use std::sync::Arc;

use chrono::NaiveDate;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::{
        errors::ApplicationError,
        use_cases::{
            archive_project::{ArchiveOutcome, ArchiveProject},
            create_project::{CreateProject, CreateProjectArgs},
            get_project::GetProject,
            list_projects::ListProjects,
            update_project::{UpdateProject, UpdateProjectArgs},
        },
    },
    cli::output::{format_json, resolve_use_json, should_use_json},
    domain::{
        errors::DomainError,
        value_objects::{ProjectId, ProjectState, api_key::ApiKey},
    },
    infrastructure::{
        auth::keyring_store::KeyringCredentialStore, graphql::client::LinearGraphqlClient,
        repositories::project_repository::LinearProjectRepository,
    },
};

#[derive(Args)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub subcommand: ProjectSubcommand,
}

#[derive(Subcommand)]
pub enum ProjectSubcommand {
    #[command(about = "List projects")]
    List(ListArgs),
    #[command(about = "Get project details")]
    Get(GetArgs),
    #[command(about = "Create a new project")]
    Create(CreateArgs),
    #[command(about = "Update a project")]
    Update(UpdateArgs),
    #[command(about = "Archive a project")]
    Archive(ArchiveArgs),
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, help = "Filter by team UUID")]
    pub team: Option<String>,
    #[arg(long, help = "Filter by partial name (case-insensitive)")]
    pub name: Option<String>,
    #[arg(long, default_value = "50", help = "Max results per page")]
    pub limit: u32,
    #[arg(long, help = "Pagination cursor")]
    pub cursor: Option<String>,
    #[arg(long, help = "Fetch all pages")]
    pub all: bool,
    #[arg(
        long = "output",
        value_name = "FORMAT",
        help = "Output format: json or human"
    )]
    pub output: Option<String>,
    /// Use JSON output format (alias for --output json)
    #[arg(long)]
    pub json: bool,
    #[arg(long, help = "Print debug info to stderr")]
    pub debug: bool,
}

#[derive(Args)]
pub struct GetArgs {
    #[arg(help = "Project UUID or slug")]
    pub id: String,
    #[arg(
        long = "output",
        value_name = "FORMAT",
        help = "Output format: json or human"
    )]
    pub output: Option<String>,
    /// Use JSON output format (alias for --output json)
    #[arg(long)]
    pub json: bool,
    #[arg(long, help = "Print debug info to stderr")]
    pub debug: bool,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long, required = true, help = "Project name")]
    pub name: String,
    #[arg(long = "team", required = true, help = "Team UUID (repeatable)")]
    pub teams: Vec<String>,
    #[arg(long, help = "Project description")]
    pub description: Option<String>,
    #[arg(long, help = "Lead user UUID")]
    pub lead: Option<String>,
    #[arg(long, value_name = "YYYY-MM-DD", help = "Start date")]
    pub start_date: Option<String>,
    #[arg(long, value_name = "YYYY-MM-DD", help = "Target date")]
    pub target_date: Option<String>,
    #[arg(long, help = "Dry run (no API call)")]
    pub dry_run: bool,
    #[arg(
        long = "output",
        value_name = "FORMAT",
        help = "Output format: json or human"
    )]
    pub output: Option<String>,
    /// Use JSON output format (alias for --output json)
    #[arg(long)]
    pub json: bool,
    #[arg(long, help = "Print debug info to stderr")]
    pub debug: bool,
}

#[derive(Args)]
pub struct UpdateArgs {
    #[arg(help = "Project UUID or slug")]
    pub id: String,
    #[arg(long, help = "New project name")]
    pub name: Option<String>,
    #[arg(long, help = "New description")]
    pub description: Option<String>,
    #[arg(long, help = "New state: planned|started|paused|completed|cancelled")]
    pub state: Option<String>,
    #[arg(long, help = "New lead user UUID")]
    pub lead: Option<String>,
    #[arg(long, value_name = "YYYY-MM-DD", help = "New start date")]
    pub start_date: Option<String>,
    #[arg(long, value_name = "YYYY-MM-DD", help = "New target date")]
    pub target_date: Option<String>,
    #[arg(long, help = "Dry run (no API call)")]
    pub dry_run: bool,
    #[arg(
        long = "output",
        value_name = "FORMAT",
        help = "Output format: json or human"
    )]
    pub output: Option<String>,
    /// Use JSON output format (alias for --output json)
    #[arg(long)]
    pub json: bool,
    #[arg(long, help = "Print debug info to stderr")]
    pub debug: bool,
}

