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

        let (line, row, at) = get_error_line(&$src.code, $at);
        let path_row = match $src.path {
            Some(ref p) => format!("{}:{}: ", p, row + 1),
            None => format!("-:{}: ", row + 1),
        };
        let at = at + path_row.chars().count();

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
        error_at!($tok.common.src, $tok.common.pos, $fmt);
    };
    ($tok:expr, $fmt:expr, $($arg:tt)*) => {
        error_at!($tok.common.src, $tok.common.pos, $fmt, $($arg)*);
    };
}

pub fn get_error_line(src: &str, pos: usize) -> (String, usize, usize) {
    let mut line = String::new();
    let mut row = 0;
    let mut col = 0;

    for (i, c) in src.chars().enumerate() {
        if c == '\n' {
            if i >= pos {
                break;
            }

            line.clear();
            row += 1;
            col = 0;

            continue;
        }

        let nr_col = if c == '\t' {
            // タブはスペース4つに変換する
            line.push_str("    ");
            4
        } else {
            line.push(c);
            1
        };

        if i < pos {
            col += nr_col;
        }
    }

    (line, row, col)
}
