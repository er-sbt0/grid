#[cfg(test)]
mod subtree_collision_tests {
    use grid::{LayoutConfig, LayoutManager, TreeNode, TreeStructure};

    /// Helper function to create a layout manager and calculate layout without validation
    fn create_layout(tree: &TreeStructure, terminal_size: (u16, u16)) -> LayoutManager {
        let mut layout_manager = LayoutManager::new(LayoutConfig::default());
        // We'll call the internal method without validation for debugging
        layout_manager.calculate_layout_for_testing(tree, terminal_size);
        layout_manager
    }

    /// Helper function to print node positions for debugging
    fn print_node_positions(layout_manager: &LayoutManager, tree: &TreeStructure) {
        println!("\n=== Node Positions ===");
        for (i, layout) in layout_manager.state.node_layouts.iter().enumerate() {
            if let Some(ref rt_data) = layout.rt_data {
                println!(
                    "Node {} ({}): prelim={:.2}, modifier={:.2}, final=({:.2}, {:.2}), terminal=({}, {})",
                    i, tree.nodes[i].label, rt_data.prelim, rt_data.modifier,
                    rt_data.final_x, rt_data.final_y, layout.x, layout.y
                );
            }
        }
    }

    /// Helper function to check for collisions and return details
    fn find_collisions(
        layout_manager: &LayoutManager,
        min_separation: f32,
    ) -> Vec<(usize, usize, f32)> {
        let mut collisions = Vec::new();

        for i in 0..layout_manager.state.node_layouts.len() {
            for j in (i + 1)..layout_manager.state.node_layouts.len() {
                if let (Some(rt_data_i), Some(rt_data_j)) = (
                    &layout_manager.state.node_layouts[i].rt_data,
                    &layout_manager.state.node_layouts[j].rt_data,
                ) {
                    // Check if nodes are on the same level
                    if (rt_data_i.final_y - rt_data_j.final_y).abs() < 0.1 {
                        let distance = (rt_data_i.final_x - rt_data_j.final_x).abs();
                        if distance < min_separation {
                            collisions.push((i, j, distance));
                        }
                    }
                }
            }
        }

        collisions
    }

    #[test]
    fn test_simple_subtree_collision() {
        // Test case: Two parent nodes with children that should not overlap
        // Tree structure:
        //     Root(0)
        //    /       \
        //   P1(1)    P2(2)
        //  /  \        \
        // L1(3) L2(4)  L3(5)
        //
        // Expected issue: L1(3) and L3(5) might collide because they're positioned
        // independently by their parents without considering global conflicts

        let tree = TreeStructure {
            nodes: vec![
                TreeNode {
                    label: "Root".to_string(),
                    children: vec![1, 2],
                },
                TreeNode {
                    label: "Parent1".to_string(),
                    children: vec![3, 4],
                },
                TreeNode {
                    label: "Parent2".to_string(),
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

        let layout_manager = create_layout(&tree, (80, 24));
        print_node_positions(&layout_manager, &tree);

        let config = LayoutConfig::default();
        let min_separation = match config.strategy {
            grid::LayoutStrategy::ReingoldTilford {
                node_separation, ..
            } => node_separation,
            _ => 4.0,
        };

        let collisions = find_collisions(&layout_manager, min_separation);

        if !collisions.is_empty() {
            println!("\n=== COLLISIONS DETECTED ===");
            for (i, j, distance) in &collisions {
                println!(
                    "Nodes {} and {} collide: distance={:.2}, required={:.2}",
                    i, j, distance, min_separation
                );
            }
        }

        // This test is expected to fail initially, showing the collision bug
        assert!(
            collisions.is_empty(),
            "Found {} collision(s) - this demonstrates the subtree collision bug",
            collisions.len()
        );
    }

    #[test]
    fn test_asymmetric_subtree_collision() {
        // Test case: Asymmetric tree where one subtree is larger than the other
        // Tree structure:
        //       Root(0)
        //      /       \
        //     P1(1)    P2(2)
        //    / | \        \
        //   L1 L2 L3     L4
        //   (3)(4)(5)    (6)
        //
        // Expected issue: L3(5) and L4(6) might collide because P1 has 3 children
        // but P2 has only 1 child, creating asymmetric spacing

        let tree = TreeStructure {
            nodes: vec![
                TreeNode {
                    label: "Root".to_string(),
                    children: vec![1, 2],
                },
                TreeNode {
                    label: "Parent1".to_string(),
                    children: vec![3, 4, 5],
                },
                TreeNode {
                    label: "Parent2".to_string(),
                    children: vec![6],
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
                TreeNode {
                    label: "Leaf4".to_string(),
                    children: vec![],
                },
            ],
            root: 0,
            current_node: 0,
            node_counter: 7,
        };

        let layout_manager = create_layout(&tree, (80, 24));
        print_node_positions(&layout_manager, &tree);

        let config = LayoutConfig::default();
        let min_separation = match config.strategy {
            grid::LayoutStrategy::ReingoldTilford {
                node_separation, ..
            } => node_separation,
            _ => 4.0,
        };

        let collisions = find_collisions(&layout_manager, min_separation);

        if !collisions.is_empty() {
            println!("\n=== COLLISIONS DETECTED ===");
            for (i, j, distance) in &collisions {
                println!(
                    "Nodes {} and {} collide: distance={:.2}, required={:.2}",
                    i, j, distance, min_separation
                );
            }
        }

        assert!(
            collisions.is_empty(),
            "Found {} collision(s) in asymmetric tree",
            collisions.len()
        );
    }

    #[test]
    fn test_minimal_collision_case() {
        // Minimal test case to isolate the exact collision scenario
        // Tree structure:
        //   Root(0)
        //   /     \
        //  A(1)   B(2)
        //  |       |
        //  C(3)   D(4)
        //
        // This is the simplest case where two single-child subtrees might collide

        let tree = TreeStructure {
            nodes: vec![
                TreeNode {
                    label: "Root".to_string(),
                    children: vec![1, 2],
                },
                TreeNode {
                    label: "A".to_string(),
                    children: vec![3],
                },
                TreeNode {
                    label: "B".to_string(),
                    children: vec![4],
                },
                TreeNode {
                    label: "C".to_string(),
                    children: vec![],
                },
                TreeNode {
                    label: "D".to_string(),
                    children: vec![],
                },
            ],
            root: 0,
            current_node: 0,
            node_counter: 5,
        };

        let layout_manager = create_layout(&tree, (80, 24));
        print_node_positions(&layout_manager, &tree);

        let config = LayoutConfig::default();
        let min_separation = match config.strategy {
            grid::LayoutStrategy::ReingoldTilford {
                node_separation, ..
            } => node_separation,
            _ => 4.0,
        };

        let collisions = find_collisions(&layout_manager, min_separation);

        if !collisions.is_empty() {
            println!("\n=== COLLISIONS DETECTED ===");
            for (i, j, distance) in &collisions {
                println!(
                    "Nodes {} and {} collide: distance={:.2}, required={:.2}",
                    i, j, distance, min_separation
                );
            }
        }

        assert!(
            collisions.is_empty(),
            "Found {} collision(s) in minimal collision case",
            collisions.len()
        );
    }
}
