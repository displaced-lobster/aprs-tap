# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Runs clippy
check:
    cargo clippy --locked --features server -- -D warnings

# Runs rustfmt
fmt:
    cargo +nightly fmt

# Run the server
server:
    cargo run --features server --bin server

# Run the server with hot reload (requires cargo-watch: cargo install cargo-watch)
dev:
    cargo watch -x "run --features server --bin server"
