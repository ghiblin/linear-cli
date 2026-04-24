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

// ---- List Projects ----

#[derive(Serialize)]
pub struct ProjectsVariables {
    pub first: i32,
    pub after: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectsData {
    pub projects: ProjectConnection,
}

#[derive(Deserialize, Debug)]
pub struct TeamProjectsData {
    pub team: TeamWithProjects,
}

#[derive(Deserialize, Debug)]
pub struct TeamWithProjects {
    pub projects: ProjectConnection,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConnection {
    pub nodes: Vec<ProjectNode>,
    pub page_info: PageInfoNode,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slug_id: String,
    pub progress: f64,
    pub state: String,
    pub lead: Option<LeadNode>,
    pub teams: TeamConnection,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub updated_at: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LeadNode {
    pub id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TeamConnection {
    pub nodes: Vec<TeamNode>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TeamNode {
    pub id: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageInfoNode {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

const PROJECTS_QUERY: &str = r#"
query Projects($first: Int!, $after: String) {
  projects(first: $first, after: $after) {
    nodes {
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
    pageInfo { hasNextPage endCursor }
  }
}
"#;

const TEAM_PROJECTS_QUERY: &str = r#"
query TeamProjects($teamId: String!, $first: Int!, $after: String) {
  team(id: $teamId) {
    projects(first: $first, after: $after) {
      nodes {
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
      pageInfo { hasNextPage endCursor }
    }
  }
}
"#;

pub async fn fetch_projects(
    client: &reqwest::Client,
    api_key: &str,
    first: i32,
    after: Option<String>,
    team_id: Option<&str>,
) -> Result<(Vec<ProjectNode>, PageInfoNode), crate::domain::errors::DomainError> {
    if let Some(tid) = team_id {
        #[derive(Serialize)]
        struct Vars<'a> {
            #[serde(rename = "teamId")]
            team_id: &'a str,
            first: i32,
            after: Option<String>,
        }
        let resp = execute_with_retry(
            client,
            api_key,
            TEAM_PROJECTS_QUERY,
            Vars { team_id: tid, first, after },
        )
        .await?;
        let body: GraphqlResponse<TeamProjectsData> = resp;
        if let Some(errors) = body.errors {
            return Err(map_errors(errors));
        }
        let data = body.data.ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "empty response from Linear API".to_string(),
            )
        })?;
        Ok((data.team.projects.nodes, data.team.projects.page_info))
    } else {
        #[derive(Serialize)]
        struct Vars {
            first: i32,
            after: Option<String>,
        }
        let resp = execute_with_retry(
            client,
            api_key,
            PROJECTS_QUERY,
            Vars { first, after },
        )
        .await?;
        let body: GraphqlResponse<ProjectsData> = resp;
        if let Some(errors) = body.errors {
            return Err(map_errors(errors));
        }
        let data = body.data.ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "empty response from Linear API".to_string(),
            )
        })?;
        Ok((data.projects.nodes, data.projects.page_info))
    }
}

// ---- Get Project ----

#[derive(Deserialize, Debug)]
pub struct GetProjectData {
    pub project: Option<ProjectNode>,
}

#[derive(Deserialize, Debug)]
pub struct SlugLookupData {
    pub projects: SlugProjectConnection,
}

#[derive(Deserialize, Debug)]
pub struct SlugProjectConnection {
    pub nodes: Vec<SlugProjectNode>,
}

#[derive(Deserialize, Debug)]
pub struct SlugProjectNode {
    pub id: String,
}

const GET_PROJECT_QUERY: &str = r#"
query GetProject($id: String!) {
  project(id: $id) {
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
"#;

const SLUG_LOOKUP_QUERY: &str = r#"
query SlugLookup($slugId: String!) {
  projects(filter: { slugId: { eq: $slugId } }, first: 1) {
    nodes { id }
  }
}
"#;

pub async fn fetch_project_by_id(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    #[derive(Serialize)]
    struct Vars<'a> {
        id: &'a str,
    }
    let resp: GraphqlResponse<GetProjectData> =
        execute_with_retry(client, api_key, GET_PROJECT_QUERY, Vars { id }).await?;
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
    #[derive(Serialize)]
    struct Vars<'a> {
        #[serde(rename = "slugId")]
        slug_id: &'a str,
    }
    let resp: GraphqlResponse<SlugLookupData> =
        execute_with_retry(client, api_key, SLUG_LOOKUP_QUERY, Vars { slug_id: slug })
            .await?;
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
        .map(|n| n.id)
        .ok_or_else(|| crate::domain::errors::DomainError::NotFound(slug.to_string()))
}

// ---- Workspace project statuses ----

#[derive(Deserialize, Debug)]
pub struct OrgStatusData {
    pub organization: OrgWithStatuses,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrgWithStatuses {
    pub project_statuses: Vec<ProjectStatusNode>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectStatusNode {
    pub id: String,
    #[serde(rename = "type")]
    pub status_type: String,
}

const ORG_PROJECT_STATUSES_QUERY: &str = r#"
query OrgProjectStatuses {
  organization {
    projectStatuses { id type }
  }
}
"#;

pub async fn fetch_status_id_for_type(
    client: &reqwest::Client,
    api_key: &str,
    state_type: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let resp: GraphqlResponse<OrgStatusData> = execute_with_retry(
        client,
        api_key,
        ORG_PROJECT_STATUSES_QUERY,
        serde_json::json!({}),
    )
    .await?;
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
    let normalized = match state_type {
        "cancelled" => "canceled",
        other => other,
    };
    statuses
        .into_iter()
        .find(|s| s.status_type == normalized)
        .map(|s| s.id)
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
