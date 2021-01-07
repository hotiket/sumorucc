use super::parse::Node;

// 左辺の結果をraxに、右辺の結果をrdiにセットする
fn gen_binary_operator(lhs: &Node, rhs: &Node) {
    gen(lhs);
    gen(rhs);
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

fn gen(node: &Node) {
    match node {
        Node::ASSIGN(lhs, rhs) => {
            gen_lval(lhs);
            gen(rhs);
            println!("        pop rdi");
            println!("        pop rax");
            println!("        mov [rax], rdi");
            println!("        push rdi");
            return;
        }
        Node::EQ(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        cmp rax, rdi");
            println!("        sete al");
            println!("        movzb rax, al");
        }
        Node::NEQ(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        cmp rax, rdi");
            println!("        setne al");
            println!("        movzb rax, al");
        }
        Node::LT(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        cmp rax, rdi");
            println!("        setl al");
            println!("        movzb rax, al");
        }
        Node::LTE(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        cmp rax, rdi");
            println!("        setle al");
            println!("        movzb rax, al");
        }
        Node::ADD(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        add rax, rdi");
        }
        Node::SUB(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        sub rax, rdi");
        }
        Node::MUL(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
            println!("        imul rax, rdi");
        }
        Node::DIV(lhs, rhs) => {
            gen_binary_operator(lhs, rhs);
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

pub fn codegen(nodes: &[Node]) {
    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    prologue(8 * 26);

    for node in nodes {
        // 抽象構文木を下りながらコード生成
        gen(node);

        // スタックトップに式全体の値が残っているはずなので
        // スタックが溢れないようにポップしておく
        println!("        pop rax");
    }

    epilogue();
}
