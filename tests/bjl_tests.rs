#[cfg(test)]
mod bjl_tests {
    use grid::{LayoutConfig, LayoutManager, TreeNode, TreeStructure};

    #[test]
    fn test_bjl_no_overlaps() {
        // Create test tree with various shapes
        let tree_structure = TreeStructure {
            nodes: vec![
                TreeNode {
                    label: "Root".to_string(),
                    children: vec![1, 2],
                },
                TreeNode {
                    label: "Child1".to_string(),
                    children: vec![3, 4],
                },
                TreeNode {
                    label: "Child2".to_string(),
                    children: vec![5],
                },
                TreeNode {
                    label: "Leaf1".to_string(),
                    children: vec![],
                },
                TreeNode {
                    label: "Leaf2".to_string(),
                    children: vec![],
                },
                TreeNode {
                    label: "Leaf3".to_string(),
                    children: vec![],
                },
            ],
            root: 0,
            current_node: 0,
            node_counter: 6,
        };

        let mut layout_manager = LayoutManager::new(LayoutConfig::default());
        layout_manager.calculate_layout(&tree_structure, (80, 24));

        // Print debug information
        let layouts = &layout_manager.state.node_layouts;
        for (i, layout) in layouts.iter().enumerate() {
            println!(
                "Node {}: ({}, {}) {}x{}",
                i, layout.x, layout.y, layout.width, layout.height
            );
        }

        // Verify no node overlaps exist
        for (i, layout_a) in layouts.iter().enumerate() {
            for (j, layout_b) in layouts.iter().enumerate() {
                if i != j {
                    // Check if nodes overlap
                    let overlap_x = layout_a.x < layout_b.x + layout_b.width
                        && layout_b.x < layout_a.x + layout_a.width;
                    let overlap_y = layout_a.y < layout_b.y + layout_b.height
                        && layout_b.y < layout_a.y + layout_a.height;

                    if overlap_x && overlap_y {
                        println!("Overlap detected between nodes {} and {}", i, j);
                        println!(
                            "Node {}: ({},{}) to ({},{})",
                            i,
                            layout_a.x,
                            layout_a.y,
                            layout_a.x + layout_a.width,
                            layout_a.y + layout_a.height
                        );
                        println!(
                            "Node {}: ({},{}) to ({},{})",
                            j,
                            layout_b.x,
                            layout_b.y,
                            layout_b.x + layout_b.width,
                            layout_b.y + layout_b.height
                        );
                    }

                    assert!(
                        !(overlap_x && overlap_y),
                        "Nodes {} and {} overlap at ({},{}) and ({},{})",
                        i,
                        j,
                        layout_a.x,
                        layout_a.y,
                        layout_b.x,
                        layout_b.y
                    );
                }
            }
        }
    }

    #[test]
    fn test_bjl_proper_spacing() {
        let tree_structure = TreeStructure {
            nodes: vec![
                TreeNode {
                    label: "Root".to_string(),
                    children: vec![1, 2],
                },
                TreeNode {
                    label: "Child1".to_string(),
                    children: vec![],
                },
                TreeNode {
                    label: "Child2".to_string(),
                    children: vec![],
                },
            ],
            root: 0,
            current_node: 0,
            node_counter: 3,
        };

        let mut layout_manager = LayoutManager::new(LayoutConfig::default());
        layout_manager.calculate_layout(&tree_structure, (80, 24));

        // Verify minimum spacing requirements met
        let layouts = &layout_manager.state.node_layouts;
        let child1_layout = &layouts[1];
        let child2_layout = &layouts[2];

        // Check spacing between siblings (avoiding overflow)
        // For LeftRight orientation, siblings are separated vertically (y-direction)
        let spacing = if child1_layout.y < child2_layout.y {
            child2_layout
                .y
                .saturating_sub(child1_layout.y + child1_layout.height)
        } else {
            child1_layout
                .y
                .saturating_sub(child2_layout.y + child2_layout.height)
        };

        println!(
            "Child1: ({}, {}), Child2: ({}, {}), Vertical Spacing: {}",
            child1_layout.x, child1_layout.y, child2_layout.x, child2_layout.y, spacing
        );

        assert!(
            spacing >= 1,
            "Insufficient vertical spacing between siblings: {}",
            spacing
        );
    }

