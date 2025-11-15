use grid::{CornerType, EdgeSegmentType, LayoutConfig, LayoutEngine, Tree, TreeBuilder};

#[test]
fn test_edge_computation_simple_tree() {
    // Create a simple tree: root -> child1, child2
    let tree = TreeBuilder::new("root")
        .add_child(0, "child1")
        .unwrap()
        .add_child(0, "child2")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new().with_node_size(10.0, 3.0);

    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    // Should have edges computed
    let edges = layout.edges();
    assert_eq!(edges.len(), 2, "Should have 2 edges (root to each child)");

    // Verify edge parent/child IDs
    assert!(edges.iter().any(|e| e.parent_id == 0 && e.child_id == 1));
    assert!(edges.iter().any(|e| e.parent_id == 0 && e.child_id == 2));

    // Each edge should have segments
    for edge in edges {
        assert!(!edge.segments.is_empty(), "Edge should have segments");
    }
}

#[test]
fn test_edge_horizontal_only() {
    // Create a simple parent-child relationship on same Y level
    let tree = TreeBuilder::new("root")
        .add_child(0, "child")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new()
        .with_node_size(10.0, 3.0)
        .with_level_separation(0.0); // Force same level for testing

    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    let edges = layout.edges();
    assert_eq!(edges.len(), 1);

    // If nodes are on same Y, should have single horizontal segment
    let edge = &edges[0];
    if edge.segments.len() == 1 {
        assert!(matches!(
            edge.segments[0].segment_type,
            EdgeSegmentType::Horizontal
        ));
    }
}

#[test]
fn test_edge_with_vertical_segment() {
    // Create a tree with multiple levels to ensure vertical offset
    let tree = TreeBuilder::new("root")
        .add_child(0, "child1")
        .unwrap()
        .add_child(1, "grandchild1")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new()
        .with_node_size(10.0, 3.0)
        .with_level_separation(10.0); // Large separation to ensure vertical segment

    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    let edges = layout.edges();
    // Should have 2 edges (root->child1, child1->grandchild1)
    assert_eq!(edges.len(), 2);

    // Check the second edge (child1->grandchild1) which should have vertical offset
    let edge = &edges[1];

    // Check if there's vertical offset between nodes
    let parent_pos = layout.position(1).unwrap();
    let child_pos = layout.position(2).unwrap();

    if (parent_pos.y - child_pos.y).abs() > 0.1 {
        // Should have multiple segments including vertical
        assert!(edge.segments.len() > 1);

        // Should contain at least one vertical segment
        let has_vertical = edge
            .segments
            .iter()
            .any(|s| matches!(s.segment_type, EdgeSegmentType::Vertical));
        assert!(has_vertical, "Edge should have vertical segment");

        // Should have corner segments
        let has_corners = edge
            .segments
            .iter()
            .any(|s| matches!(s.segment_type, EdgeSegmentType::Corner(_)));
        assert!(has_corners, "Edge should have corner segments");
    } else {
        // If nodes are on same Y, should have single horizontal segment
        assert_eq!(edge.segments.len(), 1);
        assert!(matches!(
            edge.segments[0].segment_type,
            EdgeSegmentType::Horizontal
        ));
    }
}

#[test]
fn test_edge_connection_points() {
    let tree = TreeBuilder::new("root")
        .add_child(0, "child")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new().with_node_size(10.0, 3.0);

    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    // Test connection points API
    let points = layout.connection_points(0, 1, 10.0, 3.0);
    assert!(points.is_some());

    let points = points.unwrap();

    // Parent exit should use fixed offset (2.0, 1.0) to match old app behavior
    let parent_pos = layout.position(0).unwrap();
    assert_eq!(points.parent_exit.x, parent_pos.x + 2.0);
    assert_eq!(points.parent_exit.y, parent_pos.y + 1.0);

    // Child entry should be at left edge with fixed vertical offset
    let child_pos = layout.position(1).unwrap();
    assert_eq!(points.child_entry.x, child_pos.x);
    assert_eq!(points.child_entry.y, child_pos.y + 1.0);
}

