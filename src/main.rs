use std::env;
use std::rc::Rc;

#[macro_use]
mod error;

mod codegen;
mod ctype;
mod node;
mod parse;
mod parse_context;
mod preprocess;
mod src;
mod token_stream;
mod tokenize;
mod util;

use codegen::codegen;
use parse::parse;
use preprocess::preprocess;
use src::read_input;
use tokenize::{tokenize, Token};

fn get_preprocessed_token(path: &str) -> Vec<Rc<Token>> {
    let src = read_input(path);
    if src.is_err() {
        error!("ソースが読み込めません: {}", path);
    }

    let token = tokenize(Rc::from(src.unwrap()));

    preprocess(&token)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("引数の個数が正しくありません");
    }

    let token = get_preprocessed_token(&args[1]);

    let (node, parse_ctx) = parse(&token);

    codegen(&node, &parse_ctx);
}
