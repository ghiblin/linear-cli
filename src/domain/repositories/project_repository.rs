use async_trait::async_trait;

use crate::domain::{
    entities::project::{CreateProjectInput, Project, UpdateProjectInput},
    errors::DomainError,
    value_objects::{ProjectId, team_id::TeamId},
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ListProjectsResult {
    pub items: Vec<Project>,
    pub page_info: PageInfo,
}

#[async_trait]
#[allow(dead_code)]
pub trait ProjectRepository: Send + Sync {
    async fn list(
        &self,
        team_id: Option<TeamId>,
        first: u32,
        after: Option<String>,
    ) -> Result<ListProjectsResult, DomainError>;

    async fn get(&self, id: ProjectId) -> Result<Project, DomainError>;

    async fn create(&self, input: CreateProjectInput) -> Result<Project, DomainError>;

    async fn update(
        &self,
        id: ProjectId,
        input: UpdateProjectInput,
    ) -> Result<Project, DomainError>;

    async fn archive(&self, id: ProjectId) -> Result<(), DomainError>;
}
