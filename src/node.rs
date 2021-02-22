use std::rc::Rc;

use super::ctype::CType;
use super::parse_context::ParseContext;
use super::tokenize::Token;

pub enum NodeKind {
    // name, params(offset), body
    Defun(String, Vec<usize>, Box<Node>),
    Block(Vec<Node>),
    Return(Box<Node>),
    // cond, then, else
    If(Box<Node>, Box<Node>, Box<Node>),
    // init, cond, update, body
    For(Box<Node>, Box<Node>, Box<Node>, Box<Node>),
    Assign(Box<Node>, Box<Node>),
    Eq(Box<Node>, Box<Node>),
    Neq(Box<Node>, Box<Node>),
    LT(Box<Node>, Box<Node>),
    LTE(Box<Node>, Box<Node>),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Addr(Box<Node>),
    Deref(Box<Node>),
    Num(isize),
    // name, type, offset
    LVar(String, CType, usize),
    // name, type
    GVar(String, CType),
    // name, args
    Call(String, Vec<Node>),
}

pub struct Node {
    pub token: Rc<Token>,
    pub kind: NodeKind,
    pub ctype: CType,
}

impl Node {
    pub fn new(token: Rc<Token>, mut kind: NodeKind) -> Self {
        let ctype_ret = CType::new(&token, &mut kind);

        if let Err(reason) = ctype_ret {
            error_tok!(token, "{}", reason);
        }

        let ctype = ctype_ret.unwrap();

        Node { token, kind, ctype }
    }

    pub fn null_statement(token: Rc<Token>) -> Self {
        Self::new(token, NodeKind::Block(Vec::new()))
    }

    pub fn var(name: &str, token: Rc<Token>, ctx: &ParseContext) -> Self {
        let var_kind = ctx.find_var(name);

        if var_kind.is_none() {
            error_tok!(token, "宣言されていません");
        }

        Self::new(token, var_kind.unwrap())
    }
}
