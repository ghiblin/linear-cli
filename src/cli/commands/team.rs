use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::use_cases::list_teams::ListTeams,
    cli::output::{format_json, should_use_json},
    domain::entities::team::Team,
    infrastructure::repositories::team_repository::LinearTeamRepository,
};

#[derive(Args)]
pub struct TeamCommand {
    #[command(subcommand)]
    pub subcommand: TeamSubcommand,
}

#[derive(Subcommand)]
pub enum TeamSubcommand {
    List,
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

pub async fn run_team(cmd: &TeamCommand, force_json: bool) -> Result<(), anyhow::Error> {
    match &cmd.subcommand {
        TeamSubcommand::List => {
            let repo = LinearTeamRepository;
            let use_case = ListTeams::new(Box::new(repo));
            let teams = use_case.execute().await?;
            let dtos: Vec<TeamDto> = teams.iter().map(TeamDto::from).collect();
            if should_use_json(force_json) {
                println!("{}", format_json(&dtos));
            } else {
                println!("Teams: {}", dtos.len());
            }
        }
    }
    Ok(())
}
