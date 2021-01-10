struct TokenCommon {
    token_str: String,
    pos: usize,
}

enum TokenKind {
    // 記号
    RESERVED,
    // 識別子
    IDENT,
    // 整数
    NUM(isize),
    // 入力の終わりを表すトークン
    EOF,
}

struct Token {
    common: TokenCommon,
    kind: TokenKind,
}

pub struct TokenStream {
    pub src: String,
    token: Vec<Token>,
    current: usize,
}

impl TokenStream {
    fn peek(&self) -> Option<&Token> {
        if self.current == self.token.len() {
            None
        } else {
            Some(&self.token[self.current])
        }
    }

    fn next(&mut self) -> Option<&Token> {
        if self.current == self.token.len() {
            None
        } else {
            self.current += 1;
            Some(&self.token[self.current - 1])
        }
    }

    pub fn pos(&self) -> usize {
        match self.peek() {
            Some(Token {
                common: TokenCommon { token_str: _, pos },
                ..
            }) => *pos,
            _ => self.src.len(),
        }
    }

    // 次のトークンが期待している記号のときには、トークンを1つ読み進めて
    // 真を返す。それ以外の場合には偽を返す。
    pub fn consume(&mut self, op: &str) -> bool {
        if let Some(Token {
            common,
            kind: TokenKind::RESERVED,
        }) = self.peek()
        {
            if common.token_str == op {
                self.next();
                return true;
            }
        }
        false
    }

    // 次のトークンが数値の場合、トークンを1つ勧めてその数値のSomeを返す。
    // それ以外の場合にはNoneを返す。
    pub fn consume_number(&mut self) -> Option<isize> {
        if let Some(&Token {
            common: _,
            kind: TokenKind::NUM(n),
        }) = self.peek()
        {
            self.next();
            Some(n)
        } else {
            None
        }
    }

    // 次のトークンが識別子の場合、トークンを1つ勧めてその識別子のSomeを返す。
    // それ以外の場合にはNoneを返す。
    pub fn consume_identifier(&mut self) -> Option<String> {
        if let Some(Token {
            common,
            kind: TokenKind::IDENT,
        }) = self.peek()
        {
            let ident = common.token_str.clone();
            self.next();
            Some(ident)
        } else {
            None
        }
    }

    // 次のトークンが期待している記号のときには、トークンを1つ読み進める。
    // それ以外の場合にはエラーを報告する。
    pub fn expect(&mut self, op: &str) {
        if self.consume(op) {
            return;
        }

        error_at!(self.src, self.pos(), "{}ではありません", op);
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。
    // それ以外の場合にはエラーを報告する。
    pub fn expect_number(&mut self) -> isize {
        if let Some(n) = self.consume_number() {
            n
        } else {
            error_at!(self.src, self.pos(), "数ではありません");

            #[allow(unreachable_code)]
            // 型を合わせるためのダミー
            0
        }
    }

    // 次のトークンが識別子の場合、トークンを1つ読み進めてその識別子を返す。
    // それ以外の場合にはエラーを報告する。
    pub fn expect_identifier(&mut self) -> String {
        if let Some(s) = self.consume_identifier() {
            s
        } else {
            error_at!(self.src, self.pos(), "識別子ではありません");

            #[allow(unreachable_code)]
            // 型を合わせるためのダミー
            String::new()
        }
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
        "==", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "(", ")", ";",
    ];

    for symbol in symbols.iter() {
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

pub fn tokenize(src: &str) -> TokenStream {
    let expr: Vec<char> = src.chars().collect();

    let mut token = Vec::new();

    let mut i = 0;
    while let Some(c) = expr.get(i) {
        let mut token_str = c.to_string();
        let pos = i;

        match c {
            '0'..='9' => {
                for c in expr.iter().skip(i + 1) {
                    if ('0'..='9').contains(c) {
                        token_str.push(*c);
                        i += 1;
                    } else {
                        break;
                    }
                }

                let n = token_str.parse::<isize>().unwrap();
                token.push(Token {
                    common: TokenCommon { token_str, pos },
                    kind: TokenKind::NUM(n),
                });
            }

            _ if is_reserved(&token_str) => {
                for c in expr.iter().skip(i + 1) {
                    token_str.push(*c);
                    if is_reserved(&token_str) {
                        i += 1;
                    } else {
                        token_str.pop();
                        break;
                    }
                }

                token.push(Token {
                    common: TokenCommon { token_str, pos },
                    kind: TokenKind::RESERVED,
                });
            }

            _ if is_ident_1(*c) => {
                for c in expr.iter().skip(i + 1) {
                    token_str.push(*c);
                    if is_ident_2(*c) {
                        i += 1;
                    } else {
                        token_str.pop();
                        break;
                    }
                }

                token.push(Token {
                    common: TokenCommon { token_str, pos },
                    kind: TokenKind::IDENT,
                });
            }

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => (),

            _ => {
                error_at!(src, i, "トークナイズできません");
            }
        }

        i += 1;
    }

    token.push(Token {
        common: TokenCommon {
            token_str: "".to_string(),
            pos: src.len(),
        },
        kind: TokenKind::EOF,
    });

    TokenStream {
        src: src.to_string(),
        token,
        current: 0,
    }
}