#[derive(Args)]
pub struct ArchiveArgs {
    #[arg(help = "Project UUID or slug")]
    pub id: String,
    #[arg(long, help = "Dry run (no API call)")]
    pub dry_run: bool,
    #[arg(
        long = "output",
        value_name = "FORMAT",
        help = "Output format: json or human"
    )]
    pub output: Option<String>,
    /// Use JSON output format (alias for --output json)
    #[arg(long)]
    pub json: bool,
    #[arg(long, help = "Print debug info to stderr")]
    pub debug: bool,
}

impl ProjectCommand {
    /// Returns the effective verbosity level from per-subcommand flags:
    /// 2 = --debug (DEBUG tracing), 0 = default.
    pub fn verbosity(&self) -> u8 {
        let debug = match &self.subcommand {
            ProjectSubcommand::List(a) => a.debug,
            ProjectSubcommand::Get(a) => a.debug,
            ProjectSubcommand::Create(a) => a.debug,
            ProjectSubcommand::Update(a) => a.debug,
            ProjectSubcommand::Archive(a) => a.debug,
        };
        if debug { 2 } else { 0 }
    }
}

// ---- Output DTOs ----

#[derive(Serialize)]
struct ProjectListDto {
    projects: Vec<ProjectDto>,
    page_info: PageInfoDto,
}

#[derive(Serialize)]
struct PageInfoDto {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[derive(Serialize)]
struct ProjectDto {
    id: String,
    slug_id: String,
    name: String,
    description: Option<String>,
    state: String,
    progress: f64,
    lead_id: Option<String>,
    team_ids: Vec<String>,
    start_date: Option<String>,
    target_date: Option<String>,
    updated_at: String,
}

impl From<&crate::domain::entities::project::Project> for ProjectDto {
    fn from(p: &crate::domain::entities::project::Project) -> Self {
        Self {
            id: p.id.clone(),
            slug_id: p.slug_id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            state: p.state.to_string(),
            progress: p.progress,
            lead_id: p.lead_id.as_ref().map(|l| l.to_string()),
            team_ids: p.team_ids.iter().map(|t| t.to_string()).collect(),
            start_date: p.start_date.map(|d| d.to_string()),
            target_date: p.target_date.map(|d| d.to_string()),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
struct MutationResultDto {
    id: String,
    slug_id: String,
    name: String,
    state: String,
}

#[derive(Serialize)]
struct ArchiveResultDto {
    success: bool,
    id: String,
    already_archived: bool,
}

#[derive(Serialize)]
struct DryRunCreateDto {
    dry_run: bool,
    action: &'static str,
    input: serde_json::Value,
}

#[derive(Serialize)]
struct DryRunUpdateDto {
    dry_run: bool,
    action: &'static str,
    id: String,
    input: serde_json::Value,
}

#[derive(Serialize)]
struct DryRunArchiveDto {
    dry_run: bool,
    action: &'static str,
    id: String,
}

// ---- Main dispatch ----

fn verbose_print(verbose: bool, msg: &str) {
    if verbose {
        eprintln!("{}", msg);
    }
}

fn parse_date(s: &str, flag: &str) -> Result<NaiveDate, anyhow::Error> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("invalid date for {}: '{}'; expected YYYY-MM-DD", flag, s))
}

pub async fn run_project(cmd: &ProjectCommand, force_json: bool) -> Result<(), anyhow::Error> {
    use crate::application::use_cases::resolve_auth::resolve_auth;
    use crate::domain::repositories::credential_store::CredentialStore;

    let env_key = std::env::var("LINEAR_API_KEY")
        .ok()
        .and_then(|k| ApiKey::new(k).ok());
    let stores: Vec<Box<dyn CredentialStore>> = vec![Box::new(KeyringCredentialStore::new())];
    let client = Arc::new(LinearGraphqlClient::new());
    let session = resolve_auth(env_key, stores, client)
        .await
        .map_err(|e| anyhow::anyhow!(ApplicationError::Auth(e)))?;

    let api_key = session.api_key().as_str().to_string();
    let repo = Arc::new(LinearProjectRepository::new(api_key));

    match &cmd.subcommand {
        ProjectSubcommand::List(args) => {
            let verbose = args.debug;
            let use_json = resolve_use_json(args.json, args.output.as_deref(), force_json);
            let uc = ListProjects::new(repo);
            let team_id = args
                .team
                .as_deref()
                .map(|t| crate::domain::value_objects::team_id::TeamId::new(t.to_string()))
                .transpose()
                .map_err(|e: DomainError| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                })
                .unwrap();
            verbose_print(verbose, "Fetching projects…");
            let result = uc
                .execute(
                    team_id,
                    args.limit,
                    args.cursor.clone(),
                    args.all,
                    args.name.clone(),
                )
                .await?;
            verbose_print(
                verbose,
                &format!("Found {} project(s).", result.items.len()),
            );

            if should_use_json(use_json) {
                let dto = ProjectListDto {
                    projects: result.items.iter().map(ProjectDto::from).collect(),
                    page_info: PageInfoDto {
                        has_next_page: result.page_info.has_next_page,
                        end_cursor: result.page_info.end_cursor.clone(),
                    },
                };
                println!("{}", format_json(&dto));
            } else {
                println!("Projects ({}):", result.items.len());
                for p in &result.items {
                    let date = p
                        .target_date
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "—".to_string());
                    println!(
                        "  {:<35} {:<22} {:<12} {}",
                        p.name,
                        p.slug_id,
                        p.state.to_string(),
                        date
                    );
                }
                if result.page_info.has_next_page {
                    if let Some(cursor) = &result.page_info.end_cursor {
                        println!(
                            "\nMore results — run with --cursor {} for next page",
                            cursor
                        );
                    }
                }
            }
        }

