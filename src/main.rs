use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use grid::{LayoutConfig, LayoutManager, TreeNode, TreeStructure};
use ratatui::{style::Color, text::Text};

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
    let node_size = 3; // 3x3 square

    // Check if the node is within the visible area (with some margin for partial visibility)
    if x < -(node_size as i32)
        || y < -(node_size as i32)
        || x >= area.width as i32
        || y >= area.height as i32
    {
        return; // Node is completely outside visible area
    }

    // Choose color based on whether this is the current node
    let color = if is_current {
        Color::Green
    } else {
        Color::Cyan
    };

    // Draw the square node
    let lines = ["┌─┐", "│ │", "└─┘"];

    for (i, line) in lines.iter().enumerate() {
        let y_pos = y + i as i32;
        if y_pos >= 0 && y_pos < area.height as i32 && x >= 0 && x < area.width as i32 {
            let text = Text::from(line.to_string()).style(color);
            let line_area = ratatui::layout::Rect {
                x: x as u16,
                y: y_pos as u16,
                width: node_size.min((area.width as i32 - x).max(0) as u16),
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }

    // Draw the label below the node
    let label_y = y + node_size as i32;
    if label_y >= 0 && label_y < area.height as i32 && x >= 0 && x < area.width as i32 {
        let label_color = if is_current {
            Color::Green
        } else {
            Color::Yellow
        };
        let label_text = Text::from(node.label.clone()).style(label_color);
        let label_width = node.label.len() as u16;
        let label_area = ratatui::layout::Rect {
            x: x as u16,
            y: label_y as u16,
            width: label_width.min((area.width as i32 - x).max(0) as u16),
            height: 1,
        };
        frame.render_widget(label_text, label_area);
    }
}

fn draw_connection(
    frame: &mut ratatui::Frame,
    from_pos: (i32, i32),
    to_pos: (i32, i32),
    area: ratatui::layout::Rect,
) {
    let from_x = from_pos.0 + 2; // Connect from right edge of source node (middle of 3-wide node)
    let from_y = from_pos.1 + 1; // Connect from middle of source node (middle of 3-high node)
    let to_x = to_pos.0; // Connect to left edge of target node
    let to_y = to_pos.1 + 1; // Connect to middle of target node

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
