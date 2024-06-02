use anyhow::Context;
use std::path::PathBuf;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

wasmtime::component::bindgen!({
    path: "../cillio-nodes/cillio-addition-node/wit/world.wit",
    world: "node",
    async: true
});

pub async fn add(path: PathBuf, x: f32, y: f32) -> wasmtime::Result<f32> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::add_to_linker_async(&mut linker).context("Failed to link command world")?;
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);

    let component = Component::from_file(&engine, path).context("Component file not found")?;

    let (instance, _) = Node::instantiate_async(&mut store, &component, &linker)
        .await
        .context("Failed to instantiate the addition world")?;
    instance
        .call_run(&mut store, x, y)
        .await
        .context("Failed to call add function")
}

struct ServerWasiView {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl ServerWasiView {
    fn new() -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new().inherit_stdio().build();

        Self { table, ctx }
    }
}

impl WasiView for ServerWasiView {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
