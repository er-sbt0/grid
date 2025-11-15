use crate::config::{LayoutConfig, TreeOrientation};
use crate::error::LayoutError;
use crate::result::LayoutResult;
use crate::tree::Tree;
use crate::types::{LayoutBounds, Position};

/// Internal data for Reingold-Tilford algorithm
#[derive(Clone, Default, Debug)]
struct RTNodeData {
    prelim: f32,
    modifier: f32,
    final_x: f32,
    final_y: f32,
    left_contour: Vec<f32>,
    right_contour: Vec<f32>,
    contour_depth: usize,
}

/// Layout engine for computing tree positions
pub struct LayoutEngine {
    config: LayoutConfig,
    version: u64,
    last_tree_version: Option<u64>,
    cached_result: Option<LayoutResult>,
}

impl LayoutEngine {
    /// Create a new layout engine with default configuration
    pub fn new() -> Self {
        Self {
            config: LayoutConfig::new(),
            version: 0,
            last_tree_version: None,
            cached_result: None,
        }
    }

    /// Create a layout engine with specific configuration
    pub fn with_config(config: LayoutConfig) -> Result<Self, LayoutError> {
        config.validate()?;
        Ok(Self {
            config,
            version: 0,
            last_tree_version: None,
            cached_result: None,
        })
    }

    /// Compute layout for a tree
    pub fn compute<T>(&mut self, tree: &Tree<T>) -> Result<LayoutResult, LayoutError> {
        tree.validate()?;

        let computation = LayoutComputation::new(&self.config, tree);
        let mut result = computation.execute()?;

        // Compute edges with node dimensions from config
        let parent_children: Vec<_> = tree
            .nodes()
            .iter()
            .enumerate()
            .map(|(id, node)| (id, node.children.clone()))
            .collect();

        result.compute_edges(
            parent_children,
            self.config.node_width(),
            self.config.node_height(),
        );

        self.version += 1;
        self.last_tree_version = Some(tree.version());
        self.cached_result = Some(result.clone());
        Ok(result)
    }

    /// Compute layout for a tree using cached result if tree hasn't changed
    pub fn compute_cached<T>(&mut self, tree: &Tree<T>) -> Result<LayoutResult, LayoutError> {
        // Return cached result if tree hasn't changed
        if let Some(last_version) = self.last_tree_version {
            if last_version == tree.version() {
                if let Some(ref cached) = self.cached_result {
                    return Ok(cached.clone());
                }
            }
        }

        // Otherwise, recompute
        self.compute(tree)
    }

    /// Check if layout needs to be recomputed for a given tree version
    pub fn needs_recompute(&self, tree_version: u64) -> bool {
        match self.last_tree_version {
            Some(last) => last != tree_version,
            None => true,
        }
    }

    /// Invalidate the cached layout result
    pub fn invalidate_cache(&mut self) {
        self.last_tree_version = None;
        self.cached_result = None;
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: LayoutConfig) -> Result<(), LayoutError> {
        config.validate()?;
        self.config = config;
        self.invalidate_cache();
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal computation state
struct LayoutComputation<'a, T> {
    config: &'a LayoutConfig,
    tree: &'a Tree<T>,
    positions: Vec<Position>,
    rt_data: Vec<RTNodeData>,
    version: u64,
}

impl<'a, T> LayoutComputation<'a, T> {
    fn new(config: &'a LayoutConfig, tree: &'a Tree<T>) -> Self {
        let node_count = tree.len();
        Self {
            config,
            tree,
            positions: vec![Position::default(); node_count],
            rt_data: vec![RTNodeData::default(); node_count],
            version: 0,
        }
    }

    fn execute(mut self) -> Result<LayoutResult, LayoutError> {
        if self.tree.is_empty() {
            return Err(LayoutError::EmptyTree);
        }

        // Phase 1: First walk (postorder) - calculate preliminary positions
        self.first_walk(self.tree.root())?;

        // Phase 2: Second walk (preorder) - apply modifiers
        self.second_walk(self.tree.root(), 0.0, 0.0)?;

        // Phase 3: Finalize positions with orientation
        self.finalize_positions()?;

        // Calculate bounds
        let bounds = self.calculate_bounds();

        Ok(LayoutResult::new(self.positions, bounds, self.version))
    }

