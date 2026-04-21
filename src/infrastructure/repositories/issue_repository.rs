use async_trait::async_trait;

use crate::domain::{
    entities::issue::Issue,
    errors::DomainError,
    repositories::issue_repository::IssueRepository,
    value_objects::{issue_id::IssueId, team_id::TeamId},
};

pub struct LinearIssueRepository;

#[async_trait]
impl IssueRepository for LinearIssueRepository {
    async fn list(&self, _team_id: TeamId) -> Result<Vec<Issue>, DomainError> {
        Ok(vec![])
    }

    async fn get(&self, _id: IssueId) -> Result<Issue, DomainError> {
        Err(DomainError::NotImplemented)
    }
}
