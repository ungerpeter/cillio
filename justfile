set shell := ["bash", "-cu"]

CORE_DIR := "crates/cillio-core"
CONFIG_DIR := "crates/cillio-config"
GRAPH_DIR := "crates/cillio-graph"
NODE_IMPLEMENTATIONS_DIR := "crates/cillio-nodes"
TARGET_DIR := "target"

default:
  just --list

build:
    just build-node-implementations
    just build-core
    just build-config
    just build-graph

build-core:
    cargo build --manifest-path {{CORE_DIR}}/Cargo.toml

build-config:
    cargo build --manifest-path {{CONFIG_DIR}}/Cargo.toml

build-graph:
    cargo build --manifest-path {{GRAPH_DIR}}/Cargo.toml

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

print-dot:
    cargo run -p cillio-cli dot -c assets/sum_graph.json | dot -T svg -o assets/sum_graph.svg
