use std::env;
use std::rc::Rc;

#[macro_use]
mod error;

mod codegen;
mod ctype;
mod node;
mod parse;
mod parse_context;
mod src;
mod token_stream;
mod tokenize;
mod util;

use codegen::codegen;
use parse::parse;
use src::read_input;
use tokenize::tokenize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("引数の個数が正しくありません");
    }

    let src = read_input(&args[1]);
    if src.is_err() {
        error!("ソースが読み込めません");
    }

    let token = tokenize(Rc::from(src.unwrap()));

    let (node, parse_ctx) = parse(&token);

    codegen(&node, &parse_ctx);
}
