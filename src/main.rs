use std::env;
use std::process::exit;

macro_rules! error {
    () => {
        eprintln!();
        exit(1);
    };
    ($fmt:expr) => {
        eprintln!($fmt);
        exit(1);
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!($fmt, $($arg)*);
        exit(1);
    };
}

enum Token {
    // 記号
    RESERVED(String),
    // 整数
    NUM(isize, String),
    // 入力の終わりを表すトークン
    EOF,
}

struct TokenStream {
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

    // 次のトークンが期待している記号のときには、トークンを1つ読み進めて
    // 真を返す。それ以外の場合には偽を返す。
    fn consume(&mut self, op: char) -> bool {
        if let Some(Token::RESERVED(s)) = self.peek() {
            if *s == op.to_string() {
                self.next();
                return true;
            }
        }
        false
    }

    // 次のトークンが期待している記号のときには、トークンを1つ読み進める。
    // それ以外の場合にはエラーを報告する。
    fn except(&mut self, op: char) {
        if let Some(Token::RESERVED(s)) = self.next() {
            if *s == op.to_string() {
                return;
            }
        }

        error!("{}ではありません", op);
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。
    // それ以外の場合にはエラーを報告する。
    fn except_number(&mut self) -> isize {
        if let Some(Token::NUM(n, _)) = self.next() {
            *n
        } else {
            error!("数ではありません。");

            #[allow(unreachable_code)]
            // 型を合わせるためのダミー
            0
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Some(Token::EOF))
    }
}

fn tokenize(src: &str) -> TokenStream {
    let expr: Vec<char> = src.chars().collect();

    let mut token = Vec::new();
    let mut num_string = String::from("");

    for i in 0..expr.len() {
        let c = expr[i];
        match c {
            '0'..='9' => {
                num_string.push(c);

                let is_last_digit = if i == expr.len() - 1 {
                    true
                } else {
                    let next_c = expr[i + 1];
                    !('0'..='9').contains(&next_c)
                };

                if is_last_digit {
                    let n = num_string.parse::<isize>().unwrap();
                    token.push(Token::NUM(n, num_string));

                    num_string = String::from("");
                }
            }

            '+' | '-' => token.push(Token::RESERVED(c.to_string())),

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => continue,

            _ => {
                error!("トークナイズできません");
            }
        }
    }

    token.push(Token::EOF);

    TokenStream { token, current: 0 }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("引数の個数が正しくありません");
    }

    let mut token_stream = tokenize(&args[1]);

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // 式の最初は数でなければならないので、それをチェックして
    // 最初のmov命令を出力
    println!("        mov rax, {}", token_stream.except_number());

    while !token_stream.at_eof() {
        if token_stream.consume('+') {
            println!("        add rax, {}", token_stream.except_number());
            continue;
        }

        token_stream.except('-');
        println!("        sub rax, {}", token_stream.except_number());
    }

    println!("        ret");
}
