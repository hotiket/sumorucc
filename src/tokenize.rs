use std::iter::{Enumerate, Peekable};
use std::rc::Rc;
use std::str::CharIndices;

use super::Source;

pub struct TokenCommon {
    pub token_str: String,
    pub src: Rc<Source>,
    // ソースにおけるトークンの
    // コードポイント単位での開始位置
    pub pos: usize,
}

enum TokenKind {
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
    EOF,
}

pub struct Token {
    pub common: TokenCommon,
    kind: TokenKind,
}

pub struct TokenStream<'vec> {
    token: &'vec [Rc<Token>],
    current: usize,
}

impl<'vec> TokenStream<'vec> {
    pub fn new(token: &'vec [Rc<Token>]) -> Self {
        Self { token, current: 0 }
    }

    fn get_src(&self) -> Rc<Source> {
        // 終端にEOFがあるので0要素目は必ず存在する
        Rc::clone(&self.token.get(0).unwrap().common.src)
    }

    fn peek(&self) -> Option<Rc<Token>> {
        if self.current >= self.token.len() {
            None
        } else {
            Some(Rc::clone(&self.token[self.current]))
        }
    }

    fn next(&mut self) -> Option<Rc<Token>> {
        if self.current >= self.token.len() {
            None
        } else {
            self.current += 1;
            Some(Rc::clone(&self.token[self.current - 1]))
        }
    }

    pub fn save(&self) -> usize {
        self.current
    }

    pub fn restore(&mut self, pos: usize) {
        self.current = pos;
    }

    pub fn pos(&self) -> usize {
        if let Some(token) = self.peek() {
            token.common.pos
        } else {
            self.get_src().code.chars().count()
        }
    }

    pub fn current(&self) -> Option<Rc<Token>> {
        self.token.get(self.current).map(|token| Rc::clone(token))
    }

    // 次のトークンが期待している記号のときには、trueを返す。
    // それ以外の場合にはfalseを返す。
    pub fn is_punctuator(&self, op: &str) -> bool {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Punctuator,
            }) => common.token_str == op,
            _ => false,
        }
    }

    fn is_number_impl(&self) -> Option<isize> {
        match self.peek().as_deref() {
            Some(Token {
                kind: TokenKind::Num(n),
                ..
            }) => Some(*n),
            _ => None,
        }
    }

    // 次のトークンが数値の場合、trueを返す。
    // それ以外の場合にはfalseを返す。
    #[allow(dead_code)]
    pub fn is_number(&self) -> bool {
        self.is_number_impl().is_some()
    }

    fn is_string_impl(&self) -> Option<Vec<u8>> {
        match self.peek().as_deref() {
            Some(Token {
                kind: TokenKind::Str(s),
                ..
            }) => Some(s.clone()),
            _ => None,
        }
    }

    // 次のトークンが文字列の場合、trueを返す。
    // それ以外の場合にはfalseを返す。
    #[allow(dead_code)]
    pub fn is_string(&self) -> bool {
        self.is_string_impl().is_some()
    }

    fn is_identifier_impl(&self) -> Option<String> {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Ident,
            }) => Some(common.token_str.clone()),
            _ => None,
        }
    }

    // 次のトークンが識別子の場合、trueを返す。
    // それ以外の場合にはfalseを返す。
    #[allow(dead_code)]
    fn is_identifier(&self) -> bool {
        self.is_identifier_impl().is_some()
    }

    // 次のトークンが期待しているキーワードのときには、trueを返す。
    // それ以外の場合にはfalseを返す。
    pub fn is_keyword(&self, keyword: &str) -> bool {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Keyword,
            }) => common.token_str == keyword,
            _ => false,
        }
    }

    // 次のトークンが期待している記号のときには、
    // そのトークンをSomeで包んで返し、トークンを1つ読み進める。
    // それ以外の場合にはNoneを返す。
    pub fn consume_punctuator(&mut self, op: &str) -> Option<Rc<Token>> {
        if self.is_punctuator(op) {
            self.next()
        } else {
            None
        }
    }

    // 次のトークンが数値の場合、そのトークンと数値をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_number(&mut self) -> Option<(Rc<Token>, isize)> {
        match self.is_number_impl() {
            Some(n) => Some((self.next().unwrap(), n)),
            _ => None,
        }
    }

    // 次のトークンが文字列の場合、そのトークンと文字列をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_string(&mut self) -> Option<(Rc<Token>, Vec<u8>)> {
        match self.is_string_impl() {
            Some(s) => Some((self.next().unwrap(), s)),
            _ => None,
        }
    }

    // 次のトークンが識別子の場合、そのトークンと識別子をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_identifier(&mut self) -> Option<(Rc<Token>, String)> {
        match self.is_identifier_impl() {
            Some(ident) => Some((self.next().unwrap(), ident)),
            _ => None,
        }
    }

    // 次のトークンが期待しているキーワードの場合、そのトークンを
    // Someで包んで返しトークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_keyword(&mut self, keyword: &str) -> Option<Rc<Token>> {
        if self.is_keyword(keyword) {
            self.next()
        } else {
            None
        }
    }

    // 次のトークンが期待している記号のときには、そのトークンを返し
    // トークンを1つ読み進める。それ以外の場合にはエラーを報告する。
    pub fn expect_punctuator(&mut self, op: &str) -> Rc<Token> {
        let token = self.consume_punctuator(op);

        if token.is_none() {
            error_at!(self.get_src(), self.pos(), "{}ではありません", op);
        }

        token.unwrap()
    }

    // 次のトークンが数値の場合、そのトークンと数値を返し、トークンを
    // 1つ読み進める。それ以外の場合にはエラーを報告する。
    #[allow(dead_code)]
    pub fn expect_number(&mut self) -> (Rc<Token>, isize) {
        let token_num = self.consume_number();

        if token_num.is_none() {
            error_at!(self.get_src(), self.pos(), "数値ではありません");
        }

        token_num.unwrap()
    }

    // 次のトークンが識別子の場合、そのトークンを返し、トークンを
    // 1つ読み進める。それ以外の場合にはエラーを報告する。
    pub fn expect_identifier(&mut self) -> (Rc<Token>, String) {
        let token_ident = self.consume_identifier();

        if token_ident.is_none() {
            error_at!(self.get_src(), self.pos(), "識別子ではありません");
        }

        token_ident.unwrap()
    }

    // 次のトークンが期待しているキーワードの場合、そのトークンを返し
    // トークンを1つ読み進める。それ以外の場合にはエラーを報告する。
    #[allow(dead_code)]
    pub fn expect_keyword(&mut self, keyword: &str) -> Rc<Token> {
        let token = self.consume_keyword(keyword);

        if token.is_none() {
            error_at!(self.get_src(), self.pos(), "{}ではありません", keyword);
        }

        token.unwrap()
    }

    pub fn at_eof(&self) -> bool {
        matches!(
            self.peek().as_deref(),
            Some(Token {
                kind: TokenKind::EOF,
                ..
            })
        )
    }
}

