use tracing::instrument;

use crate::{
    application::errors::ApplicationError,
    domain::{
        entities::issue::{CreateIssueInput, Issue},
        errors::DomainError,
        repositories::issue_repository::IssueRepository,
    },
};

pub struct CreateIssue {
    repo: Box<dyn IssueRepository>,
}

impl CreateIssue {
    pub fn new(repo: Box<dyn IssueRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        mut input: CreateIssueInput,
        dry_run: bool,
    ) -> Result<Issue, ApplicationError> {
        if input.title.is_empty() {
            return Err(ApplicationError::Domain(DomainError::InvalidInput(
                "--title is required".to_string(),
            )));
        }
        if input.team_id.to_string().is_empty() {
            return Err(ApplicationError::Domain(DomainError::InvalidInput(
                "--team is required".to_string(),
            )));
        }
        if let Some(ref date) = input.due_date {
            validate_date(date)?;
        }
        if let Some(estimate) = input.estimate {
            if estimate < 0.0 {
                return Err(ApplicationError::Domain(DomainError::InvalidInput(
                    "--estimate must be non-negative".to_string(),
                )));
            }
        }

        // Resolve display ID for parent if needed
        if let Some(ref parent_id) = input.parent_id.clone() {
            if is_display_id(parent_id.as_str()) && !dry_run {
                let resolved = self.repo.get(parent_id.clone()).await?;
                input.parent_id = Some(resolved.id().clone());
            }
        }

        if dry_run {
            return Err(ApplicationError::Domain(DomainError::InvalidInput(
                "dry-run: issue not created".to_string(),
            )));
        }

        Ok(self.repo.create(input).await?)
    }
}

fn validate_date(date: &str) -> Result<(), ApplicationError> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].chars().all(|c| c.is_ascii_digit())
        && parts[2].chars().all(|c| c.is_ascii_digit())
    {
        Ok(())
    } else {
        Err(ApplicationError::Domain(DomainError::InvalidInput(
            format!("--due-date '{}' must be in YYYY-MM-DD format", date),
        )))
    }
}

fn is_display_id(s: &str) -> bool {
    let parts: Vec<&str> = s.rsplitn(2, '-').collect();
    if parts.len() != 2 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_digit())
        && !parts[0].is_empty()
        && parts[1].chars().all(|c| c.is_ascii_uppercase())
        && !parts[1].is_empty()
}
