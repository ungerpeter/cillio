## Requirements:
- cargo-component: cargo install cargo-component --locked

## Build wasm:
```bash
cd addition && cargo component build --release
```

## Run host:
```bash
cargo run --release -- 42 76 ./target/wasm32-wasi/release/addition.wasm
```