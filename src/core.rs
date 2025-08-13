// Pure Tree Structure (Business Logic)
#[derive(Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<usize>,
}

pub struct TreeStructure {
    pub nodes: Vec<TreeNode>,
    pub root: usize,
    pub current_node: usize,
    pub node_counter: usize,
}

// Layout Data (Presentation Logic)
#[derive(Clone, Default)]
pub struct NodeLayout {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub subtree_width: u16,  // Total width needed for this subtree
    pub subtree_height: u16, // Total height needed for this subtree

    // BJL-specific data (optional for non-BJL layouts)
    pub bjl_data: Option<BJLNodeData>,
}

#[derive(Clone, Default)]
pub struct BJLNodeData {
    // Core BJL algorithm fields
    pub prelim: f32,           // Preliminary x-coordinate
    pub modifier: f32,         // Modifier for subtree positioning
    pub thread: Option<usize>, // Thread to extreme subtree contour
    pub ancestor: usize,       // Ancestor for conflict resolution

    // Positioning refinement fields
    pub shift: f32,  // Shift value for spacing adjustment
    pub change: f32, // Change value for uniform distribution

    // Tree structure navigation
    pub left_sibling: Option<usize>,
    pub left_most_sibling: Option<usize>,

    // Final floating-point coordinates (before terminal conversion)
    pub final_x: f32,
    pub final_y: f32,
}

// Layout State Management
pub struct LayoutState {
    pub node_layouts: Vec<NodeLayout>,
    pub is_dirty: bool,
    pub subtree_dirty: Vec<bool>,
    pub last_terminal_size: (u16, u16),
    pub layout_version: u64,
}

// Layout Strategy Types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutStrategy {
    BJL {
        // Buchheim-Jünger-Leipert algorithm
        orientation: TreeOrientation,
        sibling_separation: f32,
        subtree_separation: f32,
        level_separation: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeOrientation {
    LeftRight, // Root at left, children right
}

// Configuration
#[derive(Clone)]
pub struct LayoutConfig {
    pub node_width: u16,
    pub node_height: u16,
    pub min_horizontal_spacing: u16,
    pub min_vertical_spacing: u16,
    pub margin_top: u16,
    pub margin_left: u16,

    // New field for strategy selection
    pub strategy: LayoutStrategy,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_width: 3,
            node_height: 3,
            min_horizontal_spacing: 6, // Space between levels (left-to-right)
            min_vertical_spacing: 4,   // Space between siblings (vertically)
            margin_top: 2,
            margin_left: 2,
            strategy: LayoutStrategy::BJL {
                orientation: TreeOrientation::LeftRight,
                sibling_separation: 4.0,
                subtree_separation: 6.0,
                level_separation: 6.0,
            },
        }
    }
}

// Layout Manager (Core Logic)
pub struct LayoutManager {
    pub state: LayoutState,
    pub config: LayoutConfig,
}

impl LayoutManager {
    pub fn new(config: LayoutConfig) -> Self {
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

    pub fn ensure_layout_valid(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        if self.state.is_dirty || self.terminal_size_changed(terminal_size) {
            self.calculate_layout(tree, terminal_size);
        }
    }

    pub fn terminal_size_changed(&self, terminal_size: (u16, u16)) -> bool {
        self.state.last_terminal_size != terminal_size
    }

    pub fn calculate_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Resize layout vectors to match tree size
        self.state
            .node_layouts
            .resize(tree.nodes.len(), NodeLayout::default());
        self.state.subtree_dirty.resize(tree.nodes.len(), false);

        // Use BJL algorithm
        self.calculate_bjl_layout(tree, terminal_size);

        // Mark as clean
        self.state.is_dirty = false;
        self.state.last_terminal_size = terminal_size;
        self.state.layout_version += 1;
    }

