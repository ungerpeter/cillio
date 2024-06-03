use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeImplementationData {
    pub input: Option<HashMap<String, String>>,
    pub output: Option<HashMap<String, String>>,
    pub state: Option<HashMap<String, String>>,
    pub wasm: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeData {
    pub r#type: String,
    pub state: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdgeData {
    pub from: String,
    pub to: String,
    pub from_port: Option<String>,
    pub to_port: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphConfig {
    pub node_implementations: HashMap<String, NodeImplementationData>,
    pub nodes: HashMap<String, NodeData>,
    pub edges: Vec<EdgeData>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

pub fn load_config(path: &str) -> Result<GraphConfig, ConfigError> {
    let mut file = File::open(path)?;
    let mut config_str = String::new();
    file.read_to_string(&mut config_str)?;
    let config: GraphConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}

pub fn print_config(config: &GraphConfig) {
    println!("Graph Configuration:");
    println!("Node Implementations:");
    for (key, value) in &config.node_implementations {
        println!("  {}: {:?}", key, value);
    }
    println!("Nodes:");
    for (key, value) in &config.nodes {
        println!("  {}: {:?}", key, value);
    }
    println!("Edges:");
    for edge in &config.edges {
        println!("  {:?}", edge);
    }
}
