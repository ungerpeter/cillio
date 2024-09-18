use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Rect, Size},
    Frame,
};
use tracing::error;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use crate::{action::Action, config::Config, tui::Event};

pub mod file_explorer;
pub mod fps;
pub mod graph_explorer;
pub mod home;

pub trait Component {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx; // to appease clippy
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        let _ = config; // to appease clippy
        Ok(())
    }

    fn init(&mut self, area: Size) -> Result<()> {
        let _ = area; // to appease clippy
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let _ = key; // to appease clippy
        Ok(None)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}

/// Manages all components in the application.
pub struct ComponentManager {
    components: HashMap<String, Box<dyn Component>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn register_component(&mut self, name: &str, component: Box<dyn Component>) {
        self.components.insert(name.to_string(), component);
    }

    pub fn init_components(&mut self, area: Size) -> Result<()> {
        for component in self.components.values_mut() {
            component.init(area)?;
        }
        Ok(())
    }

    pub fn register_action_handlers(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        for component in self.components.values_mut() {
            component.register_action_handler(action_tx.clone())?;
        }
        Ok(())
    }

    pub fn register_config_handlers(&mut self, config: Config) -> Result<()> {
        for component in self.components.values_mut() {
            component.register_config_handler(config.clone())?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self, event: Option<Event>, action_tx: UnboundedSender<Action>) -> Result<()> {
        for component in self.components.values_mut() {
            if let Some(action) = component.handle_events(event.clone())? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    pub fn handle_action(&mut self, action: Action, action_tx: UnboundedSender<Action>) -> Result<()> {
        for component in self.components.values_mut() {
            if let Some(new_action) = component.update(action.clone())? {
                action_tx.send(new_action)?;
            }
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        for component in self.components.values_mut() {
            if let Err(err) = component.render(frame, area) {
                error!("Failed to render component: {:?}", err);
            }
        }
        Ok(())
    }
}
