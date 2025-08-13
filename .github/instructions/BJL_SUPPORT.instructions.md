---
applyTo: '**'
---
# Buchheim-Jünger-Leipert (BJL) Algorithm Integration Guidelines

## Overview

This document provides guidelines for integrating the Buchheim-Jünger-Leipert (BJL) tree layout algorithm into the existing Layout State Management system. BJL is the gold standard for tree visualization, offering O(n) performance with superior aesthetic results.

## Current Code Analysis

### Existing Architecture Strengths
✅ **Clean Separation**: `TreeStructure` (business logic) and `LayoutManager` (presentation logic) are properly separated
✅ **Layout State Management**: `LayoutState` with dirty tracking and versioning
✅ **Extensible Foundation**: `LayoutConfig` allows configuration customization
✅ **Performance Ready**: Infrastructure for incremental updates exists

### Current Layout Strategy
The existing code implements a **horizontal tree layout** with:
- Left-to-right expansion (root on left, children to right)
- Post-order dimension calculation (`calculate_subtree_dimensions`)
- Top-down positioning (`position_nodes` -> `position_children`)
- Fixed spacing configuration (6 units horizontal, 4 units vertical)

## BJL Integration Guidelines

### Phase 1: Strategy Pattern Foundation

#### 1.1 Introduce Layout Strategy Enum
Add layout strategy selection to support multiple algorithms:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
enum LayoutStrategy {
    Horizontal,           // Current implementation - keep as fallback
    BJL {                 // Buchheim-Jünger-Leipert algorithm
        orientation: TreeOrientation,
        sibling_separation: f32,
        subtree_separation: f32,
        level_separation: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TreeOrientation {
    TopDown,    // Root at top, children below (most common)
    LeftRight,  // Root at left, children right (current)
    BottomUp,   // Root at bottom, children above
    RightLeft,  // Root at right, children left
}
```

#### 1.2 Update LayoutConfig Structure
Extend existing configuration to support strategy selection:

```rust
struct LayoutConfig {
    // Existing fields - keep for backward compatibility
    node_width: u16,
    node_height: u16,
    min_horizontal_spacing: u16,
    min_vertical_spacing: u16,
    margin_top: u16,
    margin_left: u16,
    
    // New field for strategy selection
    strategy: LayoutStrategy,
}
```

#### 1.3 Update LayoutManager Dispatch
Modify `calculate_layout` to dispatch based on strategy:

```rust
impl LayoutManager {
    fn calculate_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Resize layout vectors (existing code)
        self.state.node_layouts.resize(tree.nodes.len(), NodeLayout::default());
        self.state.subtree_dirty.resize(tree.nodes.len(), false);

        // Strategy dispatch
        match self.config.strategy {
            LayoutStrategy::Horizontal => {
                self.calculate_horizontal_layout(tree, terminal_size);
            }
            LayoutStrategy::BJL { .. } => {
                self.calculate_bjl_layout(tree, terminal_size);
            }
        }

        // Existing cleanup code
        self.state.is_dirty = false;
        self.state.last_terminal_size = terminal_size;
        self.state.layout_version += 1;
    }

    // Rename existing implementation for clarity
    fn calculate_horizontal_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        self.calculate_subtree_dimensions(tree, tree.root);
        self.position_nodes(tree, terminal_size);
    }
}
```

### Phase 2: BJL Data Structures

#### 2.1 Extend NodeLayout for BJL
Add BJL-specific data while maintaining existing structure:

```rust
#[derive(Clone, Default)]
struct NodeLayout {
    // Existing fields - maintain backward compatibility
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    subtree_width: u16,
    subtree_height: u16,
    
    // BJL-specific data (optional for non-BJL layouts)
    bjl_data: Option<BJLNodeData>,
}

#[derive(Clone, Default)]
struct BJLNodeData {
    // Core BJL algorithm fields
    prelim: f32,              // Preliminary x-coordinate
    modifier: f32,            // Modifier for subtree positioning
    thread: Option<usize>,    // Thread to extreme subtree contour
    ancestor: usize,          // Ancestor for conflict resolution
    
