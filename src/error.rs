use super::tokenize::Loc;

macro_rules! error {
    ($fmt:expr) => {
        eprintln!($fmt);
        std::process::exit(1);
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!($fmt, $($arg)*);
        std::process::exit(1);
    };
}

macro_rules! error_at_impl {
    ($src:expr, $at:expr, $msg:expr) => {
        use crate::error::get_error_line;

        let (line, corr) = get_error_line(&$src.code, $at);
        let path_row = match $src.path {
            Some(ref p) => format!("{}:{}: ", p, $at.row + 1),
            None => format!("-:{}: ", $at.row + 1),
        };
        let at = $at.col + corr + path_row.chars().count();

        eprintln!("{}{}", path_row, line);
        eprint!("{}^ ", " ".repeat(at));
        eprintln!("{}", $msg);
        std::process::exit(1);
    };
}

macro_rules! error_at {
    ($src:expr, $at:expr, $fmt:expr) => {
        error_at_impl!($src, $at, $fmt);
    };
    ($src:expr, $at:expr, $fmt:expr, $($arg:tt)*) => {
        let msg = format!($fmt, $($arg)*);
        error_at_impl!($src, $at, msg);
    };
}

macro_rules! error_tok {
    ($tok:expr, $fmt:expr) => {
        error_at!($tok.common.src, $tok.common.loc, $fmt);
    };
    ($tok:expr, $fmt:expr, $($arg:tt)*) => {
        error_at!($tok.common.src, $tok.common.loc, $fmt, $($arg)*);
    };
}

pub fn get_error_line(src: &str, loc: Loc) -> (String, usize) {
    let mut line = String::new();
    let mut cur_row = 0;
    let mut cur_col = 0;
    let mut correction = 0;

    for c in src.chars() {
        if cur_row > loc.row {
            break;
        }

        if c == '\n' {
            cur_row += 1;
            continue;
        }

        if cur_row == loc.row {
            cur_col += 1;
            if c == '\t' {
                // タブはスペース4つに変換する
                line.push_str("    ");

                // タブをスペースに変換した分colに加算する
                if cur_col < loc.col {
                    correction += 3;
                }
            } else {
                line.push(c);
            };
        }
    }

    (line, correction)
}
