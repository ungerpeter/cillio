use cillio_config::GraphConfig;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFoundError(String),

    #[error("Graph structure error: {0}")]
    GraphStructureError(String),
}

pub struct Graph {
    graph: DiGraph<String, usize>,
    node_map: HashMap<String, NodeIndex>,
}

impl Graph {
    pub fn new(config: GraphConfig) -> Result<Self, GraphError> {
        let mut graph = DiGraph::<String, usize>::new();
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
            graph.add_edge(*from_index, *to_index, 1);
        }

        Ok(Self { graph, node_map })
    }

    pub fn print(&self) {
        for node in self.graph.node_indices() {
            println!("Node: {}", self.graph[node]);
        }

        for edge in self.graph.edge_indices() {
            let (from, to) = self.graph.edge_endpoints(edge).unwrap();
            println!("Edge: {} -> {}", self.graph[from], self.graph[to]);
        }
    }

    pub fn node_map(&self) -> &HashMap<String, NodeIndex> {
        &self.node_map
    }
}
