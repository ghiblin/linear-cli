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
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub eq: Option<cynic::Id>,
    #[cynic(rename = "in", skip_serializing_if = "Option::is_none")]
    pub in_list: Option<Vec<cynic::Id>>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueIDComparator")]
pub struct IssueIdComparatorInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub eq: Option<cynic::Id>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "StringComparator")]
pub struct StringComparatorInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub eq: Option<String>,
    #[cynic(rename = "eqIgnoreCase", skip_serializing_if = "Option::is_none")]
    pub eq_ignore_case: Option<String>,
    #[cynic(rename = "containsIgnoreCase", skip_serializing_if = "Option::is_none")]
    pub contains_ignore_case: Option<String>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "NullableNumberComparator")]
pub struct NullableNumberComparatorInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub eq: Option<f64>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "TeamFilter")]
pub struct TeamFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "NullableProjectFilter")]
pub struct ProjectFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "WorkflowStateFilter")]
pub struct StateFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub name: Option<StringComparatorInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub team: Option<TeamFilterInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "NullableUserFilter")]
pub struct AssigneeFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueLabelFilter")]
pub struct IssueLabelFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdComparatorInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueLabelCollectionFilter")]
pub struct LabelCollectionFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub some: Option<IssueLabelFilterInput>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueFilter")]
pub struct IssueFilterInput {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub team: Option<TeamFilterInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectFilterInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub state: Option<StateFilterInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<AssigneeFilterInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub priority: Option<NullableNumberComparatorInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub labels: Option<LabelCollectionFilterInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub id: Option<IssueIdComparatorInput>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub title: Option<StringComparatorInput>,
    /// Compound filters — all conditions must match (AND semantics).
    /// Used to require that an issue carries every label when multiple --label
    /// flags are supplied (FR-002: "issues must carry ALL specified labels").
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub and: Option<Vec<IssueFilterInput>>,
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
            contains_ignore_case: None,
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
    let title = input
        .title_contains
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| StringComparatorInput {
            eq: None,
            eq_ignore_case: None,
            contains_ignore_case: Some(s.to_string()),
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
    let extra_label_filters: Vec<IssueFilterInput> = label_iter
        .map(|l| IssueFilterInput {
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
            title: None,
            and: None,
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
        && title.is_none()
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
        title,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{entities::issue::ListIssuesInput, value_objects::team_id::TeamId};

    fn default_list_input() -> ListIssuesInput {
        ListIssuesInput {
            team_id: None,
            project_id: None,
            state_name: None,
            assignee_id: None,
            priority: None,
            label_ids: vec![],
            limit: 50,
            cursor: None,
            all_pages: false,
            title_contains: None,
        }
    }

    // T001
    #[test]
    fn title_contains_is_threaded_to_filter() {
        let input = ListIssuesInput {
            title_contains: Some("login".to_string()),
            ..default_list_input()
        };
        let filter = build_issue_filter(&input).expect("expected a filter");
        let title = filter.title.expect("expected title field in filter");
        assert_eq!(title.contains_ignore_case, Some("login".to_string()));
    }

    // T002
    #[test]
    fn empty_title_is_treated_as_no_filter() {
        let with_empty = ListIssuesInput {
            title_contains: Some("".to_string()),
            ..default_list_input()
        };
        let with_none = ListIssuesInput {
            title_contains: None,
            ..default_list_input()
        };
        assert!(build_issue_filter(&with_empty).is_none());
        assert!(build_issue_filter(&with_none).is_none());
    }

    // T016
    #[test]
    fn title_and_state_filter_compose() {
        let input = ListIssuesInput {
            title_contains: Some("auth".to_string()),
            state_name: Some("in_progress".to_string()),
            ..default_list_input()
        };
        let filter = build_issue_filter(&input).expect("expected a filter");
        assert!(filter.title.is_some(), "title filter should be set");
        assert!(filter.state.is_some(), "state filter should be set");
    }

    // T017
    #[test]
    fn title_and_team_filter_compose() {
        let input = ListIssuesInput {
            title_contains: Some("deploy".to_string()),
            team_id: Some(TeamId::new("team-1".to_string()).unwrap()),
            ..default_list_input()
        };
        let filter = build_issue_filter(&input).expect("expected a filter");
        assert!(filter.title.is_some(), "title filter should be set");
        assert!(filter.team.is_some(), "team filter should be set");
    }
}
