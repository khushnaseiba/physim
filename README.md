# physim

play it [here](https://khushnaseiba.github.io/physim/)

A small physics simulation engine that runs in the browser.

- Balls fall, bounce, and collide with each other.
- Position-based Verlet integration with elastic collisions.
- Mass scales with size, so bigger balls push smaller ones around.
- Spawn, drag, resize, throw, and delete balls with the mouse.

## Controls

- **Left click** on empty space: spawn a ball.
- **Drag** a ball: move it around and throw it on release.
- **Scroll** over a ball: resize it.
- **Right click** a ball: delete it.

## Requirements

- Rust
- The WebAssembly target: `rustup target add wasm32-unknown-unknown`
- `wasm-bindgen-cli` matching `Cargo.lock`: `cargo install wasm-bindgen-cli --version 0.2.130`
- [just](https://github.com/casey/just) (optional, but used below)

## Build and run

```
just serve
```

and open [http://localhost:8000](http://localhost:8000).

`just serve` builds the release wasm bundle into `pkg/` and serves the project
with `python3 -m http.server`. To only build:

```
just build
```

Or without `just`:

```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/physim.wasm
python3 -m http.server 8000
```

### Nix

```
nix run
```
