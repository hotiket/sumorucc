use std::fmt;
use std::mem::{replace, swap};
use std::rc::Rc;

use super::node::{Node, NodeKind};
use super::tokenize::Token;
use super::util::align_to;

#[derive(Clone, PartialEq)]
pub enum Integer {
    Char,
    Int,
}

#[derive(Clone, PartialEq)]
pub struct Member {
    name: String,
    ctype: CType,
    offset: usize,
}

#[derive(Clone, PartialEq)]
pub enum CType {
    Integer(Integer),
    Pointer(Box<Self>),
    Array(Box<Self>, usize),
    // NOTE: タグ名, メンバーだけを持つと、それらが一致していれば
    //       異なる箇所で定義された構造体であっても同一のものと
    //       判定してしまう。定義された箇所のトークンを持つことで
    //       そのような構造体を異なるものとして判定できるようにする。
    Struct(Option<String>, Vec<Member>, Rc<Token>),
    Union(Option<String>, Vec<Member>, Rc<Token>),
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

        const ERROR_INVALID_OPERAND: &str = "無効なオペランドです";
        const ERROR_STMT_EXPR_VOID: &str = "voidを返すStatement Expressionはサポートしていません";

        match kind {
            NodeKind::Defun(..)
            | NodeKind::Block(..)
            | NodeKind::Return(..)
            | NodeKind::If(..)
            | NodeKind::For(..) => Ok(Self::Statement),
            NodeKind::StmtExpr(block) => {
                if let NodeKind::Block(body) = &block.kind {
                    if let Some(last) = body.last() {
                        if last.ctype == Self::Statement {
                            Err(ERROR_STMT_EXPR_VOID)
                        } else {
                            Ok(last.ctype.clone())
                        }
                    } else {
                        Err(ERROR_STMT_EXPR_VOID)
                    }
                } else {
                    unreachable!("StmtExprの要素がBlockではありません");
                }
            }
            NodeKind::Assign(lhs, rhs) => match (&lhs.ctype, &rhs.ctype) {
                (Self::Integer(_), Self::Integer(_)) => Ok(lhs.ctype.clone()),
                (Self::Pointer(_), Self::Pointer(_)) if lhs.ctype == rhs.ctype => {
                    Ok(lhs.ctype.clone())
                }
                (Self::Pointer(p_base), Self::Array(a_base, _)) if p_base == a_base => {
                    Self::array_to_ptr(rhs);
                    Ok(lhs.ctype.clone())
                }
                (Self::Struct(..), Self::Struct(..)) | (Self::Union(..), Self::Union(..))
                    if lhs.ctype == rhs.ctype =>
                {
                    Ok(lhs.ctype.clone())
                }
                _ => Err(ERROR_INVALID_OPERAND),
            },
            NodeKind::Eq(..) | NodeKind::Neq(..) | NodeKind::LT(..) | NodeKind::LTE(..) => {
                Ok(Self::Integer(Integer::Int))
            }
            NodeKind::Add(lhs, rhs) | NodeKind::Sub(lhs, rhs) => match (&lhs.ctype, &rhs.ctype) {
                (Self::Integer(_), Self::Integer(_)) => Ok(Self::Integer(Integer::Int)),
                (Self::Pointer(base), Self::Integer(_)) => {
                    Self::index(rhs, base.size());
                    Ok(lhs.ctype.clone())
                }
                (Self::Integer(_), Self::Pointer(base)) => {
                    Self::index(lhs, base.size());
                    Ok(rhs.ctype.clone())
                }
                (Self::Array(base, _), Self::Integer(_)) => {
                    let base = base.clone();
                    Self::array_to_ptr(lhs);
                    Self::index(rhs, base.size());
                    Ok(CType::Pointer(base))
                }
                (Self::Integer(_), Self::Array(base, _)) => {
                    let base = base.clone();
                    Self::index(lhs, base.size());
                    Self::array_to_ptr(rhs);
                    Ok(CType::Pointer(base))
                }
                _ => Err(ERROR_INVALID_OPERAND),
            },
            NodeKind::Mul(lhs, rhs) | NodeKind::Div(lhs, rhs) => match (&lhs.ctype, &rhs.ctype) {
                (Self::Integer(_), Self::Integer(_)) => Ok(Self::Integer(Integer::Int)),
                _ => Err(ERROR_INVALID_OPERAND),
            },
            NodeKind::Addr(operand) => match &operand.kind {
                NodeKind::LVar(..) | NodeKind::GVar(..) | NodeKind::Deref(..) => {
                    let base = Box::new(operand.ctype.clone());
                    Ok(Self::Pointer(base))
                }
                _ => Err(ERROR_INVALID_OPERAND),
            },
            NodeKind::Deref(operand) => match &operand.ctype {
                Self::Pointer(base) => Ok(*base.clone()),
                Self::Array(base, _) => {
                    // operandを借用したままだとarray_to_ptrが呼べないので
                    // ここでCTypeをclone()することで借用を断つ。
                    let ctype = *base.clone();
                    Self::array_to_ptr(operand);
                    Ok(ctype)
                }
                _ => Err(ERROR_INVALID_OPERAND),
            },
            NodeKind::Member(..) => {
                unreachable!("MemberはNode側でCType生成しているのでここには来ないはず")
            }
            NodeKind::Num(..) => Ok(Self::Integer(Integer::Int)),
            NodeKind::LVar(_, ctype, _) | NodeKind::GVar(_, ctype) => Ok(ctype.clone()),
            NodeKind::Call(_, ref mut args) => {
                for arg in args.iter_mut() {
                    if matches!(arg.ctype, Self::Array(..)) {
                        Self::array_to_ptr(arg);
                    }
                }

                Ok(Self::Integer(Integer::Int))
            }
        }
    }

    fn make_ctype_members<F>(
        members: Vec<(String, CType)>,
        offset_fn: F,
    ) -> Result<Vec<Member>, &'static str>
    where
        F: Fn(usize, usize) -> usize,
    {
        if members.is_empty() {
            return Err("空の構造体/共用体は定義できません");
        }

        let mut ret = Vec::<Member>::new();
        let mut current_offset = 0;

        for (name, ctype) in members.into_iter() {
            if ret.iter().any(|m| m.name == name) {
                return Err("名前が重複しているメンバーがあります");
            }

            let offset = offset_fn(current_offset, ctype.alignof());
            current_offset = offset + ctype.size();
            ret.push(Member {
                name,
                ctype,
                offset,
            });
        }

        Ok(ret)
    }

    pub fn new_struct(
        name: Option<String>,
        members: Vec<(String, CType)>,
        token: Rc<Token>,
    ) -> Result<Self, &'static str> {
        match Self::make_ctype_members(members, align_to) {
            Ok(members) => Ok(Self::Struct(name, members, token)),
            Err(msg) => Err(msg),
        }
    }

    pub fn new_union(
        name: Option<String>,
        members: Vec<(String, CType)>,
        token: Rc<Token>,
    ) -> Result<Self, &'static str> {
        match Self::make_ctype_members(members, |_, _| 0) {
            Ok(members) => Ok(Self::Union(name, members, token)),
            Err(msg) => Err(msg),
        }
    }

    pub fn get_member(&self, name: &str) -> Result<(Self, usize), &str> {
        match self {
            Self::Struct(_, members, _) | Self::Union(_, members, _) => {
                for m in members.iter() {
                    if m.name == name {
                        return Ok((m.ctype.clone(), m.offset));
                    }
                }

                Err("メンバーが存在しません")
            }
            _ => Err("構造体/共用体ではありません"),
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
            Some(Self::Integer(Integer::Int))
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
            Self::Integer(Integer::Char) => 1,
            Self::Integer(Integer::Int) => 8,
            Self::Pointer(_) => 8,
            Self::Array(base, size) => base.size() * size,
            Self::Struct(_, members, _) => {
                if let Some(m) = members.last() {
                    let raw_size = m.offset + m.ctype.size();
                    align_to(raw_size, self.alignof())
                } else {
                    unreachable!("空の構造体は定義できません");
                }
            }
            Self::Union(_, members, _) => {
                let max_size = members.iter().map(|m| m.ctype.size()).max().unwrap_or(0);
                align_to(max_size, self.alignof())
            }
            Self::Statement => 0,
        }
    }

    // 一次元の配列で現した時の要素数を返す。
    // 例えば、int[2][3]なら6, intなら1。
    pub fn flat_len(&self) -> usize {
        match self {
            Self::Integer(_) | Self::Pointer(_) | Self::Struct(..) | Self::Union(..) => 1,
            Self::Array(base, size) => base.flat_len() * size,
            Self::Statement => 0,
        }
    }

    pub fn alignof(&self) -> usize {
        match self {
            Self::Integer(_) | Self::Pointer(_) => self.size(),
            Self::Array(base, _) => base.alignof(),
            Self::Struct(_, members, _) | Self::Union(_, members, _) => {
                members.iter().map(|m| m.ctype.alignof()).max().unwrap_or(0)
            }
            Self::Statement => 0,
        }
    }

    // Arrayのbaseを再帰的に辿り返す。
    // 例えば、int[2][2]ならSome(int), (int*)[3]ならSome(int*)。
    pub fn array_base(&self) -> Option<&Self> {
        if !matches!(self, Self::Array(..)) {
            return None;
        }

        let base = self.base().unwrap();
        match base {
            Self::Array(..) => base.array_base(),
            _ => Some(base),
        }
    }

    pub fn base(&self) -> Option<&Self> {
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

        let ctype = Self::Integer(Integer::Int);
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
            Self::Integer(Integer::Char) => write!(f, "char"),
            Self::Integer(Integer::Int) => write!(f, "int"),
            Self::Pointer(base) => write!(f, "{}*", base),
            Self::Array(base, size) => write!(f, "{}[{}]", base, size),
            Self::Struct(name, members, _) | Self::Union(name, members, _) => {
                let struct_or_union = if matches!(self, Self::Struct(..)) {
                    "struct"
                } else {
                    "union"
                };

                if let Some(name) = name {
                    let _ = write!(f, "{} {} {{", struct_or_union, name);
                } else {
                    let _ = write!(f, "{} {{", struct_or_union);
                }

                let mut i = members.iter().peekable();
                while let Some(m) = i.next() {
                    let _ = write!(
                        f,
                        "{} {};{}",
                        m.ctype,
                        m.name,
                        if i.peek().is_some() { " " } else { "" }
                    );
                }

                write!(f, "}}")
            }
            Self::Statement => write!(f, "Statement"),
        }
    }
}
