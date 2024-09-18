use crate::{action::Action, state::AppState};
use redux_rs::Reducer;
use tracing::warn;

pub struct AppReducer;

impl Default for AppReducer {
    fn default() -> Self {
        Self
    }
}

impl Reducer<AppState, Action> for AppReducer {
    fn reduce(&self, state: AppState, action: Action) -> AppState {
        match action {
            Action::SetGraphConfigPath(path) => AppState {
                graph_config_path: Some(path),
                ..state
            },
            Action::SetGraphConfig(config) => AppState {
                graph_config: Some(config),
                ..state
            },
            _ => {
                warn!("Unhandled action: {:?}", action);
                state
            }
        }
    }
}
