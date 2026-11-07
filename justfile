default:
    @just --list

# Build the release web bundle into ./pkg
build:
    cargo build --target wasm32-unknown-unknown --release
    wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/physim.wasm

# Build a debug web bundle into ./pkg (faster compile)
build-dev:
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/debug/physim.wasm

# Build and serve the app at http://localhost:8000
serve: build
    python3 -m http.server 8000 --bind 127.0.0.1

# Type-check the crate for the browser target
check:
    cargo check --target wasm32-unknown-unknown

# Run clippy for the browser target
clippy:
    cargo clippy --target wasm32-unknown-unknown --all-targets -- -D warnings

# Run host tests
test:
    cargo test

# Format Rust and Nix sources
fmt:
    cargo fmt
    nixfmt .

# Build the site reproducibly with Nix
nix-build:
    nix build

# Serve the Nix-built site at http://localhost:8000
nix-serve:
    nix run

# Run all pre-commit hooks
pre-commit-all:
    pre-commit run --all-files

# Remove local build artifacts
clean:
    cargo clean
    rm -rf pkg result
