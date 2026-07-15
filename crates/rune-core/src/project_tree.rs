use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InstanceId(u64);

impl InstanceId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstanceNode {
    id: InstanceId,
    name: String,
    class_name: String,
    children: Vec<Self>,
}

impl InstanceNode {
    pub fn new(id: InstanceId, name: impl Into<String>, class_name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            class_name: class_name.into(),
            children: Vec::new(),
        }
    }

    pub fn id(&self) -> InstanceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }

    fn append_depth_first_ids(&self, ids: &mut Vec<InstanceId>) {
        ids.push(self.id);

        for child in &self.children {
            child.append_depth_first_ids(ids);
        }
    }

    pub fn depth_first_ids(&self) -> Vec<InstanceId> {
        let mut ids = Vec::new();

        self.append_depth_first_ids(&mut ids);
        ids
    }

    pub fn find(&self, id: InstanceId) -> Option<&Self> {
        if self.id == id {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find(id))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectTree {
    root: InstanceNode,
}

impl ProjectTree {
    pub fn try_new(root: InstanceNode) -> Result<Self, ProjectTreeError> {
        let mut ids = HashSet::new();

        for id in root.depth_first_ids() {
            if !ids.insert(id) {
                return Err(ProjectTreeError::DuplicateInstanceId(id));
            }
        }

        Ok(Self { root })
    }

    pub fn root(&self) -> &InstanceNode {
        &self.root
    }

    pub fn find(&self, id: InstanceId) -> Option<&InstanceNode> {
        self.root.find(id)
    }

    pub fn depth_first_ids(&self) -> Vec<InstanceId> {
        self.root.depth_first_ids()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectTreeError {
    DuplicateInstanceId(InstanceId),
}

impl fmt::Display for ProjectTreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateInstanceId(id) => {
                write!(formatter, "duplicate instance id: {}", id.value())
            }
        }
    }
}

impl std::error::Error for ProjectTreeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ProjectTree {
        let mut workspace = InstanceNode::new(InstanceId::new(2), "Workspace", "Workspace");

        workspace.add_child(InstanceNode::new(InstanceId::new(3), "Camera", "Camera"));

        let mut root = InstanceNode::new(InstanceId::new(1), "Game", "DataModel");
        root.add_child(workspace);
        root.add_child(InstanceNode::new(
            InstanceId::new(4),
            "ReplicatedStorage",
            "ReplicatedStorage",
        ));

        ProjectTree::try_new(root).expect("fixture should be valid")
    }

    #[test]
    fn preserves_depth_first_child_order() {
        let tree = fixture();

        assert_eq!(
            tree.depth_first_ids(),
            vec![
                InstanceId::new(1),
                InstanceId::new(2),
                InstanceId::new(3),
                InstanceId::new(4),
            ]
        );
    }

    #[test]
    fn finds_instanced_by_id() {
        let tree = fixture();

        let camera = tree.find(InstanceId::new(3));
        assert_eq!(camera.map(InstanceNode::name), Some("Camera"));
        assert_eq!(camera.map(InstanceNode::class_name), Some("Camera"));
        assert!(tree.find(InstanceId::new(99)).is_none());
    }

    #[test]
    fn allows_duplicate_names_when_ids_are_unique() {
        let mut root = InstanceNode::new(InstanceId::new(1), "Game", "DataModel");

        root.add_child(InstanceNode::new(InstanceId::new(2), "Folder", "Folder"));
        root.add_child(InstanceNode::new(InstanceId::new(3), "Folder", "Folder"));

        assert!(ProjectTree::try_new(root).is_ok());
    }

    #[test]
    fn rejects_duplicate_instance_ids() {
        let mut root = InstanceNode::new(InstanceId::new(1), "Game", "DataModel");

        root.add_child(InstanceNode::new(InstanceId::new(2), "First", "Folder"));
        root.add_child(InstanceNode::new(InstanceId::new(2), "Second", "Folder"));

        assert_eq!(
            ProjectTree::try_new(root),
            Err(ProjectTreeError::DuplicateInstanceId(InstanceId::new(2)))
        );
    }
}
