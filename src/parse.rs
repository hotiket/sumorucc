use std::rc::Rc;

use super::ctype::CType;
use super::tokenize::{Token, TokenStream};

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

pub struct LVar {
    // 変数の名前
    pub name: String,
    // 変数の型
    pub ctype: CType,
    // RBPからのオフセット
    pub offset: usize,
}

pub struct GVar {
    // 変数の名前
    pub name: String,
    // 変数の型
    pub ctype: CType,
}

struct Scope {
    child: Option<Box<Self>>,
    lvars: Vec<LVar>,
}

impl Scope {
    fn new() -> Self {
        Self {
            child: None,
            lvars: Vec::new(),
        }
    }

    fn add_var(&mut self, name: &str, ctype: CType, offset: usize) -> Result<(), &str> {
        if let Some(ref mut child) = self.child {
            child.add_var(name, ctype, offset)
        } else if self.find_current_var(name).is_some() {
            Err("すでに定義されています")
        } else {
            self.lvars.push(LVar {
                name: name.to_string(),
                ctype,
                offset,
            });

            Ok(())
        }
    }

    fn find_var(&self, name: &str) -> Option<NodeKind> {
        if let Some(ref child) = self.child {
            let lvar = child.find_var(name);
            if lvar.is_some() {
                return lvar;
            }
        }

        self.find_current_var(name)
    }

    fn find_current_var(&self, name: &str) -> Option<NodeKind> {
        self.lvars
            .iter()
            .find(|v| v.name == name)
            .map(|v| NodeKind::LVar(v.name.clone(), v.ctype.clone(), v.offset))
    }

    fn enter(&mut self) {
        if let Some(ref mut child) = self.child {
            child.enter();
        } else {
            self.child = Some(Box::new(Self::new()));
        }
    }

    fn exit(&mut self) -> Result<(), &str> {
        if self.child.is_none() {
            return Err("対応するスコープがありません");
        }

        self.exit_impl();
        Ok(())
    }

    // 戻り値: スコープ削除の可否
    fn exit_impl(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            if !child.exit_impl() {
                self.child = None;
            }
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn debug_print_lvars(&self) {
        self.debug_print_lvars_impl(0);
    }

    #[allow(dead_code)]
    fn debug_print_lvars_impl(&self, depth: usize) {
        eprintln!("{}DEPTH={}", " ".repeat(depth), depth);
        for lvar in self.lvars.iter() {
            eprintln!("{}{} {}", " ".repeat(depth), &lvar.ctype, &lvar.name);
        }

        if let Some(ref child) = self.child {
            child.debug_print_lvars_impl(depth + 1);
        }
    }
}

pub struct Function {
    name: String,
    stack_size: usize,
    scope: Scope,
}

impl Function {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stack_size: 0,
            scope: Scope::new(),
        }
    }

    fn add_var(&mut self, name: &str, ctype: CType) -> Result<(), &str> {
        let offset = self.stack_size + ctype.size();
        let result = self.scope.add_var(name, ctype, offset);

        // 変数の追加に成功したらスタックサイズを更新する
        if result.is_ok() {
            self.stack_size = offset;
        }

        result
    }

    fn find_var(&self, name: &str) -> Option<NodeKind> {
        self.scope.find_var(name)
    }

    fn enter(&mut self) {
        self.scope.enter();
    }

    fn exit(&mut self) -> Result<(), &str> {
        self.scope.exit()
    }

    #[allow(dead_code)]
    fn debug_print_lvars(&self) {
        self.scope.debug_print_lvars();
    }
}

pub struct ParseContext {
    pub funcs: Vec<Function>,
    pub gvars: Vec<GVar>,
    current_fn: Option<String>,
}

impl ParseContext {
    pub fn new() -> Self {
        Self {
            funcs: Vec::new(),
            gvars: Vec::new(),
            current_fn: None,
        }
    }

    pub fn stack_size(&self, name: &str) -> Option<usize> {
        self.find_fn(name).map(|func| func.stack_size)
    }

    pub fn enter_fn(&mut self, name: &str) -> Result<(), &str> {
        if self.current_fn.is_some() {
            return Err("関数内での関数定義です");
        }

        if self.find_fn(name).is_none() && self.find_gvar(name).is_none() {
            self.funcs.push(Function::new(name));
            self.current_fn = Some(name.to_string());
            Ok(())
        } else {
            Err("すでに定義されています")
        }
    }

