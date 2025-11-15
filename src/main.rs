use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use grid::{LayoutConfig, LayoutManager, TreeNode, TreeStructure};
use ratatui::{
    style::{Color, Style},
    text::Text,
    widgets::{Block, Paragraph},
};

// Main Application Structure
struct TreeApp {
    tree: TreeStructure,
    layout_manager: LayoutManager,
}

fn main() -> std::io::Result<()> {
    // Initialize the terminal
    let mut terminal = ratatui::init();

    // Initialize tree structure with a single root node
    let tree_structure = TreeStructure::new();

    // Initialize layout manager
    let layout_manager = LayoutManager::new(LayoutConfig::default());
    // Create the main application
    let mut app = TreeApp {
        tree: tree_structure,
        layout_manager,
    };

    let result = run(&mut terminal, &mut app);

    // Restore the terminal
    ratatui::restore();

    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut TreeApp) -> std::io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Ensure layout is valid before drawing
            app.layout_manager
                .ensure_layout_valid(&app.tree, (area.width, area.height));

            // Draw connections first (so they appear behind nodes)
            for (node_idx, node) in app.tree.nodes.iter().enumerate() {
                for &child_idx in &node.children {
                    if child_idx < app.tree.nodes.len() {
                        if let (Some(parent_pos), Some(child_pos)) = (
                            app.layout_manager.get_node_position(node_idx),
                            app.layout_manager.get_node_position(child_idx),
                        ) {
                            draw_connection(frame, parent_pos, child_pos, area);
                        }
                    }
                }
            }

            // Draw nodes using layout positions
            for (idx, node) in app.tree.nodes.iter().enumerate() {
                if let Some((x, y)) = app.layout_manager.get_node_position(idx) {
                    let is_current = idx == app.tree.current_node;
                    draw_node_at_position(frame, node, x, y, is_current, area);
                }
            }
        })?;

        // Handle events
        if handle_events(app)? {
            break;
        }
    }

    Ok(())
}

fn draw_node_at_position(
    frame: &mut ratatui::Frame,
    node: &TreeNode,
    x: i32,
    y: i32,
    is_current: bool,
    area: ratatui::layout::Rect,
) {
    // Calculate node dimensions - width based on label, height is 3 (border + text + border)
    let node_width = (node.label.len() + 2) as u16; // +2 for left and right borders
    let node_height = 3u16; // Single line node with top and bottom borders

    // Check if the node is within the visible area
    if x < 0 || y < 0 || x as u16 >= area.width || y as u16 >= area.height {
        return; // Node is completely outside visible area
    }

    // Choose style based on whether this is the current node
    let style = if is_current {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Cyan)
    };

    // Create a block (rectangle) with the node label inside
    let block = Block::bordered().style(style);
    
    // Create paragraph with the label
    let paragraph = Paragraph::new(node.label.as_str())
        .block(block)
        .style(style);

    // Calculate the area for this node
    let node_area = ratatui::layout::Rect {
        x: x as u16,
        y: y as u16,
        width: node_width.min(area.width.saturating_sub(x as u16)),
        height: node_height.min(area.height.saturating_sub(y as u16)),
    };

    // Render the widget
    frame.render_widget(paragraph, node_area);
}

