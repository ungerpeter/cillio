use std::collections::HashMap;
use std::f64::consts::PI;

use petgraph::{graph::NodeIndex, visit::EdgeRef};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Block, Borders, canvas::Canvas, canvas::Line},
    Frame,
};
use cillio_graph::Graph;

#[derive(Debug, Clone)]
pub struct GraphRenderer {
    positions: HashMap<NodeIndex, (f64, f64)>,
}

impl GraphRenderer {
    /// Creates a new GraphRenderer with computed node positions.
    pub fn new(graph: &Graph) -> Self {
        let positions = Self::compute_positions(graph);
        Self { positions }
    }

    /// Computes the positions of nodes using a circular layout.
    fn compute_positions(graph: &Graph) -> HashMap<NodeIndex, (f64, f64)> {
        let mut positions = HashMap::new();
        let node_indices: Vec<NodeIndex> = graph.graph().node_indices().collect();
        let n = node_indices.len();

        let radius = 10.0;
        let center_x = 0.0;
        let center_y = 0.0;

        for (i, &node_idx) in node_indices.iter().enumerate() {
            let angle = (i as f64) * 2.0 * PI / (n as f64);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            positions.insert(node_idx, (x, y));
        }

        positions
    }

    /// Computes canvas bounds based on node positions.
    fn compute_bounds(
        &self,
    ) -> ((f64, f64), (f64, f64)) {
        let xs: Vec<f64> = self.positions.values().map(|(x, _)| *x).collect();
        let ys: Vec<f64> = self.positions.values().map(|(_, y)| *y).collect();
        let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        ((x_min, x_max), (y_min, y_max))
    }

    /// Renders the graph using Ratatui.
    pub fn render(&self, graph: &Graph, f: &mut Frame, area: Rect) {
        let ((x_min, x_max), (y_min, y_max)) = self.compute_bounds();

        let canvas = Canvas::default()
            .block(Block::default().title("Graph View").borders(Borders::ALL))
            .x_bounds([x_min - 1.0, x_max + 1.0])
            .y_bounds([y_min - 1.0, y_max + 1.0])
            .paint(|ctx| {
                // Draw edges
                for edge in graph.graph().edge_references() {
                    let source = edge.source();
                    let target = edge.target();

                    if let (Some(&(x1, y1)), Some(&(x2, y2))) =
                        (self.positions.get(&source), self.positions.get(&target))
                    {
                        // Edge label with from_port and to_port
                        let edge_weight = edge.weight();
                        let edge_label = format!(
                            "{} -> {}",
                            edge_weight.from_port.clone().unwrap_or_default(),
                            edge_weight.to_port.clone().unwrap_or_default()
                        );

                        // Draw the edge line
                        ctx.draw(&Line {
                            x1,
                            y1,
                            x2,
                            y2,
                            color: Color::White,
                        });

                        // Draw edge label at midpoint
                        if !edge_label.trim().is_empty() {
                            let mid_x = (x1 + x2) / 2.0;
                            let mid_y = (y1 + y2) / 2.0;
                            ctx.print(
                                mid_x,
                                mid_y,
                                Span::styled(edge_label, Style::default().fg(Color::Gray)),
                            );
                        }
                    }
                }

                // Draw nodes
                for node_idx in graph.graph().node_indices() {
                    if let Some(&(x, y)) = self.positions.get(&node_idx) {
                        let node = &graph.graph()[node_idx];
                        let node_label = format!("{} ({})", node.id, node.data.r#type);

                        // Draw node symbol
                        ctx.print(
                            x,
                            y,
                            Span::styled(
                                symbols::Marker::Braille.to_string(),
                                Style::default().fg(Color::Yellow),
                            ),
                        );

                        // Draw node label
                        ctx.print(
                            x + 0.5,
                            y,
                            Span::styled(node_label, Style::default().fg(Color::Yellow)),
                        );
                    }
                }
            });

        f.render_widget(canvas, area);
    }
}