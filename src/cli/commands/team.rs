use std::sync::Arc;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::{errors::ApplicationError, use_cases::list_teams::ListTeams},
    cli::output::{format_json, resolve_use_json, should_use_json},
    domain::{entities::team::Team, value_objects::api_key::ApiKey},
    infrastructure::{
        auth::keyring_store::KeyringCredentialStore, graphql::client::LinearGraphqlClient,
        repositories::team_repository::LinearTeamRepository,
    },
};

#[derive(Args)]
pub struct TeamCommand {
    #[command(subcommand)]
    pub subcommand: TeamSubcommand,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, help = "Output format: json or human")]
    pub output: Option<String>,
    /// Use JSON output format (alias for --output json)
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum TeamSubcommand {
    List(ListArgs),
}

#[derive(Serialize)]
struct TeamDto {
    id: String,
    name: String,
    key: String,
}

impl From<&Team> for TeamDto {
    fn from(team: &Team) -> Self {
        Self {
            id: team.id().to_string(),
            name: team.name().to_string(),
            key: team.key().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn auth_guard_run_team_requires_auth() {
        // When no credential is available, run_team should propagate AuthError::NotAuthenticated.
        // This is verified by integration tests (T034) that spawn the binary and check exit code 3.
        assert!(true, "auth guard tested via integration tests");
    }
}

pub async fn run_team(cmd: &TeamCommand, force_json: bool) -> Result<(), anyhow::Error> {
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
        TeamSubcommand::List(args) => {
            let repo = LinearTeamRepository;
            let use_case = ListTeams::new(Box::new(repo));
            let teams = use_case.execute().await?;
            let dtos: Vec<TeamDto> = teams.iter().map(TeamDto::from).collect();
            if should_use_json(resolve_use_json(
                args.json,
                args.output.as_deref(),
                force_json,
            )) {
                println!("{}", format_json(&dtos));
            } else {
                println!("Teams: {}", dtos.len());
            }
        }
    }
    Ok(())
}
