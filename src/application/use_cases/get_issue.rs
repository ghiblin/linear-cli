use tracing::instrument;

use crate::{
    application::errors::ApplicationError,
    domain::{
        entities::issue::Issue,
        errors::DomainError,
        repositories::issue_repository::IssueRepository,
        value_objects::issue_id::IssueId,
    },
};

pub struct GetIssue {
    repo: Box<dyn IssueRepository>,
}

impl GetIssue {
    pub fn new(repo: Box<dyn IssueRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, id: String) -> Result<Issue, ApplicationError> {
        let issue_id =
            IssueId::new(id).map_err(|e| ApplicationError::Domain(DomainError::InvalidInput(e.to_string())))?;
        Ok(self.repo.get(issue_id).await?)
    }
}
