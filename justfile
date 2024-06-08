set shell := ["bash", "-cu"]

CORE_DIR := "crates/cillio-core"
CONFIG_DIR := "crates/cillio-config"
GRAPH_DIR := "crates/cillio-graph"
GRAPH_COMPONENT_DIR := "crates/cillio-graph-component"
NODE_IMPLEMENTATIONS_DIR := "crates/cillio-nodes"
TARGET_DIR := "target"
DOCS_DIR := "docs"

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
        just build-wasm ${dir}; \
    done

build-wasm target:
    @echo '🏗️ Building wasm: {{target}}…'
    @cd {{target}} && cargo component build --release;

optimize-wasm target:
    @echo '🏎️ Optimizing wasm: {{target}}…'
    @wasm-opt -Oz --enable-bulk-memory -o {{target}}.wasm {{target}}.wasm

compile-docs:
    @echo "📚 Compiling docs"
    @find {{DOCS_DIR}} -name "*.mmd" -mindepth 1 | while read input; do \
        echo "📝 Compiling ${input}"; \
        output="${input%.mmd}.svg" && \
        mmdc -i "${input}" -o "${output}" -t dark -b transparent; \
    done
    @echo "🟢 Done compiling docs"

compile-sum-graph:
    @echo "🧹 Cleaning compiled/sum-graph"
    @rm -rf compiled/sum-graph
    @echo "📦 Copy sum-graph.json"
    @mkdir -p compiled/sum-graph && cp assets/sum_graph.json compiled/sum-graph/graph.json
    @just build-node-implementations
    @echo "📦 Copying node implementations"
    cp -r target/wasm32-wasi/release/cillio_emit_number_node.wasm compiled/sum-graph
    cp -r target/wasm32-wasi/release/cillio_addition_node.wasm compiled/sum-graph
    cp -r target/wasm32-wasi/release/cillio_log_number_node.wasm compiled/sum-graph
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

run: compile-sum-graph
    cargo run -p cillio-cli run
