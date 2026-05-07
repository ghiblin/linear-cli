use std::sync::Arc;

use chrono::NaiveDate;
use serde::Serialize;
use tracing::instrument;

use crate::domain::{
    entities::project::{CreateProjectInput, Project},
    errors::DomainError,
    repositories::project_repository::ProjectRepository,
    value_objects::{UserId, team_id::TeamId},
};

#[derive(Debug, Clone, Serialize)]
pub struct CreateProjectArgs {
    pub name: String,
    pub team_ids: Vec<String>,
    pub description: Option<String>,
    pub lead_id: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

pub struct CreateProject {
    repo: Arc<dyn ProjectRepository>,
}

impl CreateProject {
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        args: CreateProjectArgs,
        dry_run: bool,
    ) -> Result<Option<Project>, DomainError> {
        if args.name.is_empty() {
            return Err(DomainError::InvalidInput(
                "project name cannot be empty".to_string(),
            ));
        }
        if args.team_ids.is_empty() {
            return Err(DomainError::InvalidInput(
                "at least one team id is required".to_string(),
            ));
        }

        if dry_run {
            return Ok(None);
        }

        let team_ids: Vec<TeamId> = args
            .team_ids
            .iter()
            .map(|s| TeamId::new(s.clone()))
            .collect::<Result<_, _>>()?;

        let lead_id = args.lead_id.map(UserId::new).transpose()?;

        let input = CreateProjectInput {
            name: args.name,
            team_ids,
            description: args.description,
            lead_id,
            start_date: args.start_date,
            target_date: args.target_date,
        };

        self.repo.create(input).await.map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;

    use crate::domain::{
        entities::project::UpdateProjectInput,
        repositories::project_repository::{ListProjectsResult, ProjectRepository},
        value_objects::{ProjectId, ProjectState},
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
                name_contains: Option<String>,
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
            ProjectState::Planned,
            0.0,
            None,
            vec![crate::domain::value_objects::team_id::TeamId::new("team-1".to_string()).unwrap()],
            None,
            None,
            Utc::now(),
            "test".to_string(),
        )
        .unwrap()
    }

    fn valid_args() -> CreateProjectArgs {
        CreateProjectArgs {
            name: "My Project".to_string(),
            team_ids: vec!["team-1".to_string()],
            description: None,
            lead_id: None,
            start_date: None,
            target_date: None,
        }
    }

    #[tokio::test]
    async fn dry_run_returns_none_without_api_call() {
        let mock = MockTestRepo::new();
        let uc = CreateProject::new(Arc::new(mock));
        let result = uc.execute(valid_args(), true).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn missing_name_fails_before_api_call() {
        let mock = MockTestRepo::new();
        let uc = CreateProject::new(Arc::new(mock));
        let mut args = valid_args();
        args.name = "".to_string();
        let result = uc.execute(args, false).await;
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn missing_team_fails_before_api_call() {
        let mock = MockTestRepo::new();
        let uc = CreateProject::new(Arc::new(mock));
        let mut args = valid_args();
        args.team_ids = vec![];
        let result = uc.execute(args, false).await;
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn valid_create_calls_repo() {
        let mut mock = MockTestRepo::new();
        mock.expect_create()
            .times(1)
            .returning(|_| Ok(sample_project()));

        let uc = CreateProject::new(Arc::new(mock));
        let result = uc.execute(valid_args(), false).await.unwrap();
        assert!(result.is_some());
    }
}
