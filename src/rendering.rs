//! Optional rendering utilities for ratatui
//!
//! This module is only available when the "rendering" feature is enabled.

use crate::{
    EdgePath, EdgeSegment, EdgeSegmentType, LayoutBounds, LayoutResult, Position, Tree, Viewport,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Style};

/// Render orthogonal edges for a tree layout
pub fn render_tree_edges<T>(
    buf: &mut Buffer,
    _tree: &Tree<T>,
    layout: &LayoutResult,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    for edge in layout.edges() {
        render_edge_path(buf, edge, viewport, area, style);
    }
}

/// Render a single edge path
pub fn render_edge_path(
    buf: &mut Buffer,
    path: &EdgePath,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    // Render segments with awareness of adjacent corners to avoid overlaps
    for (i, segment) in path.segments.iter().enumerate() {
        let has_corner_before = i > 0 && matches!(path.segments[i - 1].segment_type, EdgeSegmentType::Corner(_));
        let has_corner_after = i + 1 < path.segments.len() && matches!(path.segments[i + 1].segment_type, EdgeSegmentType::Corner(_));
        
        render_edge_segment_with_context(buf, segment, has_corner_before, has_corner_after, viewport, area, style);
    }
}

/// Render a single edge segment with context about adjacent corners
fn render_edge_segment_with_context(
    buf: &mut Buffer,
    segment: &EdgeSegment,
    has_corner_before: bool,
    has_corner_after: bool,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    use EdgeSegmentType::*;

    match segment.segment_type {
        Horizontal => render_horizontal_line_with_context(
            buf, 
            segment.from, 
            segment.to, 
            has_corner_before, 
            has_corner_after, 
            viewport, 
            area, 
            style
        ),
        Vertical => render_vertical_line_with_context(
            buf, 
            segment.from, 
            segment.to, 
            has_corner_before, 
            has_corner_after, 
            viewport, 
            area, 
            style
        ),
        Corner(corner_type) => render_corner(buf, segment.from, corner_type, viewport, area, style),
    }
}

/// Render a single edge segment
pub fn render_edge_segment(
    buf: &mut Buffer,
    segment: &EdgeSegment,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    use EdgeSegmentType::*;

    match segment.segment_type {
        Horizontal => render_horizontal_line(buf, segment.from, segment.to, viewport, area, style),
        Vertical => render_vertical_line(buf, segment.from, segment.to, viewport, area, style),
        Corner(corner_type) => render_corner(buf, segment.from, corner_type, viewport, area, style),
    }
}

fn render_horizontal_line_with_context(
    buf: &mut Buffer,
    from: crate::Position,
    to: crate::Position,
    has_corner_before: bool,
    has_corner_after: bool,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    // Transform world coordinates to screen coordinates
    let from_screen = viewport.world_to_screen(from);
    let to_screen = viewport.world_to_screen(to);

    let y = from_screen.y as i32;
    let mut start_x = from_screen.x.min(to_screen.x) as i32;
    let mut end_x = from_screen.x.max(to_screen.x) as i32;

    // Skip endpoints if they have adjacent corners
    if has_corner_before {
        start_x += 1;
    }
    if has_corner_after {
        end_x -= 1;
    }

    for x in start_x..=end_x {
        if x >= 0 && x < area.width as i32 && y >= 0 && y < area.height as i32 {
            buf.set_string(area.x + x as u16, area.y + y as u16, "─", style);
        }
    }
}

fn render_horizontal_line(
    buf: &mut Buffer,
    from: crate::Position,
    to: crate::Position,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    // Transform world coordinates to screen coordinates
    let from_screen = viewport.world_to_screen(from);
    let to_screen = viewport.world_to_screen(to);

    let y = from_screen.y as i32;
    let start_x = from_screen.x.min(to_screen.x) as i32;
    let end_x = from_screen.x.max(to_screen.x) as i32;

    for x in start_x..=end_x {
        if x >= 0 && x < area.width as i32 && y >= 0 && y < area.height as i32 {
            buf.set_string(area.x + x as u16, area.y + y as u16, "─", style);
        }
    }
}

fn render_vertical_line(
    buf: &mut Buffer,
    from: crate::Position,
    to: crate::Position,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    // Transform world coordinates to screen coordinates
    let from_screen = viewport.world_to_screen(from);
    let to_screen = viewport.world_to_screen(to);

    let x = from_screen.x as i32;
    let start_y = from_screen.y.min(to_screen.y) as i32;
    let end_y = from_screen.y.max(to_screen.y) as i32;

    // Skip endpoints (start_y+1 to end_y-1) to avoid overwriting corner characters
    // Corners are rendered separately as their own segments
    for y in (start_y + 1)..end_y {
        if x >= 0 && x < area.width as i32 && y >= 0 && y < area.height as i32 {
            buf.set_string(area.x + x as u16, area.y + y as u16, "│", style);
        }
    }
}

