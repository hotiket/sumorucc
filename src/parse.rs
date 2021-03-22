use std::rc::Rc;

use super::ctype::{CType, Integer};
use super::node::{Node, NodeKind};
use super::parse_context::ParseContext;
use super::tokenize::{Token, TokenStream};

pub fn parse(token: &[Rc<Token>]) -> (Vec<Node>, ParseContext) {
    let mut stream = TokenStream::new(token);
    let mut ctx = ParseContext::new();
    let nodes = program(&mut stream, &mut ctx);

    if !stream.at_eof() {
        error_tok!(stream.current().unwrap(), "余分なトークンがあります");
    }

    (nodes, ctx)
}

// program := (function_definition | declaration)*
fn program(stream: &mut TokenStream, ctx: &mut ParseContext) -> Vec<Node> {
    let mut nodes = Vec::new();

    while !stream.at_eof() {
        if is_function(stream) {
            nodes.push(function_definition(stream, ctx));
        } else if declaration(stream, ctx).is_none() {
            error_tok!(
                stream.current().unwrap(),
                "トップレベルでは関数定義かグローバル変数定義のみできます"
            );
        }
    }

    nodes
}

// type_specifier := "int" | "char"
fn type_specifier(stream: &mut TokenStream) -> Option<(CType, Rc<Token>)> {
    if let Some(token) = stream.consume_keyword("int") {
        Some((CType::Integer(Integer::Int), token))
    } else if let Some(token) = stream.consume_keyword("char") {
        Some((CType::Integer(Integer::Char), token))
    } else {
        None
    }
}

// type_specifier "*"* ident "(" ならば真を返す
// それ以外は偽を返す
fn is_function(stream: &mut TokenStream) -> bool {
    let mut result = false;

    let state = stream.save();

    if type_specifier(stream).is_some() {
        while stream.consume_punctuator("*").is_some() {}
        if stream.consume_identifier().is_some() && stream.consume_punctuator("(").is_some() {
            result = true;
        }
    }

    stream.restore(state);

    result
}

