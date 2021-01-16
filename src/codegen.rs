use super::parse::{AdditionalInfo, Node};

struct Context {
    label: usize,
}

enum Register {
    RDI,
    RBP,
}

fn push() {
    println!("        push rax");
}

fn pop(reg: Register) {
    let s = match reg {
        Register::RDI => "rdi",
        Register::RBP => "rbp",
    };

    println!("        pop {}", s);
}

// 左辺の結果をraxに、右辺の結果をrdiにセットする
fn gen_binary_operator(lhs: &Node, rhs: &Node, ctx: &mut Context) {
    gen(rhs, ctx);
    push();
    gen(lhs, ctx);
    pop(Register::RDI);
}

// 変数のアドレスをraxにmovする
fn gen_lval(node: &Node) {
    if let Node::LVar(offset) = node {
        println!("        mov rax, rbp");
        println!("        sub rax, {}", offset);
    } else {
        error!("代入の左辺値が変数ではありません");
    }
}

fn gen(node: &Node, ctx: &mut Context) {
    match node {
        Node::Block(nodes) => {
            for node in nodes {
                gen(node, ctx);
            }
        }
        Node::If(cond_node, then_node, else_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(cond_node, ctx);
            // 0が偽、0以外は真なので0と比較する
            println!("        cmp rax, 0");

            // 0だったら偽としてelse節にジャンプする
            println!("        je .Lelse{}", label);

            gen(then_node, ctx);
            // then節が終わったらif文の終わりにジャンプ
            println!("        jmp .Lend{}", label);

            println!(".Lelse{}:", label);

            gen(else_node, ctx);

            println!(".Lend{}:", label);
        }
        Node::For(init_node, cond_node, update_node, body_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(init_node, ctx);

            println!(".Lbegin{}:", label);

            gen(cond_node, ctx);
            // 0が偽、0以外は真なので0と比較する
            println!("        cmp rax, 0");
            println!("        je .Lend{}", label);

            gen(body_node, ctx);

            gen(update_node, ctx);

            println!("        jmp .Lbegin{}", label);
            println!(".Lend{}:", label);
        }
        Node::Return(child) => {
            gen(child, ctx);
            epilogue();
        }
        Node::Assign(lhs, rhs) => {
            gen(rhs, ctx);
            push();
            gen_lval(lhs);
            pop(Register::RDI);
            println!("        mov [rax], rdi");
            println!("        mov rax, rdi");
        }
        Node::Eq(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp rax, rdi");
            println!("        sete al");
            println!("        movzb rax, al");
        }
        Node::Neq(lhs, rhs) => {
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
        Node::Add(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        add rax, rdi");
        }
        Node::Sub(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        sub rax, rdi");
        }
        Node::Mul(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        imul rax, rdi");
        }
        Node::Div(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cqo");
            println!("        idiv rdi");
        }
        Node::Num(n) => {
            println!("        mov rax, {}", n);
        }
        Node::LVar(_) => {
            gen_lval(node);
            println!("        mov rax, [rax]");
        }
    }
}

fn prologue(stack_size: usize) {
    println!("        push rbp");
    println!("        mov rbp, rsp");
    println!("        sub rsp, {}", stack_size);
}

fn epilogue() {
    println!("        mov rsp, rbp");
    pop(Register::RBP);
    println!("        ret");
}

pub fn codegen(nodes: &[Node], add_info: &AdditionalInfo) {
    if nodes.is_empty() || nodes.len() > 1 || !matches!(nodes[0], Node::Block(_)) {
        error!("プログラムはブロックで囲まれている必要があります");
    }

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
    gen(&nodes[0], &mut ctx);

    epilogue();
}