    pub fn exit_fn(&mut self) -> Result<(), &str> {
        if self.current_fn.is_none() {
            return Err("関数定義がされていません");
        }

        self.current_fn = None;
        Ok(())
    }

    pub fn find_fn(&self, name: &str) -> Option<&Function> {
        self.funcs.iter().find(|f| f.name == name)
    }

    pub fn find_fn_mut(&mut self, name: &str) -> Option<&mut Function> {
        self.funcs.iter_mut().find(|f| f.name == name)
    }

    pub fn add_var(&mut self, name: &str, ctype: CType) -> Result<(), &str> {
        if self.current_fn.is_some() {
            // selfの再借用にならないよう処理中の関数名をクローンを作成する
            let fn_name = self.current_fn.as_ref().unwrap().clone();
            let func = self.find_fn_mut(&fn_name).unwrap();
            func.add_var(name, ctype)
        } else if self.find_gvar(name).is_some() || self.find_fn(name).is_some() {
            Err("すでに定義されています")
        } else {
            self.gvars.push(GVar {
                name: name.to_string(),
                ctype,
            });
            Ok(())
        }
    }

    pub fn find_var(&self, name: &str) -> Option<NodeKind> {
        self.find_lvar(name).or_else(|| self.find_gvar(name))
    }

    pub fn find_lvar(&self, name: &str) -> Option<NodeKind> {
        if let Some(ref fn_name) = self.current_fn {
            let func = self.find_fn(fn_name).unwrap();
            func.find_var(name)
        } else {
            None
        }
    }

    pub fn find_gvar(&self, name: &str) -> Option<NodeKind> {
        self.gvars
            .iter()
            .find(|v| v.name == name)
            .map(|v| NodeKind::GVar(v.name.clone(), v.ctype.clone()))
    }

    pub fn enter_scope(&mut self) -> Result<(), &str> {
        if self.current_fn.is_none() {
            return Err("関数定義がされていません");
        }

        let fn_name = self.current_fn.as_ref().unwrap().clone();
        let func = self.find_fn_mut(&fn_name).unwrap();
        func.enter();
        Ok(())
    }

    pub fn exit_scope(&mut self) -> Result<(), &str> {
        if self.current_fn.is_none() {
            return Err("関数定義がされていません");
        }

        let fn_name = self.current_fn.as_ref().unwrap().clone();
        let func = self.find_fn_mut(&fn_name).unwrap();
        func.exit()
    }

    #[allow(dead_code)]
    fn debug_print_lvars(&self) {
        self.find_fn(&self.current_fn.as_ref().unwrap())
            .unwrap()
            .debug_print_lvars();
    }
}

pub fn parse(token: &[Rc<Token>]) -> (Vec<Node>, ParseContext) {
    let mut stream = TokenStream::new(token);
    let mut ctx = ParseContext::new();
    let nodes = program(&mut stream, &mut ctx);

    if !stream.at_eof() {
        error_tok!(stream.current().unwrap(), "余分なトークンがあります");
    }

    (nodes, ctx)
}

// program := (function_definition | global_declaration)*
fn program(stream: &mut TokenStream, ctx: &mut ParseContext) -> Vec<Node> {
    let mut nodes = Vec::new();

    while !stream.at_eof() {
        if is_function(stream) {
            nodes.push(function_definition(stream, ctx));
        } else {
            global_declaration(stream, ctx);
        }
    }

    nodes
}

// "int" "*"* ident "(" ならば真を返す
// それ以外は偽を返す
fn is_function(stream: &mut TokenStream) -> bool {
    let mut result = false;

    let state = stream.save();

    if stream.consume_keyword("int").is_some() {
        while stream.consume_punctuator("*").is_some() {}
        if stream.consume_identifier().is_some() && stream.consume_punctuator("(").is_some() {
            result = true;
        }
    }

    stream.restore(state);

    result
}

