# FIX_COLLISIONS: Reingold-Tilford Algorithm Refactor Plan

## CRITICAL ISSUE: Node Overlap Bug Resolution

**Problem**: Current implementation has node overlaps caused by mixed algorithm concerns and position value overwriting.

**Root Cause**: The algorithm mixes subtree dimension calculation with positioning logic, causing preliminary positions to be calculated correctly but then overwritten or invalidated during subsequent phases.

## SOLUTION: Pure Reingold-Tilford Three-Phase Implementation

### **PHASE 1: Clean Algorithm Separation**

#### 1.1 Remove Mixed Concerns in `rt_first_walk`
**Current Problem**: The method calls both `calculate_subtree_dimensions_initial` AND `calculate_subtree_dimensions`, causing position values to be overwritten.

**Fix Required**:
```rust
fn rt_first_walk(&mut self, tree: &TreeStructure, node_id: usize) {
    let children = tree.nodes[node_id].children.clone();
    
    // Process all children first (postorder)
    for &child_id in &children {
        self.rt_first_walk(tree, child_id);
    }
    
    // ONLY do positioning logic - NO dimension calculation
    if children.is_empty() {
        self.set_leaf_position(node_id);
    } else {
        self.set_internal_position(tree, node_id);
    }
    
    // DO NOT call calculate_subtree_dimensions here!
}
```

#### 1.2 Eliminate `calculate_subtree_dimensions_initial`
**Problem**: This method creates circular dependencies between positioning and dimensions.

**Solution**: Remove this method entirely. Calculate subtree dimensions based on FINAL positioned coordinates, not estimated values.

#### 1.3 Fix `set_internal_position` Logic
**Current Problem**: The method tries to position children using subtree widths that may not be accurate.

**Fix Required**:
```rust
fn set_internal_position(&mut self, tree: &TreeStructure, node_id: usize) {
    let children = &tree.nodes[node_id].children;
    
    if children.len() == 1 {
        // Single child: position directly above it
        let child_prelim = self.get_rt_prelim(children[0]);
        self.set_rt_prelim(node_id, child_prelim);
        self.set_rt_modifier(node_id, 0.0);
    } else if children.len() >= 2 {
        // Position children with simple fixed separation
        self.position_children_with_fixed_separation(tree, node_id);
        
        // Center parent between first and last child
        let first_child_pos = self.get_rt_prelim(children[0]);
        let last_child_pos = self.get_rt_prelim(children[children.len() - 1]);
        let midpoint = (first_child_pos + last_child_pos) / 2.0;
        
        self.set_rt_prelim(node_id, midpoint);
        self.set_rt_modifier(node_id, 0.0);
    }
}
```

### **PHASE 2: Implement Pure Reingold-Tilford Phases**

#### 2.1 Phase 1: First Walk (Positioning Only)
**Purpose**: Calculate preliminary positions and modifiers ONLY.
**Rules**:
- No subtree dimension calculation
- No final coordinate calculation
- Only set `prelim` and `modifier` values
- Use fixed node separation (not dynamic subtree-based)

#### 2.2 Phase 2: Second Walk (Apply Modifiers Only)
**Purpose**: Apply accumulated modifiers to get final layout coordinates.
**Rules**:
- Only read `prelim` and `modifier` values
- Only write `final_x` and `final_y` values
- No position recalculation
- No dimension calculation

#### 2.3 Phase 3: Terminal Coordinate Transformation
**Purpose**: Transform layout coordinates to terminal screen coordinates.
**Rules**:
- Only read `final_x` and `final_y` values
- Only write terminal `x` and `y` coordinates
- Handle scaling and bounds checking
- No layout recalculation

### **PHASE 3: Implementation Steps**

#### Step 1: Remove Problematic Methods
- [ ] Remove `calculate_subtree_dimensions_initial` method
- [ ] Remove the second `calculate_subtree_dimensions` call from `rt_first_walk`
- [ ] Remove all debug print statements

#### Step 2: Fix Child Positioning Logic
- [ ] Replace `position_children_with_separation` with `position_children_with_fixed_separation`
- [ ] Use simple fixed separation instead of subtree-width-based separation
- [ ] Ensure positioning logic doesn't depend on subtree dimensions

