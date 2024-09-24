use std::collections::{HashMap, VecDeque};

use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::Widget,
    Frame,
};

use cillio_graph::{Edge, Graph, Node};

/// Defines the layout direction for the graph.
#[derive(Debug, Clone, Copy)]
pub enum LayoutDirection {
    LeftToRight,
    TopToBottom,
}

/// Struct representing the layout information of a node.
#[derive(Debug, Clone)]
struct NodeLayout {
    position: (u16, u16),
    size: (u16, u16),
    input_ports: Vec<(String, (u16, u16))>,
    output_ports: Vec<(String, (u16, u16))>,
}

#[derive(Debug, Clone)]
pub struct GraphRenderer {
    node_layouts: HashMap<NodeIndex, NodeLayout>,
    recompute_layout: bool,
    layout_direction: LayoutDirection,
}

impl GraphRenderer {
    /// Creates a new `GraphRenderer` with the specified layout direction.
    pub fn new(layout_direction: LayoutDirection) -> Self {
        Self {
            node_layouts: HashMap::new(),
            recompute_layout: true,
            layout_direction,
        }
    }

    /// Marks the renderer as needing to recompute positions.
    pub fn mark_dirty(&mut self) {
        self.recompute_layout = true;
    }

    /// Renders the graph onto the frame within the specified area.
    pub fn render(&mut self, graph: &Graph, frame: &mut Frame, area: Rect) {
        if self.recompute_layout {
            self.compute_layout(graph, area);
            self.recompute_layout = false;
        }

        let mut buffer = Buffer::empty(area);

        // Draw nodes
        for (&node_idx, node_layout) in &self.node_layouts {
            let node = &graph.graph()[node_idx];
            self.draw_node(&mut buffer, node, node_layout);
        }

        // Draw connections
        self.draw_connections(&mut buffer, graph);

        // Render the buffer
        frame.render_widget(AsciiArt { buffer }, area);
    }

    /// Computes the layout of the graph.
    fn compute_layout(&mut self, graph: &Graph, area: Rect) {
        self.node_layouts.clear();

        // Assign levels to nodes
        let levels = self.assign_levels(graph);

        // Layout nodes and compute port positions
        self.layout_nodes(graph, area, &levels);
    }

    /// Assigns levels to nodes using BFS for hierarchical layout.
    fn assign_levels(&self, graph: &Graph) -> HashMap<NodeIndex, usize> {
        let mut levels = HashMap::new();
        let mut queue = VecDeque::new();

        // Calculate in-degrees for all nodes
        let mut in_degrees = HashMap::new();
        for node in graph.graph().node_indices() {
            in_degrees.insert(node, 0);
        }
        for edge in graph.graph().edge_references() {
            let target = edge.target();
            *in_degrees.entry(target).or_insert(0) += 1;
        }

        // Identify root nodes (in-degree 0)
        for (&node, &degree) in &in_degrees {
            if degree == 0 {
                levels.insert(node, 0);
                queue.push_back(node);
            }
        }

        // Assign levels using BFS
        while let Some(node) = queue.pop_front() {
            let level = levels[&node];
            for neighbor in graph.graph().neighbors_directed(node, Direction::Outgoing) {
                if !levels.contains_key(&neighbor) {
                    levels.insert(neighbor, level + 1);
                    queue.push_back(neighbor);
                }
            }
        }

        levels
    }

