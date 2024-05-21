use {
    crate::{ast_printer::AstPrinter, parser::Parser},
    anyhow::{anyhow, Error},
    argh::FromArgs,
    culpa::{throw, throws},
    interpreter::Interpreter,
    liso::{liso, OutputOnly, Response},
    std::sync::OnceLock,
};

mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;

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
    let _ = INTERPRETER.set(Interpreter::new(io.clone_output()));

    if args.script.len() == 1 {
        run_script(&args.script[0])?;
    } else {
        run_repl(io)?;
    }
}

static OUT: OnceLock<OutputOnly> = OnceLock::new();
static INTERPRETER: OnceLock<Interpreter> = OnceLock::new();

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

    let mut parser = Parser::new(tokens);

    let ast = parser.parse();

    if let Err(e) = ast {
        error(1, e.to_string().as_str());
        return;
    }

    let ast = ast.unwrap();

    let printer = AstPrinter::new();

    OUT.get().expect("Must be set at start").wrapln(liso!(
        fg = blue,
        &printer.print_stmt(ast.clone()),
        fg = none
    ));

    let interpreter = INTERPRETER.get().expect("Must be set at start");
    let value = interpreter.interpret(ast);

    OUT.get().expect("Must be set at start").wrapln(liso!(
        fg = green,
        format!("{:?}", value),
        fg = none
    ));
}
// @todo use nom_report to report exact parse error locations

pub fn error(line: usize, message: &str) {
    OUT.get().expect("Must be set at start").wrapln(liso!(
        fg = red,
        bold,
        format!("[line {}] {}", line, message),
        reset
    ))
}
