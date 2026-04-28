use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use tracing::instrument;

use crate::{
    domain::{
        entities::project::{CreateProjectInput, Project, UpdateProjectInput},
        errors::DomainError,
        repositories::project_repository::{ListProjectsResult, PageInfo, ProjectRepository},
        value_objects::{ProjectId, ProjectState, UserId, team_id::TeamId},
    },
    infrastructure::graphql::{
        mutations::project_mutations::{
            ProjectCreateInputVars, ProjectUpdateInputVars, archive_project, create_project,
            update_project,
        },
        queries::project_queries::{
            ProjectNode, fetch_project_by_id, fetch_projects, fetch_status_id_for_type,
            resolve_slug_to_uuid,
        },
    },
};

pub struct LinearProjectRepository {
    http: Client,
    api_key: String,
}

#[allow(dead_code)]
impl LinearProjectRepository {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
        }
    }

    async fn resolve_id(&self, id: &ProjectId) -> Result<String, DomainError> {
        match id {
            ProjectId::Uuid(uuid) => Ok(uuid.clone()),
            ProjectId::Slug(slug) => resolve_slug_to_uuid(&self.http, &self.api_key, slug).await,
        }
    }
}

fn project_state_from_str(s: &str) -> ProjectState {
    match s.to_lowercase().as_str() {
        "started" => ProjectState::Started,
        "paused" => ProjectState::Paused,
        "completed" => ProjectState::Completed,
        "cancelled" | "canceled" => ProjectState::Cancelled,
        _ => ProjectState::Planned,
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn node_to_project(node: ProjectNode) -> Result<Project, DomainError> {
    Project::new(
        node.id,
        node.name,
        if node.description.is_empty() {
            None
        } else {
            Some(node.description)
        },
        project_state_from_str(&node.state),
        node.progress,
        node.lead.and_then(|l| UserId::new(l.id).ok()),
        node.teams.nodes.into_iter().filter_map(|t| TeamId::new(t.id).ok()).collect(),
        node.start_date.as_deref().and_then(parse_date),
        node.target_date.as_deref().and_then(parse_date),
        parse_datetime(&node.updated_at),
        node.slug_id,
    )
}

#[async_trait]
impl ProjectRepository for LinearProjectRepository {
    #[instrument(skip(self))]
    async fn list(
        &self,
        team_id: Option<TeamId>,
        first: u32,
        after: Option<String>,
    ) -> Result<ListProjectsResult, DomainError> {
        let tid = team_id.as_ref().map(|t| t.as_str());
        let (nodes, page_info_node) =
            fetch_projects(&self.http, &self.api_key, first as i32, after, tid).await?;
        let items = nodes
            .into_iter()
            .map(node_to_project)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ListProjectsResult {
            items,
            page_info: PageInfo {
                has_next_page: page_info_node.has_next_page,
                end_cursor: page_info_node.end_cursor,
            },
        })
    }

    #[instrument(skip(self))]
    async fn get(&self, id: ProjectId) -> Result<Project, DomainError> {
        let uuid = self.resolve_id(&id).await?;
        let node = fetch_project_by_id(&self.http, &self.api_key, &uuid).await?;
        node_to_project(node)
    }

    #[instrument(skip(self))]
    async fn create(&self, input: CreateProjectInput) -> Result<Project, DomainError> {
        let status_id = None;
        let vars = ProjectCreateInputVars {
            name: input.name,
            team_ids: input.team_ids.iter().map(|t| t.as_str().to_string()).collect(),
            description: input.description,
            lead_id: input.lead_id.map(|l| l.as_str().to_string()),
            start_date: input.start_date.map(|d| d.to_string()),
            target_date: input.target_date.map(|d| d.to_string()),
            status_id,
        };
        let node = create_project(&self.http, &self.api_key, vars).await?;
        node_to_project(node)
    }

    #[instrument(skip(self))]
    async fn update(
        &self,
        id: ProjectId,
        input: UpdateProjectInput,
    ) -> Result<Project, DomainError> {
        let uuid = self.resolve_id(&id).await?;
        let status_id = if let Some(ref state) = input.state {
            Some(
                fetch_status_id_for_type(&self.http, &self.api_key, &state.to_string()).await?,
            )
        } else {
            None
        };
        let vars = ProjectUpdateInputVars {
            name: input.name,
            description: input.description,
            lead_id: input.lead_id.map(|l| l.as_str().to_string()),
            start_date: input.start_date.map(|d| d.to_string()),
            target_date: input.target_date.map(|d| d.to_string()),
            status_id,
        };
        let node = update_project(&self.http, &self.api_key, &uuid, vars).await?;
        node_to_project(node)
    }

    #[instrument(skip(self))]
    async fn archive(&self, id: ProjectId) -> Result<(), DomainError> {
        let uuid = self.resolve_id(&id).await?;
        archive_project(&self.http, &self.api_key, &uuid).await?;
        Ok(())
    }
}
