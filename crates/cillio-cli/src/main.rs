use cillio_config::{load_config, print_config, ConfigError};
use cillio_graph::{Graph, GraphError};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
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
    /// Print the computation graph
    Print {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    Dot {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

fn main() -> Result<(), CliError> {
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
    }

    Ok(())
}
