use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{style::Color, text::Text};

#[derive(Clone)]
struct Node {
    x: u16,
    y: u16,
    label: String,
    children: Vec<usize>, // indices of children nodes
}

struct TreeState {
    nodes: Vec<Node>,
    current_node: usize,
    node_counter: usize,
    min_horizontal_spacing: u16,
    min_vertical_spacing: u16,
}

impl TreeState {
    fn new() -> Self {
        TreeState {
            nodes: vec![Node {
                x: 5,
                y: 10,
                label: "Root".to_string(),
                children: vec![],
            }],
            current_node: 0,
            node_counter: 1,
            min_horizontal_spacing: 12,  // Horizontal distance between levels
            min_vertical_spacing: 6,    // Minimum vertical distance between siblings
        }
    }

    fn check_collision(&self, x: u16, y: u16, exclude_idx: Option<usize>) -> bool {
        let node_size = 3;
        let min_gap = 1; // Minimum gap between nodes
        
        for (idx, node) in self.nodes.iter().enumerate() {
            if let Some(exclude) = exclude_idx {
                if idx == exclude {
                    continue;
                }
            }
            
            // Check if the proposed position would overlap with existing node
            let x_overlap = (x < node.x + node_size + min_gap) && 
                           (x + node_size + min_gap > node.x);
            let y_overlap = (y < node.y + node_size + min_gap) && 
                           (y + node_size + min_gap > node.y);
            
            if x_overlap && y_overlap {
                return true;
            }
        }
        false
    }

    fn has_any_collisions(&self) -> bool {
        for (i, node1) in self.nodes.iter().enumerate() {
            for (j, node2) in self.nodes.iter().enumerate() {
                if i != j && self.check_collision(node1.x, node1.y, Some(i)) {
                    return true;
                }
            }
        }
        false
    }

    fn recalculate_layout(&mut self, terminal_width: u16, terminal_height: u16) {
        if self.nodes.is_empty() {
            return;
        }

        // Build tree structure to understand parent-child relationships
        let mut levels: Vec<Vec<usize>> = Vec::new();
        let mut visited = vec![false; self.nodes.len()];
        
        // Start with root (index 0)
        levels.push(vec![0]);
        visited[0] = true;
        
        // Build levels using BFS
        let mut current_level = 0;
        while current_level < levels.len() {
            let mut next_level = Vec::new();
            
            for &parent_idx in &levels[current_level] {
                for &child_idx in &self.nodes[parent_idx].children {
                    if child_idx < self.nodes.len() && !visited[child_idx] {
                        next_level.push(child_idx);
                        visited[child_idx] = true;
                    }
                }
            }
            
            if !next_level.is_empty() {
                levels.push(next_level);
            }
            current_level += 1;
        }

        // Calculate positions level by level
        for (level_idx, level_nodes) in levels.iter().enumerate() {
            let x = 5 + (level_idx as u16 * self.min_horizontal_spacing);
            
            // Calculate total height needed for this level
            let total_height_needed = if level_nodes.len() > 1 {
                (level_nodes.len() - 1) as u16 * self.min_vertical_spacing
            } else {
                0
            };
            
            // Start from center and spread out
            let start_y = if total_height_needed == 0 {
                10 // Default center position
            } else {
                10_u16.saturating_sub(total_height_needed / 2)
            };
            
            for (i, &node_idx) in level_nodes.iter().enumerate() {
                let y = start_y + (i as u16 * self.min_vertical_spacing);
                
                // Ensure we stay within terminal bounds
                let final_y = if y + 4 >= terminal_height {
                    terminal_height.saturating_sub(5)
                } else if y < 2 {
                    2
                } else {
                    y
                };
                
                self.nodes[node_idx].x = x;
                self.nodes[node_idx].y = final_y;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    // Initialize the terminal
    let mut terminal = ratatui::init();

    // Initialize tree state with a single root node
    let mut tree_state = TreeState::new();

    let result = run(&mut terminal, &mut tree_state);

    // Restore the terminal
    ratatui::restore();

    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, tree_state: &mut TreeState) -> std::io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Draw title
            let title_text = Text::from("Dynamic Tree Graph (Auto Layout) - hjkl to navigate, Enter to expand leaf nodes, q to quit").style(Color::White);
            let title_area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: area.width,
                height: 1,
            };
            frame.render_widget(title_text, title_area);

            // Draw connections first (so they appear behind nodes)
            for (node_idx, node) in tree_state.nodes.iter().enumerate() {
                for &child_idx in &node.children {
                    if child_idx < tree_state.nodes.len() {
                        draw_connection(frame, node, &tree_state.nodes[child_idx], area);
                    }
                }
            }

            // Draw nodes
            for (idx, node) in tree_state.nodes.iter().enumerate() {
                let is_current = idx == tree_state.current_node;
                draw_node(frame, node, is_current, area);
            }
        })?;

        // Handle events
        if handle_events(tree_state)? {
            break;
        }
    }

    Ok(())
}

