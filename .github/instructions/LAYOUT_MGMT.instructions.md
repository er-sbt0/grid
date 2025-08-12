---
applyTo: '**'
---
# Layout State Management Design Document

## Overview

This document outlines the design and implementation plan for transforming the current tree visualization from a simple position-based approach to a sophisticated Layout State Management system that eliminates node collisions and provides scalable tree layout.

## Current Problems

1. **Mixed Concerns**: Layout data (x, y) mixed with business data (label, children)
2. **Collision-Prone**: Fixed offsets (+10 horizontal, ±4 vertical) cause inevitable overlaps
3. **No Planning**: Positions calculated locally without global tree awareness
4. **Performance**: Recalculates everything on every change
5. **Inflexible**: Hardcoded spacing doesn't adapt to tree size or terminal dimensions

## Design Goals

- **Separation of Concerns**: Isolate layout logic from tree structure
- **Collision-Free**: Guarantee no node overlaps through proper spacing calculation
- **Performance**: Efficient incremental updates and caching
- **Adaptability**: Handle terminal resizing and different tree configurations
- **Maintainability**: Clean, testable architecture

## Architecture Design

### Core Components

```rust
// 1. Pure Tree Structure (Business Logic)
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

// 2. Layout Data (Presentation Logic)
struct NodeLayout {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    subtree_width: u16,   // Total width needed for this subtree
    subtree_height: u16,  // Total height needed for this subtree
}

// 3. Layout State Management
struct LayoutState {
    node_layouts: Vec<NodeLayout>,
    is_dirty: bool,
    subtree_dirty: Vec<bool>,
    last_terminal_size: (u16, u16),
    layout_version: u64,
    calculation_in_progress: bool,
}

// 4. Layout Manager (Core Logic)
struct LayoutManager {
    state: LayoutState,
    config: LayoutConfig,
}

// 5. Configuration
struct LayoutConfig {
    node_width: u16,
    node_height: u16,
    min_horizontal_spacing: u16,
    min_vertical_spacing: u16,
    margin_top: u16,
    margin_left: u16,
}
```

## Implementation Plan

### Phase 1: Data Structure Refactoring

#### Step 1.1: Separate Tree from Layout
- Create `TreeStructure` for pure tree data
- Create `NodeLayout` for position/size data
- Create `LayoutManager` wrapper

#### Step 1.2: Update Core Functions
- Modify `expand_current_node()` to only update tree structure
- Remove layout calculations from tree manipulation
- Update drawing functions to use separated data

#### Step 1.3: Basic Layout Manager
```rust
impl LayoutManager {
    fn new(config: LayoutConfig) -> Self;
    fn calculate_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16));
    fn get_node_position(&self, node_id: usize) -> Option<(u16, u16)>;
    fn mark_dirty(&mut self);
}
```

### Phase 2: Collision-Free Layout Algorithm

#### Step 2.1: Subtree Dimension Calculation
```rust
fn calculate_subtree_dimensions(
    tree: &TreeStructure, 
    node_id: usize, 
    layouts: &mut [NodeLayout],
    config: &LayoutConfig
) {
    // Post-order traversal to calculate bottom-up
    // Each node's subtree width = sum of children widths + spacing
    // Each node's subtree height = max depth in subtree
}
```

#### Step 2.2: Hierarchical Positioning
```rust
fn position_nodes(
    tree: &TreeStructure,
    layouts: &mut [NodeLayout],
    config: &LayoutConfig,
    terminal_size: (u16, u16)
) {
    // Level-by-level positioning
    // Center children under parents
    // Ensure minimum spacing requirements
}
```

#### Step 2.3: Collision Detection
```rust
fn detect_collisions(layouts: &[NodeLayout]) -> Vec<(usize, usize)>;
fn resolve_collisions(layouts: &mut [NodeLayout], collisions: Vec<(usize, usize)>);
```

### Phase 3: Performance Optimization

#### Step 3.1: Dirty Tracking
```rust
impl LayoutState {
    fn mark_node_dirty(&mut self, node_id: usize);
    fn mark_subtree_dirty(&mut self, root_id: usize);
    fn is_clean(&self) -> bool;
    fn needs_recalculation(&self) -> bool;
}
```

