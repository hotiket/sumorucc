use std::fmt;
use std::mem::{replace, swap};
use std::rc::Rc;

use super::node::{Node, NodeKind};
use super::tokenize::Token;

#[derive(Clone, PartialEq)]
pub enum CType {
    Int,
    Pointer(Box<Self>),
    Array(Box<Self>, usize),
    Statement,
}

impl CType {
    pub fn new(token: &Rc<Token>, kind: &mut NodeKind) -> Result<Self, &'static str> {
        // kindの種別によってはkindを置き換える必要があるが
        // matchの中で置き換えようとするとkindの再借用となり
        // コンパイルできない。よって、kindを置き換える場合のみ
        // matchの前に処理をする。
        if let Some(ctype) = Self::new_ptr_sub(token, kind) {
            return Ok(ctype);
        }

        let invalid_operand = "無効なオペランドです";

        match kind {
            NodeKind::Defun(..)
            | NodeKind::Block(..)
            | NodeKind::Return(..)
            | NodeKind::If(..)
            | NodeKind::For(..) => Ok(Self::Statement),
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
                (Self::Array(base, _), Self::Int) => {
                    let base = base.clone();
                    Self::array_to_ptr(lhs);
                    Self::index(rhs, base.size());
                    Ok(CType::Pointer(base))
                }
                (Self::Int, Self::Array(base, _)) => {
                    let base = base.clone();
                    Self::index(lhs, base.size());
                    Self::array_to_ptr(rhs);
                    Ok(CType::Pointer(base))
                }
                _ => Err(invalid_operand),
            },
            NodeKind::Mul(lhs, rhs) | NodeKind::Div(lhs, rhs) => match (&lhs.ctype, &rhs.ctype) {
                (Self::Int, Self::Int) => Ok(Self::Int),
                _ => Err(invalid_operand),
            },
            NodeKind::Addr(operand) => match &operand.kind {
                NodeKind::LVar(..) | NodeKind::GVar(..) | NodeKind::Deref(..) => {
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
            NodeKind::LVar(_, ctype, _) | NodeKind::GVar(_, ctype) => Ok(ctype.clone()),
            NodeKind::Call(..) => Ok(Self::Int),
        }
    }

    // ポインタ同士の減算用の処理。減算結果をポインタが指す
    // 型のサイズで割り、要素数を返すようにkindを置き換える。
    fn new_ptr_sub(token: &Rc<Token>, kind: &mut NodeKind) -> Option<Self> {
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

    // 配列からポインタへの暗黙の型変換。
    // ポインタを返すようにAddr(node)に置き換える。
    fn array_to_ptr(node: &mut Node) {
        // Arrayのときだけ呼ばれるのでunwrapして問題ない
        let base = node.ctype.base().unwrap().clone();
        let ctype = CType::Pointer(Box::new(base));

        let token = Rc::clone(&node.token);

        // ダミーノードと元のノードを入れ替える
        let dummy_node = Node::null_statement(Rc::clone(&token));
        let org_node = replace(node, dummy_node);

        // ポインタ演算するためにアドレスを返すようにする
        let kind = NodeKind::Addr(Box::new(org_node));

        // Addr(node)にしたノードと元のノードを入れ替える
        let mut new_node = Node { token, kind, ctype };
        swap(node, &mut new_node);
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Int => 8,
            Self::Pointer(_) => 8,
            Self::Array(base, size) => base.size() * size,
            Self::Statement => 0,
        }
    }

    fn base(&self) -> Option<&Self> {
        match self {
            Self::Array(base, _) => Some(&base),
            Self::Pointer(base) => Some(&base),
            _ => None,
        }
    }

    // ptr + nがptrのn番目の要素を指すようにnをsizeof(ptr)倍する
    fn index(node: &mut Node, size: usize) {
        let dummy_node = Node::null_statement(Rc::clone(&node.token));
        let org_node = Box::new(replace(node, dummy_node));

        let size_node = Box::new(Node::new(
            Rc::clone(&node.token),
            NodeKind::Num(size as isize),
        ));

        let mut new_node = Node::new(Rc::clone(&node.token), NodeKind::Mul(org_node, size_node));
        swap(node, &mut new_node);
    }

    // ptr2 - ptr1が要素数を返すように(ptr2 - ptr1) / sizeof(ptr1)にする
    fn num_of_elements(token: &Rc<Token>, kind: &mut NodeKind, size: usize) {
        let dummy_kind = NodeKind::Num(0);
        let org_kind = replace(kind, dummy_kind);

        let ctype = Self::Int;
        let org_node = Box::new(Node {
            token: Rc::clone(token),
            kind: org_kind,
            ctype,
        });

        let size_node = Box::new(Node::new(Rc::clone(token), NodeKind::Num(size as isize)));

        let mut new_kind = NodeKind::Div(org_node, size_node);
        swap(kind, &mut new_kind);
    }
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Pointer(base) => write!(f, "{}*", base),
            Self::Array(base, size) => write!(f, "{}[{}]", base, size),
            Self::Statement => write!(f, "Statement"),
        }
    }
}
