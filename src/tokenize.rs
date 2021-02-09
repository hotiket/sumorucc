use std::rc::Rc;

pub struct TokenCommon {
    pub token_str: String,
    pub src: Rc<str>,
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

    fn get_src(&self) -> Rc<str> {
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

    pub fn pos(&self) -> usize {
        if let Some(token) = self.peek() {
            token.common.pos
        } else {
            self.get_src().chars().count()
        }
    }

    pub fn current(&self) -> Option<Rc<Token>> {
        self.token.get(self.current).map(|token| Rc::clone(token))
    }

    // 次のトークンが期待している記号のときには、
    // そのトークンをSomeで包んで返し、トークンを1つ読み進める。
    // それ以外の場合にはNoneを返す。
    pub fn consume_punctuator(&mut self, op: &str) -> Option<Rc<Token>> {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Punctuator,
            }) if common.token_str == op => self.next(),
            _ => None,
        }
    }

    // 次のトークンが数値の場合、そのトークンと数値をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_number(&mut self) -> Option<(Rc<Token>, isize)> {
        match self.peek().as_deref() {
            Some(Token {
                kind: TokenKind::Num(n),
                ..
            }) => Some((self.next().unwrap(), *n)),
            _ => None,
        }
    }

    // 次のトークンが識別子の場合、そのトークンと識別子をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_identifier(&mut self) -> Option<(Rc<Token>, String)> {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Ident,
            }) => Some((self.next().unwrap(), common.token_str.clone())),
            _ => None,
        }
    }

    // 次のトークンが期待しているキーワードの場合、そのトークンを
    // Someで包んで返しトークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_keyword(&mut self, keyword: &str) -> Option<Rc<Token>> {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Keyword,
                ..
            }) if common.token_str == keyword => self.next(),
            _ => None,
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
    let keywords = ["return", "if", "else", "for", "while", "int"];

    for keyword in &keywords {
        if s == *keyword {
            return true;
        }
    }

    false
}

pub fn tokenize(src: Rc<str>) -> Vec<Rc<Token>> {
    let mut token = Vec::new();
    let mut src_iter = src.char_indices().enumerate().peekable();

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

                let token_str = src[byte_s..byte_e].to_string();
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

            // "+", "*", ";"といった記号
            _ if is_punctuator(&src[byte_s..byte_e]) => {
                while let Some((_, (_, c))) = src_iter.peek() {
                    let new_byte_e = byte_e + c.len_utf8();
                    if is_punctuator(&src[byte_s..new_byte_e]) {
                        byte_e = new_byte_e;
                        src_iter.next();
                    } else {
                        break;
                    }
                }

                let token_str = src[byte_s..byte_e].to_string();

                token.push(Rc::new(Token {
                    common: TokenCommon {
                        token_str,
                        src: Rc::clone(&src),
                        pos,
                    },
                    kind: TokenKind::Punctuator,
                }));
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

                let token_str = src[byte_s..byte_e].to_string();

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
            pos: src.chars().count(),
        },
        kind: TokenKind::EOF,
    }));

    token
}