fn render_vertical_line_with_context(
    buf: &mut Buffer,
    from: crate::Position,
    to: crate::Position,
    has_corner_before: bool,
    has_corner_after: bool,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    // Transform world coordinates to screen coordinates
    let from_screen = viewport.world_to_screen(from);
    let to_screen = viewport.world_to_screen(to);

    let x = from_screen.x as i32;
    let mut start_y = from_screen.y.min(to_screen.y) as i32;
    let mut end_y = from_screen.y.max(to_screen.y) as i32;

    // Skip endpoints if they have adjacent corners
    if has_corner_before {
        start_y += 1;
    }
    if has_corner_after {
        end_y -= 1;
    }

    for y in start_y..=end_y {
        if x >= 0 && x < area.width as i32 && y >= 0 && y < area.height as i32 {
            buf.set_string(area.x + x as u16, area.y + y as u16, "│", style);
        }
    }
}

fn render_corner(
    buf: &mut Buffer,
    pos: crate::Position,
    corner_type: crate::CornerType,
    viewport: &Viewport,
    area: Rect,
    style: Style,
) {
    use crate::CornerType::*;

    // Transform world coordinates to screen coordinates
    let screen_pos = viewport.world_to_screen(pos);
    let x = screen_pos.x as i32;
    let y = screen_pos.y as i32;

    if x >= 0 && x < area.width as i32 && y >= 0 && y < area.height as i32 {
        let char = match corner_type {
            TopRight => "┐",
            TopLeft => "┌",
            BottomRight => "┘",
            BottomLeft => "└",
        };
        buf.set_string(area.x + x as u16, area.y + y as u16, char, style);
    }
}

/// Render complete tree with edges and nodes
///
/// This is a high-level helper that renders both edges and nodes in the correct order.
/// Edges are drawn first (behind nodes), then nodes are rendered using the provided callback.
///
/// # Arguments
/// * `buf` - The ratatui buffer to draw into
/// * `tree` - The tree structure being rendered
/// * `layout` - The computed layout result with positions and edges
/// * `viewport` - The viewport for coordinate transformation
/// * `area` - The screen area to render within
/// * `edge_style` - The style to use for drawing edges
/// * `render_node` - Callback to render each visible node. Receives:
///   - `buf`: Buffer to draw into
///   - `node_id`: ID of the node being rendered
///   - `data`: Reference to the node's data
///   - `screen_pos`: Position on screen (already viewport-transformed)
///   - `area`: Screen area bounds
///   - `style`: Style to use for the node
///
/// # Example
/// ```no_run
/// use grid::{Tree, LayoutEngine, Viewport, Position};
/// use ratatui::{buffer::Buffer, layout::Rect, style::Style};
///
/// # fn example(buf: &mut Buffer, tree: &Tree<String>, layout: &grid::LayoutResult, area: Rect) {
/// let viewport = Viewport::default();
/// let edge_style = Style::default();
///
/// grid::render_tree_with_edges(
///     buf,
///     tree,
///     layout,
///     &viewport,
///     area,
///     edge_style,
///     |buf, node_id, data, screen_pos, area, style| {
///         // Custom node rendering logic here
///         let x = screen_pos.x as u16;
///         let y = screen_pos.y as u16;
///         if x < area.width && y < area.height {
///             buf.set_string(area.x + x, area.y + y, data, style);
///         }
///     },
/// );
/// # }
/// ```
pub fn render_tree_with_edges<T, F>(
    buf: &mut Buffer,
    tree: &Tree<T>,
    layout: &LayoutResult,
    viewport: &Viewport,
    area: Rect,
    edge_style: Style,
    render_node: F,
) where
    F: Fn(&mut Buffer, usize, &T, Position, Rect, Style),
{
    // 1. Render edges first (behind nodes)
    render_tree_edges(buf, tree, layout, viewport, area, edge_style);

    // 2. Render nodes using callback
    let viewport_layout = layout.with_viewport(viewport);
    let screen_bounds = LayoutBounds::new(0.0, area.width as f32, 0.0, area.height as f32);

    for (node_id, screen_pos) in viewport_layout.iter_visible(screen_bounds) {
        if let Ok(node) = tree.node(node_id) {
            render_node(buf, node_id, &node.data, screen_pos, area, Style::default());
        }
    }
}
