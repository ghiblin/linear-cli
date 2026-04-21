use crate::{
    application::errors::ApplicationError,
    domain::{
        entities::issue::Issue, repositories::issue_repository::IssueRepository,
        value_objects::team_id::TeamId,
    },
};

pub struct ListIssues {
    repo: Box<dyn IssueRepository>,
}

impl ListIssues {
    pub fn new(repo: Box<dyn IssueRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, team_id: Option<TeamId>) -> Result<Vec<Issue>, ApplicationError> {
        let id = team_id.unwrap_or_else(|| TeamId::new("default".to_string()).unwrap());
        Ok(self.repo.list(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::issue::Issue,
        errors::DomainError,
        value_objects::{issue_id::IssueId, team_id::TeamId},
    };

    mockall::mock! {
        IssueRepo {}

        #[async_trait::async_trait]
        impl IssueRepository for IssueRepo {
            async fn list(&self, team_id: TeamId) -> Result<Vec<Issue>, DomainError>;
            async fn get(&self, id: IssueId) -> Result<Issue, DomainError>;
        }
    }

    #[tokio::test]
    async fn returns_empty_vec_when_repo_returns_empty() {
        let mut mock = MockIssueRepo::new();
        mock.expect_list().returning(|_| Ok(vec![]));

        let use_case = ListIssues::new(Box::new(mock));
        let team_id = TeamId::new("team-1".to_string()).unwrap();
        let result = use_case.execute(Some(team_id)).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn propagates_domain_error_as_application_error() {
        let mut mock = MockIssueRepo::new();
        mock.expect_list()
            .returning(|_| Err(DomainError::NotImplemented));

        let use_case = ListIssues::new(Box::new(mock));
        let team_id = TeamId::new("team-1".to_string()).unwrap();
        let result = use_case.execute(Some(team_id)).await;
        assert!(result.is_err());
    }
}
