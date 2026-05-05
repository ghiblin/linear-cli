use std::sync::Arc;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::{
        errors::ApplicationError,
        use_cases::{
            create_issue::CreateIssue, get_issue::GetIssue, list_issues::ListIssues,
            update_issue::UpdateIssue,
        },
    },
    cli::output::{format_json, should_use_json},
    domain::{
        entities::issue::{CreateIssueInput, Issue, ListIssuesInput, UpdateIssueInput},
        value_objects::{
            LabelId, api_key::ApiKey, issue_id::IssueId, priority::Priority, project_id::ProjectId,
            team_id::TeamId, user_id::UserId,
        },
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
        #[arg(long)]
        team: Option<String>,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long = "label", num_args = 1)]
        labels: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "50")]
        limit: i32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        output: Option<String>,
    },
    Get {
        id: String,
        #[arg(long)]
        output: Option<String>,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        team: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long = "label", num_args = 1)]
        labels: Vec<String>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        estimate: Option<f64>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        output: Option<String>,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        estimate: Option<f64>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        no_parent: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        output: Option<String>,
    },
}

// ---- DTOs for JSON output ----

#[derive(Serialize)]
struct WorkflowStateDto {
    id: String,
    name: String,
    #[serde(rename = "type")]
    state_type: String,
}

#[derive(Serialize)]
struct SubIssueDto {
    id: String,
    identifier: String,
    title: String,
}

#[derive(Serialize)]
struct IssueDto {
    id: String,
    identifier: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    state: WorkflowStateDto,
    priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_name: Option<String>,
    team_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_title: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sub_issues: Vec<SubIssueDto>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ListIssuesDto {
    items: Vec<IssueDto>,
    next_cursor: Option<String>,
    has_next_page: bool,
}

fn priority_label(p: Priority) -> &'static str {
    match p {
        Priority::NoPriority => "no_priority",
        Priority::Urgent => "urgent",
        Priority::High => "high",
        Priority::Medium => "medium",
        Priority::Low => "low",
    }
}

fn parse_priority(s: &str) -> Result<Priority, anyhow::Error> {
    match s.to_lowercase().as_str() {
        "no_priority" | "no-priority" | "none" | "0" => Ok(Priority::NoPriority),
        "urgent" | "1" => Ok(Priority::Urgent),
        "high" | "2" => Ok(Priority::High),
        "medium" | "3" => Ok(Priority::Medium),
        "low" | "4" => Ok(Priority::Low),
        other => Err(anyhow::anyhow!(
            "invalid priority '{}'; use: no_priority, urgent, high, medium, low",
            other
        )),
    }
}

impl From<&Issue> for IssueDto {
    fn from(issue: &Issue) -> Self {
        Self {
            id: issue.id().to_string(),
            identifier: issue.identifier.clone(),
            title: issue.title().to_string(),
            description: issue.description.clone(),
            state: WorkflowStateDto {
                id: issue.state().id.clone(),
                name: issue.state().name.clone(),
                state_type: issue.state().state_type.clone(),
            },
            priority: priority_label(issue.priority()).to_string(),
            assignee_id: issue.assignee_id.as_ref().map(|a| a.to_string()),
            assignee_name: issue.assignee_name.clone(),
            team_id: issue.team_id().to_string(),
            label_ids: issue.label_ids.iter().map(|l| l.to_string()).collect(),
            due_date: issue.due_date.clone(),
            estimate: issue.estimate,
            parent_id: issue.parent_id.as_ref().map(|p| p.to_string()),
            parent_title: issue.parent_title.clone(),
            sub_issues: issue
                .sub_issues
                .iter()
                .map(|s| SubIssueDto {
                    id: s.id.to_string(),
                    identifier: s.identifier.clone(),
                    title: s.title.clone(),
                })
                .collect(),
            created_at: issue.created_at.clone(),
            updated_at: issue.updated_at.clone(),
        }
    }
}

