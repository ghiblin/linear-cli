use std::sync::Arc;

use tracing::instrument;

use crate::domain::{
    errors::DomainError,
    repositories::project_repository::ProjectRepository,
    value_objects::ProjectId,
};

pub enum ArchiveOutcome {
    Archived,
    AlreadyArchived,
}

pub struct ArchiveProject {
    repo: Arc<dyn ProjectRepository>,
}

impl ArchiveProject {
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, id: ProjectId, dry_run: bool) -> Result<ArchiveOutcome, DomainError> {
        if dry_run {
            return Ok(ArchiveOutcome::Archived);
        }
        match self.repo.archive(id).await {
            Ok(()) => Ok(ArchiveOutcome::Archived),
            Err(DomainError::NotFound(msg)) if msg.to_lowercase().contains("archived") => {
                Ok(ArchiveOutcome::AlreadyArchived)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    use crate::domain::{
        entities::project::{CreateProjectInput, Project, UpdateProjectInput},
        repositories::project_repository::{ListProjectsResult, ProjectRepository},
        value_objects::team_id::TeamId,
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

    #[tokio::test]
    async fn dry_run_returns_ok_without_api_call() {
        let mock = MockTestRepo::new();
        let uc = ArchiveProject::new(Arc::new(mock));
        let result = uc
            .execute(ProjectId::parse("uuid-1").unwrap(), true)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn success_returns_ok() {
        let mut mock = MockTestRepo::new();
        mock.expect_archive().times(1).returning(|_| Ok(()));
        let uc = ArchiveProject::new(Arc::new(mock));
        let result = uc
            .execute(ProjectId::parse("uuid-1").unwrap(), false)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let mut mock = MockTestRepo::new();
        mock.expect_archive()
            .times(1)
            .returning(|_| Err(DomainError::NotFound("uuid-1".to_string())));
        let uc = ArchiveProject::new(Arc::new(mock));
        let result = uc
            .execute(ProjectId::parse("uuid-1").unwrap(), false)
            .await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }

    #[tokio::test]
    async fn already_archived_signal_returns_already_archived_outcome() {
        // The repo must signal already-archived via NotFound("...archived...").
        // The mutation layer is responsible for mapping success==false to this error.
        let mut mock = MockTestRepo::new();
        mock.expect_archive()
            .times(1)
            .returning(|_| Err(DomainError::NotFound("project is already archived".to_string())));
        let uc = ArchiveProject::new(Arc::new(mock));
        let result = uc
            .execute(ProjectId::parse("uuid-1").unwrap(), false)
            .await;
        assert!(matches!(result, Ok(ArchiveOutcome::AlreadyArchived)));
    }
}
