use super::tokenize::TokenStream;

pub enum Node {
    ASSIGN(Box<Node>, Box<Node>),
    EQ(Box<Node>, Box<Node>),
    NEQ(Box<Node>, Box<Node>),
    LT(Box<Node>, Box<Node>),
    LTE(Box<Node>, Box<Node>),
    ADD(Box<Node>, Box<Node>),
    SUB(Box<Node>, Box<Node>),
    MUL(Box<Node>, Box<Node>),
    DIV(Box<Node>, Box<Node>),
    NUM(isize),
    LVAR(usize),
}

pub struct LVar {
    // 変数の名前
    pub name: String,
    // RBPからのオフセット
    pub offset: usize,
    // 変数のサイズ
    pub size: usize,
}

pub struct AdditionalInfo {
    pub lvars: Vec<LVar>,
}

pub fn parse(token: &mut TokenStream) -> (Vec<Node>, AdditionalInfo) {
    let mut add_info = AdditionalInfo { lvars: Vec::new() };
    let nodes = program(token, &mut add_info);

    if !token.at_eof() {
        error_at!(token.src, token.pos(), "余分なトークンがあります");
    }

    (nodes, add_info)
}

// program := stmt*
fn program(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Vec<Node> {
    let mut nodes = Vec::new();

    while !token.at_eof() {
        nodes.push(stmt(token, add_info));
    }

    nodes
}

// stmt := expr ";"
fn stmt(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let node = expr(token, add_info);
    token.expect(";");
    node
}

// expr := assign
fn expr(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    assign(token, add_info)
}

// assign := equality ("=" assign)?
fn assign(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = equality(token, add_info);

    if token.consume("=") {
        let lhs = Box::new(node);
        let rhs = Box::new(assign(token, add_info));
        node = Node::ASSIGN(lhs, rhs);
    }

    node
}

// equality := relational ("==" relational | "!=" relational)*
fn equality(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = relational(token, add_info);

    loop {
        if token.consume("==") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(token, add_info));
            node = Node::EQ(lhs, rhs);
        } else if token.consume("!=") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(token, add_info));
            node = Node::NEQ(lhs, rhs);
        } else {
            return node;
        }
    }
}

// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = add(token, add_info);

    loop {
        if token.consume("<") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token, add_info));
            node = Node::LT(lhs, rhs);
        } else if token.consume("<=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token, add_info));
            node = Node::LTE(lhs, rhs);
        } else if token.consume(">") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token, add_info));
            // LTの左右のオペランドを入れ替えてGTにする
            node = Node::LT(rhs, lhs);
        } else if token.consume(">=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token, add_info));
            // LTEの左右のオペランドを入れ替えてGTEにする
            node = Node::LTE(rhs, lhs);
        } else {
            return node;
        }
    }
}

// expr := mul ("+" mul | "-" mul)*
fn add(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = mul(token, add_info);

    loop {
        if token.consume("+") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token, add_info));
            node = Node::ADD(lhs, rhs);
        } else if token.consume("-") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token, add_info));
            node = Node::SUB(lhs, rhs);
        } else {
            return node;
        }
    }
}

// mul := unary ("*" unary | "/" unary)*
fn mul(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    let mut node = unary(token, add_info);

    loop {
        if token.consume("*") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token, add_info));
            node = Node::MUL(lhs, rhs);
        } else if token.consume("/") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token, add_info));
            node = Node::DIV(lhs, rhs);
        } else {
            return node;
        }
    }
}

// unary := ("+" | "-")? primary
fn unary(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    if token.consume("+") {
        primary(token, add_info)
    } else if token.consume("-") {
        let lhs = Box::new(Node::NUM(0));
        let rhs = Box::new(primary(token, add_info));
        Node::SUB(lhs, rhs)
    } else {
        primary(token, add_info)
    }
}

// primary := "(" expr ")" | num | ident
fn primary(token: &mut TokenStream, add_info: &mut AdditionalInfo) -> Node {
    if token.consume("(") {
        let node = expr(token, add_info);
        token.expect(")");
        node
    } else if let Some(n) = token.consume_number() {
        Node::NUM(n)
    } else {
        let name = token.expect_identifier();

        // ローカル変数のスタックのオフセットを取得
        let offset = if let Some(lv) = add_info.lvars.iter().find(|lv| lv.name == name) {
            // ローカル変数のリストにあればそのオフセットを使う
            lv.offset
        } else {
            let offset = if let Some(lv) = add_info.lvars.last() {
                // ローカル変数のリストになければ、リスト最後の
                // ローカル変数の次に配置する
                lv.offset + lv.size
            } else {
                // ローカル変数のリスト自体が空ならスタックに
                // 積まれているRBPの次となる8から始める
                8
            };

            add_info.lvars.push(LVar {
                name: name.clone(),
                offset,
                // 変数はintしかないので8固定
                size: 8,
            });

            offset
        };

        Node::LVAR(offset)
    }
}