fn is_punctuator(test_op: &str) -> bool {
    let symbols = [
        "==", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "(", ")", ";", "{", "}", "&", ",",
        "[", "]",
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
        "return", "if", "else", "for", "while", "int", "char", "sizeof", "struct",
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

fn read_oct_escape_sequence(
    src_iter: &mut Peekable<Enumerate<CharIndices>>,
) -> Option<(u8, usize)> {
    const DIGITS: [char; 8] = ['0', '1', '2', '3', '4', '5', '6', '7'];

    read_num_escape_sequence(src_iter, 8, &DIGITS, Some(3))
}

fn read_hex_escape_sequence(
    src_iter: &mut Peekable<Enumerate<CharIndices>>,
) -> Option<(u8, usize)> {
    const DIGITS: [char; 22] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B',
        'C', 'D', 'E', 'F',
    ];

    read_num_escape_sequence(src_iter, 16, &DIGITS, None)
}

fn read_num_escape_sequence(
    src_iter: &mut Peekable<Enumerate<CharIndices>>,
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
    // とりあえず0から255でclampして返すことにする。
    if num < 0 {
        Some((0, nr_read_bytes))
    } else if num > 255 {
        Some((255, nr_read_bytes))
    } else {
        Some((num as u8, nr_read_bytes))
    }
}

fn is_comment(src_iter: &mut Peekable<Enumerate<CharIndices>>, first: char, second: char) -> bool {
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

fn read_string(
    src_iter: &mut Peekable<Enumerate<CharIndices>>,
    terminator: char,
) -> Option<(Vec<u8>, usize)> {
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
    let mut src_iter = src.code.char_indices().enumerate().peekable();

    while let Some((pos, (byte_s, c))) = src_iter.next() {
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
                        pos,
                    },
                    kind: TokenKind::Num(n),
                }));
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
                            pos,
                        },
                        kind: TokenKind::Str(string),
                    }));
                } else {
                    error_at!(src, pos, "終端されていません");
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
                        error_at!(src, pos, "ブロックコメントの終端が存在しません");
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
                            pos,
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
                    pos,
                };

                token.push(Rc::new(Token { common, kind }));
            }

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => (),

            _ => {
                error_at!(src, pos, "トークナイズできません");
            }
        }
    }

    token.push(Rc::new(Token {
        common: TokenCommon {
            token_str: String::new(),
            src: Rc::clone(&src),
            pos: src.code.chars().count(),
        },
        kind: TokenKind::EOF,
    }));

    token
}
