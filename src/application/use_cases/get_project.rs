use std::sync::Arc;

use tracing::instrument;

use crate::domain::{
    entities::project::Project,
    errors::DomainError,
    repositories::project_repository::ProjectRepository,
    value_objects::ProjectId,
};

pub struct GetProject {
    repo: Arc<dyn ProjectRepository>,
}

impl GetProject {
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, id: ProjectId) -> Result<Project, DomainError> {
        self.repo.get(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;

    use crate::domain::{
        entities::project::{CreateProjectInput, UpdateProjectInput},
        repositories::project_repository::{ListProjectsResult, ProjectRepository},
        value_objects::{ProjectState, team_id::TeamId},
    };

    mock! {
        TestRepo {}
        #[async_trait]
        impl ProjectRepository for TestRepo {
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
    }

    fn sample_project() -> Project {
        Project::new(
            "uuid-1".to_string(),
            "Test Project".to_string(),
            None,
            ProjectState::Planned,
            0.0,
            None,
            vec![crate::domain::value_objects::team_id::TeamId::new("team-1".to_string()).unwrap()],
            None,
            None,
            Utc::now(),
            "test-project".to_string(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn returns_project_when_found() {
        let mut mock = MockTestRepo::new();
        mock.expect_get()
            .times(1)
            .returning(|_| Ok(sample_project()));

        let uc = GetProject::new(Arc::new(mock));
        let result = uc.execute(ProjectId::parse("uuid-1").unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn propagates_not_found_error() {
        let mut mock = MockTestRepo::new();
        mock.expect_get()
            .times(1)
            .returning(|_| Err(DomainError::NotFound("uuid-1".to_string())));

        let uc = GetProject::new(Arc::new(mock));
        let result = uc.execute(ProjectId::parse("uuid-1").unwrap()).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }
}
