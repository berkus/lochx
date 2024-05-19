use {anyhow::{anyhow, Result}, argh::FromArgs};

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

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    if args.version {
        println!("{} {}", APP_NAME, APP_VERSION);
        return Ok(());
    }

    if args.script.len() > 1 {
        return Err(anyhow!("Usage: lochx [script file]"));
    }

    if args.script.len() == 1 {
        run_script(&args.script[0]);
    } else {
        run_repl();
    }

    Ok(())
}

fn run_repl() {}
fn run_script(_script: &str) {}
