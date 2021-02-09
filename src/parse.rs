use std::rc::Rc;

use super::ctype::CType;
use super::tokenize::{Token, TokenStream};

pub enum NodeKind {
    // name, args(offset), body
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

    pub fn lvar(name: String, token: Rc<Token>, add_info: &AdditionalInfo) -> Self {
        // ローカル変数のスタックのオフセットを取得
        let lvar = add_info
            .current_fn()
            .expect("関数定義外でのローカル変数参照です")
            .find_lvar(&name);
        if lvar.is_none() {
            error_tok!(token, "宣言されていません");
        }
        let lvar = lvar.unwrap();
        let offset = lvar.offset;

        Self::new(token, NodeKind::LVar(name, lvar.ctype.clone(), offset))
    }
}

pub struct LVar {
    // 変数の名前
    pub name: String,
    // RBPからのオフセット
    pub offset: usize,
    // 変数の型
    pub ctype: CType,
}

pub struct Function {
    pub name: String,
    pub lvars: Vec<LVar>,
}

impl Function {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            lvars: Vec::new(),
        }
    }

    pub fn add_lvar(&mut self, name: &str, ctype: CType, token: &Rc<Token>) {
        // 同名のローカル変数がすでに宣言されているかチェック
        let lvar = self.find_lvar(name);
        if lvar.is_some() {
            error_tok!(token, "すでに宣言されています");
        }

        // リスト最後のローカル変数の次に登録する
        let last_offset = self.lvars.last().map_or(0, |lvar| lvar.offset);
        let offset = last_offset + ctype.size();

        self.lvars.push(LVar {
            name: name.to_string(),
            offset,
            ctype,
        });
    }

    pub fn find_lvar(&self, name: &str) -> Option<&LVar> {
        self.lvars.iter().find(|lvar| lvar.name == name)
    }
}

pub struct AdditionalInfo {
    functions: Vec<Function>,
}

impl AdditionalInfo {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    pub fn add_fn(&mut self, name: &str, token: &Rc<Token>) {
        // 同名の関数がすでに宣言されているかチェック
        let lvar = self.find_fn(name);
        if lvar.is_some() {
            error_tok!(token, "すでに宣言されています");
        }

        self.functions.push(Function::new(name));
    }

    pub fn find_fn(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|function| function.name == name)
    }

    pub fn current_fn(&self) -> Option<&Function> {
        self.functions.last()
    }

    pub fn current_fn_mut(&mut self) -> Option<&mut Function> {
        self.functions.last_mut()
    }
}

pub fn parse(token: &[Rc<Token>]) -> (Vec<Node>, AdditionalInfo) {
    let mut stream = TokenStream::new(token);
    let mut add_info = AdditionalInfo::new();
    let nodes = program(&mut stream, &mut add_info);

    if !stream.at_eof() {
        error_tok!(stream.current().unwrap(), "余分なトークンがあります");
    }

    (nodes, add_info)
}

