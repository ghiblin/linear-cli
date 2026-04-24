use serde::{Deserialize, Serialize};

use crate::infrastructure::graphql::queries::project_queries::{
    GraphqlResponse, ProjectNode, execute_with_retry, map_errors,
};

// ---- Create ----

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateInputVars {
    pub name: String,
    pub team_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ProjectPayload {
    pub project: Option<ProjectNode>,
    pub success: bool,
    pub last_sync_id: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateData {
    pub project_create: ProjectPayload,
}

const PROJECT_CREATE_MUTATION: &str = r#"
mutation ProjectCreate($input: ProjectCreateInput!) {
  projectCreate(input: $input) {
    success
    lastSyncId
    project {
      id
      name
      description
      slugId
      progress
      state
      lead { id }
      teams(first: 50) { nodes { id } }
      startDate
      targetDate
      updatedAt
    }
  }
}
"#;

pub async fn create_project(
    client: &reqwest::Client,
    api_key: &str,
    input: ProjectCreateInputVars,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    #[derive(Serialize)]
    struct Vars {
        input: ProjectCreateInputVars,
    }
    let resp: GraphqlResponse<ProjectCreateData> =
        execute_with_retry(client, api_key, PROJECT_CREATE_MUTATION, Vars { input }).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project_create
        .project
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "project not returned in create response".to_string(),
            )
        })
}

// ---- Update ----

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateInputVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateData {
    pub project_update: ProjectPayload,
}

const PROJECT_UPDATE_MUTATION: &str = r#"
mutation ProjectUpdate($id: String!, $input: ProjectUpdateInput!) {
  projectUpdate(id: $id, input: $input) {
    success
    lastSyncId
    project {
      id
      name
      description
      slugId
      progress
      state
      lead { id }
      teams(first: 50) { nodes { id } }
      startDate
      targetDate
      updatedAt
    }
  }
}
"#;

pub async fn update_project(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
    input: ProjectUpdateInputVars,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    #[derive(Serialize)]
    struct Vars<'a> {
        id: &'a str,
        input: ProjectUpdateInputVars,
    }
    let resp: GraphqlResponse<ProjectUpdateData> =
        execute_with_retry(client, api_key, PROJECT_UPDATE_MUTATION, Vars { id, input }).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project_update
        .project
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "project not returned in update response".to_string(),
            )
        })
}

// ---- Archive ----

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArchivePayload {
    pub success: bool,
    pub entity: Option<ArchivedProjectEntity>,
}

#[derive(Deserialize, Debug)]
pub struct ArchivedProjectEntity {
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArchiveData {
    pub project_archive: ProjectArchivePayload,
}

const PROJECT_ARCHIVE_MUTATION: &str = r#"
mutation ProjectArchive($id: String!) {
  projectArchive(id: $id) {
    success
    entity { id }
  }
}
"#;

pub async fn archive_project(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    #[derive(Serialize)]
    struct Vars<'a> {
        id: &'a str,
    }
    let resp: GraphqlResponse<ProjectArchiveData> =
        execute_with_retry(client, api_key, PROJECT_ARCHIVE_MUTATION, Vars { id }).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let data = resp.data.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
    })?;
    if !data.project_archive.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "archive operation reported failure".to_string(),
        ));
    }
    Ok(data
        .project_archive
        .entity
        .map(|e| e.id)
        .unwrap_or_else(|| id.to_string()))
}