fn draw_node(frame: &mut ratatui::Frame, node: &Node, is_current: bool, area: ratatui::layout::Rect) {
    let node_size = 3; // 3x3 square

    // Ensure the node fits within the terminal area
    if node.x + node_size >= area.width || node.y + node_size >= area.height {
        return;
    }

    // Choose color based on whether this is the current node
    let color = if is_current { Color::Green } else { Color::Cyan };

    // Draw the square node
    let lines = ["┌─┐", "│ │", "└─┘"];

    for (i, line) in lines.iter().enumerate() {
        let y_pos = node.y + i as u16;
        if y_pos < area.height {
            let text = Text::from(line.to_string()).style(color);
            let line_area = ratatui::layout::Rect {
                x: node.x,
                y: y_pos,
                width: node_size as u16,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }

    // Draw the label below the node
    if node.y + node_size + 1 < area.height {
        let label_color = if is_current { Color::Green } else { Color::Yellow };
        let label_text = Text::from(node.label.clone()).style(label_color);
        let label_area = ratatui::layout::Rect {
            x: node.x,
            y: node.y + node_size as u16,
            width: node.label.len() as u16,
            height: 1,
        };
        frame.render_widget(label_text, label_area);
    }
}

fn draw_connection(
    frame: &mut ratatui::Frame,
    from: &Node,
    to: &Node,
    area: ratatui::layout::Rect,
) {
    let from_x = from.x + 2; // Connect from right edge of source node (middle of 3-wide node)
    let from_y = from.y + 1; // Connect from middle of source node (middle of 3-high node)
    let to_x = to.x; // Connect to left edge of target node
    let to_y = to.y + 1; // Connect to middle of target node

    // Draw horizontal line from source to intermediate point
    let mid_x = (from_x + to_x) / 2;

    // Horizontal line from source node
    for x in (from_x + 1)..mid_x {
        if x < area.width && from_y < area.height {
            let text = Text::from("─").style(Color::Green);
            let line_area = ratatui::layout::Rect {
                x,
                y: from_y,
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
            if mid_x < area.width && y < area.height {
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
                    x: mid_x,
                    y,
                    width: 1,
                    height: 1,
                };
                frame.render_widget(text, line_area);
            }
        }
    }

    // Horizontal line to target node
    for x in (mid_x + 1)..to_x {
        if x < area.width && to_y < area.height {
            let text = Text::from("─").style(Color::Green);
            let line_area = ratatui::layout::Rect {
                x,
                y: to_y,
                width: 1,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }
}

fn handle_events(tree_state: &mut TreeState) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            
            // Navigation with hjkl keys
            KeyCode::Char('h') => move_left(tree_state),
            KeyCode::Char('j') => move_down(tree_state),
            KeyCode::Char('k') => move_up(tree_state),
            KeyCode::Char('l') => move_right(tree_state),
            
            // Expand node with Enter
            KeyCode::Enter => {
                // Get terminal size using crossterm
                if let Ok((width, height)) = crossterm::terminal::size() {
                    expand_current_node(tree_state, width, height);
                } else {
                    // Fallback to default dimensions
                    expand_current_node(tree_state, 120, 30);
                }
            },
            
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}

fn move_left(tree_state: &mut TreeState) {
    // Find parent of current node
    for (parent_idx, parent) in tree_state.nodes.iter().enumerate() {
        if parent.children.contains(&tree_state.current_node) {
            tree_state.current_node = parent_idx;
            return;
        }
    }
}

fn move_right(tree_state: &mut TreeState) {
    // Move to first child if available
    let current = &tree_state.nodes[tree_state.current_node];
    if !current.children.is_empty() {
        tree_state.current_node = current.children[0];
    }
}

fn move_up(tree_state: &mut TreeState) {
    // Find siblings and move to previous sibling
    for parent in &tree_state.nodes {
        if let Some(current_pos) = parent.children.iter().position(|&x| x == tree_state.current_node) {
            if current_pos > 0 {
                tree_state.current_node = parent.children[current_pos - 1];
                return;
            }
        }
    }
}

fn move_down(tree_state: &mut TreeState) {
    // Find siblings and move to next sibling
    for parent in &tree_state.nodes {
        if let Some(current_pos) = parent.children.iter().position(|&x| x == tree_state.current_node) {
            if current_pos + 1 < parent.children.len() {
                tree_state.current_node = parent.children[current_pos + 1];
                return;
            }
        }
    }
}

fn expand_current_node(tree_state: &mut TreeState, terminal_width: u16, terminal_height: u16) {
    let current_node = &tree_state.nodes[tree_state.current_node];
    
    // Only expand if it's a leaf node (no children)
    if current_node.children.is_empty() {
        // Create two children with temporary positions
        let child1 = Node {
            x: 0, // Temporary position
            y: 0, // Temporary position
            label: format!("{}.1", tree_state.node_counter),
            children: vec![],
        };
        
        let child2 = Node {
            x: 0, // Temporary position
            y: 0, // Temporary position
            label: format!("{}.2", tree_state.node_counter),
            children: vec![],
        };
        
        // Add the new nodes to the tree
        let child1_idx = tree_state.nodes.len();
        let child2_idx = tree_state.nodes.len() + 1;
        
        tree_state.nodes.push(child1);
        tree_state.nodes.push(child2);
        
        // Add children to current node
        tree_state.nodes[tree_state.current_node].children.push(child1_idx);
        tree_state.nodes[tree_state.current_node].children.push(child2_idx);
        
        tree_state.node_counter += 1;
        
        // Recalculate the entire tree layout
        tree_state.recalculate_layout(terminal_width, terminal_height);
    }
}
