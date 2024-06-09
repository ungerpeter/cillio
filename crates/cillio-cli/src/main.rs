use cillio_config::{load_config, print_config, ConfigError};
use cillio_graph::{Graph, GraphError};
use cillio_runtime::execution_plan::ExecutionPlan;
use cillio_runtime::Runtime;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::time::Instant;
use std::{
    io::{self},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Failed to load configuration: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Configuration file path not provided")]
    ConfigPathNotProvided,

    #[error("Failed to build graph: {0}")]
    GraphError(#[from] GraphError),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[clap(
    name = "Cillio CLI",
    version = "1.0",
    about = "CLI for Cillio Computation Graph"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Print {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    Dot {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    Run,
    TestComposition,
}

#[async_std::main]
async fn main() -> anyhow::Result<(), CliError> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Print { config } => {
            let config_path = config.as_ref().ok_or(CliError::ConfigPathNotProvided)?;
            let config = load_config(
                config_path
                    .to_str()
                    .ok_or(CliError::ConfigPathNotProvided)?,
            )?;
            print_config(&config);
            println!("Printing graph:");
            let graph = Graph::new(&config)?;
            graph.print_dot();
        }
        Commands::Dot { config } => {
            let config_path = config.as_ref().ok_or(CliError::ConfigPathNotProvided)?;
            let config = load_config(
                config_path
                    .to_str()
                    .ok_or(CliError::ConfigPathNotProvided)?,
            )?;
            let graph = Graph::new(&config)?;
            graph.print_dot();
        }
        Commands::Run => test_sum_graph().await.expect("Error testing sum graph"),
        Commands::TestComposition => test_graph_component_composition()
            .await
            .expect("Error testing composition"),
    }

    Ok(())
}

fn load_wasm_module<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

fn get_plugins_from_path(path: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut plugins = std::fs::read_dir(path)?
        .filter_map(|res| res.ok())
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "wasm") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    plugins.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(plugins)
}

async fn test_sum_graph() -> Result<(), anyhow::Error> {
    let total_start_time = Instant::now();
    let start_time = Instant::now();
    println!("Load graph...");
    let config_path = "compiled/sum-graph/graph.json";
    let config = load_config(config_path)?;
    let graph = Graph::new(&config)?;
    println!("Time taken: {} ms\n", start_time.elapsed().as_millis());

    let start_time = Instant::now();
    println!("Load Plugins...");
    let plugins = get_plugins_from_path("compiled/sum-graph")?;

    let graph_node_implementations =
        config
            .clone()
            .node_implementations
            .into_iter()
            .map(|(node_id, node_implementation)| {
                let plugin_path = plugins.iter().find(|path| {
                    path.file_stem().unwrap().to_str().unwrap() == node_implementation.wasm
                });
                (node_id, plugin_path)
            });
    println!("Plugins: {:?}", graph_node_implementations);
    println!("Time taken: {} ms\n", start_time.elapsed().as_millis());

    let start_time = Instant::now();
    println!("Create execution plan...");
    let execution_plan = ExecutionPlan::from_graph(&graph);
    println!("Execution Plan: {:?}", execution_plan);
    println!("Time taken: {} ms\n", start_time.elapsed().as_millis());

    let start_time = Instant::now();
    println!("Create runtime...");
    let mut runtime = Runtime::new();
    println!("Time taken: {} ms\n", start_time.elapsed().as_millis());

    let start_time = Instant::now();
    println!("Load node implementations...");
    for (node_id, plugin_path) in graph_node_implementations {
        let plugin_path = plugin_path
            .ok_or_else(|| anyhow::anyhow!("Plugin not found for node_id: {}", node_id))?;
        let wasm_module_buffer = load_wasm_module(plugin_path)?;
        println!("Loading component: {}", node_id);
        runtime
            .load_component(&node_id, &wasm_module_buffer)
            .await?;
    }
    println!("Time taken: {} ms\n", start_time.elapsed().as_millis());

    let start_time = Instant::now();
    println!("Execute plan...");
    let results = execution_plan.execute(&mut runtime).await?;
    println!("Results: {:?}", results);
    println!("Time taken: {} ms\n", start_time.elapsed().as_millis());

    println!(
        "Total Time taken: {} ms",
        total_start_time.elapsed().as_millis()
    );
    Ok(())
}

async fn test_graph_component_composition() -> Result<(), anyhow::Error> {
    // let start_time = Instant::now();
    // println!("Create runtime...");
    // let mut runtime = Runtime::new();
    // println!("Time taken: {} ms", start_time.elapsed().as_millis());

    // let start_time = Instant::now();
    // println!("Load graph component...");
    // let (graph_world, _) = runtime
    //     .load_graph(PathBuf::from(
    //         "target/wasm32-wasi/release/cillio_graph_component.wasm",
    //     ))
    //     .await
    //     .expect("Failed to load wasm module");
    // println!("Time taken: {} ms", start_time.elapsed().as_millis());

    // let start_time = Instant::now();
    // println!("Add component to graph...");
    // graph_world
    //     .cillio_graph_composition_graph()
    //     .graph()
    //     .call_add_component(
    //         runtime.get_store(),
    //         composition_graph_res,
    //         "test",
    //         wasm_module_buffer.as_slice().into(),
    //     )
    //     .await
    //     .expect("Failed to call add_component")
    //     .expect("Failed to add component");
    // println!("Time taken: {} ms", start_time.elapsed().as_millis());

    // let start_time = Instant::now();
    // println!("Compute graph...");
    // let computed_graph = graph_world
    //     .cillio_graph_composition_graph()
    //     .graph()
    //     .call_print_graph(runtime.get_store(), composition_graph_res)
    //     .await
    //     .expect("Failed to call print_graph");
    // println!("Time taken: {} ms", start_time.elapsed().as_millis());

    // println!("Computed graph:\n{}", computed_graph);

    Ok(())
}