    /// Layouts nodes and computes port positions.
    fn layout_nodes(&mut self, graph: &Graph, area: Rect, levels: &HashMap<NodeIndex, usize>) {
        // Group nodes by level
        let mut nodes_by_level: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
        for (&node, &level) in levels {
            nodes_by_level.entry(level).or_default().push(node);
        }

        let node_spacing = 4;
        let mut x_offset = 0;

        for level in 0.. {
            if let Some(nodes) = nodes_by_level.get(&level) {
                let mut y_offset = 0;
                let mut max_width = 0;

                for &node_idx in nodes {
                    let node = &graph.graph()[node_idx];
                    let title = format!("{} ({})", node.id, node.data.r#type);
                    let title_len = title.len() as u16;

                    // Collect ports for the node
                    let (input_ports, output_ports) = self.collect_ports(graph, node_idx);

                    // Calculate node dimensions
                    let port_count = input_ports.len().max(output_ports.len()) as u16;
                    let node_height = 3 + port_count; // Base height + ports
                    let node_width = title_len.max(10) + 4; // Ensure minimum width

                    if x_offset + node_width > area.width || y_offset + node_height > area.height {
                        continue;
                    }

                    // Compute port positions within the node
                    let (input_port_positions, output_port_positions) = self.compute_port_positions(
                        x_offset,
                        y_offset,
                        node_width,
                        port_count,
                        &input_ports,
                        &output_ports,
                    );

                    self.node_layouts.insert(
                        node_idx,
                        NodeLayout {
                            position: (x_offset, y_offset),
                            size: (node_width, node_height),
                            input_ports: input_port_positions,
                            output_ports: output_port_positions,
                        },
                    );

                    y_offset += node_height + node_spacing;
                    max_width = max_width.max(node_width);
                }

                x_offset += max_width + node_spacing;

                if x_offset > area.width {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Collects ports for a node.
    fn collect_ports(&self, graph: &Graph, node_idx: NodeIndex) -> (Vec<String>, Vec<String>) {
        let mut input_ports = Vec::new();
        let mut output_ports = Vec::new();

        // Collect input ports
        for edge in graph.graph().edges_directed(node_idx, Direction::Incoming) {
            if let Some(port) = &edge.weight().to_port {
                if !input_ports.contains(port) {
                    input_ports.push(port.clone());
                }
            }
        }

        // Collect output ports
        for edge in graph.graph().edges_directed(node_idx, Direction::Outgoing) {
            if let Some(port) = &edge.weight().from_port {
                if !output_ports.contains(port) {
                    output_ports.push(port.clone());
                }
            }
        }

        (input_ports, output_ports)
    }

    /// Computes port positions within a node.
    fn compute_port_positions(
        &self,
        x_offset: u16,
        y_offset: u16,
        node_width: u16,
        port_count: u16,
        input_ports: &[String],
        output_ports: &[String],
    ) -> (
        Vec<(String, (u16, u16))>,
        Vec<(String, (u16, u16))>,
    ) {
        let mut input_port_positions = Vec::new();
        let mut output_port_positions = Vec::new();
        let mut port_y = y_offset + 2;

        for i in 0..port_count as usize {
            if i < input_ports.len() {
                let port_name = &input_ports[i];
                let port_x = x_offset + 2; // Left side of the node
                input_port_positions.push((port_name.clone(), (port_x, port_y)));
            }

            if i < output_ports.len() {
                let port_name = &output_ports[i];
                let port_x = x_offset + node_width - 3; // Right side of the node
                output_port_positions.push((port_name.clone(), (port_x, port_y)));
            }

            port_y += 1;
        }

        (input_port_positions, output_port_positions)
    }

    /// Draws a node.
    fn draw_node(&self, buffer: &mut Buffer, node: &Node, layout: &NodeLayout) {
        let style = Style::default().bg(Color::DarkGray).fg(Color::White);
        let (x, y) = layout.position;
        let (width, height) = layout.size;

        // Fill node area
        for dy in 0..height {
            for dx in 0..width {
                buffer
                    .cell_mut(Position::new(x + dx, y + dy))
                    .unwrap()
                    .set_symbol(" ")
                    .set_style(style);
            }
        }

        // Draw borders
        self.draw_borders(buffer, x, y, width, height, style);

        // Draw title
        let title = format!("{} ({})", node.id, node.data.r#type);
        let title_x = x + (width - title.len() as u16) / 2;
        buffer.set_stringn(title_x, y + 1, &title, title.len(), style);

        // Draw ports
        let mut port_y = y + 2;
        let port_count = layout.input_ports.len().max(layout.output_ports.len());

        for i in 0..port_count {
            if i < layout.input_ports.len() {
                let (port_name, _) = &layout.input_ports[i];
                let port_line = format!("{:<width$}", port_name, width = ((width / 2) - 2) as usize);
                buffer.set_stringn(x + 2, port_y, &port_line, port_line.len(), style);
            }

            if i < layout.output_ports.len() {
                let (port_name, _) = &layout.output_ports[i];
                let port_line = format!("{:>width$}", port_name, width = ((width / 2) - 2) as usize);
                buffer.set_stringn(x + width / 2, port_y, &port_line, port_line.len(), style);
            }

            port_y += 1;
        }
    }

    /// Draws the borders of a node.
    fn draw_borders(
        &self,
        buffer: &mut Buffer,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        style: Style,
    ) {
        // Top border
        buffer.set_stringn(x, y, "╭", 1, style);
        buffer.set_stringn(x + width - 1, y, "╮", 1, style);
        for dx in 1..(width - 1) {
            buffer.set_stringn(x + dx, y, "─", 1, style);
        }

        // Bottom border
        buffer.set_stringn(x, y + height - 1, "╰", 1, style);
        buffer.set_stringn(x + width - 1, y + height - 1, "╯", 1, style);
        for dx in 1..(width - 1) {
            buffer.set_stringn(x + dx, y + height - 1, "─", 1, style);
        }

        // Side borders
        for dy in (y + 1)..(y + height - 1) {
            buffer.set_stringn(x, dy, "│", 1, style);
            buffer.set_stringn(x + width - 1, dy, "│", 1, style);
        }
    }

    /// Draws all connections between nodes.
    fn draw_connections(&self, buffer: &mut Buffer, graph: &Graph) {
        let style = Style::default().fg(Color::White);

        for edge in graph.graph().edge_references() {
            let source_idx = edge.source();
            let target_idx = edge.target();
            let edge_data = edge.weight();

            let source_layout = &self.node_layouts[&source_idx];
            let target_layout = &self.node_layouts[&target_idx];

            // Get port positions
            let source_port = edge_data.from_port.clone().unwrap_or_default();
            let target_port = edge_data.to_port.clone().unwrap_or_default();

            let (sx, sy) = source_layout
                .output_ports
                .iter()
                .find(|(name, _)| name == &source_port)
                .map(|(_, pos)| *pos)
                .unwrap_or_else(|| {
                    (
                        source_layout.position.0 + source_layout.size.0 - 1,
                        source_layout.position.1 + source_layout.size.1 / 2,
                    )
                });

            let (tx, ty) = target_layout
                .input_ports
                .iter()
                .find(|(name, _)| name == &target_port)
                .map(|(_, pos)| *pos)
                .unwrap_or_else(|| {
                    (
                        target_layout.position.0,
                        target_layout.position.1 + target_layout.size.1 / 2,
                    )
                });

            // Draw connection line
            self.draw_connection_line(buffer, (sx, sy), (tx, ty), style);
        }
    }

    /// Draws a connection line between two points.
    fn draw_connection_line(
        &self,
        buffer: &mut Buffer,
        start: (u16, u16),
        end: (u16, u16),
        style: Style,
    ) {
        let (sx, sy) = start;
        let (ex, ey) = end;

        // Adjust for buffer boundaries
        let ex = ex;
        let ey = ey;

        // Determine the midpoints for the L-shaped connection
        let mid_x = ex;
        let mid_y = sy;

        // Horizontal line from (sx, sy) to (mid_x, sy)
        let x_range = if sx <= mid_x { sx..=mid_x } else { mid_x..=sx };
        for x in x_range {
            buffer
                .cell_mut(Position::new(x, sy))
                .unwrap()
                .set_symbol("─")
                .set_style(style);
        }

        // Vertical line from (mid_x, sy) to (mid_x, ey)
        let y_range = if sy <= ey { sy..=ey } else { ey..=sy };
        for y in y_range {
            buffer
                .cell_mut(Position::new(mid_x, y))
                .unwrap()
                .set_symbol("│")
                .set_style(style);
        }
    }
}

/// A widget to render the ASCII art buffer.
struct AsciiArt {
    buffer: Buffer,
}

impl Widget for AsciiArt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.merge(&self.buffer);
    }
}