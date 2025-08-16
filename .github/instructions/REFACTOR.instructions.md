---
applyTo: '**'
---

# Reingold-Tilford Algorithm Refactor Plan

## CRITICAL MANDATE: COMPLETE BJL REMOVAL

**⚠️ IMPORTANT: This refactor requires the complete elimination of the BJL algorithm.**

### What Must Be Removed:
- All `BJLNodeData` structures and related fields
- All BJL algorithm methods (`bjl_first_walk`, `bjl_second_walk`, `execute_shifts`, `apportion`, etc.)
- BJL-specific configuration parameters (`sibling_separation`, `subtree_separation` in BJL context)
- BJL strategy enum variants
- All BJL helper methods and data accessors

### What Must Be Preserved:
- **Left-to-Right Layout Direction**: The new Reingold-Tilford implementation must maintain horizontal tree expansion
- Terminal coordinate system with root on left, children expanding rightward
- User interaction patterns (hjkl navigation, Enter to expand)
- Visual rendering (nodes, connections, current node highlighting)

## Overview

This document outlines the refactor plan to **completely discard** the current BJL (Buchheim-Jünger-Leipert) algorithm and replace it with the classic Reingold-Tilford algorithm. The new implementation must **maintain the left-to-right tree layout direction** as a core requirement. The Reingold-Tilford algorithm is simpler, more maintainable, and specifically designed for tidy tree layouts.

**CRITICAL REQUIREMENTS:**
1. **DISCARD BJL COMPLETELY**: Remove all BJL-related code, data structures, and logic
2. **PRESERVE LEFT-TO-RIGHT LAYOUT**: The new algorithm must maintain horizontal tree expansion (root on left, children to the right)

## Current Code Analysis

### Existing Architecture Assessment
✅ **Good Separation**: `TreeStructure` and `LayoutManager` are properly separated  
✅ **Clean State Management**: `LayoutState` with dirty tracking and versioning  
✅ **Flexible Configuration**: `LayoutConfig` supports different strategies  
❌ **Over-Complex Algorithm**: BJL is unnecessarily complex for our use case  
❌ **Performance Issues**: Current BJL implementation has redundant calculations  
❌ **Maintenance Burden**: Complex apportion algorithm is hard to debug  

### Current Implementation Problems
1. **BJL Over-Engineering**: The current BJL implementation includes complex threading and apportion logic that's unnecessary for our tree visualization needs
2. **Maintenance Complexity**: The BJL algorithm's multi-phase approach with threading, conflict resolution, and shift operations is difficult to debug and extend
3. **Coordinate Confusion**: The coordinate transformation between layout space and terminal space is convoluted and error-prone
4. **Performance Overhead**: Multiple passes and complex data structures create unnecessary computational overhead
5. **Debug Difficulty**: The multi-phase BJL algorithm is hard to debug and understand, making maintenance costly

**DECISION: DISCARD BJL ENTIRELY** - The complexity is not justified for our use case

## Reingold-Tilford Algorithm Benefits

### Why Reingold-Tilford Over BJL?
- **Simplicity**: Two clear phases (postorder dimension calculation, preorder positioning)
- **Maintainability**: Easier to understand, debug, and extend
- **Performance**: More straightforward algorithm with fewer passes
- **Proven**: Classic algorithm with well-understood behavior
- **Sufficient**: Provides all required features (no overlaps, aesthetic layout, O(n) complexity)

### Algorithm Properties
- **No Node Overlaps**: Guaranteed minimum spacing between all nodes
- **Subtree Compactness**: Subtrees placed as close as possible while respecting constraints
- **Aesthetic Symmetry**: Children centered under parents when possible
- **O(n) Time Complexity**: Linear performance scaling
- **Left-to-Right Layout**: Perfect fit for our horizontal tree requirement
- **Complete BJL Replacement**: Simpler algorithm that meets all our needs without BJL complexity

## Refactor Plan

### Phase 1: Simplify Data Structures

#### 1.1 Replace BJL-Specific Data
Remove complex BJL data structures and replace with simpler Reingold-Tilford data:

