use std::env;
use std::rc::Rc;

macro_rules! error {
    ($fmt:expr) => {
        eprintln!($fmt);
        std::process::exit(1);
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!($fmt, $($arg)*);
        std::process::exit(1);
    };
}

macro_rules! error_at {
    ($src:expr, $at:expr, $fmt:expr) => {
        eprintln!("{}", $src);
        eprint!("{}^ ", " ".repeat($at));
        eprintln!($fmt);
        std::process::exit(1);
    };
    ($src:expr, $at:expr, $fmt:expr, $($arg:tt)*) => {
        eprintln!("{}", $src);
        eprint!("{}^ ", " ".repeat($at));
        eprintln!($fmt, $($arg)*);
        std::process::exit(1);
    };
}

macro_rules! error_tok {
    ($tok:expr, $fmt:expr) => {
        error_at!($tok.common.src, $tok.common.pos, $fmt);
    };
    ($tok:expr, $fmt:expr, $($arg:tt)*) => {
        error_at!($tok.common.src, $tok.common.pos, $fmt, $($arg)*);
    };
}

mod codegen;
mod ctype;
mod node;
mod parse;
mod parse_context;
mod tokenize;

use codegen::codegen;
use parse::parse;
use tokenize::tokenize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("引数の個数が正しくありません");
    }

    let src = Rc::from(args[1].clone());

    let token = tokenize(src);

    let (node, parse_ctx) = parse(&token);

    codegen(&node, &parse_ctx);
}
