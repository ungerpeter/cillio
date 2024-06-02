set shell := ["bash", "-cu"]

CORE_DIR := "crates/cillio-core"
NODE_IMPLEMENTATIONS_DIR := "crates/cillio-nodes"
TARGET_DIR := "target"

default:
  just --list

build:
    just build-node-implementations
    just build-core

build-core:
    cargo build --manifest-path {{CORE_DIR}}/Cargo.toml

build-node-implementations:
    @find {{NODE_IMPLEMENTATIONS_DIR}} -maxdepth 1 -mindepth 1 -type d | while read dir; do \
        echo "Building node in ${dir}"; \
        (cd ${dir} && cargo component build); \
    done

clean:
    cargo clean

fmt:
    cargo fmt --all

clippy:
    cargo clippy --all -- -D warnings

test:
    cargo test --all

doc:
    cargo doc --no-deps --open