    /// Phase 1: Postorder traversal to calculate preliminary positions
    fn first_walk(&mut self, node_id: usize) -> Result<(), LayoutError> {
        let node = self.tree.node(node_id)?;
        let children = node.children.clone();

        // Process children first (postorder)
        for &child_id in &children {
            self.first_walk(child_id)?;
        }

        if children.is_empty() {
            self.set_leaf_position(node_id);
        } else {
            self.set_internal_position(node_id)?;
        }

        self.calculate_contours(node_id)?;
        Ok(())
    }

    /// Set position for leaf nodes
    fn set_leaf_position(&mut self, node_id: usize) {
        self.rt_data[node_id].prelim = 0.0;
        self.rt_data[node_id].modifier = 0.0;

        let half_width = self.config.node_width / 2.0;
        self.rt_data[node_id].left_contour = vec![-half_width];
        self.rt_data[node_id].right_contour = vec![half_width];
        self.rt_data[node_id].contour_depth = 1;
    }

    /// Set position for internal nodes
    fn set_internal_position(&mut self, node_id: usize) -> Result<(), LayoutError> {
        let node = self.tree.node(node_id)?;
        let children = &node.children;

        if children.len() == 1 {
            // Single child: position directly above it
            let child_prelim = self.rt_data[children[0]].prelim;
            self.rt_data[node_id].prelim = child_prelim;
            self.rt_data[node_id].modifier = 0.0;
        } else {
            // Multiple children: use contour-based positioning
            self.position_children_with_contours(node_id)?;

            // Center parent between first and last child
            let first_child_pos = self.rt_data[children[0]].prelim;
            let last_child_pos = self.rt_data[children[children.len() - 1]].prelim;
            let midpoint = (first_child_pos + last_child_pos) / 2.0;

            self.rt_data[node_id].prelim = midpoint;
            self.rt_data[node_id].modifier = 0.0;
        }

        Ok(())
    }

    /// Position children using contour-based collision detection
    fn position_children_with_contours(&mut self, parent_id: usize) -> Result<(), LayoutError> {
        let node = self.tree.node(parent_id)?;
        let children = &node.children;

        if children.is_empty() {
            return Ok(());
        }

        let min_separation = self.config.node_separation;

        // Calculate positions starting from 0
        let mut child_positions = vec![0.0; children.len()];
        child_positions[0] = 0.0;

        // Position subsequent children using contour detection
        for i in 1..children.len() {
            let left_sibling = children[i - 1];
            let current_child = children[i];

            let min_distance = self.find_min_distance_between_subtrees(
                left_sibling,
                current_child,
                min_separation,
            );

            child_positions[i] = child_positions[i - 1] + min_distance;
        }

        // Center children for symmetrical layout
        let total_width = child_positions[children.len() - 1] - child_positions[0];
        let center_offset = -total_width / 2.0;

        for (i, &child_id) in children.iter().enumerate() {
            self.rt_data[child_id].prelim = child_positions[i] + center_offset;
        }

        Ok(())
    }

    /// Find minimum distance between two subtrees to avoid collisions
    fn find_min_distance_between_subtrees(
        &self,
        left_node: usize,
        right_node: usize,
        min_separation: f32,
    ) -> f32 {
        let left_contour = &self.rt_data[left_node].right_contour;
        let right_contour = &self.rt_data[right_node].left_contour;

        let max_depth = left_contour.len().max(right_contour.len());
        let mut max_separation_needed = min_separation;

        for level in 0..max_depth {
            let left_edge = if level < left_contour.len() {
                left_contour[level]
            } else {
                left_contour.last().copied().unwrap_or(0.0)
            };

            let right_edge = if level < right_contour.len() {
                right_contour[level]
            } else {
                right_contour.last().copied().unwrap_or(0.0)
            };

            let separation_needed = left_edge - right_edge + min_separation;
            max_separation_needed = max_separation_needed.max(separation_needed);
        }

        max_separation_needed
    }

