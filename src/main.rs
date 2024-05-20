use {
    anyhow::{anyhow, Error},
    argh::FromArgs,
    culpa::{throw, throws},
    liso::{liso, OutputOnly, Response},
    std::sync::OnceLock,
};

mod ast_printer;
mod expr;
mod scanner;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Execute a lochx script or run a REPL.
#[derive(FromArgs)]
struct Args {
    /// print version information
    #[argh(switch, short = 'v')]
    version: bool,

    /// script file
    #[argh(positional)]
    script: Vec<String>,
}

#[throws]
fn main() {
    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", APP_NAME, APP_VERSION);
        return;
    }

    if args.script.len() > 1 {
        throw!(anyhow!("Usage: lochx [script file]"));
    }

    let io = liso::InputOutput::new();
    let _ = OUT.set(io.clone_output());

    // sdsdf
    let expr = expr::Expr::Binary(Box::new(expr::Binary {
        left: Box::new(expr::Expr::Unary(Box::new(expr::Unary {
            op: scanner::Token::new(scanner::TokenType::Minus, "-", 1, None),
            right: Box::new(expr::Expr::Literal(Box::new(expr::Literal {
                value: scanner::LiteralValue::Num(123.0),
            }))),
        }))),
        op: scanner::Token::new(scanner::TokenType::Star, "*", 1, None),
        right: Box::new(expr::Expr::Grouping(Box::new(expr::Grouping {
            expr: Box::new(expr::Expr::Literal(Box::new(expr::Literal {
                value: scanner::LiteralValue::Num(45.67),
            }))),
        }))),
    }));

    OUT.get().expect("OOPS").wrapln(liso!(
        fg = green,
        dim,
        ast_printer::AstPrinter::new().print(&expr),
        reset
    ));
    // sdsdf

    if args.script.len() == 1 {
        run_script(&args.script[0])?;
    } else {
        run_repl(io)?;
    }
}

static OUT: OnceLock<OutputOnly> = OnceLock::new();

#[throws]
fn run_repl(mut io: liso::InputOutput) {
    io.prompt(liso!(fg = green, bold, "> ", reset), true, false);
    loop {
        match io.read_blocking() {
            Response::Input(line) => {
                io.echoln(liso!(fg = green, dim, "> ", fg = none, &line));
                run(line.as_str())?
            }
            Response::Discarded(line) => {
                io.echoln(liso!(bold + dim, "X ", -bold, line));
            }
            Response::Dead => break,
            Response::Quit => break,
            Response::Finish => break,
            _ => {}
        }
    }
}

#[throws]
fn run_script(script: &str) {
    let contents = std::fs::read_to_string(script)?;
    run(&contents)?
}

#[throws]
fn run(source: &str) {
    use crate::scanner::Scanner;

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    // For now just print the tokens
    for token in tokens {
        OUT.get().expect("Must be set at start").wrapln(liso!(
            fg = blue,
            format!("{:?}", token),
            fg = none
        ));
    }
}

pub fn error(line: usize, message: &str) {
    OUT.get().expect("Must be set at start").wrapln(liso!(
        fg = red,
        bold,
        format!("[line {}] {}", line, message),
        reset
    ))
}
