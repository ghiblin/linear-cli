use std::sync::Arc;

use chrono::NaiveDate;
use serde::Serialize;
use tracing::instrument;

use crate::domain::{
    entities::project::{Project, UpdateProjectInput},
    errors::DomainError,
    repositories::project_repository::ProjectRepository,
    value_objects::{ProjectId, ProjectState, UserId},
};

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProjectArgs {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<ProjectState>,
    pub lead_id: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

impl UpdateProjectArgs {
    pub fn has_changes(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.state.is_some()
            || self.lead_id.is_some()
            || self.start_date.is_some()
            || self.target_date.is_some()
    }
}

pub struct UpdateProject {
    repo: Arc<dyn ProjectRepository>,
}

impl UpdateProject {
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        id: ProjectId,
        args: UpdateProjectArgs,
        dry_run: bool,
    ) -> Result<Option<Project>, DomainError> {
        if !args.has_changes() {
            return Err(DomainError::InvalidInput(
                "at least one update field must be provided".to_string(),
            ));
        }

        if dry_run {
            return Ok(None);
        }

        let lead_id = args.lead_id.map(UserId::new).transpose()?;

        let input = UpdateProjectInput {
            name: args.name,
            description: args.description,
            state: args.state,
            lead_id,
            start_date: args.start_date,
            target_date: args.target_date,
        };

        self.repo.update(id, input).await.map(Some)
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

    fn sample_project() -> Project {
        Project::new(
            "uuid-1".to_string(),
            "Test".to_string(),
            None,
            ProjectState::Started,
            0.0,
            None,
            vec!["team-1".to_string()],
            None,
            None,
            Utc::now(),
            "test".to_string(),
        )
        .unwrap()
    }

    fn args_with_state(state: ProjectState) -> UpdateProjectArgs {
        UpdateProjectArgs {
            name: None,
            description: None,
            state: Some(state),
            lead_id: None,
            start_date: None,
            target_date: None,
        }
    }

    #[tokio::test]
    async fn no_fields_returns_error() {
        let mock = MockTestRepo::new();
        let uc = UpdateProject::new(Arc::new(mock));
        let args = UpdateProjectArgs {
            name: None,
            description: None,
            state: None,
            lead_id: None,
            start_date: None,
            target_date: None,
        };
        let result = uc
            .execute(ProjectId::parse("uuid-1").unwrap(), args, false)
            .await;
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn dry_run_returns_none_without_api_call() {
        let mock = MockTestRepo::new();
        let uc = UpdateProject::new(Arc::new(mock));
        let result = uc
            .execute(
                ProjectId::parse("uuid-1").unwrap(),
                args_with_state(ProjectState::Started),
                true,
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn valid_update_calls_repo() {
        let mut mock = MockTestRepo::new();
        mock.expect_update()
            .times(1)
            .returning(|_, _| Ok(sample_project()));

        let uc = UpdateProject::new(Arc::new(mock));
        let result = uc
            .execute(
                ProjectId::parse("uuid-1").unwrap(),
                args_with_state(ProjectState::Started),
                false,
            )
            .await
            .unwrap();
        assert!(result.is_some());
    }
}
