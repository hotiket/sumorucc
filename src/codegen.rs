use super::parse::{AdditionalInfo, Node};

struct Context {
    label: usize,
}

// 左辺の結果をraxに、右辺の結果をrdiにセットする
fn gen_binary_operator(lhs: &Node, rhs: &Node, ctx: &mut Context) {
    gen(lhs, ctx);
    gen(rhs, ctx);
    println!("        pop rdi");
    println!("        pop rax");
}

// 変数のアドレスをスタックにプッシュする
fn gen_lval(node: &Node) {
    if let Node::LVAR(offset) = node {
        println!("        mov rax, rbp");
        println!("        sub rax, {}", offset);
        println!("        push rax");
    } else {
        error!("代入の左辺値が変数ではありません");
    }
}

fn gen(node: &Node, ctx: &mut Context) {
    // NOTE: gen呼び出し元でスタックをポップするので
    //       なんらかの値を必ずプッシュしておく必要がある
    //       式だけでなく、if文などの制御構文もプッシュすること

    match node {
        Node::IF(cond_node, then_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(cond_node, ctx);
            // 条件式の結果を判定用にポップしておく
            println!("        pop rax");
            // 0だったら偽としてif文の終わりにジャンプする
            println!("        cmp rax, 0");
            println!("        je .Lend{}", label);

            gen(then_node, ctx);
            // .Lendのあとでスタックにプッシュするので
            // if文全体として一つの値がプッシュされるように
            // thenの結果はポップしておく
            println!("        pop rax");

            println!(".Lend{}:", label);

            // なにかしらの値をスタックに積む必要があるので
            // returnせずにgenの最後でプッシュする
        }
        Node::RETURN(child) => {
            gen(child, ctx);
            // childの結果を戻り値としてポップしておく
            println!("        pop rax");
            epilogue();
            return;
        }
        Node::ASSIGN(lhs, rhs) => {
            gen_lval(lhs);
            gen(rhs, ctx);
            println!("        pop rdi");
            println!("        pop rax");
            println!("        mov [rax], rdi");
            println!("        push rdi");
            return;
        }
        Node::EQ(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp rax, rdi");
            println!("        sete al");
            println!("        movzb rax, al");
        }
        Node::NEQ(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp rax, rdi");
            println!("        setne al");
            println!("        movzb rax, al");
        }
        Node::LT(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp rax, rdi");
            println!("        setl al");
            println!("        movzb rax, al");
        }
        Node::LTE(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp rax, rdi");
            println!("        setle al");
            println!("        movzb rax, al");
        }
        Node::ADD(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        add rax, rdi");
        }
        Node::SUB(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        sub rax, rdi");
        }
        Node::MUL(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        imul rax, rdi");
        }
        Node::DIV(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cqo");
            println!("        idiv rdi");
        }
        Node::NUM(n) => {
            println!("        push {}", n);
            return;
        }
        Node::LVAR(_) => {
            gen_lval(node);
            println!("        pop rax");
            println!("        mov rax, [rax]");
            println!("        push rax");
            return;
        }
    }

    println!("        push rax");
}

fn prologue(stack_size: usize) {
    println!("        push rbp");
    println!("        mov rbp, rsp");
    println!("        sub rsp, {}", stack_size);
}

fn epilogue() {
    println!("        mov rsp, rbp");
    println!("        pop rbp");
    println!("        ret");
}

pub fn codegen(nodes: &[Node], add_info: &AdditionalInfo) {
    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // ローカル変数はRBPからのオフセット順に並んでいるので
    // 最後の要素のオフセットがスタックサイズとなる
    let stack_size = if let Some(lvar) = add_info.lvars.last() {
        lvar.offset
    } else {
        0
    };

    prologue(stack_size);

    let mut ctx = Context { label: 0 };

    for node in nodes {
        // 抽象構文木を下りながらコード生成
        gen(node, &mut ctx);

        // スタックトップに式全体の値が残っているはずなので
        // スタックが溢れないようにポップしておく
        println!("        pop rax");
    }

    epilogue();
}