fn draw_connection(
    frame: &mut ratatui::Frame,
    from_pos: (i32, i32),
    to_pos: (i32, i32),
    area: ratatui::layout::Rect,
) {
    // Adjust connection points for the new 3-high node structure
    // Connect from the right middle of the source node
    let from_x = from_pos.0 + 2; // Connect from right edge 
    let from_y = from_pos.1 + 1; // Middle of 3-high node
    let to_x = to_pos.0; // Connect to left edge of target node
    let to_y = to_pos.1 + 1; // Middle of 3-high node

    // Draw horizontal line from source to intermediate point
    let mid_x = (from_x + to_x) / 2;

    // Horizontal line from source node
    for x in (from_x + 1)..mid_x {
        if x >= 0 && x < area.width as i32 && from_y >= 0 && from_y < area.height as i32 {
            let text = Text::from("─").style(Color::Green);
            let line_area = ratatui::layout::Rect {
                x: x as u16,
                y: from_y as u16,
                width: 1,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }

    // Vertical line (if needed)
    let min_y = from_y.min(to_y);
    let max_y = from_y.max(to_y);

    if from_y != to_y {
        for y in min_y..=max_y {
            if mid_x >= 0 && mid_x < area.width as i32 && y >= 0 && y < area.height as i32 {
                let char = if y == min_y && from_y < to_y {
                    "┐"
                } else if y == max_y && from_y > to_y {
                    "┘"
                } else if y == min_y && from_y > to_y {
                    "┌"
                } else if y == max_y && from_y < to_y {
                    "└"
                } else {
                    "│"
                };

                let text = Text::from(char).style(Color::Green);
                let line_area = ratatui::layout::Rect {
                    x: mid_x as u16,
                    y: y as u16,
                    width: 1,
                    height: 1,
                };
                frame.render_widget(text, line_area);
            }
        }
    }

    // Horizontal line to target node
    for x in (mid_x + 1)..to_x {
        if x >= 0 && x < area.width as i32 && to_y >= 0 && to_y < area.height as i32 {
            let text = Text::from("─").style(Color::Green);
            let line_area = ratatui::layout::Rect {
                x: x as u16,
                y: to_y as u16,
                width: 1,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }
}

fn handle_events(app: &mut TreeApp) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),

            // Navigation with hjkl keys (node selection)
            KeyCode::Char('h') => move_left(app),
            KeyCode::Char('j') => move_down(app),
            KeyCode::Char('k') => move_up(app),
            KeyCode::Char('l') => move_right(app),

            // Graph movement with arrow keys (viewport movement)
            KeyCode::Left => move_graph_left(app),
            KeyCode::Down => move_graph_down(app),
            KeyCode::Up => move_graph_up(app),
            KeyCode::Right => move_graph_right(app),

            // Reset viewport
            KeyCode::Char('r') => reset_graph_position(app),

            // Expand node with different numbers of children
            KeyCode::Char('1') => expand_current_node(app, 1),
            KeyCode::Char('2') => expand_current_node(app, 2),
            KeyCode::Char('3') => expand_current_node(app, 3),
            KeyCode::Char('4') => expand_current_node(app, 4),

            // Keep Enter for backward compatibility (2 children)
            KeyCode::Enter => expand_current_node(app, 2),

            _ => {}
        },
        _ => {}
    }
    Ok(false)
}

fn move_left(app: &mut TreeApp) {
    // Find parent of current node
    for (parent_idx, parent) in app.tree.nodes.iter().enumerate() {
        if parent.children.contains(&app.tree.current_node) {
            app.tree.current_node = parent_idx;
            return;
        }
    }
}

fn move_right(app: &mut TreeApp) {
    // Move to first child if available
    let current = &app.tree.nodes[app.tree.current_node];
    if !current.children.is_empty() {
        app.tree.current_node = current.children[0];
    }
}

fn move_up(app: &mut TreeApp) {
    // Find siblings and move to previous sibling
    for parent in &app.tree.nodes {
        if let Some(current_pos) = parent
            .children
            .iter()
            .position(|&x| x == app.tree.current_node)
        {
            if current_pos > 0 {
                app.tree.current_node = parent.children[current_pos - 1];
                return;
            }
        }
    }
}

fn move_down(app: &mut TreeApp) {
    // Find siblings and move to next sibling
    for parent in &app.tree.nodes {
        if let Some(current_pos) = parent
            .children
            .iter()
            .position(|&x| x == app.tree.current_node)
        {
            if current_pos + 1 < parent.children.len() {
                app.tree.current_node = parent.children[current_pos + 1];
                return;
            }
        }
    }
}

fn expand_current_node(app: &mut TreeApp, num_children: usize) {
    // Use the tree's expand method with specified number of children
    app.tree.expand_node(app.tree.current_node, num_children);

    // Mark layout as needing recalculation
    app.layout_manager.mark_subtree_dirty(app.tree.current_node);
}

// graph movement functions (move entire viewport)
fn move_graph_left(app: &mut TreeApp) {
    app.layout_manager.move_viewport(-3, 0);
}

fn move_graph_right(app: &mut TreeApp) {
    app.layout_manager.move_viewport(3, 0);
}

fn move_graph_up(app: &mut TreeApp) {
    app.layout_manager.move_viewport(0, -2);
}

fn move_graph_down(app: &mut TreeApp) {
    app.layout_manager.move_viewport(0, 2);
}

fn reset_graph_position(app: &mut TreeApp) {
    app.layout_manager.reset_viewport();
}
