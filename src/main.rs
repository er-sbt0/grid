use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{style::Color, text::Text};

// Pure Tree Structure (Business Logic)
#[derive(Clone)]
struct TreeNode {
    label: String,
    children: Vec<usize>,
}

struct TreeStructure {
    nodes: Vec<TreeNode>,
    root: usize,
    current_node: usize,
    node_counter: usize,
}

// Layout Data (Presentation Logic)
#[derive(Clone, Default)]
struct NodeLayout {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    subtree_width: u16,   // Total width needed for this subtree
    subtree_height: u16,  // Total height needed for this subtree
}

// Layout State Management
struct LayoutState {
    node_layouts: Vec<NodeLayout>,
    is_dirty: bool,
    subtree_dirty: Vec<bool>,
    last_terminal_size: (u16, u16),
    layout_version: u64,
}

// Configuration
#[derive(Clone)]
struct LayoutConfig {
    node_width: u16,
    node_height: u16,
    min_horizontal_spacing: u16,
    min_vertical_spacing: u16,
    margin_top: u16,
    margin_left: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_width: 3,
            node_height: 3,
            min_horizontal_spacing: 4,
            min_vertical_spacing: 6,
            margin_top: 2,
            margin_left: 2,
        }
    }
}

// Layout Manager (Core Logic)
struct LayoutManager {
    state: LayoutState,
    config: LayoutConfig,
}

impl LayoutManager {
    fn new(config: LayoutConfig) -> Self {
        Self {
            state: LayoutState {
                node_layouts: Vec::new(),
                is_dirty: true,
                subtree_dirty: Vec::new(),
                last_terminal_size: (0, 0),
                layout_version: 0,
            },
            config,
        }
    }

    fn ensure_layout_valid(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        if self.state.is_dirty || self.terminal_size_changed(terminal_size) {
            self.calculate_layout(tree, terminal_size);
        }
    }

    fn terminal_size_changed(&self, terminal_size: (u16, u16)) -> bool {
        self.state.last_terminal_size != terminal_size
    }

    fn calculate_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Resize layout vectors to match tree size
        self.state.node_layouts.resize(tree.nodes.len(), NodeLayout::default());
        self.state.subtree_dirty.resize(tree.nodes.len(), false);

        // Calculate subtree dimensions (post-order traversal)
        self.calculate_subtree_dimensions(tree, tree.root);

        // Position nodes (level-by-level)
        self.position_nodes(tree, terminal_size);

        // Mark as clean
        self.state.is_dirty = false;
        self.state.last_terminal_size = terminal_size;
        self.state.layout_version += 1;
    }

    fn calculate_subtree_dimensions(&mut self, tree: &TreeStructure, node_id: usize) {
        let node = &tree.nodes[node_id];
        
        // First, calculate dimensions for all children
        for &child_id in &node.children {
            self.calculate_subtree_dimensions(tree, child_id);
        }

        // Now calculate this node's dimensions
        // Get node width and height first
        let node_width = self.config.node_width;
        let node_height = self.config.node_height;
        
        // Collect child dimensions before modifying current layout
        let child_subtree_widths: Vec<u16> = node.children.iter()
            .map(|&child_id| self.state.node_layouts[child_id].subtree_width)
            .collect();
        let child_subtree_heights: Vec<u16> = node.children.iter()
            .map(|&child_id| self.state.node_layouts[child_id].subtree_height)
            .collect();

        // Now safely modify the current node's layout
        let layout = &mut self.state.node_layouts[node_id];
        layout.width = node_width;
        layout.height = node_height;

        if node.children.is_empty() {
            // Leaf node
            layout.subtree_width = node_width;
            layout.subtree_height = node_height;
        } else {
            // Calculate subtree width as sum of children widths + spacing
            let child_widths: u16 = child_subtree_widths.iter().sum();
            let spacing_width = if node.children.len() > 1 {
                (node.children.len() - 1) as u16 * self.config.min_horizontal_spacing
            } else {
                0
            };
            
            layout.subtree_width = child_widths + spacing_width;
            
            // Ensure subtree width is at least as wide as this node
            layout.subtree_width = layout.subtree_width.max(node_width);
            
            // Calculate subtree height as max child subtree height + vertical spacing
            let max_child_height = child_subtree_heights.iter().max().copied().unwrap_or(0);
            
            layout.subtree_height = node_height + self.config.min_vertical_spacing + max_child_height;
        }
    }

    fn position_nodes(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Position root node at center top
        let root_layout = &mut self.state.node_layouts[tree.root];
        root_layout.x = terminal_size.0 / 2 - self.config.node_width / 2;
        root_layout.y = self.config.margin_top;

        // Position children recursively
        self.position_children(tree, tree.root);
    }

    fn position_children(&mut self, tree: &TreeStructure, parent_id: usize) {
        let parent_node = &tree.nodes[parent_id];
        let parent_layout = self.state.node_layouts[parent_id].clone();

        if parent_node.children.is_empty() {
            return;
        }

        // Calculate total width needed for all children
        let total_children_width: u16 = parent_node.children.iter()
            .map(|&child_id| self.state.node_layouts[child_id].subtree_width)
            .sum();
        let total_spacing = if parent_node.children.len() > 1 {
            (parent_node.children.len() - 1) as u16 * self.config.min_horizontal_spacing
        } else {
            0
        };
        let total_width = total_children_width + total_spacing;

        // Center children under parent
        let parent_center_x = parent_layout.x + self.config.node_width / 2;
        let start_x = parent_center_x.saturating_sub(total_width / 2);

        let mut current_x = start_x;
        for &child_id in &parent_node.children {
            let child_layout = &mut self.state.node_layouts[child_id];
            let child_subtree_width = child_layout.subtree_width;
            
            // Center the child within its allocated subtree width
            child_layout.x = current_x + child_subtree_width / 2 - self.config.node_width / 2;
            child_layout.y = parent_layout.y + self.config.min_vertical_spacing;

            // Recursively position this child's children
            self.position_children(tree, child_id);

            // Move to next position
            current_x += child_subtree_width + self.config.min_horizontal_spacing;
        }
    }

    fn get_node_position(&self, node_id: usize) -> Option<(u16, u16)> {
        if node_id < self.state.node_layouts.len() {
            let layout = &self.state.node_layouts[node_id];
            Some((layout.x, layout.y))
        } else {
            None
        }
    }

    fn mark_dirty(&mut self) {
        self.state.is_dirty = true;
    }

    fn mark_subtree_dirty(&mut self, node_id: usize) {
        if node_id < self.state.subtree_dirty.len() {
            self.state.subtree_dirty[node_id] = true;
            self.state.is_dirty = true;
        }
    }
}

