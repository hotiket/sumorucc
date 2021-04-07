use std::iter::Peekable;
use std::rc::Rc;
use std::str::CharIndices;

use super::src::Source;

#[derive(Clone, Copy, PartialEq)]
pub struct Loc {
    pub row: usize,
    pub col: usize,
}

#[derive(PartialEq)]
pub struct TokenCommon {
    pub token_str: String,
    pub src: Rc<Source>,
    pub loc: Loc,
}

#[derive(PartialEq)]
pub enum TokenKind {
    // 記号
    Punctuator,
    // 識別子
    Ident,
    // キーワード
    Keyword,
    // 整数
    Num(isize),
    // 文字列
    Str(Vec<u8>),
    // 入力の終わりを表すトークン
    #[allow(clippy::upper_case_acronyms)]
    EOF,
}

#[derive(PartialEq)]
pub struct Token {
    pub common: TokenCommon,
    pub kind: TokenKind,
}

struct LocIter<'a> {
    iter: CharIndices<'a>,
    loc: Loc,
}

impl<'a> LocIter<'a> {
    fn new(iter: CharIndices<'a>) -> Self {
        LocIter {
            iter,
            loc: Loc { row: 0, col: 0 },
        }
    }
}

impl<'a> Iterator for LocIter<'a> {
    type Item = (Loc, (usize, char));
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(elem) => {
                let ret_loc = self.loc;

                if elem.1 == '\n' {
                    self.loc.row += 1;
                    self.loc.col = 0;
                } else {
                    self.loc.col += 1;
                }

                Some((ret_loc, elem))
            }
            None => None,
        }
    }
}

trait CharIndicesExt<'a> {
    fn loc_iter(self) -> LocIter<'a>;
}

impl<'a> CharIndicesExt<'a> for CharIndices<'a> {
    fn loc_iter(self) -> LocIter<'a> {
        LocIter::new(self)
    }
}

fn is_punctuator(test_op: &str) -> bool {
    let symbols = [
        "==", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "(", ")", ";", "{", "}", "&", ",",
        "[", "]", ".", "->",
    ];

    for symbol in &symbols {
        if symbol.starts_with(test_op) {
            return true;
        }
    }

    false
}

fn is_ident_1(c: char) -> bool {
    ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || '_' == c
}

fn is_ident_2(c: char) -> bool {
    is_ident_1(c) || ('0'..='9').contains(&c)
}

fn is_keyword(s: &str) -> bool {
    let keywords = [
        "return", "if", "else", "for", "while", "int", "char", "sizeof", "struct", "union",
    ];

    for keyword in &keywords {
        if s == *keyword {
            return true;
        }
    }

    false
}

fn push_char_as_u8(v: &mut Vec<u8>, c: char) {
    let mut buf = [0; 4];
    let u8_s = c.encode_utf8(&mut buf);
    for b in u8_s.bytes() {
        v.push(b);
    }
}

fn read_oct_escape_sequence(src_iter: &mut Peekable<LocIter>) -> Option<(u8, usize)> {
    const DIGITS: [char; 8] = ['0', '1', '2', '3', '4', '5', '6', '7'];

    read_num_escape_sequence(src_iter, 8, &DIGITS, Some(3))
}

fn read_hex_escape_sequence(src_iter: &mut Peekable<LocIter>) -> Option<(u8, usize)> {
    const DIGITS: [char; 22] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B',
        'C', 'D', 'E', 'F',
    ];

    read_num_escape_sequence(src_iter, 16, &DIGITS, None)
}