// function_definition := type_specifier function_declarator "{" compound_stmt
fn function_definition(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    if type_specifier(stream).is_none() {
        error_tok!(stream.current().unwrap(), "型ではありません");
    }

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
        ctype,
    } in params.into_iter()
    {
        if let Some(NodeKind::LVar(_, _, offset)) = ctx.find_lvar(&name) {
            offsets.push((offset, ctype));
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

// function_declarator := ident "(" (type_specifier ident ("," type_specifier ident)*)? ")"
fn function_declarator(stream: &mut TokenStream) -> (Rc<Token>, String, Vec<Parameter>) {
    let (func_token, func_name) = stream.expect_identifier();

    stream.expect_punctuator("(");

    let mut params = Vec::new();

    if stream.consume_punctuator(")").is_some() {
        return (func_token, func_name, params);
    }

    loop {
        let type_spec = type_specifier(stream);
        if type_spec.is_none() {
            error_tok!(stream.current().unwrap(), "型ではありません");
        }
        let ctype = type_spec.unwrap().0;

        let (param_token, param_name) = stream.expect_identifier();

        params.push(Parameter::new(param_token, param_name, ctype));

        if stream.consume_punctuator(",").is_none() {
            break;
        }
    }

    stream.expect_punctuator(")");

    (func_token, func_name, params)
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

// declaration := type_specifier init_declarator
fn declaration(stream: &mut TokenStream, ctx: &mut ParseContext) -> Option<Vec<Node>> {
    if let Some((ctype, token)) = type_specifier(stream) {
        let mut init_nodes = init_declarator(stream, ctx, &ctype);
        // ({int x=1;})のようなstatement expressionの値がintに
        // ならないように、最後にCType::Statementとなるノードを入れる。
        init_nodes.push(Node::null_statement(token));
        Some(init_nodes)
    } else {
        None
    }
}

// init_declarator := (declarator ("=" initializer)? ("," declarator ("=" initializer)?)*)? ";"
fn init_declarator(stream: &mut TokenStream, ctx: &mut ParseContext, base: &CType) -> Vec<Node> {
    let mut init_nodes = Vec::new();

    if stream.consume_punctuator(";").is_some() {
        return init_nodes;
    }

    loop {
        let (ident_name, ident_token) = declarator(stream, ctx, base);

        if let Some(assign_token) = stream.consume_punctuator("=") {
            // 変数定義してるのでunwrapして問題ない
            let var_kind = ctx.find_var(&ident_name).unwrap();
            let ctype = match &var_kind {
                NodeKind::LVar(_, ctype, _) | NodeKind::GVar(_, ctype) => ctype.clone(),
                _ => unreachable!(),
            };

            // 配列だったらinitializerが"{"で始まるかチェックする
            if matches!(&ctype, CType::Array(..)) {
                let state = stream.save();
                stream.expect_punctuator("{");
                stream.restore(state);
            }

            let initializer_nodes = initializer(stream, ctx, &ctype, &ident_token);

            match var_kind {
                NodeKind::LVar(..) => {
                    let new_init_nodes = set_init_val_to_lvar(
                        &ident_name,
                        ident_token,
                        ctype,
                        initializer_nodes,
                        assign_token,
                        ctx,
                    );
                    init_nodes.extend(new_init_nodes);
                }
                NodeKind::GVar(..) => set_init_val_to_gvar(&ident_name, initializer_nodes, ctx),
                _ => unreachable!(),
            }
        }

        if stream.consume_punctuator(",").is_none() {
            break;
        }
    }

    stream.expect_punctuator(";");

    init_nodes
}

// declarator := "*"* ident ("[" expr "]")*
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
        let n_node = expr(stream, ctx);

        if let Some(n) = n_node.to_isize() {
            if n <= 0 {
                error_tok!(n_node.token, "要素数が0以下の配列は定義できません");
            }
            array_sizes.push(n as usize);
        } else {
            error_tok!(n_node.token, "要素数が定数式ではありません");
        }

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

// initializer := expr | "{" initializer ("," initializer)* ","? "}"
fn initializer(
    stream: &mut TokenStream,
    ctx: &mut ParseContext,
    ctype: &CType,
    dummy_token: &Rc<Token>,
) -> Vec<Node> {
    let mut nodes = Vec::new();

    if stream.consume_punctuator("{").is_some() {
        let base = if let CType::Array(base, _) = ctype {
            *base.clone()
        } else {
            // intのような配列でない型でも括弧でくくることが
            // 許されているので、配列でなければ自分自身をbaseとして返す。
            ctype.clone()
        };

        nodes.extend(initializer(stream, ctx, &base, dummy_token));

        let mut has_trailing_comma = false;
        while stream.consume_punctuator(",").is_some() {
            if stream.consume_punctuator("}").is_some() {
                has_trailing_comma = true;
                break;
            }

            nodes.extend(initializer(stream, ctx, &base, dummy_token));
        }

        if !has_trailing_comma {
            stream.expect_punctuator("}");
        }

        // 余りを0で埋める、もしくははみ出た分を切り捨てる
        let flat_len = ctype.flat_len();
        if nodes.len() != flat_len {
            let zero = Node::new(Rc::clone(dummy_token), NodeKind::Num(0));
            nodes.resize(flat_len, zero);
        }
    } else {
        nodes.push(expr(stream, ctx));
    }

    nodes
}

fn set_init_val_to_lvar(
    ident_name: &str,
    ident_token: Rc<Token>,
    ctype: CType,
    mut initializer_nodes: Vec<Node>,
    assign_token: Rc<Token>,
    ctx: &mut ParseContext,
) -> Vec<Node> {
    let mut init_nodes = Vec::new();

    match &ctype {
        CType::Integer(_) | CType::Pointer(_) => {
            let lhs = Box::new(Node::var(ident_name, ident_token, ctx));
            let rhs = Box::new(initializer_nodes.pop().unwrap());
            let init_node = Node::new(assign_token, NodeKind::Assign(lhs, rhs));
            init_nodes.push(init_node);
        }

        // int x[2][2] = {1, 2, 3, 4}の場合
        //   ((int*)&x)[0] = 1;
        //   ((int*)&x)[1] = 2;
        //   ((int*)&x)[2] = 3;
        //   ((int*)&x)[3] = 4;
        // のようなコードを生成する。
        CType::Array(..) => {
            // 配列先頭を指すポインタを用意する
            let var_node = Node::var(ident_name, Rc::clone(&ident_token), ctx);
            let mut ptr_node =
                Node::new(Rc::clone(&ident_token), NodeKind::Addr(Box::new(var_node)));
            let base = ctype.array_base().unwrap();
            let base_ptr = CType::Pointer(Box::new(base.clone()));
            ptr_node.cast(base_ptr);

            // 配列先頭から順に各初期値を代入する
            for (i, initializer_node) in initializer_nodes.into_iter().enumerate() {
                // lhs = *(p + i)
                let index = Box::new(Node::new(
                    Rc::clone(&ident_token),
                    NodeKind::Num(i as isize),
                ));
                let p = Box::new(ptr_node.clone());
                let lhs_addr =
                    Box::new(Node::new(Rc::clone(&ident_token), NodeKind::Add(p, index)));
                let lhs = Box::new(Node::new(
                    Rc::clone(&ident_token),
                    NodeKind::Deref(lhs_addr),
                ));

                let rhs = Box::new(initializer_node);

                let init_node = Node::new(Rc::clone(&assign_token), NodeKind::Assign(lhs, rhs));
                init_nodes.push(init_node);
            }
        }
        _ => unreachable!(),
    }

    init_nodes
}

fn set_init_val_to_gvar(ident_name: &str, initializer_nodes: Vec<Node>, ctx: &mut ParseContext) {
    if ctx.set_val(&ident_name, initializer_nodes).is_err() {
        unreachable!();
    }
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

// unary := (("+" | "-" | "&" | "*" | "sizeof")? unary) | postfix
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
    } else if let Some(token) = stream.consume_keyword("sizeof") {
        let operand = unary(stream, ctx);
        Node::new(token, NodeKind::Num(operand.ctype.size() as isize))
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

// primary := "(" "{" compound_stmt ")" | "(" expr ")" | num | str | ident call_args?
fn primary(stream: &mut TokenStream, ctx: &mut ParseContext) -> Node {
    if let Some(token) = stream.consume_punctuator("(") {
        if stream.consume_punctuator("{").is_some() {
            let block = Box::new(compound_stmt(stream, ctx));
            stream.expect_punctuator(")");
            Node::new(token, NodeKind::StmtExpr(block))
        } else {
            let node = expr(stream, ctx);
            stream.expect_punctuator(")");
            node
        }
    } else if let Some((token, n)) = stream.consume_number() {
        Node::new(token, NodeKind::Num(n))
    } else if let Some((token, s)) = stream.consume_string() {
        let (label, ctype) = ctx.add_str(s);
        Node::new(token, NodeKind::GVar(label, ctype))
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
