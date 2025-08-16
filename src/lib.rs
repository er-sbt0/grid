// Core library module
mod core;

// Library exports for testing and external use
pub use core::{
    RTNodeData, LayoutConfig, LayoutManager, LayoutState, LayoutStrategy, NodeLayout, TreeNode,
    TreeOrientation, TreeStructure,
    // Deprecated exports for migration compatibility
    BJLNodeData,
};
