use cillio_config::{load_config, GraphConfig};
use cillio_graph::Graph as CillioGraph;
use color_eyre::Result;
use ratatui::{layout::{Rect, Size}, Frame};
use tracing::info;

use super::Component;

use crate::action::Action;

// Import the GraphRenderer
use crate::graph_renderer::GraphRenderer;

#[derive(Default, Debug, Clone)]
pub struct GraphExplorer {
    graph_config: Option<GraphConfig>,
    graph: Option<CillioGraph>,
    renderer: Option<GraphRenderer>,
}

impl GraphExplorer {
    pub fn new() -> Self {
        Self {
            graph_config: None,
            graph: None,
            renderer: None,
        }
    }

    pub fn with_graph_config(graph_config: GraphConfig) -> Self {
        let graph = CillioGraph::new(&graph_config).unwrap();
        let renderer = GraphRenderer::new(&graph);
        Self {
            graph_config: Some(graph_config),
            graph: Some(graph),
            renderer: Some(renderer),
        }
    }

    pub fn set_graph_config(&mut self, graph_config: GraphConfig) {
        let graph = CillioGraph::new(&graph_config).unwrap();
        let renderer = GraphRenderer::new(&graph);
        self.graph_config = Some(graph_config);
        self.graph = Some(graph);
        self.renderer = Some(renderer);
    }

    // ... (other methods remain the same)
}

impl Component for GraphExplorer {
    fn init(&mut self, _: Size) -> Result<()> {
        info!("Graph explorer initialized");
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SetGraphConfigPath(path) => {
                info!("Setting graph config path: {:?}", path);
                let graph_config = load_config(path.to_str().unwrap())?;
                self.set_graph_config(graph_config);
            }
            _ => {}
        };
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let (Some(graph), Some(renderer)) = (self.graph.as_ref(), self.renderer.as_ref()) {
            renderer.render(graph, frame, area);
        } else {
            let paragraph = ratatui::widgets::Paragraph::new("No graph configuration set")
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(paragraph, area);
        }
        Ok(())
    }
}