#![feature(iterator_try_collect)]

pub mod execution_plan;

use anyhow::Context;
use cillio::node::host::{Host, State};
use component::types::{ComponentFunc, ComponentItem};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use wasmtime::component::{Component, Instance, Linker};
use wasmtime::Engine;
use wasmtime::*;
use wasmtime_wasi::*;

wasmtime::component::bindgen!({
    path: "../../wit/node/world.wit",
    world: "node",
    async: true,
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
    nodes_state: HashMap<String, Vec<u8>>,
}

impl std::fmt::Debug for ServerWasiView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerWasiView")
            .field("table", &self.table)
            .field("ctx", &"WasiCtx")
            .field("nodes_state", &self.nodes_state)
            .finish()
    }
}

impl ServerWasiView {
    fn new() -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new().inherit_stdio().build();
        let nodes_state = HashMap::new();

        Self {
            table,
            ctx,
            nodes_state,
        }
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

impl Host for ServerWasiView {
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn get_state<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Option<State>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}

pub struct Runtime {
    engine: Engine,
    linker: Linker<ServerWasiView>,
    store: Store<ServerWasiView>,
    components: HashMap<String, Component>,
    runtime_data: HashMap<String, Val>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut config = Config::default();
        config.wasm_component_model(true);
        config.async_support(true);
        let engine = Engine::new(&config).unwrap();
        let wasi_view = ServerWasiView::new();
        let store = Store::new(&engine, wasi_view);
        let components = HashMap::new();
        let mut linker = Linker::new(&engine);
        let runtime_data = HashMap::new();

        Node::add_to_linker(&mut linker, |state| state)
            .context("Failed to link node world")
            .unwrap();

        wasmtime_wasi::add_to_linker_async(&mut linker)
            .context("Failed to link command world")
            .unwrap();

        Self {
            engine,
            linker,
            store,
            components,
            runtime_data,
        }
    }

    pub fn get_data(&self, key: &str) -> Option<&Val> {
        self.runtime_data.get(key)
    }

    pub fn get_data_hashmap(&self) -> &HashMap<String, Val> {
        &self.runtime_data
    }

    pub fn set_data(&mut self, key: &str, value: Val) {
        self.runtime_data.insert(key.to_string(), value);
    }

    pub async fn load_component(
        &mut self,
        id: &str,
        bytes: &Vec<u8>,
    ) -> Result<Component, anyhow::Error> {
        let start_time = Instant::now();
        println!("-- Component from bytes...");
        let component =
            Component::new(&self.engine, bytes).context("Failed to load component from binary")?;
        println!("-- Time taken: {} ms", start_time.elapsed().as_millis());
        // self.get_component_run_fn(id)
        //     .ok_or(anyhow::anyhow!("Component run function not found: {}", id))?;
        self.components.insert(id.to_string(), component.clone());
        Ok(component)
    }

    pub fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn get_component_run_fn(&self, id: &str) -> Option<ComponentFunc> {
        let component = self.components.get(id)?;
        let component_type = component.component_type();
        let component_item = component_type
            .exports(&self.engine)
            .find(|x| x.0 == "run")
            .unwrap();
        match component_item {
            (_, ComponentItem::ComponentFunc(handle)) => Some(handle),
            _ => None,
        }
    }

    // pub async fn load_graph(
    //     &mut self,
    //     path: PathBuf,
    // ) -> Result<(GraphWorld, wasmtime::component::Instance), anyhow::Error> {
    //     let start_time = Instant::now();
    //     println!("--Component from file...");
    //     let component =
    //         Component::from_file(&self.engine, path).context("Component file not found")?;
    //     println!("Time taken: {} ms", start_time.elapsed().as_millis());
    //     let start_time = Instant::now();
    //     println!("--Instantiate component...");
    //     let ret = GraphWorld::instantiate_async(&mut self.store, &component, &self.linker)
    //         .await
    //         .context("Failed to instantiate the graph world");
    //     println!("Time taken: {} ms", start_time.elapsed().as_millis());
    //     ret
    // }

    pub fn get_store(&self) -> &Store<ServerWasiView> {
        &self.store
    }

    pub fn get_linker(&self) -> &Linker<ServerWasiView> {
        &self.linker
    }

    pub async fn initialize_node<S>(
        &mut self,
        node_id: &str,
        _state: Option<S>,
    ) -> Result<Instance, RuntimeError> {
        let component = self.components.get(node_id).ok_or_else(|| {
            RuntimeError::NodeNotFoundError(format!("Node not found: {}", node_id))
        })?;
        let instance = self
            .linker
            .instantiate_async(&mut self.store, component)
            .await?;
        Ok(instance)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