#[test]
fn test_edge_iterator() {
    let tree = TreeBuilder::new("root")
        .add_child(0, "child1")
        .unwrap()
        .add_child(0, "child2")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new();
    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    // Test iter_edges_with_connections
    let parent_children = vec![(0, vec![1, 2])];

    let edges: Vec<_> = layout
        .iter_edges_with_connections(parent_children, 10.0, 3.0)
        .collect();

    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].0, 0); // parent_id
    assert_eq!(edges[0].1, 1); // child_id
    assert_eq!(edges[1].0, 0);
    assert_eq!(edges[1].1, 2);
}

#[test]
fn test_edge_specific_lookup() {
    let tree = TreeBuilder::new("root")
        .add_child(0, "child1")
        .unwrap()
        .add_child(0, "child2")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new();
    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    // Test getting specific edge
    let edge = layout.edge(0, 1);
    assert!(edge.is_some());
    assert_eq!(edge.unwrap().parent_id, 0);
    assert_eq!(edge.unwrap().child_id, 1);

    // Non-existent edge
    let edge = layout.edge(1, 2);
    assert!(edge.is_none());
}

#[test]
fn test_corner_types() {
    let tree = TreeBuilder::new("root")
        .add_child(0, "child1")
        .unwrap()
        .build()
        .unwrap();

    let config = LayoutConfig::new().with_level_separation(10.0);

    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    let edges = layout.edges();
    assert_eq!(edges.len(), 1);

    let edge = &edges[0];

    // Find corner segments
    let corners: Vec<_> = edge
        .segments
        .iter()
        .filter_map(|s| {
            if let EdgeSegmentType::Corner(ct) = s.segment_type {
                Some(ct)
            } else {
                None
            }
        })
        .collect();

    // Should have corners for vertical routing
    if edge.segments.len() > 1 {
        assert!(!corners.is_empty(), "Should have corner segments");

        // Verify corner types are valid
        for corner in corners {
            assert!(matches!(
                corner,
                CornerType::TopRight
                    | CornerType::TopLeft
                    | CornerType::BottomRight
                    | CornerType::BottomLeft
            ));
        }
    }
}

#[test]
fn test_complex_tree_edges() {
    // Create a more complex tree
    let tree = Tree::new("root");
    let mut tree = tree;
    let child1 = tree.add_child(0, "child1").unwrap();
    let child2 = tree.add_child(0, "child2").unwrap();
    let _grandchild1 = tree.add_child(child1, "grandchild1").unwrap();
    let _grandchild2 = tree.add_child(child1, "grandchild2").unwrap();
    let _grandchild3 = tree.add_child(child2, "grandchild3").unwrap();

    let config = LayoutConfig::new();
    let mut engine = LayoutEngine::with_config(config).unwrap();
    let layout = engine.compute(&tree).unwrap();

    // Should have 5 edges total (root->2 children, child1->2 grandchildren, child2->1 grandchild)
    let edges = layout.edges();
    assert_eq!(edges.len(), 5);

    // All edges should have valid segments
    for edge in edges {
        assert!(!edge.segments.is_empty());

        // Verify segment connectivity
        for i in 0..edge.segments.len() - 1 {
            // Note: corners have same from/to, so we skip those for connectivity check
            if !matches!(edge.segments[i].segment_type, EdgeSegmentType::Corner(_)) {
                // Next segment should start where this one ends (or be a corner at same point)
                let current_end = edge.segments[i].to;
                let next_start = edge.segments[i + 1].from;

                // Allow for floating point precision
                let distance = ((current_end.x - next_start.x).powi(2)
                    + (current_end.y - next_start.y).powi(2))
                .sqrt();
                assert!(distance < 0.01, "Segments should connect");
            }
        }
    }
}
