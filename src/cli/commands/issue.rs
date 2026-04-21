use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::use_cases::list_issues::ListIssues,
    cli::output::{format_json, should_use_json},
    domain::{entities::issue::Issue, value_objects::team_id::TeamId},
    infrastructure::repositories::issue_repository::LinearIssueRepository,
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

pub async fn run_issue(cmd: &IssueCommand, force_json: bool) -> Result<(), anyhow::Error> {
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
