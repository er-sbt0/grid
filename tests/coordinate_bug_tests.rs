#[cfg(test)]
mod coordinate_rendering_tests {
    use grid::{LayoutConfig, LayoutEngine, Tree, TreeOrientation};

    #[test]
    fn test_position_interpretation() {
        // Create a simple tree with 2 nodes
        let mut tree = Tree::new("Root");
        let root = tree.root();
        tree.add_child(root, "Child").unwrap();

        let config = LayoutConfig::new()
            .with_orientation(TreeOrientation::LeftRight)
            .with_node_size(20.0, 3.0)
            .with_node_separation(4.0)
            .with_level_separation(25.0); // Must be > node_width to avoid overlap

        let mut engine = LayoutEngine::with_config(config).unwrap();
        let result = engine.compute(&tree).unwrap();

        // Get positions
        let root_pos = result.position(root).unwrap();
        let child_id = 1; // First child
        let child_pos = result.position(child_id).unwrap();

        println!("\n=== Position Analysis ===");
        println!("Root position: ({}, {})", root_pos.x, root_pos.y);
        println!("Child position: ({}, {})", child_pos.x, child_pos.y);
        println!("Node dimensions: {}x{}", 20.0, 3.0);

        // For LeftRight orientation:
        // - X axis is depth (horizontal)
        // - Y axis is offset (vertical)
        // - Root should be at depth 0, child at depth 5 (level_separation)

        // If positions are TOP-LEFT corners:
        // Root: (0, -1.5) - centered vertically
        // Child: (5, -1.5) - same vertical position
        // Connection: Root exit at (20, 1.5), Child entry at (5, 1.5)

        // If positions are CENTERS:
        // Root: (0, 0)
        // Child: (5, 0)
        // Connection: Root exit at (10, 0), Child entry at (-10, 0) <- WRONG!

        // Get connection points
        let connection = result.connection_points(root, child_id, 20.0, 3.0).unwrap();

        println!("\n=== Connection Points ===");
        println!(
            "Parent exit: ({}, {})",
            connection.parent_exit.x, connection.parent_exit.y
        );
        println!(
            "Child entry: ({}, {})",
            connection.child_entry.x, connection.child_entry.y
        );

        // The connection_points function does:
        // parent_exit.x = parent_pos.x + node_width (= root_pos.x + 20)
        // parent_exit.y = parent_pos.y + node_height/2 (= root_pos.y + 1.5)
        // child_entry.x = child_pos.x
        // child_entry.y = child_pos.y + node_height/2

        // For a horizontal left-right tree, parent_exit.x should be LESS than child_entry.x
        assert!(
            connection.parent_exit.x < connection.child_entry.x,
            "Parent exit X ({}) should be less than child entry X ({}) for LeftRight orientation.\n\
             This indicates positions are being interpreted incorrectly.",
            connection.parent_exit.x,
            connection.child_entry.x
        );

        // The parent exit and child entry should be at roughly the same Y coordinate
        let y_diff = (connection.parent_exit.y - connection.child_entry.y).abs();
        assert!(
            y_diff < 0.1,
            "Parent exit Y and child entry Y should be approximately equal (diff: {})",
            y_diff
        );
    }

    #[test]
    fn test_bounds_include_node_dimensions() {
        let mut tree = Tree::new("Root");
        let root = tree.root();

        let config = LayoutConfig::new().with_node_size(20.0, 3.0);

        let mut engine = LayoutEngine::with_config(config).unwrap();
        let result = engine.compute(&tree).unwrap();

        let bounds = result.bounds();
        let root_pos = result.position(root).unwrap();

        println!("\n=== Bounds Analysis ===");
        println!("Root position: ({}, {})", root_pos.x, root_pos.y);
        println!(
            "Bounds: min=({}, {}), max=({}, {})",
            bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y
        );
        println!("Bounds dimensions: {}x{}", bounds.width(), bounds.height());

        // If positions are top-left corners, bounds should be:
        // min = root_pos, max = root_pos + (node_width, node_height)
        // If positions are centers, bounds should be:
        // min = root_pos - (node_width/2, node_height/2)
        // max = root_pos + (node_width/2, node_height/2)

        let expected_width = 20.0;
        let expected_height = 3.0;

        println!(
            "Expected dimensions: {}x{}",
            expected_width, expected_height
        );

        // The bounds should encompass the entire node
        let actual_width = bounds.width();
        let actual_height = bounds.height();

        assert!(
            actual_width >= expected_width - 0.1,
            "Bounds width ({}) should be at least node width ({}). \
             This indicates bounds calculation doesn't account for node dimensions.",
            actual_width,
            expected_width
        );

        assert!(
            actual_height >= expected_height - 0.1,
            "Bounds height ({}) should be at least node height ({}). \
             This indicates bounds calculation doesn't account for node dimensions.",
            actual_height,
            expected_height
        );
    }
}
