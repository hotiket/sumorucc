use super::parse::Node;

fn gen_binary_operator(lhs: &Node, rhs: &Node) {
    gen(lhs);
    gen(rhs);
    println!("        pop rdi");
    println!("        pop rax");
}

fn gen(node: &Node) {
    match node {
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
    }

    println!("        push rax");
}

pub fn codegen(node: &Node) {
    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // 抽象構文木を下りながらコード生成
    gen(&node);

    // スタックトップに式全体の値が残っているはずなので
    // それをRAXにロードして関数からの返り値とする
    println!("        pop rax");
    println!("        ret");
}
