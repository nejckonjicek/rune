use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use rune_core::{InstanceId, InstanceNode, ProjectTree, ProjectTreeError};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug)]
pub struct LoadedProject {
    pub name: String,
    pub project_file: PathBuf,
    pub tree: ProjectTree,
    pub scripts: Vec<ScriptDocument>,
}

#[derive(Debug)]
pub struct ScriptDocument {
    pub id: InstanceId,
    pub name: String,
    pub class_name: String,
    pub relative_path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project file not found: {0}")]
    ProjectFileNotFound(PathBuf),

    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("expected a directory at {0}")]
    ExpectedDirectory(PathBuf),

    #[error("path is outside the project root: {0}")]
    PathOutsideProject(PathBuf),

    #[error("init scripts are not supported yet: {0}")]
    UnsupportedScriptLayout(PathBuf),

    #[error(transparent)]
    Tree(#[from] ProjectTreeError),
}

#[derive(Debug, Deserialize)]
struct RawProject {
    name: String,
    tree: RawInstanceDescription,
}

#[derive(Debug, Deserialize)]
struct RawInstanceDescription {
    #[serde(rename = "$className")]
    class_name: Option<String>,

    #[serde(rename = "$path")]
    path: Option<PathBuf>,

    #[serde(rename = "$properties")]
    _properties: Option<Value>,

    #[serde(rename = "$ignoreUnknownInstances")]
    _ignore_unknown_instances: Option<bool>,

    #[serde(flatten)]
    children: BTreeMap<String, RawInstanceDescription>,
}

struct ProjectBuilder {
    project_root: PathBuf,
    next_id: u64,
    scripts: Vec<ScriptDocument>,
}

impl ProjectBuilder {
    fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            next_id: 1,
            scripts: Vec::new(),
        }
    }

    fn next_id(&mut self) -> InstanceId {
        let id = InstanceId::new(self.next_id);
        self.next_id += 1;
        id
    }

    fn build_node(
        &mut self,
        name: &str,
        description: &RawInstanceDescription,
        fallback_class_name: &str,
    ) -> Result<InstanceNode, ProjectError> {
        let class_name = description
            .class_name
            .as_deref()
            .unwrap_or(fallback_class_name);

        let mut node = InstanceNode::new(self.next_id(), name, class_name);

        if let Some(relative_path) = &description.path {
            let path = self.project_root.join(relative_path);
            self.add_filesystem_children(&mut node, &path)?;
        }

        for (child_name, child_description) in &description.children {
            let child = self.build_node(child_name, child_description, "Folder")?;

            node.add_child(child);
        }

        Ok(node)
    }

    fn add_filesystem_children(
        &mut self,
        parent: &mut InstanceNode,
        directory: &Path,
    ) -> Result<(), ProjectError> {
        let metadata = fs::metadata(directory).map_err(|source| ProjectError::Read {
            path: directory.to_path_buf(),
            source,
        })?;

        if !metadata.is_dir() {
            return Err(ProjectError::ExpectedDirectory(directory.to_path_buf()));
        }

        let mut entries = fs::read_dir(directory)
            .map_err(|source| ProjectError::Read {
                path: directory.to_path_buf(),
                source,
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| ProjectError::Read {
                path: directory.to_path_buf(),
                source,
            })?;

        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let metadata = entry.metadata().map_err(|source| ProjectError::Read {
                path: path.clone(),
                source,
            })?;

            if metadata.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let mut folder = InstanceNode::new(self.next_id(), &name, "Folder");

                self.add_filesystem_children(&mut folder, &path)?;
                parent.add_child(folder);

                continue;
            }

            if !metadata.is_file() {
                continue;
            }

            let Some((name, class_name)) = script_descriptor(&path)? else {
                continue;
            };

            let id = self.next_id();
            parent.add_child(InstanceNode::new(id, &name, class_name));

            let relative_path = path
                .strip_prefix(&self.project_root)
                .map_err(|_| ProjectError::PathOutsideProject(path.clone()))?
                .to_path_buf();

            self.scripts.push(ScriptDocument {
                id,
                name,
                class_name: class_name.to_owned(),
                relative_path,
            });
        }

        Ok(())
    }
}

fn script_descriptor(path: &Path) -> Result<Option<(String, &'static str)>, ProjectError> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };

    let Some((name, class_name)) = file_name
        .strip_suffix(".server.lua")
        .map(|name| (name, "Script"))
        .or_else(|| {
            file_name
                .strip_suffix(".client.lua")
                .map(|name| (name, "LocalScript"))
        })
        .or_else(|| {
            file_name
                .strip_suffix(".lua")
                .map(|name| (name, "ModuleScript"))
        })
    else {
        return Ok(None);
    };

    if name == "init" {
        return Err(ProjectError::UnsupportedScriptLayout(path.to_path_buf()));
    }

    if name.is_empty() {
        return Ok(None);
    }

    Ok(Some((name.to_owned(), class_name)))
}

fn locate_project_file(path: &Path) -> Result<PathBuf, ProjectError> {
    let candidate = if path.is_dir() {
        path.join("default.project.json")
    } else {
        path.to_path_buf()
    };

    if !candidate.is_file() {
        return Err(ProjectError::ProjectFileNotFound(candidate));
    }

    fs::canonicalize(&candidate).map_err(|source| ProjectError::Read {
        path: candidate,
        source,
    })
}

pub fn load_project(path: impl AsRef<Path>) -> Result<LoadedProject, ProjectError> {
    let project_file = locate_project_file(path.as_ref())?;
    let contents = fs::read_to_string(&project_file).map_err(|source| ProjectError::Read {
        path: project_file.clone(),
        source,
    })?;

    let raw_project: RawProject =
        serde_json::from_str(&contents).map_err(|source| ProjectError::Parse {
            path: project_file.clone(),
            source,
        })?;
    let project_root = project_file
        .parent()
        .expect("canonical project paths always have a parent")
        .to_path_buf();

    let mut builder = ProjectBuilder::new(project_root);
    let root = builder.build_node(&raw_project.name, &raw_project.tree, "DataModel")?;

    let scripts = builder.scripts;
    let tree = ProjectTree::try_new(root)?;

    Ok(LoadedProject {
        name: raw_project.name,
        project_file,
        tree,
        scripts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_script_only_project() {
        let project_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/script-only/default.project.json");
        let project = load_project(project_file).unwrap();

        assert_eq!(project.name, "Rune Script Fixture");
        assert_eq!(project.tree.root().class_name(), "DataModel");
        assert_eq!(project.scripts.len(), 4);

        assert!(project
            .scripts
            .iter()
            .any(|script| script.class_name == "Script"));

        assert!(project
            .scripts
            .iter()
            .any(|script| script.class_name == "LocalScript"));

        assert!(project
            .scripts
            .iter()
            .any(|script| script.class_name == "ModuleScript"));
    }
}
