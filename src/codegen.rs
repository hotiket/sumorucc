use std::fmt;

use super::ctype::{CType, Integer};
use super::node::{Node, NodeKind};
use super::parse_context::{GVar, ParseContext, Str};
use super::util::align_to;

macro_rules! code {
    ($fmt:expr) => {
        println!(concat!("        ", $fmt));
    };
    ($fmt:expr, $($arg:tt)*) => {
        println!(concat!("        ", $fmt), $($arg)*);
    };
}

macro_rules! label {
    ($fmt:expr) => {
        println!(concat!($fmt, ":"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        println!(concat!($fmt, ":"), $($arg)*);
    };
}

struct Debug {
    file: Vec<(String, usize)>,
}

impl Debug {
    fn new() -> Self {
        Self { file: Vec::new() }
    }

    fn find_file(&self, file: &str) -> Option<usize> {
        match self.file.iter().find(|f| f.0 == file) {
            Some(f) => Some(f.1),
            None => None,
        }
    }

    fn add_file(&mut self, file: &str) -> Result<usize, ()> {
        if self.find_file(file).is_some() {
            return Err(());
        }

        let fileno = if self.file.is_empty() {
            // filenoは正の整数なので1から始める
            1
        } else {
            self.file.last().unwrap().1 + 1
        };
        self.file.push((file.to_string(), fileno));

        Ok(fileno)
    }
}

struct Context {
    fname: String,
    label: usize,
    stack: usize,
    debug: Debug,
}

impl Context {
    fn new() -> Self {
        Self {
            fname: String::new(),
            label: 0,
            stack: 0,
            debug: Debug::new(),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
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
    #[allow(dead_code)]
    AL,
    DIL,
    #[allow(dead_code)]
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
            Self::RAX => write!(f, "%rax"),
            Self::RDI => write!(f, "%rdi"),
            Self::RBP => write!(f, "%rbp"),
            Self::RSI => write!(f, "%rsi"),
            Self::RDX => write!(f, "%rdx"),
            Self::RCX => write!(f, "%rcx"),
            Self::R8 => write!(f, "%r8"),
            Self::R9 => write!(f, "%r9"),

            Self::AL => write!(f, "%al"),
            Self::DIL => write!(f, "%dil"),
            Self::BPL => write!(f, "%bpl"),
            Self::SIL => write!(f, "%sil"),
            Self::DL => write!(f, "%dl"),
            Self::CL => write!(f, "%cl"),
            Self::R8B => write!(f, "%r8b"),
            Self::R9B => write!(f, "%r9b"),
        }
    }
}

fn push(reg: Register, ctx: &mut Context) {
    ctx.stack += 1;
    code!("push {}", reg);
}

fn pop(reg: Register, ctx: &mut Context) {
    ctx.stack -= 1;
    code!("pop {}", reg);
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
            code!("lea -{}(%rbp), %rax", offset);
        }
        NodeKind::GVar(name, _) => {
            code!("lea {}(%rip), %rax", name);
        }
        NodeKind::Deref(operand) => {
            gen(operand, ctx);
        }
        NodeKind::Member(base, offset) => {
            gen_lval(base, ctx);
            code!("add ${}, %rax", offset);
        }
        _ => {
            error_tok!(node.token, "代入の左辺値が変数ではありません");
        }
    }
}

// raxが指すアドレスの値をraxにセットする
fn gen_load(ctype: &CType) {
    match ctype {
        CType::Integer(Integer::Char) => code!("movsbq (%rax), %rax"),
        CType::Integer(Integer::Int) => code!("mov (%rax), %rax"),
        CType::Pointer(_) => code!("mov (%rax), %rax"),
        // 値がraxに入りきる保障が無い型はなにもせず
        // gen_load呼び出し元で個別に対応する。
        CType::Array(..) | CType::Struct(..) | CType::Union(..) => (),
        _ => unreachable!(),
    }
}

// デバッグ用にソース位置情報を出力
fn gen_loc(node: &Node, ctx: &mut Context) {
    let filename = match &node.token.common.src.path {
        Some(path) => &path,
        None => "<stdin>",
    };

    let fileno = match ctx.debug.find_file(filename) {
        Some(fileno) => fileno,
        None => {
            let fileno = ctx.debug.add_file(filename).unwrap();
            code!(".file {} \"{}\"", fileno, filename);
            fileno
        }
    };

    let lineno = node.token.common.loc.row + 1;

    code!(".loc {} {}", fileno, lineno);
}