    #[test]
    fn test_bjl_left_right_orientation() {
        let tree_structure = TreeStructure {
            nodes: vec![
                TreeNode {
                    label: "Root".to_string(),
                    children: vec![1, 2],
                },
                TreeNode {
                    label: "Child1".to_string(),
                    children: vec![],
                },
                TreeNode {
                    label: "Child2".to_string(),
                    children: vec![],
                },
            ],
            root: 0,
            current_node: 0,
            node_counter: 3,
        };

        let mut layout_manager = LayoutManager::new(LayoutConfig::default());
        layout_manager.calculate_layout(&tree_structure, (80, 24));

        let layouts = &layout_manager.state.node_layouts;
        let root_layout = &layouts[0];
        let child1_layout = &layouts[1];

        // In LeftRight, children should be to the right of parent
        assert!(
            child1_layout.x > root_layout.x,
            "LeftRight: Child should be to the right of parent"
        );
    }

    #[test]
    fn test_bjl_performance() {
        // Create a larger tree to test performance
        let mut tree_structure = TreeStructure {
            nodes: vec![TreeNode {
                label: "Root".to_string(),
                children: vec![],
            }],
            root: 0,
            current_node: 0,
            node_counter: 1,
        };

        // Add multiple levels
        for level in 0..4 {
            let level_start = tree_structure.nodes.len();
            let nodes_in_level = 2_usize.pow(level);

            for i in 0..nodes_in_level {
                let parent_index = if level == 0 {
                    0
                } else {
                    level_start - nodes_in_level + i
                };

                // Add two children for each node
                let child1_index = tree_structure.nodes.len();
                let child2_index = tree_structure.nodes.len() + 1;

                tree_structure.nodes.push(TreeNode {
                    label: format!("L{}N{}.1", level + 1, i),
                    children: vec![],
                });
                tree_structure.nodes.push(TreeNode {
                    label: format!("L{}N{}.2", level + 1, i),
                    children: vec![],
                });

                tree_structure.nodes[parent_index]
                    .children
                    .push(child1_index);
                tree_structure.nodes[parent_index]
                    .children
                    .push(child2_index);
            }
        }

        let start = std::time::Instant::now();
        let mut layout_manager = LayoutManager::new(LayoutConfig::default());
        layout_manager.calculate_layout(&tree_structure, (100, 30));
        let duration = start.elapsed();

        println!(
            "Layout calculation for {} nodes took: {:?}",
            tree_structure.nodes.len(),
            duration
        );

        // Should be fast for reasonable tree sizes
        assert!(
            duration.as_millis() < 100,
            "Layout calculation too slow: {:?}",
            duration
        );

        // Verify no overlaps in large tree
        let layouts = &layout_manager.state.node_layouts;
        for (i, layout_a) in layouts.iter().enumerate() {
            for (j, layout_b) in layouts.iter().enumerate() {
                if i != j {
                    let overlap_x = layout_a.x < layout_b.x + layout_b.width
                        && layout_b.x < layout_a.x + layout_a.width;
                    let overlap_y = layout_a.y < layout_b.y + layout_b.height
                        && layout_b.y < layout_a.y + layout_a.height;

                    if overlap_x && overlap_y {
                        println!(
                            "Node {}: ({},{}) size {}x{}",
                            i, layout_a.x, layout_a.y, layout_a.width, layout_a.height
                        );
                        println!(
                            "Node {}: ({},{}) size {}x{}",
                            j, layout_b.x, layout_b.y, layout_b.width, layout_b.height
                        );
                    }

                    assert!(
                        !(overlap_x && overlap_y),
                        "Large tree has overlapping nodes {} and {}",
                        i,
                        j
                    );
                }
            }
        }
    }
}
