use std::path::PathBuf;
use cillio_config::GraphConfig;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    SwitchToHome,
    SwitchToGraphExplorer,
    SwitchToFileExplorer,
    //---
    SetGraphConfigPath(PathBuf),
    SetGraphConfig(GraphConfig),
}
