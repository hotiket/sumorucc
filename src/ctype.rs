use std::fmt;
use std::mem::{replace, swap};

use super::parse::{Node, NodeKind};
use super::tokenize::Token;

#[derive(Clone, PartialEq)]
pub enum CType {
    Int,
    Pointer(Box<Self>),
    Statement,
}

impl CType {
    pub fn new<'token, 'vec>(
        token: &'vec Token<'token>,
        kind: &mut NodeKind<'token, 'vec>,
    ) -> Result<Self, String> {
        // ポインタ同士の減算の場合、減算結果をポインタが指す
        // 型のサイズで割り、要素数を返すようにkindを置き換える
        // 必要がある。matchの中だとkindが再借用となりコンパイル
        // できないため、ポインタの減算のみmatchの前に処理する。
        if let Some(ctype) = Self::new_ptr_sub(token, kind) {
            return Ok(ctype);
        }

        let invalid_operand = "無効なオペランドです".to_string();

        match kind {
            NodeKind::Block(..) | NodeKind::Return(..) | NodeKind::If(..) | NodeKind::For(..) => {
                Ok(Self::Statement)
            }
            NodeKind::Assign(lhs, rhs) => {
                if lhs.ctype == rhs.ctype {
                    Ok(lhs.ctype.clone())
                } else {
                    Err(invalid_operand)
                }
            }
            NodeKind::Eq(..) | NodeKind::Neq(..) | NodeKind::LT(..) | NodeKind::LTE(..) => {
                Ok(Self::Int)
            }
            NodeKind::Add(lhs, rhs) | NodeKind::Sub(lhs, rhs) => match (&lhs.ctype, &rhs.ctype) {
                (Self::Int, Self::Int) => Ok(Self::Int),
                (Self::Pointer(base), Self::Int) => {
                    Self::index(rhs, base.size());
                    Ok(lhs.ctype.clone())
                }
                (Self::Int, Self::Pointer(base)) => {
                    Self::index(lhs, base.size());
                    Ok(rhs.ctype.clone())
                }
                _ => Err(invalid_operand),
            },
            NodeKind::Mul(lhs, rhs) | NodeKind::Div(lhs, rhs) => match (&lhs.ctype, &rhs.ctype) {
                (Self::Int, Self::Int) => Ok(Self::Int),
                _ => Err(invalid_operand),
            },
            NodeKind::Addr(operand) => match &operand.kind {
                NodeKind::LVar(..) | NodeKind::Deref(..) => {
                    let base = Box::new(operand.ctype.clone());
                    Ok(Self::Pointer(base))
                }
                _ => Err(invalid_operand),
            },
            NodeKind::Deref(operand) => match &operand.ctype {
                Self::Pointer(base) => Ok(*base.clone()),
                _ => Err(invalid_operand),
            },
            NodeKind::Num(..) => Ok(Self::Int),
            NodeKind::LVar(_, ctype, _) => Ok(ctype.clone()),
        }
    }

    // ポインタ同士の減算用の処理
    fn new_ptr_sub<'token, 'vec>(
        token: &'vec Token<'token>,
        kind: &mut NodeKind<'token, 'vec>,
    ) -> Option<Self> {
        // kindが同じ型のポインタ同士の減算かチェック
        let base_size = if let NodeKind::Sub(lhs, rhs) = kind {
            if let Self::Pointer(base) = &lhs.ctype {
                if lhs.ctype == rhs.ctype {
                    Some(base.size())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 同じ型のポインタ同士の減算なら
        // ポインタが指す型で割るようにkindを置き換える
        if let Some(base_size) = base_size {
            Self::num_of_elements(token, kind, base_size);
            Some(Self::Int)
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Int => 8,
            Self::Pointer(_) => 8,
            Self::Statement => 0,
        }
    }

    // ptr + nがptrのn番目の要素を指すようにnをsizeof(ptr)倍する
    fn index(node: &mut Node, size: usize) {
        let dummy_node = Node::null_statement(node.token);
        let org_node = Box::new(replace(node, dummy_node));

        let size_node = Box::new(Node::new(node.token, NodeKind::Num(size as isize)));

        let mut new_node = Node::new(node.token, NodeKind::Mul(org_node, size_node));
        swap(node, &mut new_node);
    }

    // ptr2 - ptr1が要素数を返すように(ptr2 - ptr1) / sizeof(ptr1)にする
    fn num_of_elements<'token, 'vec>(
        token: &'vec Token<'token>,
        kind: &mut NodeKind<'token, 'vec>,
        size: usize,
    ) {
        let dummy_kind = NodeKind::Num(0);
        let org_kind = replace(kind, dummy_kind);

        let ctype = Self::Int;
        let org_node = Box::new(Node {
            token,
            kind: org_kind,
            ctype,
        });

        let size_node = Box::new(Node::new(token, NodeKind::Num(size as isize)));

        let mut new_kind = NodeKind::Div(org_node, size_node);
        swap(kind, &mut new_kind);
    }
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Pointer(base) => write!(f, "{}*", base),
            Self::Statement => write!(f, "Statement"),
        }
    }
}