// program := stmt*
fn program(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Vec<Node> {
    let mut nodes = Vec::new();

    while !stream.at_eof() {
        nodes.push(function_definition(stream, add_info));
    }

    nodes
}

// function_definition := "int" function_declarator "{" compound_stmt
fn function_definition(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    stream.expect_keyword("int");

    let (token, name, args) = function_declarator(stream, add_info);

    if args.len() > 6 {
        error_tok!(token, "引数が6つを超える関数定義はサポートしていません");
    }

    stream.expect_punctuator("{");
    let body = Box::new(compound_stmt(stream, add_info));

    Node::new(token, NodeKind::Defun(name, args, body))
}

// function_declarator := ident "(" ("int" ident ("," "int" ident)*)? ")"
fn function_declarator(
    stream: &mut TokenStream,
    add_info: &mut AdditionalInfo,
) -> (Rc<Token>, String, Vec<usize>) {
    let (func_token, func_name) = stream.expect_identifier();
    add_info.add_fn(&func_name, &func_token);

    stream.expect_punctuator("(");

    let mut args = Vec::new();

    if stream.consume_punctuator(")").is_some() {
        return (func_token, func_name, args);
    }

    loop {
        stream.expect_keyword("int");
        let ctype = CType::Int;

        let (arg_token, arg_name) = stream.expect_identifier();

        add_info
            .current_fn_mut()
            .unwrap()
            .add_lvar(&arg_name, ctype, &arg_token);

        let lvar = add_info.current_fn().unwrap().find_lvar(&arg_name).unwrap();
        args.push(lvar.offset);

        if stream.consume_punctuator(",").is_none() {
            break;
        }
    }

    stream.expect_punctuator(")");

    (func_token, func_name, args)
}

// stmt := "return" expr ";"
//       | "{" compound_stmt
//       | "if" "(" expr ")" stmt ("else" stmt)?
//       | "for" "(" expr_stmt expr? ";" expr? ")" stmt
//       | "while" "(" expr ")" stmt
//       | expr_stmt ";"
fn stmt(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    if let Some(token) = stream.consume_keyword("return") {
        let node = expr(stream, add_info);
        stream.expect_punctuator(";");
        Node::new(token, NodeKind::Return(Box::new(node)))
    } else if stream.consume_punctuator("{").is_some() {
        compound_stmt(stream, add_info)
    } else if let Some(token) = stream.consume_keyword("if") {
        stream.expect_punctuator("(");
        let cond_node = Box::new(expr(stream, add_info));
        stream.expect_punctuator(")");

        let then_node = Box::new(stmt(stream, add_info));

        let else_node = if stream.consume_keyword("else").is_some() {
            Box::new(stmt(stream, add_info))
        } else {
            // 紐付けるトークンがないのでif自体と紐付ける
            Box::new(Node::null_statement(Rc::clone(&token)))
        };

        Node::new(token, NodeKind::If(cond_node, then_node, else_node))
    } else if let Some(token) = stream.consume_keyword("for") {
        stream.expect_punctuator("(");

        let init_node = Box::new(expr_stmt(stream, add_info));

        let cond_node = if let Some(token) = stream.consume_punctuator(";") {
            // 終了条件が無い場合は非0の値に置き換える
            Box::new(Node::new(token, NodeKind::Num(1)))
        } else {
            let node = Box::new(expr(stream, add_info));
            stream.expect_punctuator(";");
            node
        };

        let update_node = if let Some(token) = stream.consume_punctuator(")") {
            Box::new(Node::null_statement(token))
        } else {
            let node = Box::new(expr(stream, add_info));
            stream.expect_punctuator(")");
            node
        };

        let body_node = Box::new(stmt(stream, add_info));

        Node::new(
            token,
            NodeKind::For(init_node, cond_node, update_node, body_node),
        )
    } else if let Some(token) = stream.consume_keyword("while") {
        // 紐付けるトークンがないのでwhile自体と紐付ける
        let init_node = Box::new(Node::null_statement(Rc::clone(&token)));
        let update_node = Box::new(Node::null_statement(Rc::clone(&token)));

        stream.expect_punctuator("(");

        let cond_node = Box::new(expr(stream, add_info));

        stream.expect_punctuator(")");

        let body_node = Box::new(stmt(stream, add_info));

        // initとupdateが空のfor文として生成する
        Node::new(
            token,
            NodeKind::For(init_node, cond_node, update_node, body_node),
        )
    } else {
        expr_stmt(stream, add_info)
    }
}

// compound_stmt := (declaration | stmt)* "}"
fn compound_stmt(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut nodes = Vec::new();
    let mut token = stream.consume_punctuator("}");

    while token.is_none() {
        if let Some(init_nodes) = declaration(stream, add_info) {
            nodes.extend(init_nodes);
        } else {
            nodes.push(stmt(stream, add_info));
        }

        token = stream.consume_punctuator("}");
    }

    Node::new(token.unwrap(), NodeKind::Block(nodes))
}

// declaration := "int" init_declarator
fn declaration(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Option<Vec<Node>> {
    if stream.consume_keyword("int").is_some() {
        let ctype = CType::Int;
        let init_nodes = init_declarator(stream, add_info, &ctype);
        Some(init_nodes)
    } else {
        None
    }
}

// init_declarator := (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
fn init_declarator(
    stream: &mut TokenStream,
    add_info: &mut AdditionalInfo,
    base: &CType,
) -> Vec<Node> {
    let mut init_nodes = Vec::new();

    if stream.consume_punctuator(";").is_some() {
        return init_nodes;
    }

    loop {
        let (ident_name, ident_token) = declarator(stream, add_info, base);

        if let Some(assign_token) = stream.consume_punctuator("=") {
            let lhs = Box::new(Node::lvar(ident_name, ident_token, add_info));
            let rhs = Box::new(expr(stream, add_info));
            let init_node = Node::new(assign_token, NodeKind::Assign(lhs, rhs));
            init_nodes.push(init_node);
        }

        if stream.consume_punctuator(",").is_none() {
            break;
        }
    }

    stream.expect_punctuator(";");

    init_nodes
}

// declarator := "*"* ident ("[" num "]")?
fn declarator(
    stream: &mut TokenStream,
    add_info: &mut AdditionalInfo,
    base: &CType,
) -> (String, Rc<Token>) {
    let mut ctype = base.clone();
    while stream.consume_punctuator("*").is_some() {
        ctype = CType::Pointer(Box::new(ctype));
    }

    let (token, name) = stream.expect_identifier();

    if stream.consume_punctuator("[").is_some() {
        let (token, n) = stream.expect_number();
        if n <= 0 {
            error_tok!(token, "要素数が0以下の配列は定義できません");
        }
        ctype = CType::Array(Box::new(ctype), n as usize);
        stream.expect_punctuator("]");
    }

    add_info
        .current_fn_mut()
        .expect("関数定義外での宣言です")
        .add_lvar(&name, ctype, &token);
    (name, token)
}

// expr_stmt := expr? ";"
fn expr_stmt(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    if let Some(token) = stream.consume_punctuator(";") {
        Node::null_statement(token)
    } else {
        let node = expr(stream, add_info);
        stream.expect_punctuator(";");
        node
    }
}

// expr := assign
fn expr(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    assign(stream, add_info)
}

// assign := equality ("=" assign)?
fn assign(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = equality(stream, add_info);

    if let Some(token) = stream.consume_punctuator("=") {
        let lhs = Box::new(node);
        let rhs = Box::new(assign(stream, add_info));
        node = Node::new(token, NodeKind::Assign(lhs, rhs));
    }

    node
}

// equality := relational ("==" relational | "!=" relational)*
fn equality(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = relational(stream, add_info);

    loop {
        if let Some(token) = stream.consume_punctuator("==") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(stream, add_info));
            node = Node::new(token, NodeKind::Eq(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("!=") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(stream, add_info));
            node = Node::new(token, NodeKind::Neq(lhs, rhs));
        } else {
            return node;
        }
    }
}

// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = add(stream, add_info);

    loop {
        if let Some(token) = stream.consume_punctuator("<") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            node = Node::new(token, NodeKind::LT(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("<=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            node = Node::new(token, NodeKind::LTE(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator(">") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            // LTの左右のオペランドを入れ替えてGTにする
            node = Node::new(token, NodeKind::LT(rhs, lhs));
        } else if let Some(token) = stream.consume_punctuator(">=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            // LTEの左右のオペランドを入れ替えてGTEにする
            node = Node::new(token, NodeKind::LTE(rhs, lhs));
        } else {
            return node;
        }
    }
}

// expr := mul ("+" mul | "-" mul)*
fn add(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = mul(stream, add_info);

    loop {
        if let Some(token) = stream.consume_punctuator("+") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(stream, add_info));
            node = Node::new(token, NodeKind::Add(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("-") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(stream, add_info));
            node = Node::new(token, NodeKind::Sub(lhs, rhs));
        } else {
            return node;
        }
    }
}

// mul := unary ("*" unary | "/" unary)*
fn mul(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = unary(stream, add_info);

    loop {
        if let Some(token) = stream.consume_punctuator("*") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(stream, add_info));
            node = Node::new(token, NodeKind::Mul(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("/") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(stream, add_info));
            node = Node::new(token, NodeKind::Div(lhs, rhs));
        } else {
            return node;
        }
    }
}

