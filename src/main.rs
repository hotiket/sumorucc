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

enum Token {
    // 記号
    RESERVED(String, usize),
    // 整数
    NUM(isize, String, usize),
    // 入力の終わりを表すトークン
    EOF,
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
            Some(Token::RESERVED(_, pos)) => *pos,
            Some(Token::NUM(_, _, pos)) => *pos,
            _ => self.src.len(),
        }
    }

    // 次のトークンが期待している記号のときには、トークンを1つ読み進めて
    // 真を返す。それ以外の場合には偽を返す。
    fn consume(&mut self, op: char) -> bool {
        if let Some(Token::RESERVED(s, _)) = self.peek() {
            if *s == op.to_string() {
                self.next();
                return true;
            }
        }
        false
    }

    // 次のトークンが期待している記号のときには、トークンを1つ読み進める。
    // それ以外の場合にはエラーを報告する。
    fn expect(&mut self, op: char) {
        if let Some(Token::RESERVED(s, _)) = self.peek() {
            if *s == op.to_string() {
                self.next();
                return;
            }
        }

        error_at!(self.src, self.pos(), "{}ではありません", op);
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。
    // それ以外の場合にはエラーを報告する。
    fn expect_number(&mut self) -> isize {
        if let Some(&Token::NUM(n, _, _)) = self.peek() {
            self.next();
            return n;
        }

        error_at!(self.src, self.pos(), "数ではありません");

        #[allow(unreachable_code)]
        // 型を合わせるためのダミー
        0
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Some(Token::EOF))
    }
}

enum Node {
    ADD(Box<Node>, Box<Node>),
    SUB(Box<Node>, Box<Node>),
    MUL(Box<Node>, Box<Node>),
    DIV(Box<Node>, Box<Node>),
    NUM(isize),
}

// expr := mul ("+" mul | "-" mul)*
fn expr(token: &mut TokenStream) -> Node {
    let mut node = mul(token);

    loop {
        if token.consume('+') {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token));
            node = Node::ADD(lhs, rhs);
        } else if token.consume('-') {
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
        if token.consume('*') {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token));
            node = Node::MUL(lhs, rhs);
        } else if token.consume('/') {
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
    if token.consume('+') {
        primary(token)
    } else if token.consume('-') {
        let lhs = Box::new(Node::NUM(0));
        let rhs = Box::new(primary(token));
        Node::SUB(lhs, rhs)
    } else {
        primary(token)
    }
}

// primary := "(" expr ")" | num
fn primary(token: &mut TokenStream) -> Node {
    if token.consume('(') {
        let node = expr(token);
        token.expect(')');
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

fn tokenize(src: &str) -> TokenStream {
    let expr: Vec<char> = src.chars().collect();

    let mut token = Vec::new();
    let mut num = (String::new(), 0);

    for i in 0..expr.len() {
        let c = expr[i];
        match c {
            '0'..='9' => {
                if num.0.is_empty() {
                    num.1 = i;
                }
                num.0.push(c);

                let is_last_digit = if i == expr.len() - 1 {
                    true
                } else {
                    let next_c = expr[i + 1];
                    !('0'..='9').contains(&next_c)
                };

                if is_last_digit {
                    let n = num.0.parse::<isize>().unwrap();
                    token.push(Token::NUM(n, num.0, num.1));

                    num.0 = String::new();
                }
            }

            '+' | '-' | '*' | '/' | '(' | ')' => token.push(Token::RESERVED(c.to_string(), i)),

            // 空白文字をスキップ
            _ if c.is_ascii_whitespace() => continue,

            _ => {
                error_at!(src, i, "トークナイズできません");
            }
        }
    }

    token.push(Token::EOF);

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