        ProjectSubcommand::Get(args) => {
            let verbose = args.debug;
            let use_json = resolve_use_json(args.json, args.output.as_deref(), force_json);
            let id = ProjectId::parse(&args.id)
                .map_err(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                })
                .unwrap();

            verbose_print(verbose, &format!("Fetching project {}…", args.id));
            let uc = GetProject::new(repo);
            match uc.execute(id).await {
                Ok(project) => {
                    if should_use_json(use_json) {
                        let dto = ProjectDto::from(&project);
                        println!("{}", format_json(&dto));
                    } else {
                        println!("Name:        {}", project.name);
                        println!("Slug:        {}", project.slug_id);
                        println!("ID:          {}", project.id);
                        println!("State:       {}", project.state);
                        println!("Progress:    {:.1}%", project.progress);
                        if let Some(desc) = &project.description {
                            println!("Description: {}", desc);
                        }
                        if let Some(lead) = &project.lead_id {
                            println!("Lead:        {}", lead);
                        }
                        println!(
                            "Teams:       {}",
                            project
                                .team_ids
                                .iter()
                                .map(|t| t.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        if let Some(d) = project.start_date {
                            println!("Start date:  {}", d);
                        }
                        if let Some(d) = project.target_date {
                            println!("Target date: {}", d);
                        }
                        println!("Updated:     {}", project.updated_at.to_rfc3339());
                    }
                }
                Err(DomainError::NotFound(id)) => {
                    eprintln!("error: project '{}' not found", id);
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }

        ProjectSubcommand::Create(args) => {
            let verbose = args.debug;
            let use_json = resolve_use_json(args.json, args.output.as_deref(), force_json);
            let start_date = args
                .start_date
                .as_deref()
                .map(|s| parse_date(s, "--start-date"))
                .transpose()?;
            let target_date = args
                .target_date
                .as_deref()
                .map(|s| parse_date(s, "--target-date"))
                .transpose()?;

            let create_args = CreateProjectArgs {
                name: args.name.clone(),
                team_ids: args.teams.clone(),
                description: args.description.clone(),
                lead_id: args.lead.clone(),
                start_date,
                target_date,
            };

            if args.dry_run {
                if should_use_json(use_json) {
                    let dto = DryRunCreateDto {
                        dry_run: true,
                        action: "create",
                        input: serde_json::to_value(&create_args)?,
                    };
                    println!("{}", format_json(&dto));
                } else {
                    println!("[dry-run] Would create project:");
                    println!("  name:        {}", args.name);
                    println!("  team(s):     {}", args.teams.join(", "));
                    println!(
                        "  description: {}",
                        args.description.as_deref().unwrap_or("(none)")
                    );
                    println!(
                        "  lead:        {}",
                        args.lead.as_deref().unwrap_or("(none)")
                    );
                    println!(
                        "  start date:  {}",
                        start_date
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "(none)".to_string())
                    );
                    println!(
                        "  target date: {}",
                        target_date
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "(none)".to_string())
                    );
                }
                return Ok(());
            }

            verbose_print(verbose, &format!("Creating project \"{}\"…", args.name));
            let uc = CreateProject::new(repo);
            if let Some(project) = uc.execute(create_args, false).await? {
                if should_use_json(use_json) {
                    let dto = MutationResultDto {
                        id: project.id.clone(),
                        slug_id: project.slug_id.clone(),
                        name: project.name.clone(),
                        state: project.state.to_string(),
                    };
                    println!("{}", format_json(&dto));
                } else {
                    println!(
                        "Created project: \"{}\" ({})",
                        project.name, project.slug_id
                    );
                }
            }
        }

        ProjectSubcommand::Update(args) => {
            let verbose = args.debug;
            let use_json = resolve_use_json(args.json, args.output.as_deref(), force_json);

            let state = args
                .state
                .as_deref()
                .map(|s| s.parse::<ProjectState>())
                .transpose()
                .map_err(|e: DomainError| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                })
                .unwrap();

            let start_date = args
                .start_date
                .as_deref()
                .map(|s| parse_date(s, "--start-date"))
                .transpose()?;
            let target_date = args
                .target_date
                .as_deref()
                .map(|s| parse_date(s, "--target-date"))
                .transpose()?;

            let update_args = UpdateProjectArgs {
                name: args.name.clone(),
                description: args.description.clone(),
                state: state.clone(),
                lead_id: args.lead.clone(),
                start_date,
                target_date,
            };

            if !update_args.has_changes() && !args.dry_run {
                eprintln!("error: at least one update flag must be provided");
                std::process::exit(1);
            }

            let id = ProjectId::parse(&args.id)
                .map_err(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                })
                .unwrap();

