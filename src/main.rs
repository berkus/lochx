#![feature(sync_unsafe_cell)]

use {
    crate::{ast_printer::AstPrinter, parser::Parser, scanner::SourcePosition},
    anyhow::{anyhow, Error},
    argh::FromArgs,
    culpa::{throw, throws},
    error::RuntimeError,
    interpreter::Interpreter,
    liso::{liso, OutputOnly, Response},
    miette::{miette, LabeledSpan},
    std::sync::OnceLock,
};

mod ast_printer;
mod callable;
mod environment;
mod error;
mod expr;
mod interpreter;
mod literal;
mod parser;
mod runtime;
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

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .unicode(true) // liso doesn't wrapln! unicode output well.. use println!
                .color(false) // liso doesn't handle color codes well..
                .context_lines(3)
                .build(),
        )
    }))
    .unwrap();

    let io = liso::InputOutput::new();
    let _ = OUT.set(io.clone_output());

    if args.script.len() == 1 {
        run_script(io, &args.script[0])?;
    } else {
        run_repl(io)?;
    }
}

static OUT: OnceLock<OutputOnly> = OnceLock::new();

#[throws]
fn run_repl(mut io: liso::InputOutput) {
    let mut interpreter = Interpreter::new(io.clone_output());
    io.prompt(liso!(fg = green, bold, "> ", reset), true, false);
    loop {
        match io.read_blocking() {
            Response::Input(line) => {
                io.echoln(liso!(fg = green, dim, "> ", fg = none, &line));
                run(&mut interpreter, line.as_str())?
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
fn run_script(io: liso::InputOutput, script: &str) {
    let contents = std::fs::read_to_string(script)?;
    let mut interpreter = Interpreter::new(io.clone_output());
    run(&mut interpreter, &contents)?
}

#[throws]
fn run(interpreter: &mut Interpreter, source: &str) {
    use crate::scanner::Scanner;

    runtime::set_source(source.into());

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);

    let ast = parser.parse();

    if let Err(e) = ast {
        match e {
            RuntimeError::ParseError {
                token,
                expected,
                message,
            } => {
                error(token.position.clone(), source, &message);
            }
            e => {
                error(
                    SourcePosition {
                        line: 1,
                        span: (0..5),
                    },
                    source,
                    e.to_string().as_str(),
                );
            }
        }
        return;
    }

    let ast = ast.unwrap();

    let mut printer = AstPrinter::new();

    OUT.get().expect("Must be set at start").wrapln(liso!(
        fg = blue,
        &printer.print_stmt(ast.clone())?,
        fg = none
    ));

    let value = interpreter.interpret(ast);

    if let Err(e) = value {
        match e {
            RuntimeError::ParseError {
                token,
                expected,
                message,
            } => {
                let report = miette!(
                    labels = vec![LabeledSpan::at(token.position.span, message)],
                    help = format!("Expected {expected:?} but got {0:?}", token.r#type),
                    "Parse error"
                )
                .with_source_code(source.to_string());

                OUT.get().expect("Must be set at start").println(liso!(
                    fg = red,
                    bold,
                    format!("{:?}", report),
                    fg = none
                ));
            }
            _ => {
                OUT.get().expect("Must be set at start").wrapln(liso!(
                    fg = red,
                    bold,
                    format!("Runtime error: {}", e),
                    fg = none
                ));
            }
        }

        return;
    }
}

pub fn error(location: SourcePosition, source: &str, message: &str) {
    let report = miette!(
        labels = vec![LabeledSpan::at(location.span, message)],
        // help = message,
        "Error"
    )
    .with_source_code(source.to_string());

    OUT.get().expect("Must be set at start").println(liso!(
        fg = red,
        bold,
        format!("{:?}", report),
        fg = none
    ))
}
