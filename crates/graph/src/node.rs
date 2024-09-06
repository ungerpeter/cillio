use cillio_config::NodeData;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub data: NodeData,
}

impl Node {
    pub fn new(id: String, data: NodeData) -> Self {
        Self { id, data }
    }
    pub fn data(&self) -> &NodeData {
        &self.data
    }
}