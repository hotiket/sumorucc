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

    pub fn to_isize(&self) -> Option<isize> {
        match &self.kind {
            NodeKind::Eq(l, r) => Self::bi_op(l, r, |l, r| if l == r { 1 } else { 0 }),
            NodeKind::Neq(l, r) => Self::bi_op(l, r, |l, r| if l != r { 1 } else { 0 }),
            NodeKind::LT(l, r) => Self::bi_op(l, r, |l, r| if l < r { 1 } else { 0 }),
            NodeKind::LTE(l, r) => Self::bi_op(l, r, |l, r| if l <= r { 1 } else { 0 }),
            NodeKind::Add(l, r) => Self::bi_op(l, r, |l, r| l + r),
            NodeKind::Sub(l, r) => Self::bi_op(l, r, |l, r| l - r),
            NodeKind::Mul(l, r) => Self::bi_op(l, r, |l, r| l * r),
            NodeKind::Div(l, r) => Self::bi_op(l, r, |l, r| l / r),
            NodeKind::Num(n) => Some(*n),
            _ => None,
        }
    }

    fn bi_op<F>(lhs: &Self, rhs: &Self, bi_fn: F) -> Option<isize>
    where
        F: Fn(isize, isize) -> isize,
    {
        let lhs = lhs.to_isize();
        let rhs = rhs.to_isize();
        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => Some(bi_fn(lhs, rhs)),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn debug_print(&self) {
        self.debug_print_impl(0);
    }

    #[allow(dead_code)]
    pub fn debug_print_impl(&self, depth: usize) {
        let head = "  ".repeat(depth);
        match &self.kind {
            NodeKind::Defun(name, params, body) => {
                eprint!("{}Defun({}", head, &name);
                for param in params.iter() {
                    eprint!("{}, ", param);
                }
                eprintln!(")");
                body.debug_print_impl(depth + 1);
            }
            NodeKind::Block(nodes) => {
                eprintln!("{}Block", head);
                for node in nodes.iter() {
                    node.debug_print_impl(depth + 1);
                }
            }
            NodeKind::Return(node) => {
                eprintln!("{}Return", head);
                node.debug_print_impl(depth + 1);
            }
            NodeKind::If(cond, then, els) => {
                eprintln!("{}If", head);
                eprintln!("{}cond", head);
                cond.debug_print_impl(depth + 1);
                eprintln!("{}then", head);
                then.debug_print_impl(depth + 1);
                eprintln!("{}else", head);
                els.debug_print_impl(depth + 1);
            }
            NodeKind::For(init, cond, update, body) => {
                eprintln!("{}For", head);
                eprintln!("{}init", head);
                init.debug_print_impl(depth + 1);
                eprintln!("{}cond", head);
                cond.debug_print_impl(depth + 1);
                eprintln!("{}update", head);
                update.debug_print_impl(depth + 1);
                eprintln!("{}body", head);
                body.debug_print_impl(depth + 1);
            }
            NodeKind::Assign(lhs, rhs) => {
                eprintln!("{}Assign", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Eq(lhs, rhs) => {
                eprintln!("{}Eq", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Neq(lhs, rhs) => {
                eprintln!("{}Neq", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::LT(lhs, rhs) => {
                eprintln!("{}LT", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::LTE(lhs, rhs) => {
                eprintln!("{}LTE", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Add(lhs, rhs) => {
                eprintln!("{}Add", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Sub(lhs, rhs) => {
                eprintln!("{}Sub", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Mul(lhs, rhs) => {
                eprintln!("{}Mul", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Div(lhs, rhs) => {
                eprintln!("{}Div", head);
                eprintln!("{}lhs", head);
                lhs.debug_print_impl(depth + 1);
                eprintln!("{}rhs", head);
                rhs.debug_print_impl(depth + 1);
            }
            NodeKind::Addr(node) => {
                eprintln!("{}Addr", head);
                node.debug_print_impl(depth + 1);
            }
            NodeKind::Deref(node) => {
                eprintln!("{}Deref", head);
                node.debug_print_impl(depth + 1);
            }
            NodeKind::Num(n) => {
                eprintln!("{}Num({})", head, n);
            }
            NodeKind::LVar(name, ctype, offset) => {
                eprintln!("{}LVar({}, {}, {})", head, &name, &ctype, &offset);
            }
            NodeKind::GVar(name, ctype) => {
                eprintln!("{}GVar({}, {})", head, &name, &ctype);
            }
            NodeKind::Call(name, args) => {
                eprintln!("{}Call({})", head, name);
                for arg in args.iter() {
                    arg.debug_print_impl(depth + 1);
                }
            }
        }
    }
}
