use std::env;
use std::process::exit;

// #[derive(PartialEq)]
enum Operator {
    MOV,
    ADD,
    SUB,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        exit(1);
    }

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    let expr: Vec<char> = args[1].chars().collect();
    let expr_len = expr.len();

    let mut num_string = String::from("");
    let mut operator = Operator::MOV;

    for i in 0..expr_len {
        let c = expr[i];
        match c {
            '0'..='9' => {
                num_string.push(c);

                let output_asm = if i == expr_len - 1 {
                    true
                } else {
                    let next_c = expr[i+1];
                    !('0'..='9').contains(&next_c)
                };

                if output_asm {
                    let n = num_string.parse::<isize>().unwrap();
                    num_string = String::from("");

                    match operator {
                        Operator::MOV => println!("        mov rax, {}", n),
                        Operator::ADD => println!("        add rax, {}", n),
                        Operator::SUB => println!("        sub rax, {}", n),
                    }
                }
            },

            '+' => operator = Operator::ADD,
            '-' => operator = Operator::SUB,

            _ => {
                eprintln!("予期しない文字です: {}", c);
                exit(1);
            },
        }
    }

    println!("        ret");
}
