use async_trait::async_trait;

use crate::domain::{
    entities::issue::Issue,
    errors::DomainError,
    value_objects::{issue_id::IssueId, team_id::TeamId},
};

#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn list(&self, team_id: TeamId) -> Result<Vec<Issue>, DomainError>;
    #[allow(dead_code)]
    async fn get(&self, id: IssueId) -> Result<Issue, DomainError>;
}