#### Step 3: Implement Fixed Separation Positioning
```rust
fn position_children_with_fixed_separation(&mut self, tree: &TreeStructure, parent_id: usize) {
    let children = &tree.nodes[parent_id].children;
    let separation = self.get_rt_node_separation();
    
    // Position children with fixed separation
    for (i, &child_id) in children.iter().enumerate() {
        let position = i as f32 * (self.config.node_width as f32 + separation);
        self.set_rt_prelim(child_id, position);
    }
}
```

#### Step 4: Remove Subtree Width Dependencies
- [ ] Remove all calls to `get_rt_subtree_width` from positioning logic
- [ ] Only calculate subtree dimensions after positioning is complete
- [ ] Use positioned coordinates to determine actual subtree bounds

#### Step 5: Add Position Validation
```rust
fn validate_no_overlaps(&self, tree: &TreeStructure) -> Result<(), String> {
    // Check that no two nodes have identical positions
    // Check that minimum separation is maintained
    // Return error with details if overlaps found
}
```

#### Step 6: Clean Algorithm Flow
```rust
fn calculate_rt_layout(&mut self, tree: &TreeStructure, terminal_size: (u16, u16)) {
    // Initialize RT data
    self.initialize_rt_data(tree);
    
    // Phase 1: Calculate preliminary positions (postorder)
    self.rt_first_walk(tree, tree.root);
    
    // Phase 2: Apply modifiers to get final positions (preorder)
    self.rt_second_walk(tree, tree.root, 0.0, 0.0);
    
    // Phase 3: Convert to terminal coordinates
    self.rt_to_terminal_coordinates(tree, terminal_size);
    
    // Validation (debug builds only)
    #[cfg(debug_assertions)]
    if let Err(msg) = self.validate_no_overlaps(tree) {
        panic!("Layout validation failed: {}", msg);
    }
}
```

### **PHASE 4: Testing Strategy**

#### 4.1 Unit Tests for Each Phase
- [ ] Test Phase 1: Verify preliminary positions are calculated correctly
- [ ] Test Phase 2: Verify modifiers are applied correctly to get final positions
- [ ] Test Phase 3: Verify terminal coordinate transformation

#### 4.2 Integration Tests
- [ ] Test the complete three-phase flow
- [ ] Verify no position values are overwritten between phases
- [ ] Test with various tree shapes (linear, balanced, unbalanced)

#### 4.3 Overlap Detection Tests
- [ ] Add comprehensive overlap detection in tests
- [ ] Test minimum separation requirements
- [ ] Test edge cases (single node, two nodes, deep trees)

### **PHASE 5: Key Success Criteria**

#### 5.1 Algorithm Purity
- ✅ Each phase has single responsibility
- ✅ No circular dependencies between positioning and dimensions
- ✅ Position values are never overwritten after calculation

#### 5.2 Collision Resolution
- ✅ Zero node overlaps in all test cases
- ✅ Minimum separation requirements met
- ✅ Consistent positioning across different tree shapes

#### 5.3 Performance
- ✅ O(n) time complexity maintained
- ✅ No redundant calculations
- ✅ Clean, maintainable code structure

### **IMPLEMENTATION PRIORITY**

#### Week 1: Core Algorithm Cleanup
- Remove mixed concerns from `rt_first_walk`
- Implement fixed separation positioning
- Remove problematic dimension calculation methods

#### Week 2: Phase Separation
- Ensure clean separation between the three phases
- Add validation and error detection
- Comprehensive testing

#### Week 3: Optimization and Polish
- Performance optimization
- Code cleanup and documentation
- Final integration testing

### **CRITICAL RULES FOR IMPLEMENTATION**

1. **Never mix positioning with dimension calculation**
2. **Each phase must have single, clear responsibility**
3. **Position values must never be overwritten after calculation**
4. **Use fixed separation in positioning, not dynamic subtree-based**
5. **Validate algorithm correctness at each step**

This refactor will result in a clean, maintainable Reingold-Tilford implementation that eliminates the node overlap bugs while maintaining the required left-to-right tree layout.
