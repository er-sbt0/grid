use crate::error::LayoutError;

/// A node in the tree containing user data and child references
#[derive(Clone, Debug, PartialEq)]
pub struct Node<T> {
    pub data: T,
    pub children: Vec<usize>,
}

impl<T> Node<T> {
    /// Create a new node with the given data
    pub fn new(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
        }
    }

    /// Create a new node with pre-allocated child capacity
    pub fn with_capacity(data: T, capacity: usize) -> Self {
        Self {
            data,
            children: Vec::with_capacity(capacity),
        }
    }
}

/// Generic tree structure using arena-style indices
#[derive(Clone, Debug, PartialEq)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
    root: usize,
    version: u64,
}

impl<T> Tree<T> {
    /// Create a new tree with a root node
    pub fn new(root_data: T) -> Self {
        Self {
            nodes: vec![Node::new(root_data)],
            root: 0,
            version: 0,
        }
    }

    /// Create a tree with pre-allocated capacity
    pub fn with_capacity(root_data: T, capacity: usize) -> Self {
        let mut nodes = Vec::with_capacity(capacity);
        nodes.push(Node::new(root_data));
        Self {
            nodes,
            root: 0,
            version: 0,
        }
    }

    /// Create a tree from a vector of nodes with a specified root
    pub fn from_nodes(nodes: Vec<Node<T>>, root: usize) -> Result<Self, LayoutError> {
        if nodes.is_empty() {
            return Err(LayoutError::EmptyTree);
        }
        if root >= nodes.len() {
            return Err(LayoutError::InvalidNode(root));
        }

        let tree = Self {
            nodes,
            root,
            version: 0,
        };
        tree.validate()?;
        Ok(tree)
    }

    /// Add a child to a parent node, returning the child's index
    pub fn add_child(&mut self, parent_id: usize, child_data: T) -> Result<usize, LayoutError> {
        if parent_id >= self.nodes.len() {
            return Err(LayoutError::InvalidNode(parent_id));
        }

        let child_id = self.nodes.len();
        self.nodes.push(Node::new(child_data));
        self.nodes[parent_id].children.push(child_id);
        self.version += 1;
        Ok(child_id)
    }

    /// Get an immutable reference to a node
    pub fn node(&self, id: usize) -> Result<&Node<T>, LayoutError> {
        self.nodes.get(id).ok_or(LayoutError::InvalidNode(id))
    }

    /// Get a mutable reference to a node
    pub fn node_mut(&mut self, id: usize) -> Result<&mut Node<T>, LayoutError> {
        self.nodes.get_mut(id).ok_or(LayoutError::InvalidNode(id))
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    /// Get the root node index
    pub fn root(&self) -> usize {
        self.root
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the current version of the tree (increments on mutation)
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Validate the tree structure
    pub fn validate(&self) -> Result<(), LayoutError> {
        if self.is_empty() {
            return Err(LayoutError::EmptyTree);
        }

        if self.root >= self.nodes.len() {
            return Err(LayoutError::InvalidNode(self.root));
        }

        // Check that all child indices are valid
        for (node_id, node) in self.nodes.iter().enumerate() {
            for &child_id in &node.children {
                if child_id >= self.nodes.len() {
                    return Err(LayoutError::InvalidTree(format!(
                        "Node {} has invalid child index {}",
                        node_id, child_id
                    )));
                }
            }
        }

        // Check for cycles using DFS
        let mut visited = vec![false; self.nodes.len()];
        let mut rec_stack = vec![false; self.nodes.len()];

        if self.has_cycle(self.root, &mut visited, &mut rec_stack) {
            return Err(LayoutError::InvalidTree("Tree contains cycles".into()));
        }

        Ok(())
    }

    /// Helper function to detect cycles using DFS
    fn has_cycle(&self, node_id: usize, visited: &mut [bool], rec_stack: &mut [bool]) -> bool {
        visited[node_id] = true;
        rec_stack[node_id] = true;

        for &child_id in &self.nodes[node_id].children {
            if !visited[child_id] {
                if self.has_cycle(child_id, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack[child_id] {
                return true;
            }
        }

        rec_stack[node_id] = false;
        false
    }
}

/// Builder for fluent tree construction
pub struct TreeBuilder<T> {
    tree: Tree<T>,
}

impl<T> TreeBuilder<T> {
    /// Create a new builder with a root node
    pub fn new(root_data: T) -> Self {
        Self {
            tree: Tree::new(root_data),
        }
    }

    /// Add a child to a parent node
    pub fn add_child(mut self, parent_id: usize, child_data: T) -> Result<Self, LayoutError> {
        self.tree.add_child(parent_id, child_data)?;
        Ok(self)
    }

    /// Build and return the tree
    pub fn build(self) -> Result<Tree<T>, LayoutError> {
        self.tree.validate()?;
        Ok(self.tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let tree = Tree::new("root");
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root(), 0);
        assert_eq!(tree.node(0).unwrap().data, "root");
    }

    #[test]
    fn test_add_child() {
        let mut tree = Tree::new("root");
        let child_id = tree.add_child(0, "child").unwrap();
        assert_eq!(child_id, 1);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.node(0).unwrap().children, vec![1]);
    }

    #[test]
    fn test_invalid_parent() {
        let mut tree = Tree::new("root");
        let result = tree.add_child(99, "child");
        assert!(matches!(result, Err(LayoutError::InvalidNode(99))));
    }

    #[test]
    fn test_tree_builder() {
        let tree = TreeBuilder::new("root")
            .add_child(0, "child1")
            .unwrap()
            .add_child(0, "child2")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.node(0).unwrap().children.len(), 2);
    }

    #[test]
    fn test_cycle_detection() {
        let nodes = vec![
            Node {
                data: "a",
                children: vec![1],
            },
            Node {
                data: "b",
                children: vec![0],
            }, // Cycle!
        ];
        let result = Tree::from_nodes(nodes, 0);
        assert!(matches!(result, Err(LayoutError::InvalidTree(_))));
    }

    #[test]
    fn test_empty_tree() {
        let result = Tree::<String>::from_nodes(vec![], 0);
        assert!(matches!(result, Err(LayoutError::EmptyTree)));
    }

    #[test]
    fn test_invalid_root() {
        let nodes = vec![Node::new("a")];
        let result = Tree::from_nodes(nodes, 5);
        assert!(matches!(result, Err(LayoutError::InvalidNode(5))));
    }

    #[test]
    fn test_version_increments_on_mutation() {
        let mut tree = Tree::new("root");
        let v0 = tree.version();
        assert_eq!(v0, 0);

        tree.add_child(0, "child1").unwrap();
        let v1 = tree.version();
        assert!(v1 > v0);

        tree.add_child(0, "child2").unwrap();
        let v2 = tree.version();
        assert!(v2 > v1);
    }

    #[test]
    fn test_version_immutable_on_read() {
        let tree = Tree::new("root");
        let v0 = tree.version();

        let _ = tree.node(0);
        let _ = tree.nodes();
        let _ = tree.root();

        assert_eq!(tree.version(), v0);
    }
}
