pub struct TokenCommon<'src> {
    pub token_str: &'src str,
    pub src: &'src str,
    // ソースにおけるトークンの
    // コードポイント単位での開始位置
    pub pos: usize,
}

enum TokenKind {
    // 記号
    RESERVED,
    // 識別子
    IDENT,
    // キーワード
    KEYWORD,
    // 整数
    NUM(isize),
    // 入力の終わりを表すトークン
    EOF,
}

pub struct Token<'src> {
    pub common: TokenCommon<'src>,
    kind: TokenKind,
}

pub struct TokenStream<'token, 'vec> {
    token: &'vec [Token<'token>],
    current: usize,
}

impl<'token, 'vec> TokenStream<'token, 'vec> {
    pub fn new(token: &'vec [Token<'token>]) -> Self {
        Self { token, current: 0 }
    }

    fn get_src(&self) -> &'token str {
        // 終端にEOFがあるので0要素目は必ず存在する
        self.token.get(0).unwrap().common.src
    }

    fn peek(&self) -> Option<&'vec Token<'token>> {
        if self.current >= self.token.len() {
            None
        } else {
            Some(&self.token[self.current])
        }
    }

    fn next(&mut self) -> Option<&'vec Token<'token>> {
        if self.current >= self.token.len() {
            None
        } else {
            self.current += 1;
            Some(&self.token[self.current - 1])
        }
    }

    pub fn pos(&self) -> usize {
        match self.peek() {
            Some(Token {
                common:
                    TokenCommon {
                        token_str: _,
                        src: _,
                        pos,
                    },
                ..
            }) => *pos,
            _ => self.get_src().len(),
        }
    }

    pub fn current(&self) -> Option<&'vec Token<'token>> {
        self.token.get(self.current)
    }

    // 次のトークンが期待している記号のときには、
    // そのトークンをSomeで包んで返し、トークンを1つ読み進める。
    // それ以外の場合にはNoneを返す。
    pub fn consume(self: &mut TokenStream<'token, 'vec>, op: &str) -> Option<&'vec Token<'token>> {
        if let Some(Token {
            common,
            kind: TokenKind::RESERVED,
        }) = self.peek()
        {
            if common.token_str == op {
                return self.next();
            }
        }
        None
    }

    // 次のトークンが数値の場合、そのトークンと数値をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_number(&mut self) -> Option<(&'vec Token<'token>, isize)> {
        if let Some(&Token {
            common: _,
            kind: TokenKind::NUM(n),
        }) = self.peek()
        {
            Some((self.next().unwrap(), n))
        } else {
            None
        }
    }

    // 次のトークンが識別子の場合、そのトークンと識別子をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_identifier(&mut self) -> Option<(&'vec Token<'token>, String)> {
        if let Some(Token {
            common,
            kind: TokenKind::IDENT,
        }) = self.peek()
        {
            Some((self.next().unwrap(), common.token_str.to_string()))
        } else {
            None
        }
    }

    // 次のトークンが期待しているキーワードの場合、そのトークンを
    // Someで包んで返しトークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_keyword(&mut self, keyword: &str) -> Option<&'vec Token<'token>> {
        if let Some(Token {
            common,
            kind: TokenKind::KEYWORD,
        }) = self.peek()
        {
            if common.token_str == keyword {
                return self.next();
            }
        }
        None
    }

    // 次のトークンが期待している記号のときには、そのトークンを返し
    // トークンを1つ読み進める。それ以外の場合にはエラーを報告する。
    pub fn expect(&mut self, op: &str) -> &'vec Token<'token> {
        let token = self.consume(op);

        if token.is_none() {
            error_at!(self.get_src(), self.pos(), "{}ではありません", op);
        }

        token.unwrap()
    }

    // 次のトークンが識別子の場合、そのトークンを返し、トークンを
    // 1つ読み進める。それ以外の場合にはエラーを報告する。
    pub fn expect_identifier(&mut self) -> (&'vec Token<'token>, String) {
        let token_ident = self.consume_identifier();

        if token_ident.is_none() {
            error_at!(self.get_src(), self.pos(), "識別子ではありません");
        }

        token_ident.unwrap()
    }

    pub fn at_eof(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token {
                common: _,
                kind: TokenKind::EOF,
            })
        )
    }
}

fn is_reserved(test_op: &str) -> bool {
    let symbols = [
        "==", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "(", ")", ";", "{", "}", "&", ",",
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

pub fn tokenize<'src>(src: &'src str) -> Vec<Token<'src>> {
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

                let token_str = &src[byte_s..byte_e];
                let n = token_str.parse::<isize>().unwrap();

                token.push(Token {
                    common: TokenCommon {
                        token_str,
                        src,
                        pos,
                    },
                    kind: TokenKind::NUM(n),
                });
            }

            // "+", "*", ";"といった記号
            _ if is_reserved(&src[byte_s..byte_e]) => {
                while let Some((_, (_, c))) = src_iter.peek() {
                    let new_byte_e = byte_e + c.len_utf8();
                    if is_reserved(&src[byte_s..new_byte_e]) {
                        byte_e = new_byte_e;
                        src_iter.next();
                    } else {
                        break;
                    }
                }

                let token_str = &src[byte_s..byte_e];

                token.push(Token {
                    common: TokenCommon {
                        token_str,
                        src,
                        pos,
                    },
                    kind: TokenKind::RESERVED,
                });
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

                let token_str = &src[byte_s..byte_e];

                let kind = if is_keyword(&token_str) {
                    TokenKind::KEYWORD
                } else {
                    TokenKind::IDENT
                };
                let common = TokenCommon {
                    token_str,
                    src,
                    pos,
                };

                token.push(Token { common, kind });
            }

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => (),

            _ => {
                error_at!(src, pos, "トークナイズできません");
            }
        }
    }

    token.push(Token {
        common: TokenCommon {
            token_str: "",
            src,
            pos: src.len(),
        },
        kind: TokenKind::EOF,
    });

    token
}
