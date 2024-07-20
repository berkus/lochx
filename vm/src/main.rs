use {
    argh::FromArgs,
    chunk::Chunk,
    culpa::{throw, throws},
    error::RuntimeError,
    liso::{liso, OutputOnly, Response},
    miette::{miette, LabeledSpan, MietteDiagnostic, Report},
    opcode::OpCode,
    std::sync::OnceLock,
    vm::VM,
};

mod chunk;
mod error;
mod opcode;
mod value;
mod vm;

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

#[throws(RuntimeError)]
fn main() {
    let mut chunk = Chunk::new();
    let a = chunk.append_const(1.2);
    let b = chunk.append_const(3.4);
    let c = chunk.append_const(5.6);
    chunk.append_op(OpCode::Constant(a), 1);
    chunk.append_op(OpCode::Constant(b), 1);
    chunk.append_op(OpCode::Add, 1);
    chunk.append_op(OpCode::Constant(c), 2);
    chunk.append_op(OpCode::Divide, 2);
    chunk.append_op(OpCode::Return, 3);
    chunk.disassemble("Test")?;

    let mut vm = VM::new(&chunk, 0);
    let _ = vm.run();

    return;

    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", APP_NAME, APP_VERSION);
        return;
    }

    if args.script.len() > 1 {
        throw!(RuntimeError::Usage(miette!("lochx [script file]")));
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

#[throws(RuntimeError)]
fn run_repl(mut io: liso::InputOutput) {
    //...
    io.prompt(liso!(fg = green, bold, "> ", reset), true, false);
    loop {
        match io.read_blocking() {
            Response::Input(line) => {
                let source = line.as_str();
                io.echoln(liso!(fg = green, dim, "> ", fg = none, source));
                //...
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

#[throws(RuntimeError)]
fn run_script(_io: liso::InputOutput, script: &str) {
    let contents = std::fs::read_to_string(script)?;
    //...
    run(&contents, 0)?
}

#[throws(RuntimeError)]
fn run(/*interpreter: &mut Interpreter,*/ _source: &str, _scan_offset: usize) {
    //...
}

pub fn wrapln(args: impl AsRef<str>) {
    OUT.get()
        .expect("Must be set at start")
        .wrapln(liso!(fg = blue, args.as_ref(), fg = none));
}

pub fn error(_runtime_error: RuntimeError, message: &str) {
    let note = "";
    let span = 0..1;
    let inner_message = "";

    let diag = MietteDiagnostic::new(message).with_label(LabeledSpan::at(span, inner_message));
    let diag = if note.is_empty() {
        diag
    } else {
        diag.with_help(note)
    };

    let report = Report::new(diag).with_source_code("");

    OUT.get().expect("Must be set at start").println(liso!(
        fg = red,
        bold,
        format!("{:?}", report),
        fg = none
    ));
}
