use std::sync::Arc;

use tracing::instrument;

use crate::domain::{
    errors::DomainError, repositories::issue_repository::IssueRepository,
    value_objects::issue_id::IssueId,
};

pub enum DeleteOutcome {
    Deleted,
}

pub struct DeleteIssue {
    repo: Arc<dyn IssueRepository>,
}

impl DeleteIssue {
    pub fn new(repo: Arc<dyn IssueRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, id: IssueId, dry_run: bool) -> Result<DeleteOutcome, DomainError> {
        if dry_run {
            return Ok(DeleteOutcome::Deleted);
        }
        self.repo.delete(id).await?;
        Ok(DeleteOutcome::Deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    use crate::domain::{
        entities::issue::{
            CreateIssueInput, Issue, ListIssuesInput, ListIssuesResult, UpdateIssueInput,
            WorkflowStateInfo,
        },
        repositories::issue_repository::IssueRepository,
        value_objects::{issue_id::IssueId, team_id::TeamId},
    };

    mock! {
        TestRepo {}
        #[async_trait]
        impl IssueRepository for TestRepo {
            async fn list(&self, input: ListIssuesInput) -> Result<ListIssuesResult, DomainError>;
            async fn get(&self, id: IssueId) -> Result<Issue, DomainError>;
            async fn create(&self, input: CreateIssueInput) -> Result<Issue, DomainError>;
            async fn update(&self, id: IssueId, input: UpdateIssueInput) -> Result<Issue, DomainError>;
            async fn list_workflow_states(&self, team_id: TeamId) -> Result<Vec<WorkflowStateInfo>, DomainError>;
            async fn delete(&self, id: IssueId) -> Result<(), DomainError>;
        }
    }

    #[tokio::test]
    async fn success_returns_deleted() {
        let mut mock = MockTestRepo::new();
        mock.expect_delete().times(1).returning(|_| Ok(()));
        let uc = DeleteIssue::new(Arc::new(mock));
        let result = uc
            .execute(IssueId::new("uuid-1".to_string()).unwrap(), false)
            .await;
        assert!(matches!(result, Ok(DeleteOutcome::Deleted)));
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let mut mock = MockTestRepo::new();
        mock.expect_delete()
            .times(1)
            .returning(|_| Err(DomainError::NotFound("uuid-1".to_string())));
        let uc = DeleteIssue::new(Arc::new(mock));
        let result = uc
            .execute(IssueId::new("uuid-1".to_string()).unwrap(), false)
            .await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }

    #[tokio::test]
    async fn dry_run_returns_ok_without_api_call() {
        let mock = MockTestRepo::new();
        let uc = DeleteIssue::new(Arc::new(mock));
        let result = uc
            .execute(IssueId::new("uuid-1".to_string()).unwrap(), true)
            .await;
        assert!(matches!(result, Ok(DeleteOutcome::Deleted)));
    }
}