#### Step 3.2: Incremental Updates
- Only recalculate dirty subtrees
- Propagate changes up the tree hierarchy
- Cache intermediate calculations

#### Step 3.3: Lazy Calculation
```rust
impl LayoutManager {
    fn ensure_layout_valid(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        if self.state.is_dirty || self.terminal_size_changed(terminal_size) {
            self.calculate_layout(tree, terminal_size);
        }
    }
}
```

### Phase 4: Advanced Features

#### Step 4.1: Terminal Adaptation
- Detect terminal size changes
- Implement layout compaction for small screens
- Add horizontal scrolling for wide trees

#### Step 4.2: Layout Strategies
```rust
enum LayoutStrategy {
    Compact,      // Minimize space usage
    Balanced,     // Balance between space and readability
    Spacious,     // Maximize readability
}
```

#### Step 4.3: Animation Support
- Store previous positions for smooth transitions
- Implement position interpolation
- Add layout change animations

## Detailed Algorithm

### Subtree Width Calculation
```
For each node (post-order):
  If leaf node:
    subtree_width = node_width
  Else:
    child_widths = sum(child.subtree_width for child in children)
    spacing_width = (children.count - 1) * min_horizontal_spacing
    subtree_width = max(node_width, child_widths + spacing_width)
```

### Node Positioning
```
For each level (breadth-first):
  For each node in level:
    If root:
      x = terminal_width / 2 - node_width / 2
      y = margin_top
    Else:
      parent_center_x = parent.x + parent.width / 2
      total_children_width = sum(child.subtree_width) + spacing
      start_x = parent_center_x - total_children_width / 2
      
      For each child:
        child.x = start_x + accumulated_width
        child.y = parent.y + min_vertical_spacing
        accumulated_width += child.subtree_width + spacing
```

## Integration Points

### Main Application Changes
```rust
struct TreeApp {
    tree: TreeStructure,           // Separated tree data
    layout_manager: LayoutManager, // Layout management
    terminal: DefaultTerminal,
}

impl TreeApp {
    fn draw(&mut self) {
        // Ensure layout is valid before drawing
        self.layout_manager.ensure_layout_valid(&self.tree, terminal_size);
        
        // Draw using layout positions
        for (node_id, node) in self.tree.nodes.iter().enumerate() {
            if let Some((x, y)) = self.layout_manager.get_node_position(node_id) {
                draw_node_at_position(frame, node, x, y, is_current);
            }
        }
    }
    
    fn expand_node(&mut self, node_id: usize) {
        // Only modify tree structure
        self.tree.expand_node(node_id);
        
        // Mark layout as needing recalculation
        self.layout_manager.mark_subtree_dirty(node_id);
    }
}
```

## Testing Strategy

### Unit Tests
- Test subtree dimension calculations
- Test positioning algorithms
- Test collision detection
- Test dirty tracking logic

### Integration Tests
- Test full layout calculation pipeline
- Test terminal resize handling
- Test incremental updates

### Performance Tests
- Benchmark large tree layouts
- Test memory usage with deep trees
- Measure layout calculation times

## Migration Path

1. **Backward Compatibility**: Keep current Node struct during transition
2. **Gradual Migration**: Implement new system alongside old one
3. **Feature Parity**: Ensure all current functionality works
4. **Performance Validation**: Verify improvements before removing old code
5. **Clean Removal**: Remove old layout code once new system is stable

## Success Metrics

- **No Collisions**: Zero node overlaps in any tree configuration
- **Performance**: Layout calculation < 16ms for trees up to 1000 nodes
- **Memory**: Layout data < 2x tree structure memory usage
- **Maintainability**: Layout logic in separate, testable modules
- **Adaptability**: Handles terminal resize smoothly

## Future Enhancements

1. **Different Tree Layouts**: Radial, force-directed, balloon layouts
2. **Node Grouping**: Visual clustering of related nodes
3. **Zoom/Pan**: Navigate large trees interactively
4. **Export**: Save tree layouts to files
5. **Custom Styling**: Per-node visual customization
