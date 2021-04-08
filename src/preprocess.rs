use std::rc::Rc;

use super::token_stream::TokenStream;
use super::tokenize::Token;

pub fn preprocess(token: &[Rc<Token>]) -> Vec<Rc<Token>> {
    let mut stream = TokenStream::new(token);
    let mut preprocessed = Vec::new();

    preprocessing_file(&mut stream, &mut preprocessed);

    preprocessed
}

// preprocessing_file := ("#" LF | text_line)*
fn preprocessing_file(stream: &mut TokenStream, preprocessed: &mut Vec<Rc<Token>>) {
    while !stream.at_eof() {
        if stream.consume_punctuator("#").is_some() {
            stream.expect_lf();
        } else {
            text_line(stream, preprocessed);
        }
    }

    // 末尾にEOFをつける
    preprocessed.push(stream.next().unwrap());
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