    /// Calculate contours for a subtree
    fn calculate_contours(&mut self, node_id: usize) -> Result<(), LayoutError> {
        let node = self.tree.node(node_id)?;
        let children = &node.children;
        let half_width = self.config.node_width / 2.0;

        if children.is_empty() {
            // Leaf node - already set in set_leaf_position
            return Ok(());
        }

        // Internal node: merge child contours
        let mut merged_left = vec![-half_width];
        let mut merged_right = vec![half_width];
        let mut max_depth = 1;

        for &child_id in children {
            let child_prelim = self.rt_data[child_id].prelim;
            let child_data = &self.rt_data[child_id];
            let child_depth = child_data.contour_depth;
            let total_child_depth = child_depth + 1;

            // Extend merged contours if needed
            while merged_left.len() < total_child_depth {
                merged_left.push(merged_left.last().copied().unwrap_or(-half_width));
                merged_right.push(merged_right.last().copied().unwrap_or(half_width));
            }

            // Merge child's contours
            for level in 0..child_depth {
                let contour_level = level + 1;
                if contour_level < merged_left.len() {
                    let child_left = child_prelim + child_data.left_contour[level];
                    let child_right = child_prelim + child_data.right_contour[level];

                    merged_left[contour_level] = merged_left[contour_level].min(child_left);
                    merged_right[contour_level] = merged_right[contour_level].max(child_right);
                }
            }

            max_depth = max_depth.max(total_child_depth);
        }

        self.rt_data[node_id].left_contour = merged_left;
        self.rt_data[node_id].right_contour = merged_right;
        self.rt_data[node_id].contour_depth = max_depth;

        Ok(())
    }

    /// Phase 2: Preorder traversal to apply modifiers
    fn second_walk(
        &mut self,
        node_id: usize,
        x_offset: f32,
        depth: f32,
    ) -> Result<(), LayoutError> {
        let prelim = self.rt_data[node_id].prelim;
        let modifier = self.rt_data[node_id].modifier;
        let final_x = prelim + x_offset + modifier;

        self.rt_data[node_id].final_x = final_x;
        self.rt_data[node_id].final_y = depth;

        let node = self.tree.node(node_id)?;
        let level_separation = self.config.level_separation;

        for &child_id in &node.children {
            self.second_walk(child_id, final_x, depth + level_separation)?;
        }

        Ok(())
    }

    /// Phase 3: Apply orientation transformation and finalize positions
    fn finalize_positions(&mut self) -> Result<(), LayoutError> {
        // Note: RT algorithm produces center coordinates for nodes
        // For compatibility with the deprecated LayoutManager, we do NOT
        // convert to top-left corners here. Instead, positions represent
        // node centers, and rendering/connection logic must account for this.
        // This matches the behavior of the old core::LayoutManager.
        //
        // Additionally, we truncate to integers to match the app's behavior
        // which uses: `let base_x = ((rt_data.final_y - min_y) * scale + offset_x) as i32;`
        // Note: `as i32` truncates toward zero, not floor

        for i in 0..self.tree.len() {
            let x = self.rt_data[i].final_x; // horizontal offset (center)
            let y = self.rt_data[i].final_y; // depth (center)

            // RT algorithm: both x and y represent node centers
            // Apply orientation transform without corner adjustment
            // Then round to integer coordinates for predictable rendering
            self.positions[i] = match self.config.orientation {
                TreeOrientation::LeftRight => Position::new(
                    y.round(), // depth → X (rounded for consistency)
                    x.round(), // offset → Y (rounded for consistency)
                ),
                TreeOrientation::TopDown => Position::new(
                    x.round(), // offset → X (rounded for consistency)
                    y.round(), // depth → Y (rounded for consistency)
                ),
                TreeOrientation::RightLeft => Position::new(
                    (-y).round(), // depth → X (flipped, rounded to nearest)
                    x.round(),    // offset → Y (rounded to nearest)
                ),
                TreeOrientation::BottomUp => Position::new(
                    x.round(),    // offset → X (rounded to nearest)
                    (-y).round(), // depth → Y (flipped, rounded to nearest)
                ),
            };
        }

        Ok(())
    }

    /// Calculate bounding box for the layout
    fn calculate_bounds(&self) -> LayoutBounds {
        if self.positions.is_empty() {
            return LayoutBounds::default();
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        // Positions are top-left corners, so bounds must include full node dimensions
        for pos in &self.positions {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x + self.config.node_width);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y + self.config.node_height);
        }

