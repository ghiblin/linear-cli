use async_trait::async_trait;

use crate::domain::{
    entities::issue::{
        CreateIssueInput, Issue, ListIssuesInput, ListIssuesResult, UpdateIssueInput,
        WorkflowStateInfo,
    },
    errors::DomainError,
    value_objects::{issue_id::IssueId, team_id::TeamId},
};

#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn list(&self, input: ListIssuesInput) -> Result<ListIssuesResult, DomainError>;
    async fn get(&self, id: IssueId) -> Result<Issue, DomainError>;
    async fn create(&self, input: CreateIssueInput) -> Result<Issue, DomainError>;
    async fn update(&self, id: IssueId, input: UpdateIssueInput) -> Result<Issue, DomainError>;
    async fn list_workflow_states(
        &self,
        team_id: TeamId,
    ) -> Result<Vec<WorkflowStateInfo>, DomainError>;
}
