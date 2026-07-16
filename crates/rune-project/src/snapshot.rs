use rune_core::InstanceNode;
use serde::Serialize;

use crate::rojo::{LoadedProject, ScriptDocument};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub name: String,
    pub project_file: String,
    pub root: InstanceSnapshot,
    pub scripts: Vec<ScriptSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSnapshot {
    pub id: u64,
    pub name: String,
    pub class_name: String,
    pub children: Vec<InstanceSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptSnapshot {
    pub id: u64,
    pub name: String,
    pub class_name: String,
    pub relative_path: String,
}

impl From<&LoadedProject> for ProjectSnapshot {
    fn from(project: &LoadedProject) -> Self {
        Self {
            name: project.name.clone(),
            project_file: project.project_file.to_string_lossy().into_owned(),
            root: InstanceSnapshot::from_node(project.tree.root()),
            scripts: project.scripts.iter().map(ScriptSnapshot::from).collect(),
        }
    }
}

impl InstanceSnapshot {
    fn from_node(node: &InstanceNode) -> Self {
        Self {
            id: node.id().value(),
            name: node.name().to_owned(),
            class_name: node.class_name().to_owned(),
            children: node
                .children()
                .iter()
                .map(Self::from_node)
                .collect(),
        }
    }
}

impl From<&ScriptDocument> for ScriptSnapshot {
    fn from(script: &ScriptDocument) -> Self {
        Self {
            id: script.id.value(),
            name: script.name.clone(),
            class_name: script.class_name.clone(),
            relative_path: script.relative_path.to_string_lossy().into_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_project;
    use std::path::PathBuf;

    #[test]
    fn creates_serializable_project_snapshot() {
        let project_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/script-only/default.project.json");

        let project = load_project(project_file).unwrap();
        let snapshot = ProjectSnapshot::from(&project);
        let json = serde_json::to_value(&snapshot).unwrap();

        assert_eq!(json["root"]["className"], "DataModel");
        assert_eq!(json["scripts"].as_array().unwrap().len(), 4);
    }
}
