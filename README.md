## Requirements:
- just: brew install just
- cargo-component: cargo install cargo-component --locked

## Build:
```bash
just build
```

## Run host:
```bash
cargo run -- 42 76 ./target/wasm32-wasi/debug/cillio_addition_node.wasm
```