```rust
// REMOVE: Complex BJLNodeData
// REPLACE WITH: Simple RTNodeData
#[derive(Clone, Default)]
pub struct RTNodeData {
    // Reingold-Tilford specific fields
    pub prelim: f32,           // Preliminary horizontal position
    pub modifier: f32,         // Modifier value for subtree positioning
    pub subtree_width: f32,    // Width of entire subtree
    pub subtree_height: f32,   // Height of entire subtree
    
    // Final coordinates
    pub final_x: f32,
    pub final_y: f32,
}

#[derive(Clone, Default)]
pub struct NodeLayout {
    // Terminal coordinates (final output)
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    
    // Algorithm-specific data
    pub rt_data: Option<RTNodeData>,
}
```

#### 1.2 Simplify Layout Strategy
Replace complex strategy enum with focused Reingold-Tilford configuration:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutStrategy {
    ReingoldTilford {
        orientation: TreeOrientation,
        node_separation: f32,     // Minimum distance between sibling nodes
        level_separation: f32,    // Distance between tree levels
        subtree_separation: f32,  // Minimum distance between subtrees
    },
}

// Keep TreeOrientation focused on left-right
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeOrientation {
    LeftRight,  // Root at left, children to the right
}
```

#### 1.3 Update Default Configuration
```rust
impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_width: 3,
            node_height: 3,
            min_horizontal_spacing: 6,  // Level separation (left-to-right distance)
            min_vertical_spacing: 4,    // Node separation (vertical distance between siblings)
            margin_top: 2,
            margin_left: 2,
            strategy: LayoutStrategy::ReingoldTilford {
                orientation: TreeOrientation::LeftRight,
                node_separation: 4.0,      // Vertical spacing between siblings
                level_separation: 6.0,     // Horizontal spacing between levels
                subtree_separation: 2.0,   // Extra space between subtrees
            },
        }
    }
}
```

### Phase 2: Implement Reingold-Tilford Algorithm

#### 2.1 Algorithm Overview
The Reingold-Tilford algorithm consists of two main phases:

**Phase 1 - Postorder Traversal (Bottom-Up)**:
1. Calculate subtree dimensions for each node
2. Set preliminary positions for nodes
3. Apply modifiers to handle subtree positioning

**Phase 2 - Preorder Traversal (Top-Down)**:
1. Apply accumulated modifiers to get final positions
2. Convert to terminal coordinates

#### 2.2 Core Algorithm Structure
```rust
impl LayoutManager {
    fn calculate_rt_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Initialize RT data for all nodes
        self.initialize_rt_data(tree);
        
        // Phase 1: Calculate subtree dimensions and preliminary positions (postorder)
        self.rt_first_walk(tree, tree.root);
        
        // Phase 2: Apply modifiers and set final positions (preorder)
        self.rt_second_walk(tree, tree.root, 0.0, 0.0);
        
