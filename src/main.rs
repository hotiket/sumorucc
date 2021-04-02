use std::env;
use std::fs::File;
use std::io::{stdin, Read};
use std::rc::Rc;

#[macro_use]
mod error;

mod codegen;
mod ctype;
mod node;
mod parse;
mod parse_context;
mod tokenize;
mod util;

use codegen::codegen;
use parse::parse;
use tokenize::tokenize;

#[derive(PartialEq)]
pub struct Source {
    pub path: Option<String>,
    pub code: String,
}

fn read_input(path: &str) -> Result<Source, ()> {
    if path == "-" {
        // 標準入力から読み込み
        let mut code = String::new();

        match stdin().read_to_string(&mut code) {
            Ok(_) => Ok(Source { path: None, code }),
            Err(_) => Err(()),
        }
    } else {
        // ファイルから読み込み
        let f = File::open(path);
        if f.is_err() {
            return Err(());
        }

        let mut code = String::new();
        match f.unwrap().read_to_string(&mut code) {
            Ok(_) => Ok(Source {
                path: Some(path.to_string()),
                code,
            }),
            Err(_) => Err(()),
        }
    }
}

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