// function_definition := "int" function_declarator "{" compound_stmt
fn function_definition(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    stream.expect_keyword("int");

    let (token, name, params) = function_declarator(stream);

    if params.len() > 6 {
        error_tok!(token, "引数が6つを超える関数定義はサポートしていません");
    }

    if let Err(msg) = ctx.enter_fn(&name) {
        error_tok!(token, "{}", msg);
    }

    // 引数をローカル変数として登録する
    for Parameter { token, name, ctype } in params.iter() {
        if let Err(msg) = ctx.add_var(name, ctype.clone()) {
            error_tok!(token, "{}", msg);
        }
    }

    // Defun構築用に引数のオフセットを取得する
    let mut offsets = Vec::new();
    for Parameter {
        token: _,
        name,
        ctype: _,
    } in params.into_iter()
    {
        if let Some(NodeKind::LVar(_, _, offset)) = ctx.find_lvar(&name) {
            offsets.push(offset);
        } else {
            unreachable!();
        }
    }

    stream.expect_punctuator("{");
    let body = Box::new(compound_stmt(stream, ctx));

    if ctx.exit_fn().is_err() {
        unreachable!();
    }

    Node::new(token, NodeKind::Defun(name, offsets, body))
}

struct Parameter {
    token: Rc<Token>,
    name: String,
    ctype: CType,
}

impl Parameter {
    fn new(token: Rc<Token>, name: String, ctype: CType) -> Self {
        Self { token, name, ctype }
    }
}

// function_declarator := ident "(" ("int" ident ("," "int" ident)*)? ")"
fn function_declarator(stream: &mut TokenStream) -> (Rc<Token>, String, Vec<Parameter>) {
    let (func_token, func_name) = stream.expect_identifier();

    stream.expect_punctuator("(");

    let mut params = Vec::new();

    if stream.consume_punctuator(")").is_some() {
        return (func_token, func_name, params);
    }

    loop {
        stream.expect_keyword("int");
        let ctype = CType::Int;

        let (param_token, param_name) = stream.expect_identifier();

        params.push(Parameter::new(param_token, param_name, ctype));

        if stream.consume_punctuator(",").is_none() {
            break;
        }
    }

    stream.expect_punctuator(")");

    (func_token, func_name, params)
}

// global_declaration := "int" (declarator ("," declarator)*)? ";"
fn global_declaration(stream: &mut TokenStream, ctx: &mut ParseContext) {
    stream.expect_keyword("int");
    let base = CType::Int;

    if stream.consume_punctuator(";").is_some() {
        return;
    }

    loop {
        let _ = declarator(stream, ctx, &base);

        if stream.consume_punctuator(",").is_none() {
            break;
        }
    }

    stream.expect_punctuator(";");
}

// stmt := "return" expr ";"
//       | "{" compound_stmt
//       | "if" "(" expr ")" stmt ("else" stmt)?
//       | "for" "(" expr_stmt expr? ";" expr? ")" stmt
//       | "while" "(" expr ")" stmt
//       | expr_stmt ";"
fn stmt(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    if let Some(token) = stream.consume_keyword("return") {
        let node = expr(stream, ctx);
        stream.expect_punctuator(";");
        Node::new(token, NodeKind::Return(Box::new(node)))
    } else if stream.consume_punctuator("{").is_some() {
        compound_stmt(stream, ctx)
    } else if let Some(token) = stream.consume_keyword("if") {
        stream.expect_punctuator("(");
        let cond_node = Box::new(expr(stream, ctx));
        stream.expect_punctuator(")");

        let then_node = Box::new(stmt(stream, ctx));

        let else_node = if stream.consume_keyword("else").is_some() {
            Box::new(stmt(stream, ctx))
        } else {
            // 紐付けるトークンがないのでif自体と紐付ける
            Box::new(Node::null_statement(Rc::clone(&token)))
        };

        Node::new(token, NodeKind::If(cond_node, then_node, else_node))
    } else if let Some(token) = stream.consume_keyword("for") {
        stream.expect_punctuator("(");

        let init_node = Box::new(expr_stmt(stream, ctx));

        let cond_node = if let Some(token) = stream.consume_punctuator(";") {
            // 終了条件が無い場合は非0の値に置き換える
            Box::new(Node::new(token, NodeKind::Num(1)))
        } else {
            let node = Box::new(expr(stream, ctx));
            stream.expect_punctuator(";");
            node
        };

        let update_node = if let Some(token) = stream.consume_punctuator(")") {
            Box::new(Node::null_statement(token))
        } else {
            let node = Box::new(expr(stream, ctx));
            stream.expect_punctuator(")");
            node
        };

        let body_node = Box::new(stmt(stream, ctx));

        Node::new(
            token,
            NodeKind::For(init_node, cond_node, update_node, body_node),
        )
    } else if let Some(token) = stream.consume_keyword("while") {
        // 紐付けるトークンがないのでwhile自体と紐付ける
        let init_node = Box::new(Node::null_statement(Rc::clone(&token)));
        let update_node = Box::new(Node::null_statement(Rc::clone(&token)));

        stream.expect_punctuator("(");

        let cond_node = Box::new(expr(stream, ctx));

        stream.expect_punctuator(")");

        let body_node = Box::new(stmt(stream, ctx));

        // initとupdateが空のfor文として生成する
        Node::new(
            token,
            NodeKind::For(init_node, cond_node, update_node, body_node),
        )
    } else {
        expr_stmt(stream, ctx)
    }
}

