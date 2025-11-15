// New API modules
mod config;
mod edge_router;
mod engine;
mod error;
mod result;
mod tree;
mod types;

#[cfg(feature = "rendering")]
pub mod rendering;

// Public API
pub use config::{LayoutConfig, TreeOrientation};
pub use engine::LayoutEngine;
pub use error::LayoutError;
pub use result::{LayoutResult, Viewport, ViewportLayout};
pub use tree::{Node, Tree, TreeBuilder};
pub use types::{
    CornerType, EdgeConnectionPoints, EdgePath, EdgeSegment, EdgeSegmentType, LayoutBounds,
    Position,
};

// Re-export rendering functions when feature is enabled
#[cfg(feature = "rendering")]
pub use rendering::{render_tree_edges, render_tree_with_edges};
