set shell := ["bash", "-cu"]

CORE_DIR := "crates/cillio-core"
CONFIG_DIR := "crates/cillio-config"
GRAPH_DIR := "crates/cillio-graph"
GRAPH_COMPONENT_DIR := "crates/cillio-graph-component"
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

build-graph-component:
    cargo component build --manifest-path {{GRAPH_COMPONENT_DIR}}/Cargo.toml --release
    wasm-tools strip {{TARGET_DIR}}/wasm32-wasi/release/cillio_graph_component.wasm -o {{TARGET_DIR}}/wasm32-wasi/release/cillio_graph_component.wasm

build-node-implementations:
    @find {{NODE_IMPLEMENTATIONS_DIR}} -maxdepth 1 -mindepth 1 -type d | while read dir; do \
        echo "Building node in ${dir}"; \
        (cd ${dir} && cargo component build); \
    done

build-wasm target target_dir:
    @echo '🏗️ Building wasm: {{target}}…'
    @if [ -n "{{target_dir}}" ]; then \
        cargo component build --manifest-path {{target}}/Cargo.toml --release --out-dir {{target_dir}} -Z unstable-options; \
    else \
        cargo component build --manifest-path {{target}}/Cargo.toml --release; \
    fi

optimize-wasm target:
    @echo '🏎️ Optimizing wasm: {{target}}…'
    @wasm-opt -Oz --enable-bulk-memory -o {{target}}.wasm {{target}}.wasm

compile-sum-graph:
    @echo "🧹 Cleaning compiled/sum-graph"
    @rm -rf compiled/sum-graph
    @just build-wasm "crates/cillio-nodes/cillio-emit-number-node" "compiled/sum-graph"
    @just optimize-wasm "compiled/sum-graph/cillio_emit_number_node"
    @just build-wasm "crates/cillio-nodes/cillio-addition-node" "compiled/sum-graph"
    @just optimize-wasm "compiled/sum-graph/cillio_addition_node"
    @just build-wasm "crates/cillio-nodes/cillio-log-number-node" "compiled/sum-graph"
    @just optimize-wasm "compiled/sum-graph/cillio_log_number_node"
    @echo "📦 Copying assets/sum_graph.json to compiled/sum-graph/graph.json"
    @cp assets/sum_graph.json compiled/sum-graph/graph.json
    @echo "🟢 Done compiling:"
    @ls -lh compiled/sum-graph

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

print: 
    cargo run -p cillio-cli print -c assets/sum_graph.json

save-dot:
    cargo run -p cillio-cli dot -c assets/sum_graph.json | dot -T svg -o assets/sum_graph.svg

run:
    just build-graph-component
    cargo run -p cillio-cli --release run
