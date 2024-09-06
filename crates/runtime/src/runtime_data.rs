use std::collections::HashMap;

use wasmtime::component::Val;

#[derive(Debug, Clone)]
pub struct RuntimeData {
    data: HashMap<String, Val>,
}

impl RuntimeData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get_node_input(&self, node_id: &str) -> Option<&Val> {
        self.data.get(node_id)
    }
}