// Main application structure
struct TreeApp {
    tree: TreeStructure,
    layout_manager: LayoutManager,
}

fn main() -> std::io::Result<()> {
    // Initialize the terminal
    let mut terminal = ratatui::init();

    // Initialize tree structure with a single root node
    let tree_structure = TreeStructure {
        nodes: vec![TreeNode {
            label: "Root".to_string(),
            children: vec![],
        }],
        root: 0,
        current_node: 0,
        node_counter: 1,
    };

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
            app.layout_manager.ensure_layout_valid(&app.tree, (area.width, area.height));

            // Draw title
            let title_text = Text::from("Dynamic Tree Graph - hjkl to navigate, Enter to expand leaf nodes, q to quit").style(Color::White);
            let title_area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: area.width,
                height: 1,
            };
            frame.render_widget(title_text, title_area);

            // Draw connections first (so they appear behind nodes)
            for (node_idx, node) in app.tree.nodes.iter().enumerate() {
                for &child_idx in &node.children {
                    if child_idx < app.tree.nodes.len() {
                        if let (Some(parent_pos), Some(child_pos)) = (
                            app.layout_manager.get_node_position(node_idx),
                            app.layout_manager.get_node_position(child_idx)
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

fn draw_node_at_position(frame: &mut ratatui::Frame, node: &TreeNode, x: u16, y: u16, is_current: bool, area: ratatui::layout::Rect) {
    let node_size = 3; // 3x3 square

    // Ensure the node fits within the terminal area
    if x + node_size >= area.width || y + node_size >= area.height {
        return;
    }

    // Choose color based on whether this is the current node
    let color = if is_current { Color::Green } else { Color::Cyan };

    // Draw the square node
    let lines = ["┌─┐", "│ │", "└─┘"];

    for (i, line) in lines.iter().enumerate() {
        let y_pos = y + i as u16;
        if y_pos < area.height {
            let text = Text::from(line.to_string()).style(color);
            let line_area = ratatui::layout::Rect {
                x,
                y: y_pos,
                width: node_size as u16,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }

    // Draw the label below the node
    if y + node_size + 1 < area.height {
        let label_color = if is_current { Color::Green } else { Color::Yellow };
        let label_text = Text::from(node.label.clone()).style(label_color);
        let label_area = ratatui::layout::Rect {
            x,
            y: y + node_size as u16,
            width: node.label.len() as u16,
            height: 1,
        };
        frame.render_widget(label_text, label_area);
    }
}

fn draw_connection(
    frame: &mut ratatui::Frame,
    from_pos: (u16, u16),
    to_pos: (u16, u16),
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

fn handle_events(app: &mut TreeApp) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            
            // Navigation with hjkl keys
            KeyCode::Char('h') => move_left(app),
            KeyCode::Char('j') => move_down(app),
            KeyCode::Char('k') => move_up(app),
            KeyCode::Char('l') => move_right(app),
            
            // Expand node with Enter
            KeyCode::Enter => expand_current_node(app),
            
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
        if let Some(current_pos) = parent.children.iter().position(|&x| x == app.tree.current_node) {
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
        if let Some(current_pos) = parent.children.iter().position(|&x| x == app.tree.current_node) {
            if current_pos + 1 < parent.children.len() {
                app.tree.current_node = parent.children[current_pos + 1];
                return;
            }
        }
    }
}

fn expand_current_node(app: &mut TreeApp) {
    let current_node = &app.tree.nodes[app.tree.current_node];
    
    // Only expand if it's a leaf node (no children)
    if current_node.children.is_empty() {
        // Create two children
        let child1_idx = app.tree.nodes.len();
        let child2_idx = app.tree.nodes.len() + 1;
        
        let child1 = TreeNode {
            label: format!("{}.1", app.tree.node_counter),
            children: vec![],
        };
        
        let child2 = TreeNode {
            label: format!("{}.2", app.tree.node_counter),
            children: vec![],
        };
        
        app.tree.nodes.push(child1);
        app.tree.nodes.push(child2);
        
        // Add children to current node
        app.tree.nodes[app.tree.current_node].children.push(child1_idx);
        app.tree.nodes[app.tree.current_node].children.push(child2_idx);
        
        app.tree.node_counter += 1;
        
        // Mark layout as needing recalculation
        app.layout_manager.mark_subtree_dirty(app.tree.current_node);
    }
}
