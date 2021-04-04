use std::fs::File;
use std::io::{stdin, Read};

#[derive(PartialEq)]
pub struct Source {
    pub path: Option<String>,
    pub code: String,
}

pub fn read_input(path: &str) -> Result<Source, ()> {
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