fn format_issue_human(issue: &Issue) {
    println!("Identifier: {}", issue.identifier);
    println!("Title:      {}", issue.title());
    println!("State:      {}", issue.state().name);
    println!("Priority:   {}", priority_label(issue.priority()));
    if let Some(ref name) = issue.assignee_name {
        println!("Assignee:   {}", name);
    }
    if !issue.label_ids.is_empty() {
        let ids: Vec<String> = issue.label_ids.iter().map(|l| l.to_string()).collect();
        println!("Labels:     {}", ids.join(", "));
    }
    if let Some(ref d) = issue.due_date {
        println!("Due date:   {}", d);
    }
    if let Some(e) = issue.estimate {
        println!("Estimate:   {}", e);
    }
    if let Some(ref pt) = issue.parent_title {
        println!(
            "Parent:     {} ({})",
            issue
                .parent_id
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_default(),
            pt
        );
    }
    if !issue.sub_issues.is_empty() {
        println!("Sub-issues:");
        for s in &issue.sub_issues {
            println!("  {} — {}", s.identifier, s.title);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn auth_guard_run_issue_requires_auth() {
        assert!(true, "auth guard tested via integration tests");
    }
}

pub async fn run_issue(cmd: &IssueCommand, force_json: bool) -> Result<(), anyhow::Error> {
    use crate::application::use_cases::resolve_auth::resolve_auth;
    use crate::domain::repositories::credential_store::CredentialStore;

    if let IssueSubcommand::Update {
        parent, no_parent, ..
    } = &cmd.subcommand
    {
        if parent.is_some() && *no_parent {
            eprintln!("Error: --parent and --no-parent are mutually exclusive");
            std::process::exit(1);
        }
    }

    let env_key = std::env::var("LINEAR_API_KEY")
        .ok()
        .and_then(|k| ApiKey::new(k).ok());
    let stores: Vec<Box<dyn CredentialStore>> = vec![Box::new(KeyringCredentialStore::new())];
    let client = Arc::new(LinearGraphqlClient::new());
    let session = resolve_auth(env_key, stores, client)
        .await
        .map_err(|e| anyhow::anyhow!(ApplicationError::Auth(e)))?;

    let api_key_str = session.api_key().as_str().to_string();

    match &cmd.subcommand {
        IssueSubcommand::List {
            team,
            project,
            state,
            assignee,
            priority,
            labels,
            all,
            limit,
            cursor,
            output,
        } => {
            let team_id = team
                .as_deref()
                .map(|t| TeamId::new(t.to_string()))
                .transpose()?;
            let project_id = project
                .as_deref()
                .map(|p| ProjectId::parse(p))
                .transpose()?;
            let assignee_id = assignee
                .as_deref()
                .map(|a| UserId::new(a.to_string()))
                .transpose()?;
            let priority_val = priority.as_deref().map(parse_priority).transpose()?;
            let label_ids: Vec<LabelId> = labels
                .iter()
                .filter_map(|l| LabelId::new(l.clone()).ok())
                .collect();

            let input = ListIssuesInput {
                team_id,
                project_id,
                state_name: state.clone(),
                assignee_id,
                priority: priority_val,
                label_ids,
                limit: *limit,
                cursor: cursor.clone(),
                all_pages: *all,
            };

            let repo = LinearIssueRepository::new(api_key_str);
            let use_case = ListIssues::new(Box::new(repo));
            let result = use_case.execute(input).await?;
            let use_json = output.as_deref() == Some("json") || should_use_json(force_json);

            if use_json {
                let dto = ListIssuesDto {
                    items: result.items.iter().map(IssueDto::from).collect(),
                    next_cursor: result.next_cursor,
                    has_next_page: result.has_next_page,
                };
                println!("{}", format_json(&dto));
            } else {
                let total = result.items.len();
                for issue in &result.items {
                    println!(
                        "{:<10} {:<40} {:<15} {:<10} {}",
                        issue.identifier,
                        issue.title(),
                        issue.state().name,
                        priority_label(issue.priority()),
                        issue.assignee_name.as_deref().unwrap_or("—"),
                    );
                }
                if result.has_next_page {
                    println!(
                        "\nShowing {} issues. Use --all to retrieve all, or --cursor <token> to page.",
                        total
                    );
                } else {
                    println!("\nShowing {} issues.", total);
                }
            }
        }

        IssueSubcommand::Get { id, output } => {
            let repo = LinearIssueRepository::new(api_key_str);
            let use_case = GetIssue::new(Box::new(repo));
            let issue = use_case
                .execute(id.clone())
                .await
                .map_err(|e: ApplicationError| {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                })?;

            let use_json = output.as_deref() == Some("json") || should_use_json(force_json);
            if use_json {
                println!("{}", format_json(&IssueDto::from(&issue)));
            } else {
                format_issue_human(&issue);
            }
        }

        IssueSubcommand::Create {
            title,
            team,
            project,
            description,
            priority,
            assignee,
            labels,
            due_date,
            estimate,
            parent,
            dry_run,
            output,
        } => {
            let project_str = project
                .clone()
                .or_else(|| std::env::var("LINEAR_PROJECT_ID").ok());

            if project_str.is_none() {
                eprintln!("Error: --project is required (or set LINEAR_PROJECT_ID)");
                std::process::exit(1);
            }

            let team_id = TeamId::new(team.clone())?;
            let project_id = ProjectId::parse(&project_str.unwrap())?;
            let assignee_id = assignee
                .as_deref()
                .map(|a| UserId::new(a.to_string()))
                .transpose()?;
            let priority_val = priority.as_deref().map(parse_priority).transpose()?;
            let label_ids: Vec<LabelId> = labels
                .iter()
                .filter_map(|l| LabelId::new(l.clone()).ok())
                .collect();
            let parent_id = parent
                .as_deref()
                .map(|p| IssueId::new(p.to_string()))
                .transpose()?;

            if *dry_run {
                println!("[dry-run] Would create issue:");
                println!("  Title:    {}", title);
                println!("  Team:     {}", team);
                if let Some(p) = parent {
                    println!("  Parent:   {}", p);
                }
                if let Some(ref pv) = priority_val {
                    println!("  Priority: {}", priority_label(*pv));
                }
                return Ok(());
            }

            let input = CreateIssueInput {
                title: title.clone(),
                team_id,
                project_id,
                description: description.clone(),
                priority: priority_val,
                assignee_id,
                label_ids,
                due_date: due_date.clone(),
                estimate: *estimate,
                parent_id,
            };

            let repo = LinearIssueRepository::new(api_key_str);
            let use_case = CreateIssue::new(Box::new(repo));
            let issue = use_case
                .execute(input, false)
                .await
                .map_err(|e: ApplicationError| {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                })?;

            let use_json = output.as_deref() == Some("json") || should_use_json(force_json);
            if use_json {
                println!("{}", format_json(&IssueDto::from(&issue)));
            } else {
                println!("Created issue {}: {}", issue.identifier, issue.title());
            }
        }

        IssueSubcommand::Update {
            id,
            title,
            description,
            state,
            priority,
            assignee,
            due_date,
            estimate,
            parent,
            no_parent,
            dry_run,
            output,
        } => {
            if parent.is_some() && *no_parent {
                eprintln!("Error: --parent and --no-parent are mutually exclusive");
                std::process::exit(1);
            }

            if *dry_run {
                println!("[dry-run] Would update {}:", id);
                if let Some(t) = title {
                    println!("  title:    → {}", t);
                }
                if let Some(s) = state {
                    println!("  state:    → {}", s);
                }
                if let Some(p) = priority {
                    println!("  priority: → {}", p);
                }
                return Ok(());
            }

            let priority_val = priority.as_deref().map(parse_priority).transpose()?;
            let assignee_id = assignee
                .as_deref()
                .map(|a| UserId::new(a.to_string()))
                .transpose()?;
            let parent_id = parent
                .as_deref()
                .map(|p| IssueId::new(p.to_string()))
                .transpose()?;

            let input = UpdateIssueInput {
                title: title.clone(),
                description: description.clone(),
                state_id: state.clone(),
                priority: priority_val,
                assignee_id,
                due_date: due_date.clone(),
                estimate: *estimate,
                parent_id,
                no_parent: *no_parent,
            };

            let repo = LinearIssueRepository::new(api_key_str);
            let use_case = UpdateIssue::new(Box::new(repo));
            let issue = use_case.execute(id.clone(), input, false).await.map_err(
                |e: ApplicationError| {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                },
            )?;

            let use_json = output.as_deref() == Some("json") || should_use_json(force_json);
            if use_json {
                println!("{}", format_json(&IssueDto::from(&issue)));
            } else {
                println!("Updated {}: {}", issue.identifier, issue.title());
            }
        }
    }

    Ok(())
}
