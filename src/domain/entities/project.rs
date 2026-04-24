use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    errors::DomainError,
    value_objects::{ProjectId, ProjectState, UserId, team_id::TeamId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: ProjectState,
    pub progress: f64,
    pub lead_id: Option<String>,
    pub team_ids: Vec<String>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub updated_at: DateTime<Utc>,
    pub slug_id: String,
}

#[allow(dead_code, clippy::too_many_arguments)]
impl Project {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        state: ProjectState,
        progress: f64,
        lead_id: Option<String>,
        team_ids: Vec<String>,
        start_date: Option<NaiveDate>,
        target_date: Option<NaiveDate>,
        updated_at: DateTime<Utc>,
        slug_id: String,
    ) -> Result<Self, DomainError> {
        if name.is_empty() {
            return Err(DomainError::InvalidInput(
                "project name cannot be empty".to_string(),
            ));
        }
        if team_ids.is_empty() {
            return Err(DomainError::InvalidInput(
                "project must belong to at least one team".to_string(),
            ));
        }
        if !(0.0..=100.0).contains(&progress) {
            return Err(DomainError::InvalidInput(format!(
                "progress must be between 0.0 and 100.0, got {}",
                progress
            )));
        }
        Ok(Self {
            id,
            name,
            description,
            state,
            progress,
            lead_id,
            team_ids,
            start_date,
            target_date,
            updated_at,
            slug_id,
        })
    }

    pub fn project_id(&self) -> Result<ProjectId, DomainError> {
        ProjectId::parse(&self.id)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CreateProjectInput {
    pub name: String,
    pub team_ids: Vec<TeamId>,
    pub description: Option<String>,
    pub lead_id: Option<UserId>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<ProjectState>,
    pub lead_id: Option<UserId>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

#[allow(dead_code)]
impl UpdateProjectInput {
    pub fn has_changes(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.state.is_some()
            || self.lead_id.is_some()
            || self.start_date.is_some()
            || self.target_date.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_project(name: &str, team_ids: Vec<String>, progress: f64) -> Result<Project, DomainError> {
        Project::new(
            "some-uuid".to_string(),
            name.to_string(),
            None,
            ProjectState::Planned,
            progress,
            None,
            team_ids,
            None,
            None,
            Utc::now(),
            "some-slug".to_string(),
        )
    }

    #[test]
    fn rejects_empty_name() {
        let err = make_project("", vec!["team-1".to_string()], 0.0).unwrap_err();
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn rejects_empty_teams() {
        let err = make_project("Valid Name", vec![], 0.0).unwrap_err();
        assert!(err.to_string().contains("team"));
    }

    #[test]
    fn rejects_invalid_progress() {
        let err = make_project("Valid", vec!["team-1".to_string()], 101.0).unwrap_err();
        assert!(err.to_string().contains("progress"));
    }

    #[test]
    fn accepts_valid_project() {
        let p = make_project("My Project", vec!["team-1".to_string()], 50.0).unwrap();
        assert_eq!(p.name, "My Project");
        assert_eq!(p.progress, 50.0);
    }
}
