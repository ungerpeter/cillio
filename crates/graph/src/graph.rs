use cillio_config::GraphConfig;
use petgraph::{
    dot::{Config, Dot},
    graph::{DiGraph, NodeIndex},
};
use std::collections::HashMap;
use thiserror::Error;

use crate::{Node, Edge};

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFoundError(String),

    #[error("Graph structure error: {0}")]
    GraphStructureError(String),
}

#[derive(Debug, Clone)]
pub struct Graph {
    graph: DiGraph<Node, Edge>,
    node_map: HashMap<String, NodeIndex>,
}

impl Graph {
    pub fn new(config: &GraphConfig) -> Result<Self, GraphError> {
        let mut graph = DiGraph::<Node, Edge>::new();
        let mut node_map = HashMap::new();

        for (node_id, node_data) in &config.nodes {
            let index = graph.add_node(Node::new(node_id.clone(), node_data.clone()));
            node_map.insert(node_id.clone(), index);
        }

        for edge in &config.edges {
            let from_index = node_map
                .get(&edge.from)
                .ok_or_else(|| GraphError::NodeNotFoundError(edge.from.clone()))?;
            let to_index = node_map
                .get(&edge.to)
                .ok_or_else(|| GraphError::NodeNotFoundError(edge.to.clone()))?;
            let edge_data = Edge {
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
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_, edge| {
                let edge_data = edge.weight();
                let mut labels = vec![];
                if let Some(ref from_port) = edge_data.from_port {
                    labels.push(format!("from: {}", from_port));
                }
                if let Some(ref to_port) = edge_data.to_port {
                    labels.push(format!("to: {}", to_port));
                }
                format!("label=\"{}\"", labels.join("\\n"))
            },
            &|_, (_, node)| {
                let mut label = format!("{}\\n{}", node.id, node.data.r#type);
                if let Some(ref state) = node.data.state {
                    label.push_str(&format!("\\n{}", state).replace("\"", "\\\""));
                }
                format!("label=\"{}\"", label)
            },
        );
        println!("{:?}", dot);
    }

    pub fn node_map(&self) -> &HashMap<String, NodeIndex> {
        &self.node_map
    }

    pub fn graph(&self) -> &DiGraph<Node, Edge> {
        &self.graph
    }
}
