//! Basic example of using the grid library for tree rendering
//!
//! This example demonstrates the recommended usage pattern:
//! 1. Build a tree using TreeBuilder
//! 2. Compute layout using LayoutEngine
//! 3. Render with viewport transformation using render_tree_with_edges
//!
//! Run with: cargo run --example basic_rendering --features rendering

use grid::{LayoutConfig, LayoutEngine, Position, Tree, TreeBuilder, Viewport};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    Terminal,
};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build a sample tree
    let tree = build_sample_tree()?;

    println!("Tree structure:");
    print_tree(&tree, 0, "");

    // 2. Configure layout
    let config = LayoutConfig::new()
        .with_node_size(3.0, 3.0) // Width, Height
        .with_spacing(2.0, 6.0, 2.0); // Node separation, level separation, subtree separation

    // 3. Compute layout
    let mut engine = LayoutEngine::with_config(config)?;
    let layout = engine.compute(&tree)?;

    println!("\nLayout computed:");
    println!("  Bounds: {:?}", layout.bounds());
    println!("  Edges: {} computed", layout.edges().len());

    // 4. Set up viewport (e.g., centered on root)
    let viewport = Viewport::default();

    // 5. Initialize terminal for rendering
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        let area = frame.area();

        // Create a centered viewport
        let root_pos = layout.position(0).unwrap_or(Position::new(0.0, 0.0));
        let centered_viewport = Viewport::with_offset(Position::new(
            root_pos.x - (area.width as f32 / 2.0),
            root_pos.y - (area.height as f32 / 2.0),
        ));

        // Use the high-level render function
        grid::render_tree_with_edges(
            frame.buffer_mut(),
            &tree,
            &layout,
            &centered_viewport,
            area,
            Style::default().fg(Color::Cyan),
            |buf, _node_id, data, screen_pos, area, _style| {
                render_node(buf, data, screen_pos, area);
            },
        );
    })?;

    // Keep terminal displayed briefly
    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}

/// Build a sample tree for demonstration
fn build_sample_tree() -> Result<Tree<String>, Box<dyn std::error::Error>> {
    let tree = TreeBuilder::new("Root".to_string())
        .add_child(0, "Child 1".to_string())?
        .add_child(0, "Child 2".to_string())?
        .add_child(0, "Child 3".to_string())?
        .add_child(1, "Grandchild 1.1".to_string())?
        .add_child(1, "Grandchild 1.2".to_string())?
        .add_child(2, "Grandchild 2.1".to_string())?
        .add_child(5, "Great-grandchild 2.1.1".to_string())?
        .build()?;

    Ok(tree)
}

/// Print tree structure for debugging
fn print_tree<T: std::fmt::Display>(tree: &Tree<T>, node_id: usize, prefix: &str) {
    if let Ok(node) = tree.node(node_id) {
        println!("{}{} (id: {})", prefix, node.data, node_id);
        for &child_id in &node.children {
            print_tree(tree, child_id, &format!("{}  ", prefix));
        }
    }
}

/// Render a single node (simple box with text)
fn render_node(buf: &mut Buffer, data: &String, screen_pos: Position, area: Rect) {
    let x = screen_pos.x as i32;
    let y = screen_pos.y as i32;

    // Only render if within bounds
    if x >= 0 && x < area.width as i32 && y >= 0 && y < area.height as i32 {
        let base_x = area.x + x as u16;
        let base_y = area.y + y as u16;

        // Draw a simple 3x3 box
        if base_y < area.y + area.height {
            // Top line
            buf.set_string(base_x, base_y, "┌─┐", Style::default());
        }
        if base_y + 1 < area.y + area.height {
            // Middle line with truncated text
            let text = if data.len() > 1 { &data[0..1] } else { data };
            buf.set_string(base_x, base_y + 1, &format!("│{}│", text), Style::default());
        }
        if base_y + 2 < area.y + area.height {
            // Bottom line
            buf.set_string(base_x, base_y + 2, "└─┘", Style::default());
        }
    }
}
