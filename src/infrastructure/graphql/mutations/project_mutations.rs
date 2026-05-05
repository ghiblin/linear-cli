use crate::infrastructure::graphql::schema::schema;
use cynic::MutationBuilder;

use crate::infrastructure::graphql::queries::project_queries::{
    GraphqlResponse, ProjectNode, execute_with_retry, map_errors,
};

// ---- Shared response types ----

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectPayload")]
pub struct ProjectPayload {
    pub project: Option<ProjectNode>,
    pub success: bool,
    #[allow(dead_code)]
    pub last_sync_id: f64,
}

// ---- Create ----

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "ProjectCreateInput")]
pub struct ProjectCreateInput {
    pub name: String,
    pub team_ids: Vec<String>,
    pub description: Option<String>,
    pub lead_id: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub status_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectCreateVariables {
    pub input: ProjectCreateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ProjectCreateVariables")]
pub struct ProjectCreateMutation {
    #[arguments(input: $input)]
    pub project_create: ProjectPayload,
}

// ---- Update ----

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "ProjectUpdateInput")]
pub struct ProjectUpdateInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub lead_id: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub status_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectUpdateVariables {
    pub id: String,
    pub input: ProjectUpdateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ProjectUpdateVariables")]
pub struct ProjectUpdateMutation {
    #[arguments(id: $id, input: $input)]
    pub project_update: ProjectPayload,
}

// ---- Archive ----

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Project")]
pub struct ArchivedProjectEntity {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectArchivePayload")]
pub struct ProjectArchivePayload {
    pub success: bool,
    pub entity: Option<ArchivedProjectEntity>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectArchiveVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ProjectArchiveVariables")]
pub struct ProjectArchiveMutation {
    #[arguments(id: $id)]
    pub project_archive: ProjectArchivePayload,
}

pub async fn create_project(
    client: &reqwest::Client,
    api_key: &str,
    input: ProjectCreateInput,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    let op = ProjectCreateMutation::build(ProjectCreateVariables { input });
    let resp: GraphqlResponse<ProjectCreateMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let payload = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project_create;
    if !payload.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "create operation reported failure".to_string(),
        ));
    }
    payload.project.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput(
            "project not returned in create response".to_string(),
        )
    })
}

pub async fn update_project(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
    input: ProjectUpdateInput,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    let op = ProjectUpdateMutation::build(ProjectUpdateVariables {
        id: id.to_string(),
        input,
    });
    let resp: GraphqlResponse<ProjectUpdateMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let payload = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project_update;
    if !payload.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "update operation reported failure".to_string(),
        ));
    }
    payload.project.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput(
            "project not returned in update response".to_string(),
        )
    })
}

pub async fn archive_project(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let op = ProjectArchiveMutation::build(ProjectArchiveVariables { id: id.to_string() });
    let resp: GraphqlResponse<ProjectArchiveMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let data = resp.data.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
    })?;
    if !data.project_archive.success {
        return Err(crate::domain::errors::DomainError::NotFound(
            "project is already archived".to_string(),
        ));
    }
    Ok(data
        .project_archive
        .entity
        .map(|e| e.id.into_inner())
        .unwrap_or_else(|| id.to_string()))
}
