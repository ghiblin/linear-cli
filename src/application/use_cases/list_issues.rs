use tracing::instrument;

use crate::{
    application::errors::ApplicationError,
    domain::{
        entities::issue::{ListIssuesInput, ListIssuesResult},
        repositories::issue_repository::IssueRepository,
    },
};

pub struct ListIssues {
    repo: Box<dyn IssueRepository>,
}

impl ListIssues {
    pub fn new(repo: Box<dyn IssueRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        input: ListIssuesInput,
    ) -> Result<ListIssuesResult, ApplicationError> {
        Ok(self.repo.list(input).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::issue::{
            CreateIssueInput, Issue, ListIssuesInput, ListIssuesResult, UpdateIssueInput,
            WorkflowStateInfo,
        },
        errors::DomainError,
        value_objects::{issue_id::IssueId, team_id::TeamId},
    };

    mockall::mock! {
        IssueRepo {}

        #[async_trait::async_trait]
        impl IssueRepository for IssueRepo {
            async fn list(&self, input: ListIssuesInput) -> Result<ListIssuesResult, DomainError>;
            async fn get(&self, id: IssueId) -> Result<Issue, DomainError>;
            async fn create(&self, input: CreateIssueInput) -> Result<Issue, DomainError>;
            async fn update(&self, id: IssueId, input: UpdateIssueInput) -> Result<Issue, DomainError>;
            async fn list_workflow_states(&self, team_id: TeamId) -> Result<Vec<WorkflowStateInfo>, DomainError>;
            async fn delete(&self, id: IssueId) -> Result<(), DomainError>;
        }
    }

    fn default_input() -> ListIssuesInput {
        ListIssuesInput {
            team_id: None,
            project_id: None,
            state_name: None,
            assignee_id: None,
            priority: None,
            label_ids: vec![],
            limit: 50,
            cursor: None,
            all_pages: false,
            title_contains: None,
        }
    }

    #[tokio::test]
    async fn returns_empty_result_when_repo_returns_empty() {
        let mut mock = MockIssueRepo::new();
        mock.expect_list().returning(|_| {
            Ok(ListIssuesResult {
                items: vec![],
                next_cursor: None,
                has_next_page: false,
            })
        });

        let use_case = ListIssues::new(Box::new(mock));
        let result = use_case.execute(default_input()).await.unwrap();
        assert!(result.items.is_empty());
    }

    #[tokio::test]
    async fn propagates_domain_error_as_application_error() {
        let mut mock = MockIssueRepo::new();
        mock.expect_list()
            .returning(|_| Err(DomainError::NotImplemented));

        let use_case = ListIssues::new(Box::new(mock));
        let result = use_case.execute(default_input()).await;
        assert!(result.is_err());
    }

    // T003
    #[tokio::test]
    async fn list_issues_use_case_passes_title_contains() {
        let mut mock = MockIssueRepo::new();
        mock.expect_list()
            .withf(|input| input.title_contains.as_deref() == Some("fix"))
            .returning(|_| {
                Ok(ListIssuesResult {
                    items: vec![],
                    next_cursor: None,
                    has_next_page: false,
                })
            });

        let use_case = ListIssues::new(Box::new(mock));
        let mut input = default_input();
        input.title_contains = Some("fix".to_string());
        let result = use_case.execute(input).await.unwrap();
        assert!(result.items.is_empty());
    }
}
