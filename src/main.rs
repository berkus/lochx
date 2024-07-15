#![feature(sync_unsafe_cell)]
#![feature(let_chains)]

use {
    crate::{ast_printer::AstPrinter, parser::Parser},
    anyhow::{anyhow, Error},
    argh::FromArgs,
    culpa::{throw, throws},
    error::RuntimeError,
    interpreter::Interpreter,
    liso::{liso, OutputOnly, Response},
    miette::{LabeledSpan, MietteDiagnostic, Report},
    sema::resolver::Resolver,
    std::sync::OnceLock,
};

mod ast_printer;
mod environment;
mod error;
mod interpreter;
mod parser;
mod runtime;
mod scanner;
mod sema;
mod types;

pub use types::{callable, class, expr, literal, stmt};

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
    runtime::set_source("");
    io.prompt(liso!(fg = green, bold, "> ", reset), true, false);
    loop {
        match io.read_blocking() {
            Response::Input(line) => {
                let source = line.as_str();
                io.echoln(liso!(fg = green, dim, "> ", fg = none, source));
                let scan_offset = runtime::append_source(source);
                run(&mut interpreter, source, scan_offset)?
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
    runtime::set_source(contents.clone());
    run(&mut interpreter, &contents, 0)?
}

#[throws]
fn run(interpreter: &mut Interpreter, source: &str, scan_offset: usize) {
    use crate::scanner::Scanner;

    let mut scanner = Scanner::new(source, scan_offset);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);

    let ast = parser.parse();

    if let Err(e) = ast {
        error(e, "Parsing error");
        return;
    }

    let ast = ast.unwrap();

    let mut printer = AstPrinter::new();

    let ast_printable = printer.print_stmt(ast.clone())?;

    wrapln(ast_printable);

    let mut resolver = Resolver::new(interpreter);
    let resolved = resolver.resolve(&ast);

    if let Err(e) = resolved {
        error(e, "Resolution error");
        return;
    }

    let value = interpreter.interpret(ast);

    if let Err(e) = value {
        error(e, "Runtime error");
        return;
    }
}

pub fn wrapln(args: impl AsRef<str>) {
    OUT.get()
        .expect("Must be set at start")
        .wrapln(liso!(fg = blue, args.as_ref(), fg = none));
}

pub fn error(runtime_error: RuntimeError, message: &str) {
    let (span, inner_message, note) = match runtime_error {
        RuntimeError::ParseError {
            token,
            expected,
            message,
        } => (
            token.position.span,
            message,
            format!("Expected {expected:?}"),
        ),
        RuntimeError::ScanError { location } => (location.span, "Here".into(), "".into()),
        RuntimeError::TopLevelReturn(ref t, note) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            note.into(),
        ),
        RuntimeError::NonClassThis(ref t, note) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            note.into(),
        ),
        RuntimeError::RecursiveClass(ref t) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            "".into(),
        ),
        RuntimeError::InvalidSuper(ref t, note) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            note.into(),
        ),
        RuntimeError::UndefinedVariable(ref t, _) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            "".into(),
        ),
        RuntimeError::InvalidPropertyAccess(ref t, note) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            note.into(),
        ),
        RuntimeError::DuplicateDeclaration(ref t, note) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            note.into(),
        ),
        RuntimeError::InvalidAssignmentTarget(ref t, note) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            note.into(),
        ),
        RuntimeError::ExpectedExpression(ref t) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            "".into(),
        ),
        RuntimeError::TooManyArguments(ref t) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            "".into(),
        ),
        RuntimeError::NotACallable(ref t) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            "".into(),
        ),
        RuntimeError::InvalidArity(ref t, _, _) => (
            t.position.span.clone(),
            format!("{runtime_error}"),
            "".into(),
        ),
        _ => ((0..0), format!("{runtime_error}"), "".into()), // @todo skip label if no span
    };

    let diag = MietteDiagnostic::new(message).with_label(LabeledSpan::at(span, inner_message));
    let diag = if note.is_empty() {
        diag
    } else {
        diag.with_help(note)
    };

    let report = Report::new(diag).with_source_code(runtime::source());

    OUT.get().expect("Must be set at start").println(liso!(
        fg = red,
        bold,
        format!("{:?}", report),
        fg = none
    ));
}