    pub fn get_node_position(&self, node_id: usize) -> Option<(u16, u16)> {
        if node_id < self.state.node_layouts.len() {
            let layout = &self.state.node_layouts[node_id];
            Some((layout.x, layout.y))
        } else {
            None
        }
    }

    pub fn mark_dirty(&mut self) {
        self.state.is_dirty = true;
    }

    pub fn mark_subtree_dirty(&mut self, node_id: usize) {
        if node_id < self.state.subtree_dirty.len() {
            self.state.subtree_dirty[node_id] = true;
            self.state.is_dirty = true;
        }
    }

    // BJL Algorithm Implementation
    fn calculate_bjl_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Initialize BJL data for all nodes
        self.initialize_bjl_data(tree);

        // BJL Phase 1: First Walk (post-order)
        self.bjl_first_walk(tree, tree.root);

        // BJL Phase 2: Second Walk (pre-order)
        self.bjl_second_walk(tree, tree.root, 0.0, 0.0);

        // Convert to terminal coordinates
        self.bjl_to_terminal_coordinates(tree, terminal_size);
    }

    fn initialize_bjl_data(&mut self, tree: &TreeStructure) {
        // Initialize BJL data for each node
        for (node_id, _node) in tree.nodes.iter().enumerate() {
            let layout = &mut self.state.node_layouts[node_id];
            layout.width = self.config.node_width;
            layout.height = self.config.node_height;

            layout.bjl_data = Some(BJLNodeData {
                prelim: 0.0,
                modifier: 0.0,
                thread: None,
                ancestor: node_id,
                shift: 0.0,
                change: 0.0,
                left_sibling: None,
                left_most_sibling: None,
                final_x: 0.0,
                final_y: 0.0,
            });
        }

        // Set up sibling relationships
        for parent_id in 0..tree.nodes.len() {
            let children = tree.nodes[parent_id].children.clone();

            for (i, &child_id) in children.iter().enumerate() {
                if child_id < self.state.node_layouts.len() {
                    if let Some(ref mut bjl_data) = self.state.node_layouts[child_id].bjl_data {
                        // Set left sibling
                        if i > 0 {
                            bjl_data.left_sibling = Some(children[i - 1]);
                        }

                        // Set leftmost sibling
                        if !children.is_empty() {
                            bjl_data.left_most_sibling = Some(children[0]);
                        }
                    }
                }
            }
        }
    }

    fn bjl_first_walk(&mut self, tree: &TreeStructure, node_id: usize) {
        if node_id >= tree.nodes.len() {
            return;
        }

        let children = tree.nodes[node_id].children.clone();

        // Process children first (post-order traversal)
        for &child_id in &children {
            self.bjl_first_walk(tree, child_id);
        }

        if children.is_empty() {
            // Leaf node
            self.set_preliminary_position_leaf(node_id);
        } else {
            // Internal node
            self.set_preliminary_position_internal(tree, node_id);
            self.execute_shifts(tree, node_id);
            self.apportion(tree, node_id);
        }
    }

    fn bjl_second_walk(&mut self, tree: &TreeStructure, node_id: usize, modifier: f32, depth: f32) {
        if node_id >= tree.nodes.len() {
            return;
        }

        let level_separation = self.get_level_separation();

        // Apply modifier and set final coordinates
        let prelim = self.get_prelim(node_id);
        let modifier_val = self.get_modifier(node_id);
        self.set_final_coordinates(node_id, prelim + modifier, depth);

        // Process children
        for &child_id in &tree.nodes[node_id].children {
            self.bjl_second_walk(
                tree,
                child_id,
                modifier + modifier_val,
                depth + level_separation,
            );
        }
    }

    fn execute_shifts(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = tree.nodes[node_id].children.clone();
        let mut shift = 0.0;
        let mut change = 0.0;

        // Apply shifts from right to left
        for &child_id in children.iter().rev() {
            let child_shift = self.get_shift(child_id);
            let child_change = self.get_change(child_id);

            let child_prelim = self.get_prelim(child_id);
            self.set_prelim(child_id, child_prelim + shift);
            self.set_modifier(child_id, self.get_modifier(child_id) + shift);

            change += child_change;
            shift += child_shift + change;
        }
    }

    fn apportion(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = tree.nodes[node_id].children.clone();

        if children.len() <= 1 {
            return;
        }

        // For simplicity, we'll use a basic apportion that ensures no overlaps
        let subtree_separation = match self.config.strategy {
            LayoutStrategy::BJL {
                subtree_separation, ..
            } => subtree_separation,
        };

        for i in 1..children.len() {
            let current_child = children[i];
            let prev_child = children[i - 1];

            let current_prelim = self.get_prelim(current_child);
            let prev_prelim = self.get_prelim(prev_child);

            let min_distance = prev_prelim + subtree_separation;
            if current_prelim < min_distance {
                let shift_amount = min_distance - current_prelim;
                self.shift_subtree(tree, current_child, shift_amount);
            }
        }
    }

    fn shift_subtree(&mut self, tree: &TreeStructure, root_id: usize, shift: f32) {
        let current_prelim = self.get_prelim(root_id);
        let current_modifier = self.get_modifier(root_id);
        self.set_prelim(root_id, current_prelim + shift);
        self.set_modifier(root_id, current_modifier + shift);

        // Recursively shift all descendants
        for &child_id in &tree.nodes[root_id].children {
            self.shift_subtree(tree, child_id, shift);
        }
    }

    fn set_preliminary_position_leaf(&mut self, node_id: usize) {
        let left_sibling_id = if let Some(ref bjl_data) = self.state.node_layouts[node_id].bjl_data
        {
            bjl_data.left_sibling
        } else {
            None
        };

        if let Some(left_sibling_id) = left_sibling_id {
            let left_sibling_prelim = self.get_prelim(left_sibling_id);
            let separation = match self.config.strategy {
                LayoutStrategy::BJL {
                    sibling_separation, ..
                } => sibling_separation,
            };

            self.set_prelim(node_id, left_sibling_prelim + separation);
        } else {
            self.set_prelim(node_id, 0.0);
        }
    }

    fn set_preliminary_position_internal(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = tree.nodes[node_id].children.clone();

        if children.len() >= 2 {
            let leftmost_child = children[0];
            let rightmost_child = children[children.len() - 1];

            let leftmost_prelim = self.get_prelim(leftmost_child);
            let rightmost_prelim = self.get_prelim(rightmost_child);

            // Position at midpoint of leftmost and rightmost children
            let midpoint = (leftmost_prelim + rightmost_prelim) / 2.0;

            let left_sibling_id =
                if let Some(ref bjl_data) = self.state.node_layouts[node_id].bjl_data {
                    bjl_data.left_sibling
                } else {
                    None
                };

            if let Some(left_sibling_id) = left_sibling_id {
                let left_sibling_prelim = self.get_prelim(left_sibling_id);
                let separation = match self.config.strategy {
                    LayoutStrategy::BJL {
                        sibling_separation, ..
                    } => sibling_separation,
                };

                let new_prelim = left_sibling_prelim + separation;
                let modifier = new_prelim - midpoint;
                self.set_prelim(node_id, new_prelim);
                self.set_modifier(node_id, modifier);
            } else {
                self.set_prelim(node_id, midpoint);
                self.set_modifier(node_id, 0.0);
            }
        }
    }
    fn bjl_to_terminal_coordinates(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        if tree.nodes.is_empty() {
            return;
        }

        // Find bounds of the layout
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for layout in &self.state.node_layouts {
            if let Some(ref bjl_data) = layout.bjl_data {
                min_x = min_x.min(bjl_data.final_x);
                max_x = max_x.max(bjl_data.final_x);
                min_y = min_y.min(bjl_data.final_y);
                max_y = max_y.max(bjl_data.final_y);
            }
        }

        // Calculate scaling and centering
        let layout_width = max_x - min_x;
        let layout_height = max_y - min_y;

        let available_width = terminal_size.0.saturating_sub(self.config.margin_left * 2) as f32;
        let available_height = terminal_size.1.saturating_sub(self.config.margin_top * 2) as f32;

        let scale_x = if layout_width > 0.0 {
            available_width / layout_width
        } else {
            1.0
        };
        let scale_y = if layout_height > 0.0 {
            available_height / layout_height
        } else {
            1.0
        };
        let scale = scale_x.min(scale_y).min(1.0); // Don't scale up

        let offset_x =
            self.config.margin_left as f32 + (available_width - layout_width * scale) / 2.0;
        let offset_y =
            self.config.margin_top as f32 + (available_height - layout_height * scale) / 2.0;

        // Convert to terminal coordinates
        for layout in &mut self.state.node_layouts {
            if let Some(ref bjl_data) = layout.bjl_data {
                // For LeftRight orientation: x is horizontal position, y is vertical position
                layout.x = ((bjl_data.final_y - min_y) * scale + offset_x) as u16;
                layout.y = ((bjl_data.final_x - min_x) * scale + offset_y) as u16;

                // Ensure coordinates are within bounds
                layout.x = layout
                    .x
                    .min(terminal_size.0.saturating_sub(self.config.node_width));
                layout.y = layout
                    .y
                    .min(terminal_size.1.saturating_sub(self.config.node_height));
            }
        }
    }

    // Helper methods for BJL algorithm
    fn get_prelim(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .bjl_data
            .as_ref()
            .map_or(0.0, |d| d.prelim)
    }

    fn set_prelim(&mut self, node_id: usize, value: f32) {
        if let Some(ref mut bjl_data) = self.state.node_layouts[node_id].bjl_data {
            bjl_data.prelim = value;
        }
    }

    fn get_modifier(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .bjl_data
            .as_ref()
            .map_or(0.0, |d| d.modifier)
    }

    fn set_modifier(&mut self, node_id: usize, value: f32) {
        if let Some(ref mut bjl_data) = self.state.node_layouts[node_id].bjl_data {
            bjl_data.modifier = value;
        }
    }

    fn get_shift(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .bjl_data
            .as_ref()
            .map_or(0.0, |d| d.shift)
    }

    fn get_change(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .bjl_data
            .as_ref()
            .map_or(0.0, |d| d.change)
    }

    fn set_final_coordinates(&mut self, node_id: usize, x: f32, y: f32) {
        if let Some(ref mut bjl_data) = self.state.node_layouts[node_id].bjl_data {
            bjl_data.final_x = x;
            bjl_data.final_y = y;
        }
    }

    fn get_level_separation(&self) -> f32 {
        match self.config.strategy {
            LayoutStrategy::BJL {
                level_separation, ..
            } => level_separation,
        }
    }
}

impl TreeStructure {
    pub fn new() -> Self {
        Self {
            nodes: vec![TreeNode {
                label: "Root".to_string(),
                children: vec![],
            }],
            root: 0,
            current_node: 0,
            node_counter: 1,
        }
    }

    pub fn expand_node(&mut self, node_id: usize) {
        if node_id >= self.nodes.len() {
            return;
        }

        // Only expand if it's a leaf node (no children)
        if self.nodes[node_id].children.is_empty() {
            // Create two children
            let child1_idx = self.nodes.len();
            let child2_idx = self.nodes.len() + 1;

            let child1 = TreeNode {
                label: format!("{}.1", self.node_counter),
                children: vec![],
            };

            let child2 = TreeNode {
                label: format!("{}.2", self.node_counter),
                children: vec![],
            };

            self.nodes.push(child1);
            self.nodes.push(child2);

            // Add children to current node
            self.nodes[node_id].children.push(child1_idx);
            self.nodes[node_id].children.push(child2_idx);

            self.node_counter += 1;
        }
    }
}
