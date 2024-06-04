use anyhow::Context;
use component::Component;
use std::path::PathBuf;
use thiserror::Error;
use wasmtime::component::Linker;
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
    config: Config,
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
            config,
            engine,
            linker,
            store,
        }
    }

    pub async fn load_graph(
        &mut self,
        path: PathBuf,
    ) -> Result<(GraphWorld, wasmtime::component::Instance), anyhow::Error> {
        let component =
            Component::from_file(&self.engine, path).context("Component file not found")?;
        GraphWorld::instantiate_async(&mut self.store, &component, &self.linker)
            .await
            .context("Failed to instantiate the graph world")
    }

    pub async fn compute_graph_instance(
        &mut self,
        instance: &GraphWorld,
    ) -> Result<(), anyhow::Error> {
        instance
            .call_compute(&mut self.store)
            .await
            .context("Failed to call compute function")
    }

    pub fn get_store(&mut self) -> &mut Store<ServerWasiView> {
        &mut self.store
    }
}
