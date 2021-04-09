use std::path::Path;
use std::rc::Rc;

use super::get_preprocessed_token;
use super::token_stream::TokenStream;
use super::tokenize::Token;

fn find_include_file(name: &str, search_dirs: &[String]) -> Result<String, ()> {
    let path = Path::new(name);

    // 絶対パスなら探索せずそのまま返す
    if path.is_absolute() {
        return Ok(name.to_string());
    }

    for dir in search_dirs.iter() {
        let search_dir = Path::new(dir);
        let inc = search_dir.join(path);

        if inc.is_file() {
            return Ok(inc.to_str().unwrap().to_string());
        }
    }

    Err(())
}

pub fn preprocess(token: &[Rc<Token>]) -> Vec<Rc<Token>> {
    let mut stream = TokenStream::new(token);
    let mut preprocessed = Vec::new();

    preprocessing_file(&mut stream, &mut preprocessed);

    preprocessed
}

// preprocessing_file := ("#" directive | text_line)*
fn preprocessing_file(stream: &mut TokenStream, preprocessed: &mut Vec<Rc<Token>>) {
    while !stream.at_eof() {
        if stream.consume_punctuator("#").is_some() {
            directive(stream, preprocessed);
        } else {
            text_line(stream, preprocessed);
        }
    }

    // 末尾にEOFをつける
    preprocessed.push(stream.next().unwrap());
}

// directive := "include" str LF | LF
fn directive(stream: &mut TokenStream, preprocessed: &mut Vec<Rc<Token>>) {
    if let Some((token, directive)) = stream.consume_identifier() {
        if directive != "include" {
            error_tok!(token, "無効なディレクティブです");
        }

        let (path_token, path) = stream.expect_string();

        let mut path = String::from_utf8(path).unwrap();
        // トークナイズで追加したnulを取り除く
        path.pop();

        let mut search_dirs = Vec::new();

        // ソースが格納されているディレクトリを
        // インクルードファイルの探索パスに追加。
        if let Some(src) = &token.common.src.path {
            let src = Path::new(src);
            if let Some(src_dir) = src.parent() {
                search_dirs.push(src_dir.to_str().unwrap().to_string());
            }
        }

        search_dirs.push(".".to_string());

        let path = find_include_file(&path, &search_dirs);
        if path.is_err() {
            error_tok!(path_token, "ファイルが見つかりません");
        }
        let path = path.unwrap();

        let mut inc_token = get_preprocessed_token(&path);
        // 末尾のEOFを取り除く
        inc_token.pop();

        preprocessed.extend(inc_token);

        stream.expect_lf();
    } else {
        stream.expect_lf();
    }
}

// text_line := [^LF]* LF
fn text_line(stream: &mut TokenStream, preprocessed: &mut Vec<Rc<Token>>) {
    while stream.consume_lf().is_none() {
        match stream.next() {
            Some(token) => preprocessed.push(token),
            None => unreachable!(),
        }
    }
}