fn read_num_escape_sequence(
    src_iter: &mut Peekable<LocIter>,
    radix: u32,
    digits: &[char],
    max_digits: Option<usize>,
) -> Option<(u8, usize)> {
    let mut nr_read_bytes = 0;

    let mut s = String::new();

    // 最初にhexadecimal-escape-sequenceを示す'x'があれば読み捨てる
    if let Some((_, (_, c))) = src_iter.peek() {
        if *c == 'x' {
            nr_read_bytes += c.len_utf8();
            src_iter.next();
        }
    } else {
        return None;
    }

    let mut nr_read_char = 0;

    loop {
        if let Some((_, (_, c))) = src_iter.peek() {
            if digits.contains(c) {
                s.push(*c);
                nr_read_bytes += c.len_utf8();
                nr_read_char += 1;
                src_iter.next();
            } else {
                break;
            }

            if let Some(max_digits) = max_digits {
                if nr_read_char >= max_digits {
                    break;
                }
            }
        } else {
            return None;
        }
    }

    let num = isize::from_str_radix(&s, radix).unwrap();

    // 1バイトで表現できない場合の値は処理系定義。
    // GCC/Clangでは-pedantic-errorsオプションが
    // つけられた時はコンパイルエラーにしている。
    // そちらのほうがいいかもしれないが、エラーを
    // 返すのが面倒なので、とりあえず0から255で
    // clampして返すことにする。
    if num < 0 {
        Some((0, nr_read_bytes))
    } else if num > 255 {
        Some((255, nr_read_bytes))
    } else {
        Some((num as u8, nr_read_bytes))
    }
}

fn is_comment(src_iter: &mut Peekable<LocIter>, first: char, second: char) -> bool {
    if let Some((_, (_, c))) = src_iter.peek() {
        let b = (first == '/') && (*c == second);
        if b {
            src_iter.next();
        }
        b
    } else {
        false
    }
}

fn read_string(src_iter: &mut Peekable<LocIter>, terminator: char) -> Option<(Vec<u8>, usize)> {
    const ESCAPE_SEQUENCES: [(char, u8); 12] = [
        ('\'', b'\''),
        ('\"', b'"'),
        ('?', b'?'),
        ('\\', b'\\'),
        ('a', 7),
        ('b', 8),
        ('f', 12),
        ('n', b'\n'),
        ('r', b'\r'),
        ('t', b'\t'),
        ('v', 8),
        ('e', 27),
    ];

    let mut bytes = Vec::new();
    // 文字列リテラル開始から終端までに読んだバイト数
    let mut nr_read_bytes = 0;

    let mut is_terminated = false;

    while let Some((_, (_, c))) = src_iter.next() {
        match c {
            // 終端文字
            _ if c == terminator => is_terminated = true,
            // エスケープシーケンス
            '\\' => {
                if let Some((_, (_, c))) = src_iter.peek() {
                    if let Some(e) = ESCAPE_SEQUENCES.iter().find(|e| e.0 == *c) {
                        // simple-escape-sequence
                        bytes.push(e.1);
                        nr_read_bytes += c.len_utf8();
                        src_iter.next();
                    } else if ('0'..='9').contains(c) || *c == 'x' {
                        // octal-escape-sequenceもしくはhexadecimal-escape-sequence
                        let ret = if *c == 'x' {
                            read_hex_escape_sequence(src_iter)
                        } else {
                            read_oct_escape_sequence(src_iter)
                        };

                        if let Some((c, add_bytes)) = ret {
                            bytes.push(c);
                            nr_read_bytes += add_bytes;
                        } else {
                            break;
                        }
                    } else {
                        push_char_as_u8(&mut bytes, *c);
                        nr_read_bytes += c.len_utf8();
                        src_iter.next();
                    }
                } else {
                    break;
                }
            }
            // その他の文字
            _ => push_char_as_u8(&mut bytes, c),
        }

        nr_read_bytes += c.len_utf8();

        if is_terminated {
            break;
        }
    }

    if is_terminated {
        Some((bytes, nr_read_bytes))
    } else {
        None
    }
}