    // Positioning refinement fields
    shift: f32,               // Shift value for spacing adjustment
    change: f32,              // Change value for uniform distribution
    
    // Tree structure navigation
    left_sibling: Option<usize>,
    left_most_sibling: Option<usize>,
    
    // Final floating-point coordinates (before terminal conversion)
    final_x: f32,
    final_y: f32,
}
```

### Phase 3: BJL Algorithm Implementation

#### 3.1 BJL Algorithm Structure
The BJL algorithm follows a two-phase approach:

**Phase 1 - First Walk (Post-order traversal)**:
1. Initialize BJL data and sibling relationships
2. For each node (bottom-up):
   - Process all children first
   - Set preliminary coordinates
   - Execute shifts for proper spacing
   - Run apportion algorithm for conflict resolution

**Phase 2 - Second Walk (Pre-order traversal)**:
1. Apply accumulated modifiers
2. Set final floating-point coordinates
3. Convert to terminal coordinates with scaling and centering

#### 3.2 Core BJL Methods Structure

```rust
impl LayoutManager {
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
        // Set up sibling relationships
        // Configure node dimensions
    }
    
    fn bjl_first_walk(&mut self, tree: &TreeStructure, node_id: usize) {
        // Recursive post-order traversal
        // Set preliminary coordinates
        // Execute shifts
        // Run apportion algorithm
    }
    
    fn bjl_second_walk(&mut self, tree: &TreeStructure, node_id: usize, modifier: f32, depth: f32) {
        // Apply modifiers and set final coordinates
        // Recursive pre-order traversal
    }
    
    fn bjl_to_terminal_coordinates(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
        // Calculate coordinate bounds
        // Apply scaling and centering
        // Handle orientation transforms
        // Convert to terminal u16 coordinates
    }
}
```

#### 3.3 Key Algorithm Components

**Preliminary Coordinate Assignment**:
- Leaf nodes: Position relative to left sibling with minimum separation
- Internal nodes: Position at midpoint of leftmost and rightmost children

**Execute Shifts**:
- Apply accumulated shift and change values from right to left
- Ensures uniform spacing distribution among siblings

**Apportion Algorithm**:
- Core conflict resolution between adjacent subtrees
- Walk contours of neighboring subtrees
- Detect conflicts and apply necessary shifts
- Most complex part of BJL algorithm

**Coordinate Transformation**:
- Handle different tree orientations (TopDown, LeftRight, etc.)
- Scale layout to fit within terminal bounds
- Center layout within available space
- Convert floating-point to integer coordinates

### Phase 4: Integration Guidelines

#### 4.1 Backward Compatibility Strategy
- Keep existing horizontal layout as default initially
- Maintain all existing public APIs
- Ensure existing tests continue to pass
- Provide migration path for users

#### 4.2 Configuration Strategy
```rust
impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_width: 3,
            node_height: 3,
            min_horizontal_spacing: 6,
            min_vertical_spacing: 4,
            margin_top: 2,
            margin_left: 2,
            strategy: LayoutStrategy::Horizontal, // Start with existing
        }
    }
}

// Provide BJL presets
impl LayoutConfig {
    fn bjl_top_down() -> Self {
        Self {
            strategy: LayoutStrategy::BJL {
                orientation: TreeOrientation::TopDown,
                sibling_separation: 2.0,
                subtree_separation: 4.0,
                level_separation: 6.0,
            },
            ..Default::default()
        }
    }
    
