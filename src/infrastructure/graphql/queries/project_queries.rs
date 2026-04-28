use crate::infrastructure::graphql::schema::schema;
use cynic::QueryBuilder;
use serde::{Deserialize, Serialize};

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

#[derive(Serialize)]
struct GraphqlRequest<'a, V: Serialize> {
    query: &'a str,
    variables: V,
}

#[derive(Deserialize, Debug)]
pub struct GraphqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphqlError>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphqlError {
    pub message: String,
    pub extensions: Option<GraphqlErrorExtension>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphqlErrorExtension {
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

impl GraphqlError {
    pub fn is_rate_limited(&self) -> bool {
        self.extensions
            .as_ref()
            .and_then(|e| e.error_type.as_deref())
            .map(|t| t == "RATELIMITED")
            .unwrap_or(false)
    }
}

// ---- Shared node types ----

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "User")]
pub struct LeadNode {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Team")]
pub struct TeamNode {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "TeamConnection")]
pub struct TeamConnection {
    pub nodes: Vec<TeamNode>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfoNode {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Project")]
#[allow(deprecated)]
pub struct ProjectNode {
    pub id: cynic::Id,
    pub name: String,
    pub description: String,
    pub slug_id: String,
    pub progress: f64,
    pub state: String,
    pub lead: Option<LeadNode>,
    #[arguments(first: 50)]
    pub teams: TeamConnection,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub updated_at: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectConnection")]
pub struct ProjectConnection {
    pub nodes: Vec<ProjectNode>,
    pub page_info: PageInfoNode,
}

// ---- List Projects ----

#[derive(cynic::Enum, Debug, Clone, Copy)]
#[cynic(graphql_type = "PaginationOrderBy", rename_all = "camelCase")]
pub enum PaginationOrderBy {
    UpdatedAt,
    CreatedAt,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectsVariables {
    pub first: i32,
    pub after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ProjectsVariables")]
pub struct ProjectsQuery {
    #[arguments(first: $first, after: $after, orderBy: updatedAt)]
    pub projects: ProjectConnection,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct TeamProjectsVariables {
    pub team_id: String,
    pub first: i32,
    pub after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TeamProjectsVariables")]
pub struct TeamProjectsQuery {
    #[arguments(id: $team_id)]
    pub team: TeamWithProjects,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Team", variables = "TeamProjectsVariables")]
pub struct TeamWithProjects {
    #[arguments(first: $first, after: $after, orderBy: updatedAt)]
    pub projects: ProjectConnection,
}

pub async fn fetch_projects(
    client: &reqwest::Client,
    api_key: &str,
    first: i32,
    after: Option<String>,
    team_id: Option<&str>,
) -> Result<(Vec<ProjectNode>, PageInfoNode), crate::domain::errors::DomainError> {
    if let Some(tid) = team_id {
        let op = TeamProjectsQuery::build(TeamProjectsVariables {
            team_id: tid.to_string(),
            first,
            after,
        });
        let resp: GraphqlResponse<TeamProjectsQuery> =
            execute_with_retry(client, api_key, &op.query, op.variables).await?;
        if let Some(errors) = resp.errors {
            return Err(map_errors(errors));
        }
        let data = resp.data.ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "empty response from Linear API".to_string(),
            )
        })?;
        Ok((data.team.projects.nodes, data.team.projects.page_info))
    } else {
        let op = ProjectsQuery::build(ProjectsVariables { first, after });
        let resp: GraphqlResponse<ProjectsQuery> =
            execute_with_retry(client, api_key, &op.query, op.variables).await?;
        if let Some(errors) = resp.errors {
            return Err(map_errors(errors));
        }
        let data = resp.data.ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "empty response from Linear API".to_string(),
            )
        })?;
        Ok((data.projects.nodes, data.projects.page_info))
    }
}

// ---- Get Project ----

#[derive(cynic::QueryVariables, Debug)]
pub struct GetProjectVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProjectVariables")]
pub struct GetProjectQuery {
    #[arguments(id: $id)]
    pub project: Option<ProjectNode>,
}

// ---- Slug Lookup ----

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Project")]
pub struct SlugProjectNode {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectConnection")]
pub struct SlugProjectConnection {
    pub nodes: Vec<SlugProjectNode>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "ProjectFilter")]
pub struct ProjectFilter {
    pub slug_id: Option<StringComparator>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "StringComparator")]
pub struct StringComparator {
    pub eq: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SlugLookupVariables {
    pub filter: ProjectFilter,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SlugLookupVariables")]
pub struct SlugLookupQuery {
    #[arguments(filter: $filter, first: 1)]
    pub projects: SlugProjectConnection,
}

pub async fn fetch_project_by_id(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    let op = GetProjectQuery::build(GetProjectVariables { id: id.to_string() });
    let resp: GraphqlResponse<GetProjectQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project
        .ok_or_else(|| crate::domain::errors::DomainError::NotFound(id.to_string()))
}

pub async fn resolve_slug_to_uuid(
    client: &reqwest::Client,
    api_key: &str,
    slug: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let op = SlugLookupQuery::build(SlugLookupVariables {
        filter: ProjectFilter {
            slug_id: Some(StringComparator { eq: Some(slug.to_string()) }),
        },
    });
    let resp: GraphqlResponse<SlugLookupQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .projects
        .nodes
        .into_iter()
        .next()
        .map(|n| n.id.into_inner())
        .ok_or_else(|| crate::domain::errors::DomainError::NotFound(slug.to_string()))
}

// ---- Workspace project statuses ----

#[derive(cynic::Enum, Debug, PartialEq, Clone, Copy)]
#[cynic(graphql_type = "ProjectStatusType", rename_all = "camelCase")]
pub enum ProjectStatusType {
    Backlog,
    Canceled,
    Completed,
    Paused,
    Planned,
    Started,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectStatus")]
pub struct ProjectStatusNode {
    pub id: cynic::Id,
    #[cynic(rename = "type")]
    pub status_type: ProjectStatusType,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Organization")]
pub struct OrgWithStatuses {
    pub project_statuses: Vec<ProjectStatusNode>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct OrgStatusQuery {
    pub organization: OrgWithStatuses,
}

pub async fn fetch_status_id_for_type(
    client: &reqwest::Client,
    api_key: &str,
    state_type: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let target = match state_type {
        "cancelled" | "canceled" => ProjectStatusType::Canceled,
        "backlog" => ProjectStatusType::Backlog,
        "completed" => ProjectStatusType::Completed,
        "paused" => ProjectStatusType::Paused,
        "planned" => ProjectStatusType::Planned,
        "started" => ProjectStatusType::Started,
        other => {
            return Err(crate::domain::errors::DomainError::InvalidInput(format!(
                "unknown project status type: '{}'",
                other
            )))
        }
    };
    let op = OrgStatusQuery::build(());
    let resp: GraphqlResponse<OrgStatusQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let statuses = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .organization
        .project_statuses;
    statuses
        .into_iter()
        .find(|s| s.status_type == target)
        .map(|s| s.id.into_inner())
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(format!(
                "no project status of type '{}' found in workspace",
                state_type
            ))
        })
}

// ---- Shared helpers ----

pub fn map_errors(errors: Vec<GraphqlError>) -> crate::domain::errors::DomainError {
    if let Some(first) = errors.first() {
        if first.is_rate_limited() {
            return crate::domain::errors::DomainError::InvalidInput(
                "rate limited by Linear API".to_string(),
            );
        }
        let is_auth = first
            .extensions
            .as_ref()
            .and_then(|e| e.error_type.as_deref())
            .map(|t| t == "UNAUTHENTICATED")
            .unwrap_or(false);
        if is_auth {
            return crate::domain::errors::DomainError::InvalidInput(
                "authentication error".to_string(),
            );
        }
        if first.message.to_lowercase().contains("not found") {
            return crate::domain::errors::DomainError::NotFound(first.message.clone());
        }
        return crate::domain::errors::DomainError::InvalidInput(first.message.clone());
    }
    crate::domain::errors::DomainError::InvalidInput("unknown GraphQL error".to_string())
}

pub async fn execute_with_retry<V: Serialize, T: for<'de> serde::Deserialize<'de>>(
    client: &reqwest::Client,
    api_key: &str,
    query: &str,
    variables: V,
) -> Result<GraphqlResponse<T>, crate::domain::errors::DomainError> {
    let mut last_err = None;
    for attempt in 0..3u32 {
        match try_execute(client, api_key, query, &variables).await {
            Ok(resp) => {
                let is_rate_limited = resp
                    .errors
                    .as_ref()
                    .and_then(|e| e.first())
                    .map(|e| e.is_rate_limited())
                    .unwrap_or(false);
                if is_rate_limited && attempt < 2 {
                    let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    last_err = Some(crate::domain::errors::DomainError::InvalidInput(
                        "rate limited".to_string(),
                    ));
                    continue;
                }
                return Ok(resp);
            }
            Err(e) => {
                if attempt < 2 {
                    let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    last_err = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput("request failed".to_string())
    }))
}

async fn try_execute<V: Serialize, T: for<'de> serde::Deserialize<'de>>(
    client: &reqwest::Client,
    api_key: &str,
    query: &str,
    variables: &V,
) -> Result<GraphqlResponse<T>, crate::domain::errors::DomainError> {
    let body = GraphqlRequest { query, variables };
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::domain::errors::DomainError::InvalidInput(e.to_string()))?;

    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "rate limited (HTTP 429)".to_string(),
        ));
    }

    response
        .json::<GraphqlResponse<T>>()
        .await
        .map_err(|e| crate::domain::errors::DomainError::InvalidInput(e.to_string()))
}
