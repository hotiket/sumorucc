struct TokenCommon {
    token_str: String,
    pos: usize,
}

enum TokenKind {
    // 記号
    RESERVED,
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
        if let Some(&Token {
            common: _,
            kind: TokenKind::NUM(n),
        }) = self.peek()
        {
            self.next();
            return n;
        }

        error_at!(self.src, self.pos(), "数ではありません");

        #[allow(unreachable_code)]
        // 型を合わせるためのダミー
        0
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

fn in_operators(test_op: &str) -> bool {
    let operators = [
        "==", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "(", ")",
    ];

    for op in operators.iter() {
        if op.starts_with(test_op) {
            return true;
        }
    }

    false
}

pub fn tokenize(src: &str) -> TokenStream {
    let expr: Vec<char> = src.chars().collect();

    let mut token = Vec::new();
    // 処理中のトークン
    // トークン文字列/トークン開始位置からなるタプル
    let mut in_progress = (String::new(), 0);

    for i in 0..expr.len() {
        let c = expr[i];
        let new_token = format!("{}{}", &in_progress.0, c);

        match c {
            '0'..='9' => {
                if in_progress.0.is_empty() {
                    in_progress.1 = i;
                }
                in_progress.0.push(c);

                let is_last_digit = if i == expr.len() - 1 {
                    true
                } else {
                    let next_c = expr[i + 1];
                    !('0'..='9').contains(&next_c)
                };

                if is_last_digit {
                    let n = in_progress.0.parse::<isize>().unwrap();
                    token.push(Token {
                        common: TokenCommon {
                            token_str: in_progress.0,
                            pos: in_progress.1,
                        },
                        kind: TokenKind::NUM(n),
                    });

                    in_progress.0 = String::new();
                }
            }

            _ if in_operators(&new_token) => {
                if in_progress.0.is_empty() {
                    in_progress.1 = i;
                }
                in_progress.0.push(c);

                let is_last_char = if i == expr.len() - 1 {
                    true
                } else {
                    let next_c = expr[i + 1];
                    let next_token = format!("{}{}", &in_progress.0, next_c);
                    !in_operators(&next_token)
                };

                if is_last_char {
                    token.push(Token {
                        common: TokenCommon {
                            token_str: in_progress.0,
                            pos: in_progress.1,
                        },
                        kind: TokenKind::RESERVED,
                    });

                    in_progress.0 = String::new();
                }
            }

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => continue,

            _ => {
                error_at!(src, i, "トークナイズできません");
            }
        }
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