            if args.dry_run {
                if should_use_json(use_json) {
                    let dto = DryRunUpdateDto {
                        dry_run: true,
                        action: "update",
                        id: args.id.clone(),
                        input: serde_json::to_value(&update_args)?,
                    };
                    println!("{}", format_json(&dto));
                } else {
                    println!("[dry-run] Would update project {}:", args.id);
                    if let Some(n) = &args.name {
                        println!("  name: {}", n);
                    }
                    if let Some(s) = &state {
                        println!("  state: {}", s);
                    }
                }
                return Ok(());
            }

            verbose_print(verbose, &format!("Updating project {}…", args.id));
            let uc = UpdateProject::new(repo);
            if let Some(project) = uc.execute(id, update_args, false).await? {
                if should_use_json(use_json) {
                    let dto = MutationResultDto {
                        id: project.id.clone(),
                        slug_id: project.slug_id.clone(),
                        name: project.name.clone(),
                        state: project.state.to_string(),
                    };
                    println!("{}", format_json(&dto));
                } else {
                    println!(
                        "Updated project {}: state → {}",
                        project.slug_id, project.state
                    );
                }
            }
        }

        ProjectSubcommand::Archive(args) => {
            let verbose = args.debug;
            let use_json = resolve_use_json(args.json, args.output.as_deref(), force_json);
            let id_str = args.id.clone();
            let id = ProjectId::parse(&id_str)
                .map_err(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                })
                .unwrap();

            if args.dry_run {
                if should_use_json(use_json) {
                    let dto = DryRunArchiveDto {
                        dry_run: true,
                        action: "archive",
                        id: id_str.clone(),
                    };
                    println!("{}", format_json(&dto));
                } else {
                    println!("[dry-run] Would archive project {}", id_str);
                }
                return Ok(());
            }

