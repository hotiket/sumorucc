use super::tokenize::TokenStream;

pub enum Node {
    EQ(Box<Node>, Box<Node>),
    NEQ(Box<Node>, Box<Node>),
    LT(Box<Node>, Box<Node>),
    LTE(Box<Node>, Box<Node>),
    ADD(Box<Node>, Box<Node>),
    SUB(Box<Node>, Box<Node>),
    MUL(Box<Node>, Box<Node>),
    DIV(Box<Node>, Box<Node>),
    NUM(isize),
}

pub fn parse(token: &mut TokenStream) -> Node {
    let node = expr(token);

    if !token.at_eof() {
        error_at!(token.src, token.pos(), "余分なトークンがあります");
    }

    node
}

// expr := equality
fn expr(token: &mut TokenStream) -> Node {
    equality(token)
}

// equality := relational ("==" relational | "!=" relational)*
fn equality(token: &mut TokenStream) -> Node {
    let mut node = relational(token);

    loop {
        if token.consume("==") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(token));
            node = Node::EQ(lhs, rhs);
        } else if token.consume("!=") {
            let lhs = Box::new(node);
            let rhs = Box::new(relational(token));
            node = Node::NEQ(lhs, rhs);
        } else {
            return node;
        }
    }
}

// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational(token: &mut TokenStream) -> Node {
    let mut node = add(token);

    loop {
        if token.consume("<") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token));
            node = Node::LT(lhs, rhs);
        } else if token.consume("<=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token));
            node = Node::LTE(lhs, rhs);
        } else if token.consume(">") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token));
            // LTの左右のオペランドを入れ替えてGTにする
            node = Node::LT(rhs, lhs);
        } else if token.consume(">=") {
            let lhs = Box::new(node);
            let rhs = Box::new(add(token));
            // LTEの左右のオペランドを入れ替えてGTEにする
            node = Node::LTE(rhs, lhs);
        } else {
            return node;
        }
    }
}

// expr := mul ("+" mul | "-" mul)*
fn add(token: &mut TokenStream) -> Node {
    let mut node = mul(token);

    loop {
        if token.consume("+") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token));
            node = Node::ADD(lhs, rhs);
        } else if token.consume("-") {
            let lhs = Box::new(node);
            let rhs = Box::new(mul(token));
            node = Node::SUB(lhs, rhs);
        } else {
            return node;
        }
    }
}

// mul := unary ("*" unary | "/" unary)*
fn mul(token: &mut TokenStream) -> Node {
    let mut node = unary(token);

    loop {
        if token.consume("*") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token));
            node = Node::MUL(lhs, rhs);
        } else if token.consume("/") {
            let lhs = Box::new(node);
            let rhs = Box::new(unary(token));
            node = Node::DIV(lhs, rhs);
        } else {
            return node;
        }
    }
}

// unary := ("+" | "-")? primary
fn unary(token: &mut TokenStream) -> Node {
    if token.consume("+") {
        primary(token)
    } else if token.consume("-") {
        let lhs = Box::new(Node::NUM(0));
        let rhs = Box::new(primary(token));
        Node::SUB(lhs, rhs)
    } else {
        primary(token)
    }
}

// primary := "(" expr ")" | num
fn primary(token: &mut TokenStream) -> Node {
    if token.consume("(") {
        let node = expr(token);
        token.expect(")");
        node
    } else {
        Node::NUM(token.expect_number())
    }
}
