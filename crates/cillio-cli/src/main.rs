use cillio_config::{load_config, print_config, ConfigError};
use cillio_graph::{Graph, GraphError};
use cillio_runtime::Runtime;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::time::Instant;
use std::{
    fs::File,
    io::{self, Read},
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
            let graph = Graph::new(config)?;
            graph.print_dot();
        }
        Commands::Dot { config } => {
            let config_path = config.as_ref().ok_or(CliError::ConfigPathNotProvided)?;
            let config = load_config(
                config_path
                    .to_str()
                    .ok_or(CliError::ConfigPathNotProvided)?,
            )?;
            let graph = Graph::new(config)?;
            graph.print_dot();
        }
        Commands::Run => {
            let start_time = Instant::now();
            println!("Create runtime...");
            let mut runtime = Runtime::new();
            println!("Time taken: {} ms", start_time.elapsed().as_millis());

            let start_time = Instant::now();
            println!("Load graph component...");
            let (graph_world, _) = runtime
                .load_graph(PathBuf::from(
                    "target/wasm32-wasi/debug/cillio_graph_component.wasm",
                ))
                .await
                .expect("Failed to load wasm module");
            println!("Time taken: {} ms", start_time.elapsed().as_millis());

            let start_time = Instant::now();
            println!("Create composition graph...");
            let composition_graph_res = graph_world
                .cillio_graph_composition_graph()
                .graph()
                .call_constructor(runtime.get_store())
                .await
                .expect("Failed to call constructor");
            println!("Time taken: {} ms", start_time.elapsed().as_millis());

            let start_time = Instant::now();
            println!("Get component binary...");
            let wasm_module_buffer =
                load_wasm_module("target/wasm32-wasi/debug/cillio_addition_node.wasm")
                    .expect("Failed to load wasm module");
            println!("Time taken: {} ms", start_time.elapsed().as_millis());

            let start_time = Instant::now();
            println!("Add component to graph...");
            graph_world
                .cillio_graph_composition_graph()
                .graph()
                .call_add_component(
                    runtime.get_store(),
                    composition_graph_res,
                    "test",
                    wasm_module_buffer.as_slice().into(),
                )
                .await
                .expect("Failed to call add_component")
                .expect("Failed to add component");
            println!("Time taken: {} ms", start_time.elapsed().as_millis());

            let start_time = Instant::now();
            println!("Compute graph...");
            let computed_graph = graph_world
                .cillio_graph_composition_graph()
                .graph()
                .call_print_graph(runtime.get_store(), composition_graph_res)
                .await
                .expect("Failed to call print_graph");
            println!("Time taken: {} ms", start_time.elapsed().as_millis());

            println!("Computed graph:\n{}", computed_graph);
        }
    }

    Ok(())
}

fn load_wasm_module<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
