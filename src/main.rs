mod application;
mod cli;
mod domain;
mod infrastructure;

use std::process;

use clap::Parser;
use serde::Serialize;
use tracing_subscriber::{EnvFilter, fmt};

use crate::cli::commands::{Cli, Commands};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const API_SCHEMA_DATE: &str = "2026-04-21";

#[derive(Serialize)]
struct VersionInfo {
    version: &'static str,
    api_schema: &'static str,
}

#[tokio::main]
async fn main() {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 1 } else { 0 };
            e.print().expect("failed to print clap error");
            process::exit(code);
        }
    };

    init_tracing(cli.verbose);

    if cli.version {
        let info = VersionInfo {
            version: VERSION,
            api_schema: API_SCHEMA_DATE,
        };
        println!("{}", serde_json::to_string(&info).unwrap());
        process::exit(0);
    }

    match run(&cli).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}

async fn run(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        Some(Commands::Issue(cmd)) => {
            cli::commands::issue::run_issue(cmd, cli.json).await?;
        }
        Some(Commands::Team(cmd)) => {
            cli::commands::team::run_team(cmd, cli.json).await?;
        }
        None => {
            eprintln!("No command given. Run `linear --help` for usage.");
            process::exit(1);
        }
    }
    Ok(())
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "error",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let use_json = std::env::var("LINEAR_LOG_FORMAT").as_deref() == Ok("json");

    if use_json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::new(level))
            .init();
    } else {
        fmt().with_env_filter(EnvFilter::new(level)).init();
    }
}
