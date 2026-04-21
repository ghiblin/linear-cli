use crate::{
    application::errors::ApplicationError,
    domain::{entities::team::Team, repositories::team_repository::TeamRepository},
};

pub struct ListTeams {
    repo: Box<dyn TeamRepository>,
}

impl ListTeams {
    pub fn new(repo: Box<dyn TeamRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<Team>, ApplicationError> {
        Ok(self.repo.list().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::team::Team, errors::DomainError, value_objects::team_id::TeamId,
    };

    mockall::mock! {
        TeamRepo {}

        #[async_trait::async_trait]
        impl TeamRepository for TeamRepo {
            async fn list(&self) -> Result<Vec<Team>, DomainError>;
            async fn get(&self, id: TeamId) -> Result<Team, DomainError>;
        }
    }

    #[tokio::test]
    async fn returns_empty_vec_when_repo_returns_empty() {
        let mut mock = MockTeamRepo::new();
        mock.expect_list().returning(|| Ok(vec![]));

        let use_case = ListTeams::new(Box::new(mock));
        let result = use_case.execute().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn propagates_domain_error_as_application_error() {
        let mut mock = MockTeamRepo::new();
        mock.expect_list()
            .returning(|| Err(DomainError::NotImplemented));

        let use_case = ListTeams::new(Box::new(mock));
        let result = use_case.execute().await;
        assert!(result.is_err());
    }
}