fn gen(node: &Node, ctx: &mut Context) {
    gen_loc(node, ctx);

    match &node.kind {
        NodeKind::Defun(..) => {
            error_tok!(node.token, "関数内で関数定義はできません");
        }
        NodeKind::Block(nodes) => {
            for node in nodes {
                gen(node, ctx);
            }
        }
        NodeKind::StmtExpr(block) => {
            gen(block, ctx);
        }
        NodeKind::If(cond_node, then_node, else_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(cond_node, ctx);
            // 0が偽、0以外は真なので0と比較する
            code!("cmp $0, %rax");

            // 0だったら偽としてelse節にジャンプする
            code!("je .Lelse{}", label);

            gen(then_node, ctx);
            // then節が終わったらif文の終わりにジャンプ
            code!("jmp .Lend{}", label);

            label!(".Lelse{}", label);

            gen(else_node, ctx);

            label!(".Lend{}", label);
        }
        NodeKind::For(init_node, cond_node, update_node, body_node) => {
            let label = ctx.label;
            ctx.label += 1;

            gen(init_node, ctx);

            label!(".Lbegin{}", label);

            gen(cond_node, ctx);
            // 0が偽、0以外は真なので0と比較する
            code!("cmp $0, %rax");
            code!("je .Lend{}", label);

            gen(body_node, ctx);

            gen(update_node, ctx);

            code!("jmp .Lbegin{}", label);
            label!(".Lend{}", label);
        }
        NodeKind::Return(child) => {
            gen(child, ctx);
            code!("jmp .L{}__return", &ctx.fname);
        }
        NodeKind::Assign(lhs, rhs) => {
            gen(rhs, ctx);
            push(Register::RAX, ctx);
            gen_lval(lhs, ctx);
            pop(Register::RDI, ctx);

            match &lhs.ctype {
                CType::Integer(Integer::Char) => {
                    code!("mov %dil, (%rax)");
                    code!("movsbq %dil, %rax");
                }
                CType::Struct(..) | CType::Union(..) => {
                    for i in 0..lhs.ctype.size() {
                        code!("movb {}(%rdi), %sil", i);
                        code!("movb %sil, {}(%rax)", i);
                    }
                }
                _ => {
                    code!("mov %rdi, (%rax)");
                    code!("mov %rdi, %rax");
                }
            }
        }
        NodeKind::Eq(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("cmp %rdi, %rax");
            code!("sete %al");
            code!("movzb %al, %rax");
        }
        NodeKind::Neq(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("cmp %rdi, %rax");
            code!("setne %al");
            code!("movzb %al, %rax");
        }
        NodeKind::LT(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("cmp %rdi, %rax");
            code!("setl %al");
            code!("movzb %al, %rax");
        }
        NodeKind::LTE(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("cmp %rdi, %rax");
            code!("setle %al");
            code!("movzb %al, %rax");
        }
        NodeKind::Add(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("add %rdi, %rax");
        }
        NodeKind::Sub(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("sub %rdi, %rax");
        }
        NodeKind::Mul(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("imul %rdi, %rax");
        }
        NodeKind::Div(lhs, rhs) => {
            gen_binary_operator(lhs, rhs, ctx);
            code!("cqo");
            code!("idiv %rdi");
        }
        NodeKind::Addr(operand) => {
            gen_lval(operand, ctx);
        }
        NodeKind::Deref(operand) => {
            gen_lval(node, ctx);
            let base = operand.ctype.base().unwrap();
            gen_load(&base);
        }
        NodeKind::Member(base, offset) => {
            gen_lval(base, ctx);
            code!("add ${}, %rax", offset);
            gen_load(&node.ctype);
        }
        NodeKind::Num(n) => {
            code!("mov ${}, %rax", n);
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
                code!("sub $8, %rsp");
            }

            // RAXには利用するSSEレジスタの数を入れる
            // 浮動小数点型はサポートしないので0
            code!("mov $0, %rax");

            code!("call {}", name);

            if needs_align_rsp {
                code!("add $8, %rsp");
            }
        }
    }
}

fn gen_str(string: &Str) {
    code!(".section .rodata");
    label!("{}", string.label);
    for b in string.val.iter() {
        code!(".byte 0x{:02x}", b);
    }
}

fn gen_gvar(gvar: &GVar) {
    code!(".data");
    code!(".globl {}", gvar.name);
    label!("{}", gvar.name);

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
        code!(".zero {}", gvar.ctype.size());
    }
}

fn gen_init_val(val: &Node, size: &str) {
    match &val.kind {
        NodeKind::GVar(ref name, ..) => {
            code!("{} {}", size, name);
        }
        _ => {
            let n = val.to_isize();
            if n.is_none() {
                error_tok!(val.token, "初期値が定数式ではありません");
            }
            code!("{} {}", size, n.unwrap());
        }
    }
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
    code!(".text");
    code!(".globl {}", name);
    label!("{}", name);
}

fn prologue(mut stack_size: usize, params: &[(usize, CType)], ctx: &mut Context) {
    // 関数を呼ぶ時のRSPのアライメントをしやすくするために
    // スタックサイズを16の倍数にする。
    stack_size = align_to(stack_size, 16);

    push(Register::RBP, ctx);
    code!("mov %rsp, %rbp");
    code!("sub ${}, %rsp", stack_size);

    // x86-64の呼び出し規約に従い引数をレジスタから
    // スタック上のローカル変数にセットする。
    let iter = params.iter().zip(ARG_REG8.iter()).zip(ARG_REG64.iter());
    for (((offset, ctype), reg8), reg64) in iter {
        code!("mov %rbp, %rax");
        code!("sub ${}, %rax", offset);
        match ctype.size() {
            1 => code!("movb {}, (%rax)", reg8),
            8 => code!("mov {}, (%rax)", reg64),
            _ => unreachable!(),
        }
    }
}

fn epilogue(ctx: &mut Context) {
    label!(".L{}__return", &ctx.fname);
    code!("mov %rbp, %rsp");
    pop(Register::RBP, ctx);
    code!("ret");
}

pub fn codegen(nodes: &[Node], parse_ctx: &ParseContext) {
    let mut ctx = Context::new();

    // 文字列をrodataセクションに出力
    parse_ctx.strs.iter().for_each(gen_str);

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
