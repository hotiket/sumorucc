use std::rc::Rc;

use super::src::Source;
use super::tokenize::{Token, TokenKind};

pub struct TokenStream<'vec> {
    token: &'vec [Rc<Token>],
    current: usize,
}

impl<'vec> TokenStream<'vec> {
    pub fn new(token: &'vec [Rc<Token>]) -> Self {
        Self { token, current: 0 }
    }

    fn get_src(&self) -> Rc<Source> {
        // 終端にEOFがあるので0要素目は必ず存在する
        Rc::clone(&self.token.get(0).unwrap().common.src)
    }

    fn peek(&self) -> Option<Rc<Token>> {
        if self.current >= self.token.len() {
            None
        } else {
            Some(Rc::clone(&self.token[self.current]))
        }
    }

    fn next(&mut self) -> Option<Rc<Token>> {
        if self.current >= self.token.len() {
            None
        } else {
            self.current += 1;
            Some(Rc::clone(&self.token[self.current - 1]))
        }
    }

    pub fn save(&self) -> usize {
        self.current
    }

    pub fn restore(&mut self, pos: usize) {
        self.current = pos;
    }

    pub fn pos(&self) -> usize {
        if let Some(token) = self.peek() {
            token.common.pos
        } else {
            self.get_src().code.chars().count()
        }
    }

    pub fn current(&self) -> Option<Rc<Token>> {
        self.token.get(self.current).map(|token| Rc::clone(token))
    }

    // 次のトークンが期待している記号のときには、trueを返す。
    // それ以外の場合にはfalseを返す。
    pub fn is_punctuator(&self, op: &str) -> bool {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Punctuator,
            }) => common.token_str == op,
            _ => false,
        }
    }

    fn is_number_impl(&self) -> Option<isize> {
        match self.peek().as_deref() {
            Some(Token {
                kind: TokenKind::Num(n),
                ..
            }) => Some(*n),
            _ => None,
        }
    }

    // 次のトークンが数値の場合、trueを返す。
    // それ以外の場合にはfalseを返す。
    #[allow(dead_code)]
    pub fn is_number(&self) -> bool {
        self.is_number_impl().is_some()
    }

    fn is_string_impl(&self) -> Option<Vec<u8>> {
        match self.peek().as_deref() {
            Some(Token {
                kind: TokenKind::Str(s),
                ..
            }) => Some(s.clone()),
            _ => None,
        }
    }

    // 次のトークンが文字列の場合、trueを返す。
    // それ以外の場合にはfalseを返す。
    #[allow(dead_code)]
    pub fn is_string(&self) -> bool {
        self.is_string_impl().is_some()
    }

    fn is_identifier_impl(&self) -> Option<String> {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Ident,
            }) => Some(common.token_str.clone()),
            _ => None,
        }
    }

    // 次のトークンが識別子の場合、trueを返す。
    // それ以外の場合にはfalseを返す。
    #[allow(dead_code)]
    fn is_identifier(&self) -> bool {
        self.is_identifier_impl().is_some()
    }

    // 次のトークンが期待しているキーワードのときには、trueを返す。
    // それ以外の場合にはfalseを返す。
    pub fn is_keyword(&self, keyword: &str) -> bool {
        match self.peek().as_deref() {
            Some(Token {
                common,
                kind: TokenKind::Keyword,
            }) => common.token_str == keyword,
            _ => false,
        }
    }

    // 次のトークンが期待している記号のときには、
    // そのトークンをSomeで包んで返し、トークンを1つ読み進める。
    // それ以外の場合にはNoneを返す。
    pub fn consume_punctuator(&mut self, op: &str) -> Option<Rc<Token>> {
        if self.is_punctuator(op) {
            self.next()
        } else {
            None
        }
    }

    // 次のトークンが数値の場合、そのトークンと数値をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_number(&mut self) -> Option<(Rc<Token>, isize)> {
        match self.is_number_impl() {
            Some(n) => Some((self.next().unwrap(), n)),
            _ => None,
        }
    }

    // 次のトークンが文字列の場合、そのトークンと文字列をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_string(&mut self) -> Option<(Rc<Token>, Vec<u8>)> {
        match self.is_string_impl() {
            Some(s) => Some((self.next().unwrap(), s)),
            _ => None,
        }
    }

    // 次のトークンが識別子の場合、そのトークンと識別子をSomeで包んで返し
    // トークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_identifier(&mut self) -> Option<(Rc<Token>, String)> {
        match self.is_identifier_impl() {
            Some(ident) => Some((self.next().unwrap(), ident)),
            _ => None,
        }
    }

    // 次のトークンが期待しているキーワードの場合、そのトークンを
    // Someで包んで返しトークンを1つ読み進める。それ以外の場合にはNoneを返す。
    pub fn consume_keyword(&mut self, keyword: &str) -> Option<Rc<Token>> {
        if self.is_keyword(keyword) {
            self.next()
        } else {
            None
        }
    }

    // 次のトークンが期待している記号のときには、そのトークンを返し
    // トークンを1つ読み進める。それ以外の場合にはエラーを報告する。
    pub fn expect_punctuator(&mut self, op: &str) -> Rc<Token> {
        let token = self.consume_punctuator(op);

        if token.is_none() {
            error_at!(self.get_src(), self.pos(), "{}ではありません", op);
        }

        token.unwrap()
    }

    // 次のトークンが数値の場合、そのトークンと数値を返し、トークンを
    // 1つ読み進める。それ以外の場合にはエラーを報告する。
    #[allow(dead_code)]
    pub fn expect_number(&mut self) -> (Rc<Token>, isize) {
        let token_num = self.consume_number();

        if token_num.is_none() {
            error_at!(self.get_src(), self.pos(), "数値ではありません");
        }

        token_num.unwrap()
    }

    // 次のトークンが識別子の場合、そのトークンを返し、トークンを
    // 1つ読み進める。それ以外の場合にはエラーを報告する。
    pub fn expect_identifier(&mut self) -> (Rc<Token>, String) {
        let token_ident = self.consume_identifier();

        if token_ident.is_none() {
            error_at!(self.get_src(), self.pos(), "識別子ではありません");
        }

        token_ident.unwrap()
    }

    // 次のトークンが期待しているキーワードの場合、そのトークンを返し
    // トークンを1つ読み進める。それ以外の場合にはエラーを報告する。
    #[allow(dead_code)]
    pub fn expect_keyword(&mut self, keyword: &str) -> Rc<Token> {
        let token = self.consume_keyword(keyword);

        if token.is_none() {
            error_at!(self.get_src(), self.pos(), "{}ではありません", keyword);
        }

        token.unwrap()
    }

    pub fn at_eof(&self) -> bool {
        matches!(
            self.peek().as_deref(),
            Some(Token {
                kind: TokenKind::EOF,
                ..
            })
        )
    }
}