        // Convert to terminal coordinates
        self.rt_to_terminal_coordinates(tree, terminal_size);
    }
}
```

#### 2.3 Phase 1: Subtree Dimension Calculation
```rust
fn rt_first_walk(&mut self, tree: &TreeStructure, node_id: usize) {
    let children = tree.nodes[node_id].children.clone();
    
    // Process all children first (postorder)
    for &child_id in &children {
        self.rt_first_walk(tree, child_id);
    }
    
    if children.is_empty() {
        // Leaf node
        self.set_leaf_position(node_id);
    } else {
        // Internal node
        self.set_internal_position(tree, node_id);
    }
    
    // Calculate subtree dimensions
    self.calculate_subtree_dimensions(tree, node_id);
}
```

#### 2.4 Phase 2: Final Position Calculation
```rust
fn rt_second_walk(&mut self, tree: &TreeStructure, node_id: usize, modifier: f32, depth: f32) {
    // Apply accumulated modifier to preliminary position
    let prelim = self.get_rt_prelim(node_id);
    let node_modifier = self.get_rt_modifier(node_id);
    
    // Set final coordinates
    self.set_rt_final_coordinates(node_id, prelim + modifier, depth);
    
    // Process children with accumulated modifier
    let level_separation = self.get_rt_level_separation();
    for &child_id in &tree.nodes[node_id].children {
        self.rt_second_walk(tree, child_id, modifier + node_modifier, depth + level_separation);
    }
}
```

### Phase 3: Position Calculation Logic

#### 3.1 Leaf Node Positioning
```rust
fn set_leaf_position(&mut self, node_id: usize) {
    // Leaf nodes are positioned at origin relative to their parent
    self.set_rt_prelim(node_id, 0.0);
    self.set_rt_modifier(node_id, 0.0);
}
```

#### 3.2 Internal Node Positioning
```rust
fn set_internal_position(&mut self, tree: &TreeStructure, node_id: usize) {
    let children = &tree.nodes[node_id].children;
    let node_separation = self.get_rt_node_separation();
    
    if children.len() == 1 {
        // Single child: position directly above it
        let child_prelim = self.get_rt_prelim(children[0]);
        self.set_rt_prelim(node_id, child_prelim);
        self.set_rt_modifier(node_id, 0.0);
    } else if children.len() >= 2 {
        // Multiple children: center between first and last child
        self.position_children_with_separation(tree, node_id, node_separation);
        
        let first_child_pos = self.get_rt_prelim(children[0]);
        let last_child_pos = self.get_rt_prelim(children[children.len() - 1]);
        let midpoint = (first_child_pos + last_child_pos) / 2.0;
        
        self.set_rt_prelim(node_id, midpoint);
        self.set_rt_modifier(node_id, 0.0);
    }
}
```

#### 3.3 Child Positioning with Separation
```rust
fn position_children_with_separation(&mut self, tree: &TreeStructure, parent_id: usize, separation: f32) {
    let children = &tree.nodes[parent_id].children;
    
    // Position first child at origin
    self.set_rt_prelim(children[0], 0.0);
    
    // Position subsequent children with proper separation
    for i in 1..children.len() {
        let prev_child = children[i - 1];
        let current_child = children[i];
        
        let prev_subtree_width = self.get_rt_subtree_width(prev_child);
        let prev_pos = self.get_rt_prelim(prev_child);
        
        let new_pos = prev_pos + prev_subtree_width + separation;
        self.set_rt_prelim(current_child, new_pos);
    }
}
```

### Phase 4: Subtree Dimension Calculation

#### 4.1 Width Calculation
```rust
fn calculate_subtree_dimensions(&mut self, tree: &TreeStructure, node_id: usize) {
    let children = &tree.nodes[node_id].children;
    
    if children.is_empty() {
        // Leaf node: width = node width
        let node_width = self.config.node_width as f32;
        self.set_rt_subtree_width(node_id, node_width);
        self.set_rt_subtree_height(node_id, self.config.node_height as f32);
    } else {
        // Internal node: width = span of all children + separations
        let first_child_pos = self.get_rt_prelim(children[0]);
        let last_child = children[children.len() - 1];
        let last_child_pos = self.get_rt_prelim(last_child);
        let last_child_width = self.get_rt_subtree_width(last_child);
        
        let total_width = (last_child_pos + last_child_width) - first_child_pos;
        self.set_rt_subtree_width(node_id, total_width);
        
        // Height = max child height + current node height
        let max_child_height = children.iter()
            .map(|&child_id| self.get_rt_subtree_height(child_id))
            .fold(0.0, f32::max);
        
        self.set_rt_subtree_height(node_id, max_child_height + self.config.node_height as f32);
    }
}
```

### Phase 5: Coordinate Transformation

#### 5.1 Terminal Coordinate Conversion
```rust
fn rt_to_terminal_coordinates(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
    // Find layout bounds
    let (min_x, max_x, min_y, max_y) = self.calculate_layout_bounds();
    
    // Calculate scaling to fit in terminal
    let layout_width = max_x - min_x;
    let layout_height = max_y - min_y;
    
    let available_width = terminal_size.0.saturating_sub(self.config.margin_left * 2) as f32;
    let available_height = terminal_size.1.saturating_sub(self.config.margin_top * 2) as f32;
    
    let scale = calculate_optimal_scale(layout_width, layout_height, available_width, available_height);
    
    // Transform coordinates for left-to-right layout
    for layout in &mut self.state.node_layouts {
        if let Some(ref rt_data) = layout.rt_data {
            // For LeftRight orientation:
            // - rt_data.final_y becomes terminal x (horizontal position)
            // - rt_data.final_x becomes terminal y (vertical position)
            layout.x = self.transform_coordinate(rt_data.final_y, min_y, scale, self.config.margin_left as f32);
            layout.y = self.transform_coordinate(rt_data.final_x, min_x, scale, self.config.margin_top as f32);
            
            // Ensure coordinates are within terminal bounds
            layout.x = layout.x.min(terminal_size.0.saturating_sub(self.config.node_width));
            layout.y = layout.y.min(terminal_size.1.saturating_sub(self.config.node_height));
        }
    }
}
```

### Phase 6: Helper Methods

#### 6.1 RT Data Access Methods
```rust
// Preliminary position access
fn get_rt_prelim(&self, node_id: usize) -> f32 {
    self.state.node_layouts[node_id].rt_data.as_ref().map_or(0.0, |d| d.prelim)
}

