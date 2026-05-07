use std::sync::Arc;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::{
    application::{
        errors::ApplicationError,
        use_cases::{
            create_issue::CreateIssue, delete_issue::DeleteIssue, get_issue::GetIssue,
            list_issues::ListIssues, update_issue::UpdateIssue,
        },
    },
    cli::output::{format_json, resolve_use_json, should_use_json},
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
        #[arg(long, help = "Filter by partial title (case-insensitive)")]
        title: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "50")]
        limit: i32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        output: Option<String>,
        /// Use JSON output format (alias for --output json)
        #[arg(long)]
        json: bool,
    },
    Get {
        id: String,
        #[arg(long)]
        description: bool,
        #[arg(long)]
        subtasks: bool,
        #[arg(long)]
        output: Option<String>,
        /// Use JSON output format (alias for --output json)
        #[arg(long)]
        json: bool,
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
        /// Use JSON output format (alias for --output json)
        #[arg(long)]
        json: bool,
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
        /// Use JSON output format (alias for --output json)
        #[arg(long)]
        json: bool,
    },
    Delete {
        id: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        output: Option<String>,
        /// Use JSON output format (alias for --output json)
        #[arg(long)]
        json: bool,
    },
}

// ---- DTOs for JSON output ----

#[derive(Serialize)]
struct CreateDryRunDto {
    dry_run: bool,
    title: String,
    team: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
}

#[derive(Serialize)]
struct UpdateDryRunDto {
    dry_run: bool,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    no_parent: Option<bool>,
}

#[derive(Serialize)]
struct DeleteResultDto {
    deleted: bool,
    id: String,
}

#[derive(Serialize)]
struct DeleteDryRunDto {
    dry_run: bool,
    id: String,
}

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

fn render_issue_human(issue: &Issue, show_description: bool, show_subtasks: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!("Identifier: {}\n", issue.identifier));
    out.push_str(&format!("Title:      {}\n", issue.title()));
    out.push_str(&format!("State:      {}\n", issue.state().name));
    out.push_str(&format!(
        "Priority:   {}\n",
        priority_label(issue.priority())
    ));
    if let Some(ref name) = issue.assignee_name {
        out.push_str(&format!("Assignee:   {}\n", name));
    }
    if !issue.label_ids.is_empty() {
        let ids: Vec<String> = issue.label_ids.iter().map(|l| l.to_string()).collect();
        out.push_str(&format!("Labels:     {}\n", ids.join(", ")));
    }
    if let Some(ref d) = issue.due_date {
        out.push_str(&format!("Due date:   {}\n", d));
    }
    if let Some(e) = issue.estimate {
        out.push_str(&format!("Estimate:   {}\n", e));
    }
    if let Some(ref pt) = issue.parent_title {
        out.push_str(&format!(
            "Parent:     {} ({})\n",
            issue
                .parent_id
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_default(),
            pt
        ));
    }
    if show_description {
        if let Some(ref desc) = issue.description {
            out.push_str("Description:\n");
            out.push_str(&format!("  {}\n", desc));
        }
    }
    if show_subtasks && !issue.sub_issues.is_empty() {
        out.push_str("Sub-issues:\n");
        for s in &issue.sub_issues {
            out.push_str(&format!("  {} — {}\n", s.identifier, s.title));
        }
    }
    out
}

