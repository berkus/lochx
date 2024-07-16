_default:
    @just --list

# Collect profiling information using samply, output to profile.json
samply:
    cargo build -Z build-std --target aarch64-apple-darwin --release
    samply record --save-only -o profile.json ./target/aarch64-apple-darwin/release/lochx tests/slow.lox
    rm -f profile.json.gz
    gzip -9 profile.json

# Load and view saved samply profile
sview:
    samply load profile.json.gz

# Collect profiling information using flamegraph, output to flamegraph.svg
flamegraph:
    sudo cargo flamegraph -- tests/slow.lox
