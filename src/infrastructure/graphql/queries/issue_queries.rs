use crate::infrastructure::graphql::schema::schema;
use cynic::QueryBuilder;

use crate::infrastructure::graphql::queries::project_queries::{
    GraphqlResponse, PageInfoNode, execute_with_retry, map_errors,
};

use crate::domain::{entities::issue::ListIssuesInput, errors::DomainError};

// ---- Filter input types ----

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IDComparator")]
pub struct IdComparatorInput {
    pub eq: Option<cynic::Id>,
    #[cynic(rename = "in")]
    pub in_list: Option<Vec<cynic::Id>>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueIDComparator")]
pub struct IssueIdComparatorInput {
    pub eq: Option<cynic::Id>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "StringComparator")]
pub struct StringComparatorInput {
    pub eq: Option<String>,
    #[cynic(rename = "eqIgnoreCase")]
    pub eq_ignore_case: Option<String>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "NullableNumberComparator")]
pub struct NullableNumberComparatorInput {
    pub eq: Option<f64>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "TeamFilter")]
pub struct TeamFilterInput {
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "NullableProjectFilter")]
pub struct ProjectFilterInput {
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "WorkflowStateFilter")]
pub struct StateFilterInput {
    pub name: Option<StringComparatorInput>,
    pub team: Option<TeamFilterInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "NullableUserFilter")]
pub struct AssigneeFilterInput {
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueLabelFilter")]
pub struct IssueLabelFilterInput {
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueLabelCollectionFilter")]
pub struct LabelCollectionFilterInput {
    pub some: Option<IssueLabelFilterInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueFilter")]
pub struct IssueFilterInput {
    pub team: Option<TeamFilterInput>,
    pub project: Option<ProjectFilterInput>,
    pub state: Option<StateFilterInput>,
    pub assignee: Option<AssigneeFilterInput>,
    pub priority: Option<NullableNumberComparatorInput>,
    pub labels: Option<LabelCollectionFilterInput>,
    pub id: Option<IssueIdComparatorInput>,
    /// Compound filters — all conditions must match (AND semantics).
    /// Used to require that an issue carries every label when multiple --label
    /// flags are supplied (FR-002: "issues must carry ALL specified labels").
    /// Box breaks the recursive type size cycle required by Rust.
    pub and: Option<Vec<Box<IssueFilterInput>>>,
}

// ---- Fragment types ----

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "WorkflowState")]
pub struct WorkflowStateNode {
    pub id: cynic::Id,
    pub name: String,
    #[cynic(rename = "type")]
    pub state_type: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "User")]