        LayoutBounds::new(min_x, max_x, min_y, max_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::TreeBuilder;

    #[test]
    fn test_single_node() {
        let tree = Tree::new("root");
        let mut engine = LayoutEngine::new();
        let result = engine.compute(&tree).unwrap();

        assert_eq!(result.positions().len(), 1);
        // Single node centered at (0, 0) in world space
        let pos = result.position(0).unwrap();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn test_simple_tree() {
        let mut tree = Tree::new("root");
        tree.add_child(0, "child1").unwrap();
        tree.add_child(0, "child2").unwrap();

        let mut engine = LayoutEngine::new();
        let result = engine.compute(&tree).unwrap();

        assert_eq!(result.positions().len(), 3);

        // Root centered at (0, 0)
        let root_pos = result.position(0).unwrap();
        assert_eq!(root_pos.x, 0.0);

        // Children should be at next level (default level_separation=4)
        let child1_pos = result.position(1).unwrap();
        let child2_pos = result.position(2).unwrap();

        // Both children at same depth (x coordinate in LeftRight orientation)
        assert_eq!(child1_pos.x, 4.0);
        assert_eq!(child2_pos.x, 4.0);

        // Children should be at different heights (y coordinates)
        assert_ne!(child1_pos.y, child2_pos.y);
    }

    #[test]
    fn test_orientation_topdown() {
        let mut tree = Tree::new("root");
        tree.add_child(0, "child").unwrap();

        let config = LayoutConfig::new().with_orientation(TreeOrientation::TopDown);
        let mut engine = LayoutEngine::with_config(config).unwrap();
        let result = engine.compute(&tree).unwrap();

        let root_pos = result.position(0).unwrap();
        let child_pos = result.position(1).unwrap();

        // In TopDown, child should be below (higher y)
        assert!(child_pos.y > root_pos.y);
    }

    #[test]
    fn test_bounds_calculation() {
        let mut tree = Tree::new("root");
        tree.add_child(0, "child1").unwrap();
        tree.add_child(0, "child2").unwrap();

        let mut engine = LayoutEngine::new();
        let result = engine.compute(&tree).unwrap();

        let bounds = result.bounds();
        assert!(bounds.width() > 0.0);
        assert!(bounds.height() >= 0.0);
    }

    #[test]
    fn test_no_collisions() {
        let tree = TreeBuilder::new("root")
            .add_child(0, "c1")
            .unwrap()
            .add_child(0, "c2")
            .unwrap()
            .add_child(0, "c3")
            .unwrap()
            .build()
            .unwrap();

        let mut engine = LayoutEngine::new();
        let result = engine.compute(&tree).unwrap();

        // Check that siblings don't overlap
        let c1 = result.position(1).unwrap();
        let c2 = result.position(2).unwrap();
        let c3 = result.position(3).unwrap();

        let min_sep = 2.0; // From default config
        assert!((c2.y - c1.y).abs() >= min_sep);
        assert!((c3.y - c2.y).abs() >= min_sep);
    }

    #[test]
    fn test_compute_cached_returns_same_result() {
        let tree = Tree::new("root");
        let mut engine = LayoutEngine::new();

        let layout1 = engine.compute(&tree).unwrap();
        let layout2 = engine.compute_cached(&tree).unwrap();

        assert_eq!(layout1.version(), layout2.version());
        assert_eq!(layout1.position(0), layout2.position(0));
    }

    #[test]
    fn test_needs_recompute_detects_changes() {
        let mut tree = Tree::new("root");
        let mut engine = LayoutEngine::new();

        engine.compute(&tree).unwrap();
        assert!(!engine.needs_recompute(tree.version()));

        tree.add_child(0, "child").unwrap();
        assert!(engine.needs_recompute(tree.version()));
    }

    #[test]
    fn test_invalidate_cache() {
        let tree = Tree::new("root");
        let mut engine = LayoutEngine::new();

        engine.compute(&tree).unwrap();
        assert!(!engine.needs_recompute(tree.version()));

        engine.invalidate_cache();
        assert!(engine.needs_recompute(tree.version()));
    }

    #[test]
    fn test_config_change_invalidates_cache() {
        let tree = Tree::new("root");
        let mut engine = LayoutEngine::new();

        engine.compute(&tree).unwrap();
        assert!(!engine.needs_recompute(tree.version()));

        let new_config = LayoutConfig::new().with_node_size(30.0, 5.0);
        engine.set_config(new_config).unwrap();
        assert!(engine.needs_recompute(tree.version()));
    }
}
