use std::fmt;

use super::parse::{AdditionalInfo, Node, NodeKind};

struct Context {
    fname: String,
    label: usize,
    stack: usize,
}

impl Context {
    fn new() -> Self {
        Self {
            fname: String::new(),
            label: 0,
            stack: 0,
        }
    }
}

#[derive(Clone, Copy)]
enum Register {
    RAX,
    RDI,
    RBP,
    RSI,
    RDX,
    RCX,
    R8,
    R9,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RAX => write!(f, "rax"),
            Self::RDI => write!(f, "rdi"),
            Self::RBP => write!(f, "rbp"),
            Self::RSI => write!(f, "rsi"),
            Self::RDX => write!(f, "rdx"),
            Self::RCX => write!(f, "rcx"),
            Self::R8 => write!(f, "r8"),
            Self::R9 => write!(f, "r9"),
        }
    }
}

fn push(reg: Register, ctx: &mut Context) {
    ctx.stack += 1;
    println!("        push %{}", reg);
}

fn pop(reg: Register, ctx: &mut Context) {
    ctx.stack -= 1;
    println!("        pop %{}", reg);
}

// 左辺の結果をraxに、右辺の結果をrdiにセットする
fn gen_binary_operator(lhs: &Node, rhs: &Node, ctx: &mut Context) {
    gen(rhs, ctx);
    push(Register::RAX, ctx);
    gen(lhs, ctx);
    pop(Register::RDI, ctx);
}

// 変数のアドレスをraxにmovする
fn gen_lval(node: &Node, ctx: &mut Context) {
    match &node.kind {
        NodeKind::LVar(_, _, offset) => {
            println!("        mov %rbp, %rax");
            println!("        sub ${}, %rax", offset);
        }
        NodeKind::Deref(operand) => {
            gen(operand, ctx);
        }
        _ => {
            error_tok!(node.token, "代入の左辺値が変数ではありません");
        }
    }
}

