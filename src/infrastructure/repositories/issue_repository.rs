use async_trait::async_trait;
use reqwest::Client;
use tracing::instrument;

use crate::{
    domain::{
        entities::issue::{
            CreateIssueInput, Issue, ListIssuesInput, ListIssuesResult, SubIssueRef,
            UpdateIssueInput, WorkflowStateInfo,
        },
        errors::DomainError,
        repositories::issue_repository::IssueRepository,
        value_objects::{
            LabelId, WorkflowStateRef, issue_id::IssueId, priority::Priority,
            project_id::ProjectId, team_id::TeamId, user_id::UserId,
        },
    },
    infrastructure::graphql::{
        mutations::issue_mutations::{
            IssueCreateInput, IssueUpdateInput, create_issue, delete_issue, update_issue,
        },
        queries::{
            issue_queries::{
                IssueDetailNode, IssueNode, fetch_issue, fetch_issues, fetch_workflow_states,
            },
            project_queries::resolve_slug_to_uuid,
        },
    },
};

pub struct LinearIssueRepository {
    http: Client,
    api_key: String,
}

impl LinearIssueRepository {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
        }
    }
}

fn is_display_id(s: &str) -> bool {
    let mut chars = s.chars();
    let has_upper = chars
        .by_ref()
        .take_while(|c| c.is_ascii_uppercase() || *c == '-')
        .count()
        > 0;
    if !has_upper {
        return false;
    }
    // Simple check: must match [A-Z]+-\d+
    let parts: Vec<&str> = s.rsplitn(2, '-').collect();
    if parts.len() != 2 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_digit())
        && !parts[0].is_empty()
        && parts[1].chars().all(|c| c.is_ascii_uppercase())
        && !parts[1].is_empty()
}

fn node_to_issue_light(node: IssueNode) -> Result<Issue, DomainError> {
    let state = WorkflowStateRef {
        id: node.state.id.into_inner(),
        name: node.state.name,
        state_type: node.state.state_type,
    };
    let priority = Priority::try_from(node.priority as u8)?;
    let team_id = TeamId::new(node.team.id.into_inner())?;
    let id = IssueId::new(node.id.into_inner())?;
    let assignee_id = node
        .assignee
        .as_ref()
        .and_then(|a| UserId::new(a.id.clone().into_inner()).ok());
    let assignee_name = node.assignee.map(|a| a.name);
    let label_ids: Vec<LabelId> = node
        .labels
        .nodes
        .into_iter()
        .filter_map(|l| LabelId::new(l.id.into_inner()).ok())
        .collect();

    Issue::new(
        id,
        node.identifier,
        node.title,
        None,
        state,
        priority,
        team_id,
        assignee_id,
        assignee_name,
        label_ids,
        None,
        None,
        None,
        None,
        vec![],
        node.created_at,
        node.updated_at,
    )
}

fn node_to_issue_detail(node: IssueDetailNode) -> Result<Issue, DomainError> {
    let state = WorkflowStateRef {
        id: node.state.id.into_inner(),
        name: node.state.name,
        state_type: node.state.state_type,
    };
    let priority = Priority::try_from(node.priority as u8)?;
    let team_id = TeamId::new(node.team.id.into_inner())?;
    let id = IssueId::new(node.id.into_inner())?;
    let assignee_id = node
        .assignee
        .as_ref()
        .and_then(|a| UserId::new(a.id.clone().into_inner()).ok());
    let assignee_name = node.assignee.map(|a| a.name);
    let label_ids: Vec<LabelId> = node
        .labels
        .nodes
        .into_iter()
        .filter_map(|l| LabelId::new(l.id.into_inner()).ok())
        .collect();
    let parent_id = node
        .parent
        .as_ref()
        .and_then(|p| IssueId::new(p.id.clone().into_inner()).ok());
    let parent_title = node.parent.map(|p| p.title);
    let sub_issues: Vec<SubIssueRef> = node
        .children
        .nodes
        .into_iter()
        .filter_map(|c| {
            IssueId::new(c.id.into_inner()).ok().map(|id| SubIssueRef {
                id,
                title: c.title,
                identifier: c.identifier,
            })
        })
        .collect();

    Issue::new(
        id,
        node.identifier,
        node.title,
        node.description,
        state,
        priority,
        team_id,
        assignee_id,
        assignee_name,
        label_ids,
        node.due_date,
        node.estimate,
        parent_id,
        parent_title,
        sub_issues,
        node.created_at,
        node.updated_at,
    )
}

