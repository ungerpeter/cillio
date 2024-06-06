use cillio_graph::Graph;
use petgraph::algo::toposort;
use serde_json::Value;

use crate::Runtime;

#[derive(Debug)]
struct ExecutionStep<S> {
    node_id: usize,
    node_type: String,
    node_state: Option<S>,
}

impl<S: std::fmt::Debug> ExecutionStep<S> {
    fn execute(&self) {
        println!(
            "Executing node {} - {}: {:?}",
            self.node_id,
            self.node_type,
            self.node_state.as_ref()
        );
    }
}

#[derive(Debug)]
pub struct ExecutionPlan {
    steps: Vec<ExecutionStep<Value>>,
}

impl ExecutionPlan {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn from_graph(graph: &Graph) -> Self {
        let digraph = graph.graph();
        let sorted_nodes = toposort(digraph, None).expect("The graph is cyclic!");
        let execution_steps: Vec<_> = sorted_nodes
            .into_iter()
            .map(|node_index| {
                let node = digraph.node_weight(node_index).unwrap();
                ExecutionStep {
                    node_id: node_index.index(),
                    node_type: node.data().r#type.clone(),
                    node_state: node.data().state.clone(),
                }
            })
            .collect();
        Self {
            steps: execution_steps,
        }
    }

    pub async fn execute(&self, _runtime: &mut Runtime) {
        for step in &self.steps {
            step.execute();
        }
    }
}
