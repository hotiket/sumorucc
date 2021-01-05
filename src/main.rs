use std::env;
use std::process::exit;

macro_rules! error {
    ($fmt:expr) => {
        eprintln!($fmt);
        exit(1);
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!($fmt, $($arg)*);
        exit(1);
    };
}

macro_rules! error_at {
    ($src:expr, $at:expr, $fmt:expr) => {
        eprintln!("{}", $src);
        eprint!("{}^ ", " ".repeat($at));
        eprintln!($fmt);
        exit(1);
    };
    ($src:expr, $at:expr, $fmt:expr, $($arg:tt)*) => {
        eprintln!("{}", $src);
        eprintln!("{}^", " ".repeat($at));
        eprintln!($fmt, $($arg)*);
        exit(1);
    };
}

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

struct TokenStream {
    src: String,
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

    fn pos(&self) -> usize {
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
    fn consume(&mut self, op: &str) -> bool {
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
    fn expect(&mut self, op: &str) {
        if self.consume(op) {
            return;
        }

        error_at!(self.src, self.pos(), "{}ではありません", op);
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。
    // それ以外の場合にはエラーを報告する。
    fn expect_number(&mut self) -> isize {
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

    fn at_eof(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token {
                common: _,
                kind: TokenKind::EOF,
            })
        )
    }
}

enum Node {
    EQ(Box<Node>, Box<Node>),
    ADD(Box<Node>, Box<Node>),
    SUB(Box<Node>, Box<Node>),
    MUL(Box<Node>, Box<Node>),
    DIV(Box<Node>, Box<Node>),
    NUM(isize),
}

// expr := equality
fn expr(token: &mut TokenStream) -> Node {
    equality(token)
}

// equality := add ("==" add)*
fn equality(token: &mut TokenStream) -> Node {
    let mut node = add(token);

    loop {
        if token.consume("==") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token));
            node = Node::EQ(lhs, rhs);
        } else {
            return node;
        }
    }
}

// expr := mul ("+" mul | "-" mul)*
fn add(token: &mut TokenStream) -> Node {
    let mut node = mul(token);

    loop {
        if token.consume("+") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token));
            node = Node::ADD(lhs, rhs);
        } else if token.consume("-") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token));
            node = Node::SUB(lhs, rhs);
        } else {
            return node;
        }
    }
}

// mul := unary ("*" unary | "/" unary)*
fn mul(token: &mut TokenStream) -> Node {
    let mut node = unary(token);

    loop {
        if token.consume("*") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token));
            node = Node::MUL(lhs, rhs);
        } else if token.consume("/") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token));
            node = Node::DIV(lhs, rhs);
        } else {
            return node;
        }
    }
}

// unary := ("+" | "-")? primary
fn unary(token: &mut TokenStream) -> Node {
    if token.consume("+") {
        primary(token)
    } else if token.consume("-") {
        let lhs = Box::new(Node::NUM(0));
        let rhs = Box::new(primary(token));
        Node::SUB(lhs, rhs)
    } else {
        primary(token)
    }
}

// primary := "(" expr ")" | num
fn primary(token: &mut TokenStream) -> Node {
    if token.consume("(") {
        let node = expr(token);
        token.expect(")");
        node
    } else {
        Node::NUM(token.expect_number())
    }
}

fn gen_binary_operator(lhs: &Node, rhs: &Node) {
    gen(lhs);
    gen(rhs);
    println!("        pop rdi");
    println!("        pop rax");
}

fn gen(node: &Node) {
    match node {
        Node::EQ(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        cmp rax, rdi");
            println!("        sete al");
            println!("        movzb rax, al");
        }
        Node::ADD(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        add rax, rdi");
        }
        Node::SUB(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        sub rax, rdi");
        }
        Node::MUL(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        imul rax, rdi");
        }
        Node::DIV(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        cqo");
            println!("        idiv rdi");
        }
        Node::NUM(n) => {
            println!("        push {}", n);
            return;
        }
    }

    println!("        push rax");
}

fn in_operators(test_op: &str) -> bool {
    let operators = ["==", "+", "-", "*", "/", "(", ")"];

    for op in operators.iter() {
        if op.starts_with(test_op) {
            return true;
        }
    }

    false
}

fn tokenize(src: &str) -> TokenStream {
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("引数の個数が正しくありません");
    }

    // トークナイズしてパースする
    let mut token_stream = tokenize(&args[1]);
    let node = expr(&mut token_stream);

    if !token_stream.at_eof() {
        error_at!(
            token_stream.src,
            token_stream.pos(),
            "余分なトークンがあります"
        );
    }

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // 抽象構文木を下りながらコード生成
    gen(&node);

    // スタックトップに式全体の値が残っているはずなので
    // それをRAXにロードして関数からの返り値とする
    println!("        pop rax");
    println!("        ret");
}
