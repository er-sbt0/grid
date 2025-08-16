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

    // Reingold-Tilford specific data
    pub rt_data: Option<RTNodeData>,
}

#[derive(Clone, Default)]
pub struct RTNodeData {
    // Reingold-Tilford algorithm fields
    pub prelim: f32,         // Preliminary horizontal position
    pub modifier: f32,       // Modifier value for subtree positioning
    pub subtree_width: f32,  // Width of entire subtree
    pub subtree_height: f32, // Height of entire subtree

    // Final coordinates
    pub final_x: f32,
    pub final_y: f32,

    // Contour tracking for conflict detection
    pub left_contour: Vec<f32>, // Left contour at each level relative to this node
    pub right_contour: Vec<f32>, // Right contour at each level relative to this node
    pub contour_depth: usize,   // Number of levels in contour
}

// Deprecated: Keep for migration compatibility
#[derive(Clone, Default)]
pub struct BJLNodeData {
    pub prelim: f32,
    pub modifier: f32,
    pub thread: Option<usize>,
    pub ancestor: usize,
    pub shift: f32,
    pub change: f32,
    pub left_sibling: Option<usize>,
    pub left_most_sibling: Option<usize>,
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
    ReingoldTilford {
        orientation: TreeOrientation,
        node_separation: f32,    // Minimum distance between sibling nodes
        level_separation: f32,   // Distance between tree levels
        subtree_separation: f32, // Minimum distance between subtrees
    },
    // Deprecated: Keep for migration compatibility
    #[deprecated(note = "Use ReingoldTilford strategy instead")]
    BJL {
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
            strategy: LayoutStrategy::ReingoldTilford {
                orientation: TreeOrientation::LeftRight,
                node_separation: 4.0,    // Vertical spacing between siblings
                level_separation: 6.0,   // Horizontal spacing between levels
                subtree_separation: 2.0, // Extra space between subtrees
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

        // Use Reingold-Tilford algorithm
        self.calculate_rt_layout(tree, terminal_size);

        // Mark as clean
        self.state.is_dirty = false;
        self.state.last_terminal_size = terminal_size;
        self.state.layout_version += 1;
    }

    // Testing method that skips validation to allow debugging collision issues
    pub fn calculate_layout_for_testing(
        &mut self,
        tree: &TreeStructure,
        terminal_size: (u16, u16),
    ) {
        // Resize layout vectors to match tree size
        self.state
            .node_layouts
            .resize(tree.nodes.len(), NodeLayout::default());
        self.state.subtree_dirty.resize(tree.nodes.len(), false);

        // Use Reingold-Tilford algorithm without validation
        self.calculate_rt_layout_without_validation(tree, terminal_size);

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

    // Reingold-Tilford Algorithm Implementation
    fn calculate_rt_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Initialize RT data for all nodes
        self.initialize_rt_data(tree);

        // Phase 1: Calculate preliminary positions (postorder)
        self.rt_first_walk(tree, tree.root);

        // Phase 2: Apply modifiers to get final positions (preorder)
        self.rt_second_walk(tree, tree.root, 0.0, 0.0);

        // Phase 3: Calculate subtree dimensions based on final positions
        self.rt_calculate_final_dimensions(tree, tree.root);

        // Convert to terminal coordinates
        self.rt_to_terminal_coordinates(tree, terminal_size);

        // Validation (debug builds only)
        #[cfg(debug_assertions)]
        if let Err(msg) = self.validate_no_overlaps(tree) {
            panic!("Layout validation failed: {}", msg);
        }
    }

    // Version without validation for testing
    fn calculate_rt_layout_without_validation(
        &mut self,
        tree: &TreeStructure,
        terminal_size: (u16, u16),
    ) {
        // Initialize RT data for all nodes
        self.initialize_rt_data(tree);

        // Phase 1: Calculate preliminary positions (postorder)
        self.rt_first_walk(tree, tree.root);

        // Phase 2: Apply modifiers to get final positions (preorder)
        self.rt_second_walk(tree, tree.root, 0.0, 0.0);

        // Phase 3: Calculate subtree dimensions based on final positions
        self.rt_calculate_final_dimensions(tree, tree.root);

        // Convert to terminal coordinates
        self.rt_to_terminal_coordinates(tree, terminal_size);

        // No validation - for debugging collision issues
    }

    #[allow(dead_code)] // Used in debug builds for validation
    fn validate_no_overlaps(&self, tree: &TreeStructure) -> Result<(), String> {
        let min_separation = self.get_rt_node_separation();

        for i in 0..tree.nodes.len() {
            for j in (i + 1)..tree.nodes.len() {
                let pos_i = self.get_rt_final_x(i);
                let pos_j = self.get_rt_final_x(j);
                let depth_i = self.get_rt_final_y(i);
                let depth_j = self.get_rt_final_y(j);

                // Check for overlaps on same level
                if (depth_i - depth_j).abs() < 0.1 {
                    // Same level
                    let distance = (pos_i - pos_j).abs();
                    if distance < min_separation {
                        return Err(format!(
                            "Nodes {} and {} overlap: distance={:.2}, required={:.2}",
                            i, j, distance, min_separation
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn initialize_rt_data(&mut self, tree: &TreeStructure) {
        // Initialize RT data for each node
        for (node_id, _node) in tree.nodes.iter().enumerate() {
            let layout = &mut self.state.node_layouts[node_id];
            layout.width = self.config.node_width;
            layout.height = self.config.node_height;

            layout.rt_data = Some(RTNodeData {
                prelim: 0.0,
                modifier: 0.0,
                subtree_width: 0.0,
                subtree_height: 0.0,
                final_x: 0.0,
                final_y: 0.0,
                left_contour: Vec::new(),
                right_contour: Vec::new(),
                contour_depth: 0,
            });
        }
    }

    fn rt_first_walk(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = tree.nodes[node_id].children.clone();

        // Process all children first (postorder)
        for &child_id in &children {
            self.rt_first_walk(tree, child_id);
        }

        if children.is_empty() {
            // Leaf node
            self.set_leaf_position_with_contour(node_id);
        } else {
            // Internal node - position children with contour-based conflict detection
            self.set_internal_position_with_contours(tree, node_id);
        }

        // Calculate contours for this subtree
        self.calculate_contours(tree, node_id);
    }

    fn set_internal_position_with_contours(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = &tree.nodes[node_id].children;
        let min_separation = self.get_rt_node_separation();

        if children.len() == 1 {
            // Single child: position directly above it
            let child_prelim = self.get_rt_prelim(children[0]);
            self.set_rt_prelim(node_id, child_prelim);
            self.set_rt_modifier(node_id, 0.0);
        } else if children.len() >= 2 {
            // Multiple children: use contour-based conflict detection
            self.position_children_with_contour_detection(tree, node_id, min_separation);

            // Center parent between first and last child
            let first_child_pos = self.get_rt_prelim(children[0]);
            let last_child_pos = self.get_rt_prelim(children[children.len() - 1]);
            let midpoint = (first_child_pos + last_child_pos) / 2.0;

            self.set_rt_prelim(node_id, midpoint);
            // Modifier should be 0 since children are already positioned correctly
            self.set_rt_modifier(node_id, 0.0);
        } else {
            // No children
            self.set_rt_prelim(node_id, 0.0);
            self.set_rt_modifier(node_id, 0.0);
        }
    }

    fn position_children_with_contour_detection(
        &mut self,
        tree: &TreeStructure,
        parent_id: usize,
        min_separation: f32,
    ) {
        let children = &tree.nodes[parent_id].children;

        if children.is_empty() {
            return;
        }

        // Position first child at origin
        self.set_rt_prelim(children[0], 0.0);

        // Position subsequent children using contour-based conflict detection
        for i in 1..children.len() {
            let left_sibling = children[i - 1];
            let current_child = children[i];

            // Find the minimum distance to avoid conflicts between left sibling's right contour
            // and current child's left contour
            let min_distance = self.find_min_distance_between_subtrees(
                left_sibling,
                current_child,
                min_separation,
            );

            // Position current child at the calculated distance from left sibling
            let left_sibling_pos = self.get_rt_prelim(left_sibling);
            let new_pos = left_sibling_pos + min_distance;

            self.set_rt_prelim(current_child, new_pos);
        }
    }

    fn find_min_distance_between_subtrees(
        &self,
        left_node: usize,
        right_node: usize,
        min_separation: f32,
    ) -> f32 {
        let left_data = &self.state.node_layouts[left_node].rt_data.as_ref().unwrap();
        let right_data = &self.state.node_layouts[right_node]
            .rt_data
            .as_ref()
            .unwrap();

        let left_contour = &left_data.right_contour;
        let right_contour = &right_data.left_contour;

        let max_depth = left_contour.len().max(right_contour.len());
        let mut max_separation_needed = min_separation;

        // Walk down both contours to find the maximum separation needed
        for level in 0..max_depth {
            let left_edge = if level < left_contour.len() {
                left_contour[level]
            } else {
                // Use the last known contour value
                left_contour.last().copied().unwrap_or(0.0)
            };

            let right_edge = if level < right_contour.len() {
                right_contour[level]
            } else {
                // Use the last known contour value
                right_contour.last().copied().unwrap_or(0.0)
            };

            // Calculate the separation needed at this level
            let separation_needed = left_edge - right_edge + min_separation;
            max_separation_needed = max_separation_needed.max(separation_needed);
        }

        max_separation_needed
    }

    fn calculate_contours(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = &tree.nodes[node_id].children;
        let half_width = self.config.node_width as f32 / 2.0;

        if children.is_empty() {
            // Leaf node - contours already set in set_leaf_position_with_contour
            return;
        }

        // For internal nodes, merge contours from all children
        let mut merged_left_contour = vec![-half_width]; // Node itself at level 0
        let mut merged_right_contour = vec![half_width];
        let mut max_depth = 1;

        for &child_id in children {
            let child_prelim = self.get_rt_prelim(child_id);
            let child_data = self.state.node_layouts[child_id].rt_data.as_ref().unwrap();

            // Extend contours to accommodate this child's depth
            let child_depth = child_data.contour_depth;
            let total_child_depth = child_depth + 1; // +1 for the level between parent and child

            // Extend merged contours if needed
            while merged_left_contour.len() < total_child_depth {
                merged_left_contour
                    .push(merged_left_contour.last().copied().unwrap_or(-half_width));
                merged_right_contour
                    .push(merged_right_contour.last().copied().unwrap_or(half_width));
            }

            // Merge child's contours into parent's contours
            for level in 0..child_depth {
                let contour_level = level + 1; // Offset by 1 since child is one level down
                if contour_level < merged_left_contour.len() {
                    let child_left = child_prelim + child_data.left_contour[level];
                    let child_right = child_prelim + child_data.right_contour[level];

                    merged_left_contour[contour_level] =
                        merged_left_contour[contour_level].min(child_left);
                    merged_right_contour[contour_level] =
                        merged_right_contour[contour_level].max(child_right);
                }
            }

            max_depth = max_depth.max(total_child_depth);
        }

        // Update this node's contour data
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.left_contour = merged_left_contour;
            rt_data.right_contour = merged_right_contour;
            rt_data.contour_depth = max_depth;
        }
    }

    fn rt_second_walk(&mut self, tree: &TreeStructure, node_id: usize, x_offset: f32, depth: f32) {
        // Calculate this node's final position
        let prelim = self.get_rt_prelim(node_id);
        let node_modifier = self.get_rt_modifier(node_id);
        let final_x = prelim + x_offset + node_modifier;

        // Set final coordinates
        self.set_rt_final_coordinates(node_id, final_x, depth);

        // Process children with this node's final position as their base
        let level_separation = self.get_rt_level_separation();
        for &child_id in &tree.nodes[node_id].children {
            self.rt_second_walk(
                tree,
                child_id,
                final_x, // Children positioned relative to parent's final position
                depth + level_separation,
            );
        }
    }

    fn rt_calculate_final_dimensions(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = tree.nodes[node_id].children.clone();

        // Process all children first (postorder)
        for &child_id in &children {
            self.rt_calculate_final_dimensions(tree, child_id);
        }

        // Calculate subtree dimensions based on final positioned coordinates
        self.calculate_subtree_dimensions(tree, node_id);
    }

    fn set_leaf_position_with_contour(&mut self, node_id: usize) {
        // Leaf nodes are positioned at origin relative to their parent
        self.set_rt_prelim(node_id, 0.0);
        self.set_rt_modifier(node_id, 0.0);

        // Initialize contours for leaf node
        let half_width = self.config.node_width as f32 / 2.0;
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.left_contour = vec![-half_width];
            rt_data.right_contour = vec![half_width];
            rt_data.contour_depth = 1;
        }
    }

    fn calculate_subtree_dimensions(&mut self, tree: &TreeStructure, node_id: usize) {
        let children = &tree.nodes[node_id].children;

        if children.is_empty() {
            // Leaf node: width = node width
            let node_width = self.config.node_width as f32;
            self.set_rt_subtree_width(node_id, node_width);
            self.set_rt_subtree_height(node_id, self.config.node_height as f32);
        } else {
            // Internal node: width = span of all children based on final coordinates
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;

            for &child_id in children {
                let child_final_x = self.get_rt_final_x(child_id);
                let child_width = self.get_rt_subtree_width(child_id);

                min_x = min_x.min(child_final_x);
                max_x = max_x.max(child_final_x + child_width);
            }

            let total_width = if min_x != f32::INFINITY && max_x != f32::NEG_INFINITY {
                max_x - min_x
            } else {
                self.config.node_width as f32
            };

            self.set_rt_subtree_width(node_id, total_width.max(self.config.node_width as f32));

            // Height = max child height + current node height
            let max_child_height = children
                .iter()
                .map(|&child_id| self.get_rt_subtree_height(child_id))
                .fold(0.0, f32::max);

            self.set_rt_subtree_height(node_id, max_child_height + self.config.node_height as f32);
        }
    }

    fn rt_to_terminal_coordinates(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        if tree.nodes.is_empty() {
            return;
        }

        // Find bounds of the layout
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for layout in &self.state.node_layouts {
            if let Some(ref rt_data) = layout.rt_data {
                min_x = min_x.min(rt_data.final_x);
                max_x = max_x.max(rt_data.final_x);
                min_y = min_y.min(rt_data.final_y);
                max_y = max_y.max(rt_data.final_y);
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

        // Convert to terminal coordinates for left-to-right layout
        for layout in &mut self.state.node_layouts {
            if let Some(ref rt_data) = layout.rt_data {
                // For LeftRight orientation: final_y becomes terminal x, final_x becomes terminal y
                layout.x = ((rt_data.final_y - min_y) * scale + offset_x) as u16;
                layout.y = ((rt_data.final_x - min_x) * scale + offset_y) as u16;

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

    // RT Data Access Methods
    fn get_rt_prelim(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .rt_data
            .as_ref()
            .map_or(0.0, |d| d.prelim)
    }

    fn set_rt_prelim(&mut self, node_id: usize, value: f32) {
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.prelim = value;
        }
    }

    fn get_rt_modifier(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .rt_data
            .as_ref()
            .map_or(0.0, |d| d.modifier)
    }

    fn set_rt_modifier(&mut self, node_id: usize, value: f32) {
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.modifier = value;
        }
    }

    fn get_rt_subtree_width(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .rt_data
            .as_ref()
            .map_or(0.0, |d| d.subtree_width)
    }

    fn set_rt_subtree_width(&mut self, node_id: usize, value: f32) {
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.subtree_width = value;
        }
    }

    fn get_rt_subtree_height(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .rt_data
            .as_ref()
            .map_or(0.0, |d| d.subtree_height)
    }

    fn set_rt_subtree_height(&mut self, node_id: usize, value: f32) {
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.subtree_height = value;
        }
    }

    fn set_rt_final_coordinates(&mut self, node_id: usize, x: f32, y: f32) {
        if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
            rt_data.final_x = x;
            rt_data.final_y = y;
        }
    }

    fn get_rt_final_x(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .rt_data
            .as_ref()
            .map_or(0.0, |d| d.final_x)
    }

    #[allow(dead_code)] // Used by validate_no_overlaps (debug builds only)
    fn get_rt_final_y(&self, node_id: usize) -> f32 {
        self.state.node_layouts[node_id]
            .rt_data
            .as_ref()
            .map_or(0.0, |d| d.final_y)
    }

    fn get_rt_node_separation(&self) -> f32 {
        match self.config.strategy {
            LayoutStrategy::ReingoldTilford {
                node_separation, ..
            } => node_separation,
            #[allow(deprecated)]
            LayoutStrategy::BJL {
                sibling_separation, ..
            } => sibling_separation,
        }
    }

    fn get_rt_level_separation(&self) -> f32 {
        match self.config.strategy {
            LayoutStrategy::ReingoldTilford {
                level_separation, ..
            } => level_separation,
            #[allow(deprecated)]
            LayoutStrategy::BJL {
                level_separation, ..
            } => level_separation,
        }
    }
}

impl LayoutConfig {
    #[deprecated(note = "Use ReingoldTilford strategy instead")]
    pub fn bjl_left_right() -> Self {
        Self {
            strategy: LayoutStrategy::ReingoldTilford {
                orientation: TreeOrientation::LeftRight,
                node_separation: 4.0,
                level_separation: 6.0,
                subtree_separation: 2.0,
            },
            ..Default::default()
        }
    }
}

impl Default for TreeStructure {
    fn default() -> Self {
        Self::new()
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