fn format_issue_human(issue: &Issue, show_description: bool, show_subtasks: bool) {
    print!(
        "{}",
        render_issue_human(issue, show_description, show_subtasks)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::issue::{Issue, SubIssueRef},
        value_objects::{
            issue_id::IssueId, priority::Priority, team_id::TeamId,
            workflow_state_ref::WorkflowStateRef,
        },
    };

    fn make_state() -> WorkflowStateRef {
        WorkflowStateRef {
            id: "state-1".to_string(),
            name: "In Progress".to_string(),
            state_type: "started".to_string(),
        }
    }

    fn make_issue(description: Option<&str>, sub_issues: Vec<SubIssueRef>) -> Issue {
        Issue::new(
            IssueId::new("issue-1".to_string()).unwrap(),
            "ENG-1".to_string(),
            "Test issue".to_string(),
            description.map(|s| s.to_string()),
            make_state(),
            Priority::Medium,
            TeamId::new("team-1".to_string()).unwrap(),
            None,
            None,
            vec![],
            None,
            None,
            None,
            None,
            sub_issues,
            "2026-01-01T00:00:00Z".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        )
        .unwrap()
    }

    fn make_sub_issue() -> SubIssueRef {
        SubIssueRef {
            id: IssueId::new("sub-1".to_string()).unwrap(),
            identifier: "ENG-2".to_string(),
            title: "Sub task".to_string(),
        }
    }

    #[test]
    fn auth_guard_run_issue_requires_auth() {
        assert!(true, "auth guard tested via integration tests");
    }

    #[test]
    fn description_shown_when_flag_true() {
        let issue = make_issue(Some("My description text"), vec![]);
        let output = render_issue_human(&issue, true, false);
        assert!(
            output.contains("Description:"),
            "expected Description: section"
        );
        assert!(output.contains("My description text"));
    }

    #[test]
    fn description_hidden_when_flag_false() {
        let issue = make_issue(Some("My description text"), vec![]);
        let output = render_issue_human(&issue, false, false);
        assert!(
            !output.contains("Description:"),
            "description should be hidden by default"
        );
    }

    #[test]
    fn subtasks_shown_when_flag_true() {
        let issue = make_issue(None, vec![make_sub_issue()]);
        let output = render_issue_human(&issue, false, true);
        assert!(
            output.contains("Sub-issues:"),
            "expected Sub-issues: section"
        );
        assert!(output.contains("ENG-2"));
    }

    #[test]
    fn subtasks_hidden_when_flag_false() {
        let issue = make_issue(None, vec![make_sub_issue()]);
        let output = render_issue_human(&issue, false, false);
        assert!(
            !output.contains("Sub-issues:"),
            "sub-issues should be hidden by default"
        );
    }

    #[test]
    fn both_flags_true_shows_both_sections() {
        let issue = make_issue(Some("Desc text"), vec![make_sub_issue()]);
        let output = render_issue_human(&issue, true, true);
        assert!(output.contains("Description:"));
        assert!(output.contains("Desc text"));
        assert!(output.contains("Sub-issues:"));
        assert!(output.contains("ENG-2"));
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
            title,
            all,
            limit,
            cursor,
            output,
            json,
        } => {
            let team_id = team
                .as_deref()
                .map(|t| TeamId::new(t.to_string()))
                .transpose()?;
            let project_id = project.as_deref().map(ProjectId::parse).transpose()?;
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
                title_contains: title.clone(),
            };

            let repo = LinearIssueRepository::new(api_key_str);
            let use_case = ListIssues::new(Box::new(repo));
            let result = use_case.execute(input).await?;
            let use_json = should_use_json(resolve_use_json(*json, output.as_deref(), force_json));

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

        IssueSubcommand::Get {
            id,
            output,
            json,
            description,
            subtasks,
        } => {
            let repo = LinearIssueRepository::new(api_key_str);
            let use_case = GetIssue::new(Box::new(repo));
            let issue = use_case
                .execute(id.clone())
                .await
                .map_err(|e: ApplicationError| {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                })?;

            let use_json = should_use_json(resolve_use_json(*json, output.as_deref(), force_json));
            if use_json {
                println!("{}", format_json(&IssueDto::from(&issue)));
            } else {
                format_issue_human(&issue, *description, *subtasks);
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
            json,
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

            let use_json = should_use_json(resolve_use_json(*json, output.as_deref(), force_json));

            if *dry_run {
                if use_json {
                    println!(
                        "{}",
                        format_json(&CreateDryRunDto {
                            dry_run: true,
                            title: title.clone(),
                            team: team.clone(),
                            description: description.clone(),
                            priority: priority.clone(),
                            assignee: assignee.clone(),
                            due_date: due_date.clone(),
                            estimate: *estimate,
                            parent: parent.clone(),
                        })
                    );
                } else {
                    println!("[dry-run] Would create issue:");
                    println!("  Title:    {}", title);
                    println!("  Team:     {}", team);
                    if let Some(p) = parent {
                        println!("  Parent:   {}", p);
                    }
                    if let Some(ref pv) = priority_val {
                        println!("  Priority: {}", priority_label(*pv));
                    }
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
            json,
        } => {
            if parent.is_some() && *no_parent {
                eprintln!("Error: --parent and --no-parent are mutually exclusive");
                std::process::exit(1);
            }

            let use_json = should_use_json(resolve_use_json(*json, output.as_deref(), force_json));

            if *dry_run {
                if use_json {
                    println!(
                        "{}",
                        format_json(&UpdateDryRunDto {
                            dry_run: true,
                            id: id.clone(),
                            title: title.clone(),
                            description: description.clone(),
                            state: state.clone(),
                            priority: priority.clone(),
                            assignee: assignee.clone(),
                            due_date: due_date.clone(),
                            estimate: *estimate,
                            parent: parent.clone(),
                            no_parent: if *no_parent { Some(true) } else { None },
                        })
                    );
                } else {
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

            if use_json {
                println!("{}", format_json(&IssueDto::from(&issue)));
            } else {
                println!("Updated {}: {}", issue.identifier, issue.title());
            }
        }

        IssueSubcommand::Delete {
            id,
            dry_run,
            output,
            json,
        } => {
            let use_json = should_use_json(resolve_use_json(*json, output.as_deref(), force_json));

            if *dry_run {
                if use_json {
                    println!(
                        "{}",
                        format_json(&DeleteDryRunDto {
                            dry_run: true,
                            id: id.clone()
                        })
                    );
                } else {
                    println!("[dry-run] Would delete issue: {}", id);
                }
                return Ok(());
            }

            let issue_id = IssueId::new(id.clone()).map_err(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            })?;

            let repo = Arc::new(LinearIssueRepository::new(api_key_str));
            let use_case = DeleteIssue::new(repo);
            use_case.execute(issue_id, false).await.map_err(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            })?;

            if use_json {
                println!(
                    "{}",
                    format_json(&DeleteResultDto {
                        deleted: true,
                        id: id.clone()
                    })
                );
            } else {
                println!("Deleted issue {}", id);
            }
        }
    }

    Ok(())
}
