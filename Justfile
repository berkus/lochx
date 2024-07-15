_default:
    @just --list

# Run the old AST-traversing interpreter (run without args for repl)
interpreter FILE='':
    cargo run --release -p interpreter -- {{FILE}}

# (broken) Collect profiling information using samply, output to profile.json
samply:
    cargo build -Z build-std --target aarch64-apple-darwin --release
    samply record --save-only -o profile.json ./target/aarch64-apple-darwin/release/lochx tests/slow.lox
    rm profile.json.gz
    gzip -9 profile.json

# Load and view saved samply profile
sview:
    samply load profile.json.gz

# (broken) Collect profiling information using flamegraph, output to flamegraph.svg
flamegraph:
    sudo cargo flamegraph -- tests/slow.lox
