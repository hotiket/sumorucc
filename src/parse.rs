use super::ctype::CType;
use super::tokenize::{Token, TokenStream};

pub enum NodeKind<'token, 'vec> {
    Block(Vec<Node<'token, 'vec>>),
    Return(Box<Node<'token, 'vec>>),
    // cond, then, else
    If(
        Box<Node<'token, 'vec>>,
        Box<Node<'token, 'vec>>,
        Box<Node<'token, 'vec>>,
    ),
    // init, cond, update, body
    For(
        Box<Node<'token, 'vec>>,
        Box<Node<'token, 'vec>>,
        Box<Node<'token, 'vec>>,
        Box<Node<'token, 'vec>>,
    ),
    Assign(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Eq(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Neq(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    LT(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    LTE(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Add(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Sub(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Mul(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Div(Box<Node<'token, 'vec>>, Box<Node<'token, 'vec>>),
    Addr(Box<Node<'token, 'vec>>),
    Deref(Box<Node<'token, 'vec>>),
    Num(isize),
    // name, type, offset
    LVar(String, CType, usize),
    // name, args
    Call(String, Vec<Node<'token, 'vec>>),
}

pub struct Node<'token, 'vec> {
    pub token: &'vec Token<'token>,
    pub kind: NodeKind<'token, 'vec>,
    pub ctype: CType,
}

impl<'token, 'vec> Node<'token, 'vec> {
    pub fn new(token: &'vec Token<'token>, mut kind: NodeKind<'token, 'vec>) -> Self {
        let ctype_ret = CType::new(token, &mut kind);

        if let Err(reason) = ctype_ret {
            error_tok!(token, "{}", reason);
        }

        let ctype = ctype_ret.unwrap();

        Node { token, kind, ctype }
    }

    pub fn null_statement(token: &'vec Token<'token>) -> Self {
        Self::new(token, NodeKind::Block(Vec::new()))
    }

    pub fn lvar(name: String, token: &'vec Token<'token>, add_info: &AdditionalInfo) -> Self {
        // ローカル変数のスタックのオフセットを取得
        let lvar = add_info.find_lvar(&name);
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

pub struct AdditionalInfo {
    pub lvars: Vec<LVar>,
}

impl AdditionalInfo {
    pub fn new() -> Self {
        Self { lvars: Vec::new() }
    }

    pub fn add_lvar(&mut self, name: &str, ctype: CType, token: &Token) {
        // ローカル変数がすでに宣言されているかチェック
        let lvar = self.find_lvar(name);
        if lvar.is_some() {
            error_tok!(token, "すでに宣言されています");
        }

        // ローカル変数を新しく登録する
        let offset = if let Some(lvar) = self.lvars.last() {
            // ローカル変数のリストになければ、リスト最後の
            // ローカル変数の次に配置する
            lvar.offset + lvar.ctype.size()
        } else {
            // ローカル変数のリスト自体が空ならスタックに
            // 積まれているRBPの次となる8から始める
            8
        };

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

pub fn parse<'token, 'vec>(
    token: &'vec [Token<'token>],
) -> (Vec<Node<'token, 'vec>>, AdditionalInfo) {
    let mut stream = TokenStream::new(token);
    let mut add_info = AdditionalInfo::new();
    let nodes = program(&mut stream, &mut add_info);

    if !stream.at_eof() {
        error_tok!(stream.current().unwrap(), "余分なトークンがあります");
    }

    (nodes, add_info)
}

// program := stmt*
fn program<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Vec<Node<'token, 'vec>> {
    let mut nodes = Vec::new();

    while !stream.at_eof() {
        nodes.push(stmt(stream, add_info));
    }

    nodes
}

// stmt := "return" expr ";"
//       | "{" compound_stmt
//       | "if" "(" expr ")" stmt ("else" stmt)?
//       | "for" "(" expr_stmt expr? ";" expr? ")" stmt
//       | "while" "(" expr ")" stmt
//       | expr_stmt ";"
fn stmt<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    if let Some(token) = stream.consume_keyword("return") {
        let node = expr(stream, add_info);
        stream.expect(";");
        Node::new(token, NodeKind::Return(Box::new(node)))
    } else if stream.consume("{").is_some() {
        compound_stmt(stream, add_info)
    } else if let Some(token) = stream.consume_keyword("if") {
        stream.expect("(");
        let cond_node = Box::new(expr(stream, add_info));
        stream.expect(")");

        let then_node = Box::new(stmt(stream, add_info));

        let else_node = if stream.consume_keyword("else").is_some() {
            Box::new(stmt(stream, add_info))
        } else {
            // 紐付けるトークンがないのでif自体と紐付ける
            Box::new(Node::null_statement(token))
        };

        Node::new(token, NodeKind::If(cond_node, then_node, else_node))
    } else if let Some(token) = stream.consume_keyword("for") {
        stream.expect("(");

        let init_node = Box::new(expr_stmt(stream, add_info));

        let cond_node = if let Some(token) = stream.consume(";") {
            // 終了条件が無い場合は非0の値に置き換える
            Box::new(Node::new(token, NodeKind::Num(1)))
        } else {
            let node = Box::new(expr(stream, add_info));
            stream.expect(";");
            node
        };

        let update_node = if let Some(token) = stream.consume(")") {
            Box::new(Node::null_statement(token))
        } else {
            let node = Box::new(expr(stream, add_info));
            stream.expect(")");
            node
        };

        let body_node = Box::new(stmt(stream, add_info));

        Node::new(
            token,
            NodeKind::For(init_node, cond_node, update_node, body_node),
        )
    } else if let Some(token) = stream.consume_keyword("while") {
        // 紐付けるトークンがないのでwhile自体と紐付ける
        let init_node = Box::new(Node::null_statement(token));
        let update_node = Box::new(Node::null_statement(token));

        stream.expect("(");

        let cond_node = Box::new(expr(stream, add_info));

        stream.expect(")");

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
fn compound_stmt<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    let mut nodes = Vec::new();
    let mut token = stream.consume("}");

    while token.is_none() {
        if let Some(init_nodes) = declaration(stream, add_info) {
            nodes.extend(init_nodes);
        } else {
            nodes.push(stmt(stream, add_info));
        }

        token = stream.consume("}");
    }

    Node::new(token.unwrap(), NodeKind::Block(nodes))
}

// declaration := "int" init_declarator
fn declaration<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Option<Vec<Node<'token, 'vec>>> {
    if stream.consume_keyword("int").is_some() {
        let ctype = CType::Int;
        let init_nodes = init_declarator(stream, add_info, &ctype);
        Some(init_nodes)
    } else {
        None
    }
}

// init_declarator := (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
fn init_declarator<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
    base: &CType,
) -> Vec<Node<'token, 'vec>> {
    let mut init_nodes = Vec::new();

    if stream.consume(";").is_some() {
        return init_nodes;
    }

    loop {
        let (ident_name, ident_token) = declarator(stream, add_info, base);

        if let Some(assign_token) = stream.consume("=") {
            let lhs = Box::new(Node::lvar(ident_name, ident_token, add_info));
            let rhs = Box::new(expr(stream, add_info));
            let init_node = Node::new(assign_token, NodeKind::Assign(lhs, rhs));
            init_nodes.push(init_node);
        }

        if stream.consume(",").is_none() {
            break;
        }
    }

    stream.expect(";");

    init_nodes
}

// declarator := "*"* ident
fn declarator<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
    base: &CType,
) -> (String, &'vec Token<'token>) {
    let mut ctype = base.clone();
    while stream.consume("*").is_some() {
        ctype = CType::Pointer(Box::new(ctype));
    }

    let (token, name) = stream.expect_identifier();
    add_info.add_lvar(&name, ctype, token);
    (name, token)
}

// expr_stmt := expr? ";"
fn expr_stmt<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    if let Some(token) = stream.consume(";") {
        Node::null_statement(token)
    } else {
        let node = expr(stream, add_info);
        stream.expect(";");
        node
    }
}

// expr := assign
fn expr<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    assign(stream, add_info)
}

// assign := equality ("=" assign)?
fn assign<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    let mut node = equality(stream, add_info);

    if let Some(token) = stream.consume("=") {
        let lhs = Box::new(node);
        let rhs = Box::new(assign(stream, add_info));
        node = Node::new(token, NodeKind::Assign(lhs, rhs));
    }

    node
}

// equality := relational ("==" relational | "!=" relational)*
fn equality<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    let mut node = relational(stream, add_info);

    loop {
        if let Some(token) = stream.consume("==") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(stream, add_info));
            node = Node::new(token, NodeKind::Eq(lhs, rhs));
        } else if let Some(token) = stream.consume("!=") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(stream, add_info));
            node = Node::new(token, NodeKind::Neq(lhs, rhs));
        } else {
            return node;
        }
    }
}

// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    let mut node = add(stream, add_info);