pub struct UserNode {
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueLabel")]
pub struct IssueLabelNode {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelConnection")]
pub struct IssueLabelConnection {
    pub nodes: Vec<IssueLabelNode>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Team")]
pub struct IssueTeamNode {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
pub struct SubIssueNode {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
pub struct SubIssueConnection {
    pub nodes: Vec<SubIssueNode>,
}

/// Lightweight fragment used in list queries (no sub-issues, no description)
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
pub struct IssueNode {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
    pub state: WorkflowStateNode,
    pub priority: f64,
    pub team: IssueTeamNode,
    pub assignee: Option<UserNode>,
    #[arguments(first: 50)]
    pub labels: IssueLabelConnection,
    pub created_at: String,
    pub updated_at: String,
}

/// Full fragment used in get/create/update queries
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
pub struct IssueDetailNode {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub state: WorkflowStateNode,
    pub priority: f64,
    pub team: IssueTeamNode,
    pub assignee: Option<UserNode>,
    #[arguments(first: 50)]
    pub labels: IssueLabelConnection,
    pub due_date: Option<String>,
    pub estimate: Option<f64>,
    pub parent: Option<IssueParentNode>,
    #[arguments(first: 50)]
    pub children: SubIssueConnection,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
pub struct IssueParentNode {
    pub id: cynic::Id,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
pub struct IssueConnection {
    pub nodes: Vec<IssueNode>,
    pub page_info: PageInfoNode,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
pub struct IssueDetailConnection {
    #[allow(dead_code)]
    pub nodes: Vec<IssueDetailNode>,
    #[allow(dead_code)]
    pub page_info: PageInfoNode,
}

// ---- List Issues query ----

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueListVariables {
    pub filter: Option<IssueFilterInput>,
    pub first: i32,
    pub after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "IssueListVariables")]
pub struct IssueListQuery {
    #[arguments(filter: $filter, first: $first, after: $after, orderBy: updatedAt)]
    pub issues: IssueConnection,
}

// ---- Get Issue by ID (UUID) query ----

#[derive(cynic::QueryVariables, Debug)]
pub struct GetIssueByIdVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetIssueByIdVariables")]
pub struct GetIssueByIdQuery {
    #[arguments(id: $id)]
    pub issue: IssueDetailNode,
}

// ---- Get Issue by display ID (identifier filter) query ----

#[derive(cynic::QueryVariables, Debug)]
pub struct GetIssueByIdentifierVariables {
    pub filter: IssueFilterInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetIssueByIdentifierVariables")]
pub struct GetIssueByIdentifierQuery {
    #[allow(dead_code)]
    #[arguments(filter: $filter, first: 1)]
    pub issues: IssueDetailConnection,
}

// ---- Workflow states query ----

#[derive(cynic::QueryVariables, Debug)]
pub struct WorkflowStatesVariables {
    pub filter: StateFilterInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "WorkflowState")]
pub struct WorkflowStateDetailNode {
    pub id: cynic::Id,
    pub name: String,
    #[cynic(rename = "type")]
    pub state_type: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "WorkflowStateConnection")]
pub struct WorkflowStateConnection {
    pub nodes: Vec<WorkflowStateDetailNode>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "WorkflowStatesVariables")]
pub struct WorkflowStatesQuery {
    #[arguments(filter: $filter, first: 250)]
    pub workflow_states: WorkflowStateConnection,
}

// ---- Fetch functions ----

fn build_issue_filter(input: &ListIssuesInput) -> Option<IssueFilterInput> {
    let team = input.team_id.as_ref().map(|t| TeamFilterInput {
        id: Some(IdComparatorInput {
            eq: Some(cynic::Id::new(t.to_string())),
            in_list: None,
        }),
    });
    let project = input.project_id.as_ref().map(|p| ProjectFilterInput {
        id: Some(IdComparatorInput {
            eq: Some(cynic::Id::new(p.to_string())),
            in_list: None,
        }),
    });
    let state = input.state_name.as_ref().map(|s| StateFilterInput {
        name: Some(StringComparatorInput {
            eq: Some(s.clone()),
            eq_ignore_case: None,
        }),
        team: None,
    });
    let assignee = input.assignee_id.as_ref().map(|a| AssigneeFilterInput {
        id: Some(IdComparatorInput {
            eq: Some(cynic::Id::new(a.to_string())),
            in_list: None,
        }),
    });
    let priority = input.priority.as_ref().map(|p| {
        let pv = *p as u8;
        NullableNumberComparatorInput {
            eq: Some(pv as f64),
        }
    });
    // Build per-label sub-filters. FR-002 requires AND semantics: an issue must
    // carry ALL specified labels. Linear's IssueFilter.and field chains filters
    // such that every element must match, giving us the required AND logic.
    // The first label goes into the top-level `labels` field; each additional
    // label is expressed as a separate IssueFilterInput placed in `and`.
    let mut label_iter = input.label_ids.iter();
    let labels = label_iter.next().map(|l| LabelCollectionFilterInput {
        some: Some(IssueLabelFilterInput {
            id: Some(IdComparatorInput {
                eq: Some(cynic::Id::new(l.to_string())),
                in_list: None,
            }),
        }),
    });
    let extra_label_filters: Vec<Box<IssueFilterInput>> = label_iter
        .map(|l| {
            Box::new(IssueFilterInput {
                labels: Some(LabelCollectionFilterInput {
                    some: Some(IssueLabelFilterInput {
                        id: Some(IdComparatorInput {
                            eq: Some(cynic::Id::new(l.to_string())),
                            in_list: None,
                        }),
                    }),
                }),
                team: None,
                project: None,
                state: None,
                assignee: None,
                priority: None,
                id: None,
                and: None,
            })
        })
        .collect();
    let and = if extra_label_filters.is_empty() {
        None
    } else {
        Some(extra_label_filters)
    };

    if team.is_none()
        && project.is_none()
        && state.is_none()
        && assignee.is_none()
        && priority.is_none()
        && labels.is_none()
    {
        return None;
    }

    Some(IssueFilterInput {
        team,
        project,
        state,
        assignee,
        priority,
        labels,
        id: None,
        and,
    })
}

pub async fn fetch_issues(
    client: &reqwest::Client,
    api_key: &str,
    input: &ListIssuesInput,
    after: Option<String>,
) -> Result<(Vec<IssueNode>, PageInfoNode), DomainError> {
    let filter = build_issue_filter(input);
    let op = IssueListQuery::build(IssueListVariables {
        filter,
        first: input.limit,
        after,
    });
    let resp: GraphqlResponse<IssueListQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let data = resp
        .data
        .ok_or_else(|| DomainError::InvalidInput("empty response from Linear API".to_string()))?;
    Ok((data.issues.nodes, data.issues.page_info))
}

pub async fn fetch_issue(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
    _is_display_id: bool,
) -> Result<IssueDetailNode, DomainError> {
    // The Linear `issue(id: String!)` root query accepts both UUIDs and
    // display identifiers (e.g. "ENG-123"), so a single path handles both.
    let op = GetIssueByIdQuery::build(GetIssueByIdVariables { id: id.to_string() });
    let resp: GraphqlResponse<GetIssueByIdQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| DomainError::InvalidInput("empty response from Linear API".to_string()))
        .map(|d| d.issue)
}

pub async fn fetch_workflow_states(
    client: &reqwest::Client,
    api_key: &str,
    team_id: &str,
) -> Result<Vec<WorkflowStateDetailNode>, DomainError> {
    let filter = StateFilterInput {
        name: None,
        team: Some(TeamFilterInput {
            id: Some(IdComparatorInput {
                eq: Some(cynic::Id::new(team_id.to_string())),
                in_list: None,
            }),
        }),
    };
    let op = WorkflowStatesQuery::build(WorkflowStatesVariables { filter });
    let resp: GraphqlResponse<WorkflowStatesQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| DomainError::InvalidInput("empty response from Linear API".to_string()))
        .map(|d| d.workflow_states.nodes)
}
