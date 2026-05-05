pub mod auth;
pub mod issue;
pub mod project;
pub mod team;

use clap::{Parser, Subcommand};

use crate::cli::commands::{
    auth::AuthCommand, issue::IssueCommand, project::ProjectCommand, team::TeamCommand,
};

#[derive(Parser)]
#[command(
    name = "linear",
    about = "A CLI for Linear",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(long, global = true, help = "Force JSON output")]
    pub json: bool,

    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = "Increase log verbosity (repeatable)"
    )]
    pub verbose: u8,

    #[arg(long, help = "Output version information as JSON and exit")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Manage Linear authentication")]
    Auth(AuthCommand),
    #[command(about = "Manage Linear issues")]
    Issue(IssueCommand),
    #[command(about = "Manage Linear projects")]
    Project(ProjectCommand),
    #[command(about = "Manage Linear teams")]
    Team(TeamCommand),
}