// compound_stmt := (declaration | stmt)* "}"
fn compound_stmt(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut nodes = Vec::new();

    if ctx.enter_scope().is_err() {
        unreachable!();
    }

    let mut token = stream.consume_punctuator("}");

    while token.is_none() {
        if let Some(init_nodes) = declaration(stream, ctx) {
            nodes.extend(init_nodes);
        } else {
            nodes.push(stmt(stream, ctx));
        }

        token = stream.consume_punctuator("}");
    }

    if ctx.exit_scope().is_err() {
        unreachable!();
    }

    Node::new(token.unwrap(), NodeKind::Block(nodes))
}

// declaration := "int" init_declarator
fn declaration(stream: &mut TokenStream, ctx: &mut ParseContext) -> Option<Vec<Node>> {
    if stream.consume_keyword("int").is_some() {
        let ctype = CType::Int;
        let init_nodes = init_declarator(stream, ctx, &ctype);
        Some(init_nodes)
    } else {
        None
    }
}

// init_declarator := (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
fn init_declarator(stream: &mut TokenStream, ctx: &mut ParseContext, base: &CType) -> Vec<Node> {
    let mut init_nodes = Vec::new();

    if stream.consume_punctuator(";").is_some() {
        return init_nodes;
    }

    loop {
        let (ident_name, ident_token) = declarator(stream, ctx, base);

        if let Some(assign_token) = stream.consume_punctuator("=") {
            let lhs = Box::new(Node::var(&ident_name, ident_token, ctx));
            let rhs = Box::new(expr(stream, ctx));
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

// declarator := "*"* ident ("[" num "]")*
fn declarator(
    stream: &mut TokenStream,
    ctx: &mut ParseContext,
    base: &CType,
) -> (String, Rc<Token>) {
    let mut ctype = base.clone();
    while stream.consume_punctuator("*").is_some() {
        ctype = CType::Pointer(Box::new(ctype));
    }

    let (token, name) = stream.expect_identifier();

    let mut array_sizes = Vec::new();
    while stream.consume_punctuator("[").is_some() {
        let (token, n) = stream.expect_number();
        if n <= 0 {
            error_tok!(token, "要素数が0以下の配列は定義できません");
        }
        array_sizes.push(n as usize);
        stream.expect_punctuator("]");
    }

    // int[2][3]はArray(Array(int, 3), 2)となるので
    // 逆順に配列サイズを見ていく。
    for n in array_sizes.into_iter().rev() {
        ctype = CType::Array(Box::new(ctype), n);
    }

    if let Err(msg) = ctx.add_var(&name, ctype) {
        error_tok!(&token, "{}", msg);
    }

    (name, token)
}

// expr_stmt := expr? ";"
fn expr_stmt(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    if let Some(token) = stream.consume_punctuator(";") {
        Node::null_statement(token)
    } else {
        let node = expr(stream, ctx);
        stream.expect_punctuator(";");
        node
    }
}

// expr := assign
fn expr(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    assign(stream, ctx)
}

// assign := equality ("=" assign)?
fn assign(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut node = equality(stream, ctx);

    if let Some(token) = stream.consume_punctuator("=") {
        let lhs = Box::new(node);
        let rhs = Box::new(assign(stream, ctx));
        node = Node::new(token, NodeKind::Assign(lhs, rhs));
    }

    node
}

// equality := relational ("==" relational | "!=" relational)*
fn equality(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut node = relational(stream, ctx);

    loop {
        if let Some(token) = stream.consume_punctuator("==") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(stream, ctx));
            node = Node::new(token, NodeKind::Eq(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("!=") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(stream, ctx));
            node = Node::new(token, NodeKind::Neq(lhs, rhs));
        } else {
            return node;
        }
    }
}

// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut node = add(stream, ctx);

    loop {
        if let Some(token) = stream.consume_punctuator("<") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, ctx));
            node = Node::new(token, NodeKind::LT(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("<=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, ctx));
            node = Node::new(token, NodeKind::LTE(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator(">") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, ctx));
            // LTの左右のオペランドを入れ替えてGTにする
            node = Node::new(token, NodeKind::LT(rhs, lhs));
        } else if let Some(token) = stream.consume_punctuator(">=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, ctx));
            // LTEの左右のオペランドを入れ替えてGTEにする
            node = Node::new(token, NodeKind::LTE(rhs, lhs));
        } else {
            return node;
        }
    }
}

// expr := mul ("+" mul | "-" mul)*
fn add(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut node = mul(stream, ctx);

    loop {
        if let Some(token) = stream.consume_punctuator("+") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(stream, ctx));
            node = Node::new(token, NodeKind::Add(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("-") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(stream, ctx));
            node = Node::new(token, NodeKind::Sub(lhs, rhs));
        } else {
            return node;
        }
    }
}