    loop {
        if let Some(token) = stream.consume("<") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            node = Node::new(token, NodeKind::LT(lhs, rhs));
        } else if let Some(token) = stream.consume("<=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            node = Node::new(token, NodeKind::LTE(lhs, rhs));
        } else if let Some(token) = stream.consume(">") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(stream, add_info));
            // LTの左右のオペランドを入れ替えてGTにする
            node = Node::new(token, NodeKind::LT(rhs, lhs));
        } else if let Some(token) = stream.consume(">=") {
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
fn add<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    let mut node = mul(stream, add_info);

    loop {
        if let Some(token) = stream.consume("+") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(stream, add_info));
            node = Node::new(token, NodeKind::Add(lhs, rhs));
        } else if let Some(token) = stream.consume("-") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(stream, add_info));
            node = Node::new(token, NodeKind::Sub(lhs, rhs));
        } else {
            return node;
        }
    }
}

// mul := unary ("*" unary | "/" unary)*
fn mul<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    let mut node = unary(stream, add_info);

    loop {
        if let Some(token) = stream.consume("*") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(stream, add_info));
            node = Node::new(token, NodeKind::Mul(lhs, rhs));
        } else if let Some(token) = stream.consume("/") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(stream, add_info));
            node = Node::new(token, NodeKind::Div(lhs, rhs));
        } else {
            return node;
        }
    }
}

