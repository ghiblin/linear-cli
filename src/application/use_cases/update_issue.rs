use tracing::instrument;

use crate::{
    application::errors::ApplicationError,
    domain::{
        entities::issue::{Issue, UpdateIssueInput},
        errors::DomainError,
        repositories::issue_repository::IssueRepository,
        value_objects::issue_id::IssueId,
    },
};

pub struct UpdateIssue {
    repo: Box<dyn IssueRepository>,
}

impl UpdateIssue {
    pub fn new(repo: Box<dyn IssueRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        id: String,
        mut input: UpdateIssueInput,
        dry_run: bool,
    ) -> Result<Issue, ApplicationError> {
        if input.parent_id.is_some() && input.no_parent {
            return Err(ApplicationError::Domain(DomainError::InvalidInput(
                "--parent and --no-parent are mutually exclusive".to_string(),
            )));
        }
        let all_none = input.title.is_none()
            && input.description.is_none()
            && input.state_id.is_none()
            && input.priority.is_none()
            && input.assignee_id.is_none()
            && input.due_date.is_none()
            && input.estimate.is_none()
            && input.parent_id.is_none()
            && !input.no_parent;
        if all_none {
            return Err(ApplicationError::Domain(DomainError::InvalidInput(
                "at least one update flag required".to_string(),
            )));
        }

        let issue_id = IssueId::new(id.clone())
            .map_err(|e| ApplicationError::Domain(DomainError::InvalidInput(e.to_string())))?;

        // Resolve state name to UUID
        if let Some(ref state_name) = input.state_id.clone() {
            let issue = self.repo.get(issue_id.clone()).await?;
            let team_id = issue.team_id().clone();
            let states = self.repo.list_workflow_states(team_id).await?;
            let matched = states
                .iter()
                .find(|s| s.name.to_lowercase() == state_name.to_lowercase());
            match matched {
                Some(s) => {
                    input.state_id = Some(s.id.clone());
                }
                None => {
                    let valid: Vec<&str> = states.iter().map(|s| s.name.as_str()).collect();
                    return Err(ApplicationError::Domain(DomainError::InvalidInput(
                        format!(
                            "--state \"{}\" is not valid; valid states: {}",
                            state_name,
                            valid.join(", ")
                        ),
                    )));
                }
            }
        }

        if dry_run {
            return Err(ApplicationError::Domain(DomainError::InvalidInput(
                "dry-run: issue not updated".to_string(),
            )));
        }

        Ok(self.repo.update(issue_id, input).await?)
    }
}
