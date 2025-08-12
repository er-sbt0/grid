// Core library module
mod core;

// Library exports for testing and external use
pub use core::{
    // Deprecated exports for migration compatibility
    BJLNodeData,
    LayoutConfig,
    LayoutManager,
    LayoutState,
    LayoutStrategy,
    NodeLayout,
    RTNodeData,
    TreeNode,
    TreeOrientation,
    TreeStructure,
};