// mul := unary ("*" unary | "/" unary)*
fn mul(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut node = unary(stream, ctx);

    loop {
        if let Some(token) = stream.consume_punctuator("*") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(stream, ctx));
            node = Node::new(token, NodeKind::Mul(lhs, rhs));
        } else if let Some(token) = stream.consume_punctuator("/") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(stream, ctx));
            node = Node::new(token, NodeKind::Div(lhs, rhs));
        } else {
            return node;
        }
    }
}

// unary := (("+" | "-" | "&" | "*")? unary) | postfix
fn unary(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    if stream.consume_punctuator("+").is_some() {
        unary(stream, ctx)
    } else if let Some(token) = stream.consume_punctuator("-") {
        let lhs = Box::new(Node::new(Rc::clone(&token), NodeKind::Num(0)));
        let rhs = Box::new(unary(stream, ctx));
        Node::new(token, NodeKind::Sub(lhs, rhs))
    } else if let Some(token) = stream.consume_punctuator("&") {
        let operand = Box::new(unary(stream, ctx));
        Node::new(token, NodeKind::Addr(operand))
    } else if let Some(token) = stream.consume_punctuator("*") {
        let operand = Box::new(unary(stream, ctx));
        Node::new(token, NodeKind::Deref(operand))
    } else {
        postfix(stream, ctx)
    }
}

// postfix := primary ("[" expr "]")*
fn postfix(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    let mut node = primary(stream, ctx);

    while let Some(bracket_token) = stream.consume_punctuator("[") {
        let index = Box::new(expr(stream, ctx));

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
fn primary(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    if stream.consume_punctuator("(").is_some() {
        let node = expr(stream, ctx);
        stream.expect_punctuator(")");
        node
    } else if let Some((token, n)) = stream.consume_number() {
        Node::new(token, NodeKind::Num(n))
    } else {
        let (token, name) = stream.expect_identifier();

        if let Some(args) = call_args(stream, ctx) {
            // 関数呼び出し
            if args.len() > 6 {
                error_tok!(token, "引数が6つを超える関数呼び出しはサポートしていません");
            }
            Node::new(token, NodeKind::Call(name, args))
        } else {
            // 変数
            Node::var(&name, token, ctx)
        }
    }
}

// call_args := "(" (expr ("," expr)*)? ")"
fn call_args(stream: &mut TokenStream, ctx: &mut ParseContext) -> Option<Vec<Node>> {
    if stream.consume_punctuator("(").is_some() {
        let mut args = Vec::new();

        if stream.consume_punctuator(")").is_some() {
            return Some(args);
        }

        loop {
            let arg = expr(stream, ctx);
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