    fn bjl_left_right() -> Self {
        Self {
            strategy: LayoutStrategy::BJL {
                orientation: TreeOrientation::LeftRight,
                sibling_separation: 2.0,
                subtree_separation: 4.0,
                level_separation: 6.0,
            },
            ..Default::default()
        }
    }
}
```

#### 4.3 Runtime Strategy Switching
Add keyboard shortcuts for layout switching:

```rust
fn handle_events(app: &mut TreeApp) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            // Existing navigation code...
            
            // Layout strategy switching
            KeyCode::Char('1') => {
                app.layout_manager.config.strategy = LayoutStrategy::Horizontal;
                app.layout_manager.mark_dirty();
            }
            KeyCode::Char('2') => {
                app.layout_manager.config = LayoutConfig::bjl_top_down();
                app.layout_manager.mark_dirty();
            }
            KeyCode::Char('3') => {
                app.layout_manager.config = LayoutConfig::bjl_left_right();
                app.layout_manager.mark_dirty();
            }
            
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}
```

### Phase 5: Testing Strategy

#### 5.1 Unit Testing Approach
```rust
#[cfg(test)]
mod bjl_tests {
    use super::*;

    #[test]
    fn test_bjl_no_overlaps() {
        // Create test tree with various shapes
        // Run BJL layout
        // Verify no node overlaps exist
    }
    
    #[test]
    fn test_bjl_proper_spacing() {
        // Verify minimum spacing requirements met
        // Check sibling and subtree separations
    }
    
    #[test]
    fn test_bjl_orientations() {
        // Test all four orientations produce valid layouts
        // Verify coordinate transformations work correctly
    }
    
    #[test]
    fn test_bjl_performance() {
        // Create large trees (100, 1000 nodes)
        // Measure layout calculation time
        // Verify O(n) performance characteristics
    }
    
    #[test]
    fn test_strategy_switching() {
        // Switch between strategies
        // Verify layout recalculation occurs
        // Check no data corruption
    }
}
```

#### 5.2 Integration Testing
- Test with lazy tree expansion scenarios
- Verify terminal resizing works with BJL
- Test memory usage with large trees
- Validate drawing functions work with all strategies

### Phase 6: Implementation Priority

#### Week 1: Foundation
1. Add `LayoutStrategy` enum and update `LayoutConfig`
2. Implement strategy dispatch in `LayoutManager::calculate_layout`
3. Create `BJLNodeData` structure
4. Rename existing layout method to `calculate_horizontal_layout`

#### Week 2: Basic BJL
1. Implement `initialize_bjl_data`
2. Basic `bjl_first_walk` without apportion
3. Simple `bjl_second_walk`
4. Basic coordinate conversion for TopDown orientation

#### Week 3: Full BJL Algorithm
1. Implement complete apportion algorithm
2. Add shift execution logic
3. Support all orientations
4. Add proper scaling and centering

#### Week 4: Polish and Integration
1. Add runtime strategy switching
2. Performance optimization
3. Comprehensive testing
4. Documentation and examples

### Success Criteria

#### Functional Requirements
- ✅ No node overlaps in any tree configuration
- ✅ Proper spacing according to BJL parameters
- ✅ Support for all four orientations
- ✅ Smooth strategy switching at runtime
- ✅ Maintains existing functionality

#### Performance Requirements
- ✅ Layout calculation < 16ms for trees up to 1000 nodes
- ✅ Memory overhead < 2x current usage
- ✅ No perceptible lag during tree expansion
- ✅ Linear performance scaling O(n)

#### Quality Requirements
- ✅ Visually superior to current horizontal layout
- ✅ Better space utilization
- ✅ Aesthetically pleasing node arrangement
- ✅ Consistent spacing and alignment

### Risk Mitigation

#### Technical Risks
- **Complex Algorithm**: BJL apportion algorithm is intricate
  - *Mitigation*: Implement incrementally, start with simplified version
- **Floating Point Precision**: Coordinate calculations use f32
  - *Mitigation*: Careful rounding when converting to terminal coordinates
- **Performance Regression**: BJL might be slower than current
  - *Mitigation*: Performance testing and optimization

#### Integration Risks
- **Breaking Changes**: New features might break existing code
  - *Mitigation*: Maintain backward compatibility, extensive testing
- **User Experience**: Strategy switching might be confusing
  - *Mitigation*: Clear documentation, intuitive defaults

This phased approach ensures a smooth transition to world-class tree layout quality while maintaining the robust architecture and performance characteristics of the existing system.