// unary := (("+" | "-" | "&" | "*")? unary) | primary
fn unary<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    if stream.consume("+").is_some() {
        unary(stream, add_info)
    } else if let Some(token) = stream.consume("-") {
        let lhs = Box::new(Node::new(token, NodeKind::Num(0)));
        let rhs = Box::new(unary(stream, add_info));
        Node::new(token, NodeKind::Sub(lhs, rhs))
    } else if let Some(token) = stream.consume("&") {
        let operand = Box::new(unary(stream, add_info));
        Node::new(token, NodeKind::Addr(operand))
    } else if let Some(token) = stream.consume("*") {
        let operand = Box::new(unary(stream, add_info));
        Node::new(token, NodeKind::Deref(operand))
    } else {
        primary(stream, add_info)
    }
}

// primary := "(" expr ")" | num | ident call-args?
fn primary<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Node<'token, 'vec> {
    if stream.consume("(").is_some() {
        let node = expr(stream, add_info);
        stream.expect(")");
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

// call-args := "(" (expr ("," expr)*)? ")"
fn call_args<'token, 'vec>(
    stream: &mut TokenStream<'token, 'vec>,
    add_info: &mut AdditionalInfo,
) -> Option<Vec<Node<'token, 'vec>>> {
    if stream.consume("(").is_some() {
        let mut args = Vec::new();

        if stream.consume(")").is_some() {
            return Some(args);
        }

        loop {
            let arg = expr(stream, add_info);
            args.push(arg);
            if stream.consume(",").is_none() {
                break;
            }
        }

        stream.expect(")");

        Some(args)
    } else {
        None
    }
}