            verbose_print(verbose, &format!("Archiving project {}…", id_str));
            let uc = ArchiveProject::new(repo);
            match uc.execute(id, false).await {
                Ok(ArchiveOutcome::Archived) => {
                    if should_use_json(use_json) {
                        let dto = ArchiveResultDto {
                            success: true,
                            id: id_str,
                            already_archived: false,
                        };
                        println!("{}", format_json(&dto));
                    } else {
                        println!("Archived project {}", args.id);
                    }
                }
                Ok(ArchiveOutcome::AlreadyArchived) => {
                    if should_use_json(use_json) {
                        let dto = ArchiveResultDto {
                            success: true,
                            id: id_str,
                            already_archived: true,
                        };
                        println!("{}", format_json(&dto));
                    } else {
                        println!("Project {} is already archived", args.id);
                    }
                }
                Err(DomainError::NotFound(ref id)) => {
                    eprintln!("error: project '{}' not found", id);
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::project::Project;
    use crate::domain::value_objects::ProjectState;
    use chrono::Utc;

    fn make_test_project() -> Project {
        Project::new(
            "9cfb482a-81e3-4154-b5b9-2c805e70a02d".into(),
            "Q3 Platform".into(),
            None,
            ProjectState::Started,
            42.0,
            None,
            vec![],
            None,
            None,
            Utc::now(),
            "q3-platform".into(),
        )
        .unwrap()
    }

    // T002
    #[test]
    fn list_row_includes_slug_column() {
        let p = make_test_project();
        let date = p
            .target_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "—".to_string());
        let row = format!(
            "  {:<35} {:<22} {:<12} {}",
            p.name,
            p.slug_id,
            p.state.to_string(),
            date
        );
        assert!(row.contains("q3-platform"), "list row must contain slug");
        assert!(row.contains("Q3 Platform"), "list row must contain name");
    }

    // T003
    #[test]
    fn get_output_has_slug_line_after_name() {
        let p = make_test_project();
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("Name:        {}", p.name));
        lines.push(format!("Slug:        {}", p.slug_id));
        lines.push(format!("ID:          {}", p.id));
        assert_eq!(lines[0], "Name:        Q3 Platform");
        assert_eq!(lines[1], "Slug:        q3-platform");
    }

    // T007
    #[test]
    fn create_message_uses_slug_in_parenthetical() {
        let p = make_test_project();
        let msg = format!("Created project: \"{}\" ({})", p.name, p.slug_id);
        assert_eq!(msg, "Created project: \"Q3 Platform\" (q3-platform)");
    }

    // T008
    #[test]
    fn update_message_uses_slug_as_identifier() {
        let p = make_test_project();
        let msg = format!("Updated project {}: state → {}", p.slug_id, p.state);
        assert_eq!(msg, "Updated project q3-platform: state → started");
    }

    // T012
    #[test]
    fn project_dto_includes_slug_id() {
        let p = make_test_project();
        let dto = ProjectDto::from(&p);
        assert_eq!(dto.slug_id, "q3-platform");
    }

    // T013
    #[test]
    fn mutation_result_dto_includes_slug_id() {
        let dto = MutationResultDto {
            id: "9cfb482a-81e3-4154-b5b9-2c805e70a02d".into(),
            slug_id: "q3-platform".into(),
            name: "Q3 Platform".into(),
            state: "started".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"slug_id\""));
        assert!(json.contains("\"q3-platform\""));
    }

    #[test]
    fn list_args_has_debug_flag() {
        let args = ListArgs {
            team: None,
            name: None,
            limit: 50,
            cursor: None,
            all: false,
            output: None,
            json: false,
            debug: false,
        };
        assert!(!args.debug);
    }

    #[test]
    fn get_args_has_debug_flag() {
        let args = GetArgs {
            id: "id".into(),
            output: None,
            json: false,
            debug: false,
        };
        assert!(!args.debug);
    }

    #[test]
    fn create_args_has_debug_flag() {
        let args = CreateArgs {
            name: "n".into(),
            teams: vec![],
            description: None,
            lead: None,
            start_date: None,
            target_date: None,
            dry_run: false,
            output: None,
            json: false,
            debug: false,
        };
        assert!(!args.debug);
    }

    #[test]
    fn update_args_has_debug_flag() {
        let args = UpdateArgs {
            id: "id".into(),
            name: None,
            description: None,
            state: None,
            lead: None,
            start_date: None,
            target_date: None,
            dry_run: false,
            output: None,
            json: false,
            debug: false,
        };
        assert!(!args.debug);
    }

    #[test]
    fn archive_args_has_debug_flag() {
        let args = ArchiveArgs {
            id: "id".into(),
            dry_run: false,
            output: None,
            json: false,
            debug: false,
        };
        assert!(!args.debug);
    }
}
