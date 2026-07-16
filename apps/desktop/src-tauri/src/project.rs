use rune_project::ProjectSnapshot;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCommandError {
    pub code: String,
    pub message: String,
}

impl From<rune_project::ProjectError> for ProjectCommandError {
    fn from(error: rune_project::ProjectError) -> Self {
        Self {
            code: "project_load_failed".to_owned(),
            message: error.to_string(),
        }
    }
}

#[tauri::command]
pub fn open_project(path: String) -> Result<ProjectSnapshot, ProjectCommandError> {
    let project = rune_project::load_project(path).map_err(ProjectCommandError::from)?;

    Ok(ProjectSnapshot::from(&project))
}
