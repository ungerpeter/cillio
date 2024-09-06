set shell := ["bash", "-cu"]

CLI_DIR := "crates/cli"
COMPONENTS_DIR := "crates/components"
CONFIG_DIR := "crates/config"
GRAPH_DIR := "crates/graph"
RUNTIME_DIR := "crates/runtime"

TARGET_DIR := "target"
DOCS_DIR := "docs"

default:
  just --list

build:
    just build-cli
    just build-components
    just build-config
    just build-graph
    just build-runtime

build-cli:
    cargo build --manifest-path {{CLI_DIR}}/Cargo.toml

build-components:
    @find {{COMPONENTS_DIR}} -maxdepth 1 -mindepth 1 -type d | while read dir; do \
        just build-wasm ${dir}; \
    done

build-wasm target:
    @echo '🏗️ Building wasm: {{target}}…'
    @cd {{target}} && cargo component build --release;

build-config:
    cargo build --manifest-path {{CONFIG_DIR}}/Cargo.toml

build-graph:
    cargo build --manifest-path {{GRAPH_DIR}}/Cargo.toml

build-runtime:
    cargo build --manifest-path {{RUNTIME_DIR}}/Cargo.toml

compile-docs:
    @echo "📚 Compiling docs"
    @find {{DOCS_DIR}} -name "*.mmd" -mindepth 1 | while read input; do \
        echo "📝 Compiling ${input}"; \
        output="${input%.mmd}.svg" && \
        mmdc -i "${input}" -o "${output}" -b transparent; \
    done
    @echo "🟢 Done compiling docs"

compile-sum-graph: build-components
    @echo "🧹 Cleaning compiled/sum-graph"
    @rm -rf compiled/sum-graph
    @echo "📦 Copy sum-graph.json"
    @mkdir -p compiled/sum-graph && cp assets/sum_graph/sum_graph.json compiled/sum-graph/graph.json
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
    cargo run -p cillio-cli print -c assets/sum_graph/sum_graph.json

save-dot:
    cargo run -p cillio-cli dot -c assets/sum_graph/sum_graph.json | dot -T svg -o assets/sum_graph/sum_graph.svg

run: compile-sum-graph
    cargo run -p cillio-cli run

tui:
    cargo run -p cillio-tui
