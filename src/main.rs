use {
    anyhow::{anyhow, Error},
    argh::FromArgs,
    culpa::{throw, throws},
    liso::{liso, Response},
};

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

    if args.script.len() == 1 {
        run_script(&args.script[0])?;
    } else {
        run_repl()?;
    }
}

#[throws]
fn run_repl() {
    let mut io = liso::InputOutput::new();
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
fn run(_text: &str) {}