pub fn tokenize(src: Rc<Source>) -> Vec<Rc<Token>> {
    let mut token = Vec::new();
    let mut src_iter = src.code.char_indices().loc_iter().peekable();

    while let Some((loc, (byte_s, c))) = src_iter.next() {
        let mut byte_e = byte_s + c.len_utf8();

        match c {
            // 数値
            '0'..='9' => {
                while let Some((_, (_, c))) = src_iter.peek() {
                    if ('0'..='9').contains(c) {
                        byte_e += c.len_utf8();
                        src_iter.next();
                    } else {
                        break;
                    }
                }

                let token_str = src.code[byte_s..byte_e].to_string();
                let n = token_str.parse::<isize>().unwrap();

                token.push(Rc::new(Token {
                    common: TokenCommon {
                        token_str,
                        src: Rc::clone(&src),
                        loc,
                    },
                    kind: TokenKind::Num(n),
                }));
            }

            // 文字
            '\'' => {
                if let Some((string, nr_read_bytes)) = read_string(&mut src_iter, '\'') {
                    byte_e += nr_read_bytes;
                    let token_str = src.code[byte_s..byte_e].to_string();

                    if string.is_empty() {
                        error_at!(src, loc, "空の文字定数です");
                    }

                    // 1バイトで表現できない場合の値は処理系定義。
                    // はじめの1バイトを返すこととする。
                    let n = i8::from_ne_bytes([string[0]]) as isize;

                    token.push(Rc::new(Token {
                        common: TokenCommon {
                            token_str,
                            src: Rc::clone(&src),
                            loc,
                        },
                        kind: TokenKind::Num(n),
                    }));
                } else {
                    error_at!(src, loc, "終端されていません");
                }
            }

            // 文字列
            '"' => {
                if let Some((mut string, nr_read_bytes)) = read_string(&mut src_iter, '"') {
                    string.push(b'\0');

                    byte_e += nr_read_bytes;
                    let token_str = src.code[byte_s..byte_e].to_string();

                    token.push(Rc::new(Token {
                        common: TokenCommon {
                            token_str,
                            src: Rc::clone(&src),
                            loc,
                        },
                        kind: TokenKind::Str(string),
                    }));
                } else {
                    error_at!(src, loc, "終端されていません");
                }
            }

            // "+", "*", ";"といった記号
            _ if is_punctuator(&src.code[byte_s..byte_e]) => {
                if is_comment(&mut src_iter, c, '/') {
                    // 行コメント
                    while let Some((_, (_, c))) = src_iter.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                } else if is_comment(&mut src_iter, c, '*') {
                    // ブロックコメント
                    let mut has_terminator = false;
                    let mut prev = ' ';
                    while let Some((_, (_, c))) = src_iter.next() {
                        if (prev == '*') && (c == '/') {
                            has_terminator = true;
                            break;
                        }
                        prev = c;
                    }

                    if !has_terminator {
                        error_at!(src, loc, "ブロックコメントの終端が存在しません");
                    }
                } else {
                    while let Some((_, (_, c))) = src_iter.peek() {
                        let new_byte_e = byte_e + c.len_utf8();
                        if is_punctuator(&src.code[byte_s..new_byte_e]) {
                            byte_e = new_byte_e;
                            src_iter.next();
                        } else {
                            break;
                        }
                    }

                    let token_str = src.code[byte_s..byte_e].to_string();

                    token.push(Rc::new(Token {
                        common: TokenCommon {
                            token_str,
                            src: Rc::clone(&src),
                            loc,
                        },
                        kind: TokenKind::Punctuator,
                    }));
                }
            }

            // 識別子とキーワード
            _ if is_ident_1(c) => {
                while let Some((_, (_, c))) = src_iter.peek() {
                    if is_ident_2(*c) {
                        byte_e += c.len_utf8();
                        src_iter.next();
                    } else {
                        break;
                    }
                }

                let token_str = src.code[byte_s..byte_e].to_string();

                let kind = if is_keyword(&token_str) {
                    TokenKind::Keyword
                } else {
                    TokenKind::Ident
                };
                let common = TokenCommon {
                    token_str,
                    src: Rc::clone(&src),
                    loc,
                };

                token.push(Rc::new(Token { common, kind }));
            }

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => (),

            _ => {
                error_at!(src, loc, "トークナイズできません");
            }
        }
    }

    let loc = LocIter::new(src.code.char_indices()).last().unwrap().0;

    token.push(Rc::new(Token {
        common: TokenCommon {
            token_str: String::new(),
            src: Rc::clone(&src),
            loc,
        },
        kind: TokenKind::EOF,
    }));

    token
}