fn set_rt_prelim(&mut self, node_id: usize, value: f32) {
    if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
        rt_data.prelim = value;
    }
}

// Modifier access
fn get_rt_modifier(&self, node_id: usize) -> f32 {
    self.state.node_layouts[node_id].rt_data.as_ref().map_or(0.0, |d| d.modifier)
}

fn set_rt_modifier(&mut self, node_id: usize, value: f32) {
    if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
        rt_data.modifier = value;
    }
}

// Subtree dimension access
fn get_rt_subtree_width(&self, node_id: usize) -> f32 {
    self.state.node_layouts[node_id].rt_data.as_ref().map_or(0.0, |d| d.subtree_width)
}

fn set_rt_subtree_width(&mut self, node_id: usize, value: f32) {
    if let Some(ref mut rt_data) = self.state.node_layouts[node_id].rt_data {
        rt_data.subtree_width = value;
    }
}
```

#### 6.2 Configuration Access
```rust
fn get_rt_node_separation(&self) -> f32 {
    match self.config.strategy {
        LayoutStrategy::ReingoldTilford { node_separation, .. } => node_separation,
    }
}

fn get_rt_level_separation(&self) -> f32 {
    match self.config.strategy {
        LayoutStrategy::ReingoldTilford { level_separation, .. } => level_separation,
    }
}

fn get_rt_subtree_separation(&self) -> f32 {
    match self.config.strategy {
        LayoutStrategy::ReingoldTilford { subtree_separation, .. } => subtree_separation,
    }
}
```

### Phase 7: Migration Strategy

#### 7.1 Backward Compatibility
```rust
// Keep old BJL interface temporarily for migration
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
```

#### 7.2 Incremental Migration Steps
1. **Step 1**: Add RT data structures alongside BJL data
2. **Step 2**: Implement RT algorithm as alternative code path
3. **Step 3**: Switch default strategy to Reingold-Tilford
4. **Step 4**: Update tests to verify RT algorithm correctness
5. **Step 5**: Remove BJL code and data structures completely
6. **Step 6**: Clean up and optimize RT implementation

### Phase 8: Testing Strategy

#### 8.1 Correctness Tests
```rust
#[test]
fn test_rt_no_overlaps() {
    // Verify no node overlaps in various tree configurations
    let test_cases = [
        create_linear_tree(5),
        create_balanced_tree(3, 2),
        create_unbalanced_tree(),
        create_single_node_tree(),
    ];
    
    for tree in test_cases {
        let mut layout_manager = LayoutManager::new(LayoutConfig::default());
        layout_manager.calculate_layout(&tree, (80, 24));
        
        assert_no_node_overlaps(&layout_manager.state.node_layouts);
    }
}

