use cillio_config::GraphConfig;
use petgraph::{dot::{Config, Dot}, graph::{DiGraph, NodeIndex}};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFoundError(String),

    #[error("Graph structure error: {0}")]
    GraphStructureError(String),
}

#[derive(Debug)]
pub struct EdgeData {
    pub from_port: Option<String>,
    pub to_port: Option<String>,
}

pub struct Graph {
    graph: DiGraph<String, EdgeData>,
    node_map: HashMap<String, NodeIndex>,
}

impl Graph {
    pub fn new(config: GraphConfig) -> Result<Self, GraphError> {
        let mut graph = DiGraph::<String, EdgeData>::new();
        let mut node_map = HashMap::new();

        for (node_id, _) in &config.nodes {
            let index = graph.add_node(node_id.clone());
            node_map.insert(node_id.clone(), index);
        }

        for edge in &config.edges {
            let from_index = node_map
                .get(&edge.from)
                .ok_or_else(|| GraphError::NodeNotFoundError(edge.from.clone()))?;
            let to_index = node_map
                .get(&edge.to)
                .ok_or_else(|| GraphError::NodeNotFoundError(edge.to.clone()))?;
            let edge_data = EdgeData {
                from_port: edge.from_port.clone(),
                to_port: edge.to_port.clone(),
            };
            graph.add_edge(*from_index, *to_index, edge_data);
        }

        Ok(Self { graph, node_map })
    }

    pub fn print_dot(&self) {
        let dot = Dot::with_attr_getters(
            &self.graph,
            &[Config::EdgeNoLabel],
            &|_, edge| {
                let edge_data = edge.weight();
                let mut labels = vec![];
                if let Some(ref from_port) = edge_data.from_port {
                    labels.push(format!("from: {}", from_port));
                }
                if let Some(ref to_port) = edge_data.to_port {
                    labels.push(format!("to: {}", to_port));
                }
                format!("label=\"{}\"", labels.join("\\n")).into()
            },
            &|_, _| String::new().into(),
        );
        println!("{:?}", dot);
    }

    pub fn node_map(&self) -> &HashMap<String, NodeIndex> {
        &self.node_map
    }
}