fn gen(node: &Node, ctx: &mut Context) {
    match &node.kind {
        NodeKind::Defun(..) => {
            error_tok!(node.token, "関数内で関数定義はできません");
        }
        NodeKind::Block(nodes) => {
            for node in nodes {
                gen(node, ctx);
            }
        }
        NodeKind::If(cond_node, then_node, else_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(cond_node, ctx);
            // 0が偽、0以外は真なので0と比較する
            println!("        cmp $0, %rax");

            // 0だったら偽としてelse節にジャンプする
            println!("        je .Lelse{}", label);

            gen(then_node, ctx);
            // then節が終わったらif文の終わりにジャンプ
            println!("        jmp .Lend{}", label);

            println!(".Lelse{}:", label);

            gen(else_node, ctx);

            println!(".Lend{}:", label);
        }
        NodeKind::For(init_node, cond_node, update_node, body_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(init_node, ctx);

            println!(".Lbegin{}:", label);

            gen(cond_node, ctx);
            // 0が偽、0以外は真なので0と比較する
            println!("        cmp $0, %rax");
            println!("        je .Lend{}", label);

            gen(body_node, ctx);

            gen(update_node, ctx);

            println!("        jmp .Lbegin{}", label);
            println!(".Lend{}:", label);
        }
        NodeKind::Return(child) => {
            gen(child, ctx);
            println!("        jmp .L{}__return", &ctx.fname);
        }
        NodeKind::Assign(lhs, rhs) => {
            gen(rhs, ctx);
            push(Register::RAX, ctx);
            gen_lval(lhs, ctx);
            pop(Register::RDI, ctx);
            println!("        mov %rdi, (%rax)");
            println!("        mov %rdi, %rax");
        }
        NodeKind::Eq(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp %rdi, %rax");
            println!("        sete %al");
            println!("        movzb %al, %rax");
        }
        NodeKind::Neq(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp %rdi, %rax");
            println!("        setne %al");
            println!("        movzb %al, %rax");
        }
        NodeKind::LT(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp %rdi, %rax");
            println!("        setl %al");
            println!("        movzb %al, %rax");
        }
        NodeKind::LTE(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cmp %rdi, %rax");
            println!("        setle %al");
            println!("        movzb %al, %rax");
        }
        NodeKind::Add(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        add %rdi, %rax");
        }
        NodeKind::Sub(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        sub %rdi, %rax");
        }
        NodeKind::Mul(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        imul %rdi, %rax");
        }
        NodeKind::Div(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            println!("        cqo");
            println!("        idiv %rdi");
        }
        NodeKind::Addr(operand) => {
            gen_lval(operand, ctx);
        }
        NodeKind::Deref(_) => {
            gen_lval(node, ctx);
            println!("        mov (%rax), %rax");
        }
        NodeKind::Num(n) => {
            println!("        mov ${}, %rax", n);
        }
        NodeKind::LVar(..) => {
            gen_lval(node, ctx);
            println!("        mov (%rax), %rax");
        }
        NodeKind::Call(name, args) => {
            // 関数呼び出しの際に引数をセットするレジスタ
            // RDIから順に第1引数, 第2引数, ..., 第6引数と並んでいる
            let arg_reg = vec![
                Register::RDI,
                Register::RSI,
                Register::RDX,
                Register::RCX,
                Register::R8,
                Register::R9,
            ];

            // 関数呼び出しの引数をスタックに積む
            for arg in args {
                gen(arg, ctx);
                push(Register::RAX, ctx);
            }

            // x86-64の呼び出し規約に従いレジスタに引数をセットする
            for reg in arg_reg.iter().take(args.len()).rev() {
                pop(*reg, ctx);
            }

            // x86-64では関数を呼び出す時はRSPが16の倍数でなければならない。
            // 関数呼び出しの際は呼び出し元アドレスがスタックに積まれるため
            // プッシュした回数が偶数ならば、RSPを調整する必要がある。
            let needs_align_rsp = ctx.stack % 2 == 0;

            if needs_align_rsp {
                println!("        sub $8, %rsp");
            }

            // RAXには利用するSSEレジスタの数を入れる
            // 浮動小数点型はサポートしないので0
            println!("        mov $0, %rax");

            println!("        call {}", name);

            if needs_align_rsp {
                println!("        add $8, %rsp");
            }
        }
    }
}

fn prologue(name: &str, mut stack_size: usize, ctx: &mut Context) {
    ctx.fname = name.to_string();
    println!("{}:", name);

    // 関数を呼ぶ時のRSPのアライメントをしやすくするために
    // スタックサイズを16の倍数にする。
    stack_size = (stack_size + 16 - 1) / 16 * 16;

    push(Register::RBP, ctx);
    println!("        mov %rsp, %rbp");
    println!("        sub ${}, %rsp", stack_size);
}

fn epilogue(ctx: &mut Context) {
    println!(".L{}__return:", &ctx.fname);
    println!("        mov %rbp, %rsp");
    pop(Register::RBP, ctx);
    println!("        ret");
}

pub fn codegen(nodes: &[Node], add_info: &AdditionalInfo) {
    println!(".global main");

    let mut ctx = Context::new();

    for node in nodes {
        if let NodeKind::Defun(name, body) = &node.kind {
            // ローカル変数はRBPからのオフセット順に並んでいるので
            // 最後の要素のオフセットがスタックサイズとなる
            let stack_size = if let Some(lvar) = add_info
                .find_fn(name)
                .expect("関数情報が見つかりません")
                .lvars
                .last()
            {
                lvar.offset
            } else {
                0
            };
            prologue(name, stack_size, &mut ctx);

            gen(body, &mut ctx);

            epilogue(&mut ctx);
        } else {
            error_tok!(node.token, "トップレベルでは関数定義のみできます");
        }
    }
}