#[test]
fn test_rt_minimum_spacing() {
    // Verify minimum spacing requirements are met
    let tree = create_test_tree_with_siblings();
    let mut layout_manager = LayoutManager::new(LayoutConfig::default());
    layout_manager.calculate_layout(&tree, (80, 24));
    
    verify_minimum_spacing(&layout_manager, &tree);
}

#[test]
fn test_rt_aesthetic_properties() {
    // Verify children are centered under parents when possible
    let tree = create_symmetric_tree();
    let mut layout_manager = LayoutManager::new(LayoutConfig::default());
    layout_manager.calculate_layout(&tree, (80, 24));
    
    verify_aesthetic_centering(&layout_manager, &tree);
}
```

#### 8.2 Performance Tests
```rust
#[test]
fn test_rt_performance() {
    let large_tree = create_large_test_tree(1000);
    let mut layout_manager = LayoutManager::new(LayoutConfig::default());
    
    let start = std::time::Instant::now();
    layout_manager.calculate_layout(&large_tree, (120, 40));
    let duration = start.elapsed();
    
    // Should complete in reasonable time
    assert!(duration.as_millis() < 50, "Layout took too long: {:?}", duration);
}
```

### Phase 9: Implementation Priority

#### Week 1: Data Structure Refactor
- Replace BJLNodeData with RTNodeData
- Update LayoutStrategy enum
- Modify LayoutConfig defaults
- Update initialization methods

#### Week 2: Core Algorithm Implementation
- Implement `rt_first_walk` (postorder traversal)
- Implement `rt_second_walk` (preorder traversal)
- Add subtree dimension calculation
- Implement coordinate transformation

#### Week 3: Testing and Debugging
- Create comprehensive test suite
- Debug coordinate transformation issues
- Verify left-to-right layout works correctly
- Performance testing and optimization

#### Week 4: Migration and Cleanup
- Remove BJL algorithm code completely
- Update documentation
- Clean up unused data structures
- Final integration testing

### Success Criteria

#### Functional Requirements
- ✅ **No Overlaps**: Zero node overlaps in any tree configuration
- ✅ **Proper Spacing**: Minimum spacing requirements respected
- ✅ **Left-to-Right Layout**: Horizontal tree expansion maintained
- ✅ **Aesthetic Quality**: Children centered under parents where possible
- ✅ **Terminal Adaptation**: Layout scales properly to terminal size

#### Performance Requirements
- ✅ **O(n) Complexity**: Linear scaling with tree size
- ✅ **Fast Calculation**: Layout calculation < 16ms for 1000-node trees
- ✅ **Memory Efficiency**: Reduced memory usage compared to BJL
- ✅ **Responsive UI**: No perceptible lag during interaction

#### Quality Requirements
- ✅ **Code Simplicity**: Easier to understand and maintain than BJL
- ✅ **Debugging**: Clear algorithm flow for troubleshooting
- ✅ **Extensibility**: Easy to add new features or orientations
- ✅ **Reliability**: Robust behavior across different tree shapes

### Risk Mitigation

#### Technical Risks
- **Coordinate System Confusion**: Clear documentation of coordinate transformations
- **Scaling Issues**: Comprehensive testing with various terminal sizes
- **Edge Cases**: Handle single nodes, linear trees, and deep trees properly

#### Migration Risks
- **Breaking Changes**: Maintain backward compatibility during transition
- **Performance Regression**: Benchmark against current implementation
- **Visual Changes**: Users may notice different layout aesthetics

### Future Enhancements

After successful Reingold-Tilford implementation:

1. **Multiple Orientations**: Add top-down, bottom-up orientations
2. **Dynamic Spacing**: Adjust spacing based on node content
3. **Curved Connections**: Enhance visual appeal with curved lines
4. **Animated Transitions**: Smooth transitions between layout changes
5. **Custom Node Shapes**: Support different node visualizations

This refactor will result in a cleaner, more maintainable, and equally powerful tree layout system that specifically optimizes for the left-to-right horizontal layout that the application requires.
