use futures_signals::signal::Mutable;
use cillio_config::GraphConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub graph_config_path: Mutable<Option<PathBuf>>,
    pub graph_config: Mutable<Option<GraphConfig>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            graph_config_path: Mutable::new(None),
            graph_config: Mutable::new(None),
        }
    }
}