#[async_trait]
impl IssueRepository for LinearIssueRepository {
    #[instrument(skip(self))]
    async fn list(&self, input: ListIssuesInput) -> Result<ListIssuesResult, DomainError> {
        let input = match input.project_id {
            Some(ProjectId::Slug(ref slug)) => {
                let uuid = resolve_slug_to_uuid(&self.http, &self.api_key, slug).await?;
                ListIssuesInput {
                    project_id: Some(ProjectId::Uuid(uuid)),
                    ..input
                }
            }
            _ => input,
        };
        if input.all_pages {
            let mut all_items = Vec::new();
            let mut cursor: Option<String> = input.cursor.clone();
            // Use a large page size when fetching all pages; --limit controls per-page
            // size in single-page mode only.
            let paged_input = ListIssuesInput {
                limit: 250,
                ..input.clone()
            };
            loop {
                let (nodes, page_info) =
                    fetch_issues(&self.http, &self.api_key, &paged_input, cursor).await?;
                for node in nodes {
                    all_items.push(node_to_issue_light(node)?);
                }
                if !page_info.has_next_page {
                    return Ok(ListIssuesResult {
                        items: all_items,
                        next_cursor: None,
                        has_next_page: false,
                    });
                }
                match page_info.end_cursor {
                    Some(c) => cursor = Some(c),
                    None => {
                        return Ok(ListIssuesResult {
                            items: all_items,
                            next_cursor: None,
                            has_next_page: false,
                        });
                    }
                }
            }
        } else {
            let (nodes, page_info) =
                fetch_issues(&self.http, &self.api_key, &input, input.cursor.clone()).await?;
            let items = nodes
                .into_iter()
                .map(node_to_issue_light)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ListIssuesResult {
                items,
                next_cursor: page_info.end_cursor.clone(),
                has_next_page: page_info.has_next_page,
            })
        }
    }

    #[instrument(skip(self))]
    async fn get(&self, id: IssueId) -> Result<Issue, DomainError> {
        let id_str = id.as_str();
        let display = is_display_id(id_str);
        let node = fetch_issue(&self.http, &self.api_key, id_str, display).await?;
        node_to_issue_detail(node)
    }

    #[instrument(skip(self))]
    async fn create(&self, input: CreateIssueInput) -> Result<Issue, DomainError> {
        let project_id_str = match &input.project_id {
            ProjectId::Slug(slug) => resolve_slug_to_uuid(&self.http, &self.api_key, slug).await?,
            ProjectId::Uuid(uuid) => uuid.clone(),
        };
        let cynic_input = IssueCreateInput {
            title: input.title,
            team_id: input.team_id.to_string(),
            project_id: Some(project_id_str),
            description: input.description,
            priority: input.priority.map(|p| p as u8 as i32),
            assignee_id: input.assignee_id.map(|a| a.to_string()),
            label_ids: if input.label_ids.is_empty() {
                None
            } else {
                Some(input.label_ids.iter().map(|l| l.to_string()).collect())
            },
            due_date: input.due_date,
            estimate: input.estimate.map(|e| e as i32),
            parent_id: input.parent_id.map(|p| p.to_string()),
        };
        let node = create_issue(&self.http, &self.api_key, cynic_input).await?;
        node_to_issue_detail(node)
    }

    #[instrument(skip(self))]
    async fn update(&self, id: IssueId, input: UpdateIssueInput) -> Result<Issue, DomainError> {
        let parent_id = if input.no_parent {
            Some(serde_json::Value::Null) // explicit null detaches the parent
        } else {
            input
                .parent_id
                .map(|p| serde_json::Value::String(p.to_string()))
        };
        let cynic_input = IssueUpdateInput {
            title: input.title,
            description: input.description,
            state_id: input.state_id,
            priority: input.priority.map(|p| p as u8 as i32),
            assignee_id: input.assignee_id.map(|a| a.to_string()),
            due_date: input.due_date,
            estimate: input.estimate.map(|e| e as i32),
            parent_id,
        };
        let node = update_issue(&self.http, &self.api_key, id.as_str(), cynic_input).await?;
        node_to_issue_detail(node)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: IssueId) -> Result<(), DomainError> {
        delete_issue(&self.http, &self.api_key, id.as_str()).await
    }

    #[instrument(skip(self))]
    async fn list_workflow_states(
        &self,
        team_id: TeamId,
    ) -> Result<Vec<WorkflowStateInfo>, DomainError> {
        let nodes = fetch_workflow_states(&self.http, &self.api_key, team_id.as_str()).await?;
        Ok(nodes
            .into_iter()
            .map(|n| WorkflowStateInfo {
                id: n.id.into_inner(),
                name: n.name,
                state_type: n.state_type,
            })
            .collect())
    }
}
