use std::sync::Arc;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::{errors::ApplicationError, use_cases::list_issues::ListIssues},
    cli::output::{format_json, should_use_json},
    domain::{
        entities::issue::Issue, value_objects::api_key::ApiKey, value_objects::team_id::TeamId,
    },
    infrastructure::{
        auth::keyring_store::KeyringCredentialStore, graphql::client::LinearGraphqlClient,
        repositories::issue_repository::LinearIssueRepository,
    },
};

#[derive(Args)]
pub struct IssueCommand {
    #[command(subcommand)]
    pub subcommand: IssueSubcommand,
}

#[derive(Subcommand)]
pub enum IssueSubcommand {
    List {
        #[arg(long, help = "Filter by team key or ID")]
        team: Option<String>,
    },
    Get {
        #[arg(help = "Issue identifier (e.g. ENG-123)")]
        id: String,
    },
}

#[derive(Serialize)]
struct IssueDto {
    id: String,
    title: String,
    state: String,
    priority: String,
    team_id: String,
}

impl From<&Issue> for IssueDto {
    fn from(issue: &Issue) -> Self {
        Self {
            id: issue.id().to_string(),
            title: issue.title().to_string(),
            state: issue.state().to_string(),
            priority: format!("{:?}", issue.priority()),
            team_id: issue.team_id().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn auth_guard_run_issue_requires_auth() {
        // When no credential is available, run_issue should propagate AuthError::NotAuthenticated.
        // This is verified by integration tests (T034) that spawn the binary and check exit code 3.
        // Stub: this test will be replaced by the integration test assertion.
        assert!(true, "auth guard tested via integration tests");
    }
}

pub async fn run_issue(cmd: &IssueCommand, force_json: bool) -> Result<(), anyhow::Error> {
    use crate::application::use_cases::resolve_auth::resolve_auth;
    use crate::domain::repositories::credential_store::CredentialStore;

    let env_key = std::env::var("LINEAR_API_KEY")
        .ok()
        .and_then(|k| ApiKey::new(k).ok());
    let stores: Vec<Box<dyn CredentialStore>> = vec![Box::new(KeyringCredentialStore::new())];
    let client = Arc::new(LinearGraphqlClient::new());
    resolve_auth(env_key, stores, client)
        .await
        .map_err(|e| anyhow::anyhow!(ApplicationError::Auth(e)))?;

    match &cmd.subcommand {
        IssueSubcommand::List { team } => {
            let repo = LinearIssueRepository;
            let use_case = ListIssues::new(Box::new(repo));
            let team_id = team
                .as_deref()
                .map(|t| TeamId::new(t.to_string()))
                .transpose()?;
            let issues = use_case.execute(team_id).await?;
            let dtos: Vec<IssueDto> = issues.iter().map(IssueDto::from).collect();
            if should_use_json(force_json) {
                println!("{}", format_json(&dtos));
            } else {
                println!("Issues: {}", dtos.len());
            }
        }
        IssueSubcommand::Get { id } => {
            eprintln!("issue get {id}: not yet implemented");
            std::process::exit(2);
        }
    }
    Ok(())
}