// unary := (("+" | "-" | "&" | "*")? unary) | postfix
fn unary(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    if stream.consume_punctuator("+").is_some() {
        unary(stream, add_info)
    } else if let Some(token) = stream.consume_punctuator("-") {
        let lhs = Box::new(Node::new(Rc::clone(&token), NodeKind::Num(0)));
        let rhs = Box::new(unary(stream, add_info));
        Node::new(token, NodeKind::Sub(lhs, rhs))
    } else if let Some(token) = stream.consume_punctuator("&") {
        let operand = Box::new(unary(stream, add_info));
        Node::new(token, NodeKind::Addr(operand))
    } else if let Some(token) = stream.consume_punctuator("*") {
        let operand = Box::new(unary(stream, add_info));
        Node::new(token, NodeKind::Deref(operand))
    } else {
        postfix(stream, add_info)
    }
}

// postfix := primary ("[" expr "]")?
fn postfix(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = primary(stream, add_info);

    if let Some(bracket_token) = stream.consume_punctuator("[") {
        let index = Box::new(expr(stream, add_info));

        node = Node::new(
            Rc::clone(&bracket_token),
            NodeKind::Add(Box::new(node), index),
        );
        node = Node::new(bracket_token, NodeKind::Deref(Box::new(node)));

        stream.expect_punctuator("]");
    }

    node
}

// primary := "(" expr ")" | num | ident call_args?
fn primary(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    if stream.consume_punctuator("(").is_some() {
        let node = expr(stream, add_info);
        stream.expect_punctuator(")");
        node
    } else if let Some((token, n)) = stream.consume_number() {
        Node::new(token, NodeKind::Num(n))
    } else {
        let (token, name) = stream.expect_identifier();

        if let Some(args) = call_args(stream, add_info) {
            // 関数呼び出し
            if args.len() > 6 {
                error_tok!(token, "引数が6つを超える関数呼び出しはサポートしていません");
            }
            Node::new(token, NodeKind::Call(name, args))
        } else {
            // 変数
            Node::lvar(name, token, add_info)
        }
    }
}

// call_args := "(" (expr ("," expr)*)? ")"
fn call_args(stream: &mut TokenStream, add_info: &mut AdditionalInfo) -> Option<Vec<Node>> {
    if stream.consume_punctuator("(").is_some() {
        let mut args = Vec::new();

        if stream.consume_punctuator(")").is_some() {
            return Some(args);
        }

        loop {
            let arg = expr(stream, add_info);
            args.push(arg);
            if stream.consume_punctuator(",").is_none() {
                break;
            }
        }

        stream.expect_punctuator(")");

        Some(args)
    } else {
        None
    }
}
