pub mod execution_plan;

use anyhow::Context;
use std::{path::PathBuf, time::Instant};
use thiserror::Error;
use wasmtime::component::{Component, Linker};
use wasmtime::Engine;
use wasmtime::*;
use wasmtime_wasi::*;

wasmtime::component::bindgen!({
    path: "../../wit/graph/world.wit",
    world: "graph-world",
    async: true
});

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Failed to load WASM module: {0}")]
    WasmLoadError(#[from] anyhow::Error),

    #[error("Failed to initialize node: {0}")]
    NodeInitializationError(String),

    #[error("Failed to execute compute function: {0}")]
    ComputeError(String),

    #[error("Node not found: {0}")]
    NodeNotFoundError(String),

    #[error("File read error: {0}")]
    FileReadError(#[from] std::io::Error),
}

pub struct ServerWasiView {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl std::fmt::Debug for ServerWasiView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerWasiView")
            .field("table", &self.table)
            .field("ctx", &"WasiCtx")
            .finish()
    }
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

pub struct Runtime {
    _config: Config,
    engine: Engine,
    linker: Linker<ServerWasiView>,
    store: Store<ServerWasiView>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut config = Config::default();
        config.wasm_component_model(true);
        config.async_support(true);
        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .context("Failed to link command world")
            .unwrap();
        let wasi_view = ServerWasiView::new();
        let store = Store::new(&engine, wasi_view);

        Self {
            _config: config,
            engine,
            linker,
            store,
        }
    }

    pub async fn load_component(&mut self, bytes: &Vec<u8>) -> Result<Component, anyhow::Error> {
        let start_time = Instant::now();
        println!("-- Component from bytes...");
        let component =
            Component::new(&self.engine, bytes).context("Failed to load component from binary")?;
        println!("-- Time taken: {} ms", start_time.elapsed().as_millis());
        Ok(component)
    }

    pub async fn load_graph(
        &mut self,
        path: PathBuf,
    ) -> Result<(GraphWorld, wasmtime::component::Instance), anyhow::Error> {
        let start_time = Instant::now();
        println!("--Component from file...");
        let component =
            Component::from_file(&self.engine, path).context("Component file not found")?;
        println!("Time taken: {} ms", start_time.elapsed().as_millis());
        let start_time = Instant::now();
        println!("--Instantiate component...");
        let ret = GraphWorld::instantiate_async(&mut self.store, &component, &self.linker)
            .await
            .context("Failed to instantiate the graph world");
        println!("Time taken: {} ms", start_time.elapsed().as_millis());
        ret
    }

    pub fn get_store(&mut self) -> &mut Store<ServerWasiView> {
        &mut self.store
    }

    pub fn get_linker(&mut self) -> &mut Linker<ServerWasiView> {
        &mut self.linker
    }
}
