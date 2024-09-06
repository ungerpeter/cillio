#[derive(Debug)]
pub struct Edge {
    pub from_port: Option<String>,
    pub to_port: Option<String>,
}

impl Edge {
    pub fn new(from_port: Option<String>, to_port: Option<String>) -> Self {
        Self { from_port, to_port }
    }
}