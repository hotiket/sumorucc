use std::fmt;

use super::ctype::{CType, Integer};
use super::node::{Node, NodeKind};
use super::parse_context::{GVar, ParseContext};

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

    // 下位8bit
    AL,
    DIL,
    BPL,
    SIL,
    DL,
    CL,
    R8B,
    R9B,
}

// 関数呼び出しの際に引数をセットするレジスタ(64bit)
// RDIから順に第1引数, 第2引数, ..., 第6引数と並んでいる
static ARG_REG64: [Register; 6] = [
    Register::RDI,
    Register::RSI,
    Register::RDX,
    Register::RCX,
    Register::R8,
    Register::R9,
];

// 関数呼び出しの際に引数をセットするレジスタ(8bit)
static ARG_REG8: [Register; 6] = [
    Register::DIL,
    Register::SIL,
    Register::DL,
    Register::CL,
    Register::R8B,
    Register::R9B,
];

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

            Self::AL => write!(f, "al"),
            Self::DIL => write!(f, "dil"),
            Self::BPL => write!(f, "bpl"),
            Self::SIL => write!(f, "sil"),
            Self::DL => write!(f, "dl"),
            Self::CL => write!(f, "cl"),
            Self::R8B => write!(f, "r8b"),
            Self::R9B => write!(f, "r9b"),
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

// 変数のアドレスをraxにセットする
fn gen_lval(node: &Node, ctx: &mut Context) {
    match &node.kind {
        NodeKind::LVar(_, _, offset) => {
            println!("        lea -{}(%rbp), %rax", offset);
        }
        NodeKind::GVar(name, _) => {
            println!("        lea {}(%rip), %rax", name);
        }
        NodeKind::Deref(operand) => {
            gen(operand, ctx);
        }
        _ => {
            error_tok!(node.token, "代入の左辺値が変数ではありません");
        }
    }
}

// raxが指すアドレスの値をraxにセットする
fn gen_load(ctype: &CType) {
    match ctype {
        CType::Integer(Integer::Char) => println!("        movsbq (%rax), %rax"),
        CType::Integer(Integer::Int) => println!("        mov (%rax), %rax"),
        CType::Pointer(_) => println!("        mov (%rax), %rax"),
        _ => unreachable!(),
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
            match &lhs.ctype {
                CType::Integer(Integer::Char) => {
                    println!("        mov %dil, (%rax)");
                    println!("        movsbq %dil, %rax");
                }
                _ => {
                    println!("        mov %rdi, (%rax)");
                    println!("        mov %rdi, %rax");
                }
            }
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
        NodeKind::Deref(operand) => {
            gen_lval(node, ctx);
            let base = operand.ctype.base().unwrap();
            gen_load(&base);
        }
        NodeKind::Num(n) => {
            println!("        mov ${}, %rax", n);
        }
        NodeKind::LVar(_, ref ctype, _) | NodeKind::GVar(_, ref ctype) => {
            gen_lval(node, ctx);
            gen_load(ctype);
        }
        NodeKind::Call(name, args) => {
            // 関数呼び出しの引数をスタックに積む
            for arg in args {
                gen(arg, ctx);
                push(Register::RAX, ctx);
            }

            // x86-64の呼び出し規約に従いレジスタに引数をセットする
            for reg in ARG_REG64.iter().take(args.len()).rev() {
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

fn gen_gvar(gvar: &GVar) {
    println!("        .data");
    println!("        .globl {}", gvar.name);
    println!("{}:", gvar.name);

    if let Some(val) = &gvar.val {
        match &gvar.ctype {
            CType::Integer(..) | CType::Pointer(..) => {
                let size = ctype_to_data_directive(&gvar.ctype);
                let val = val.first().unwrap();
                gen_init_val(val, size);
            }
            CType::Array(..) => {
                let base = gvar.ctype.array_base().unwrap();
                let size = ctype_to_data_directive(&base);

                for val in val.iter() {
                    gen_init_val(val, size);
                }
            }
            _ => unreachable!(),
        }
    } else {
        println!("        .zero {}", gvar.ctype.size());
    }
}

fn gen_init_val(val: &Node, size: &str) {
    let n = val.to_isize();
    if n.is_none() {
        error_tok!(val.token, "初期値が定数式ではありません");
    }
    println!("        {} {}", size, n.unwrap());
}

fn ctype_to_data_directive(ctype: &CType) -> &str {
    match ctype.size() {
        1 => ".byte",
        8 => ".quad",
        _ => unreachable!(),
    }
}

fn function_header(name: &str, ctx: &mut Context) {
    ctx.fname = name.to_string();
    println!("        .text");
    println!("        .globl {}", name);
    println!("{}:", name);
}

fn prologue(mut stack_size: usize, params: &[(usize, CType)], ctx: &mut Context) {
    // 関数を呼ぶ時のRSPのアライメントをしやすくするために
    // スタックサイズを16の倍数にする。
    stack_size = (stack_size + 16 - 1) / 16 * 16;

    push(Register::RBP, ctx);
    println!("        mov %rsp, %rbp");
    println!("        sub ${}, %rsp", stack_size);

    // x86-64の呼び出し規約に従い引数をレジスタから
    // スタック上のローカル変数にセットする。
    let iter = params.iter().zip(ARG_REG8.iter()).zip(ARG_REG64.iter());
    for (((offset, ctype), reg8), reg64) in iter {
        println!("        mov %rbp, %rax");
        println!("        sub ${}, %rax", offset);
        match ctype.size() {
            1 => println!("        movb %{}, (%rax)", reg8),
            8 => println!("        mov %{}, (%rax)", reg64),
            _ => unreachable!(),
        }
    }
}

fn epilogue(ctx: &mut Context) {
    println!(".L{}__return:", &ctx.fname);
    println!("        mov %rbp, %rsp");
    pop(Register::RBP, ctx);
    println!("        ret");
}

pub fn codegen(nodes: &[Node], parse_ctx: &ParseContext) {
    let mut ctx = Context::new();

    // グローバル変数をdataセクションに出力
    parse_ctx.gvars.iter().for_each(gen_gvar);

    // グローバル関数をtextセクションに出力
    for node in nodes {
        if let NodeKind::Defun(name, params, body) = &node.kind {
            let stack_size = parse_ctx
                .stack_size(name)
                .expect("関数情報が見つかりません");

            function_header(name, &mut ctx);

            prologue(stack_size, params, &mut ctx);

            gen(body, &mut ctx);

            epilogue(&mut ctx);
        } else {
            error_tok!(node.token, "トップレベルでは関数定義のみできます");
        }
    }
}
