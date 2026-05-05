use serde::{Deserialize, Serialize};

use crate::domain::{
    errors::DomainError,
    value_objects::{
        issue_id::IssueId, label_id::LabelId, priority::Priority, project_id::ProjectId,
        team_id::TeamId, user_id::UserId, workflow_state_ref::WorkflowStateRef,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubIssueRef {
    pub id: IssueId,
    pub title: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStateInfo {
    pub id: String,
    pub name: String,
    pub state_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    id: IssueId,
    pub identifier: String,
    title: String,
    pub description: Option<String>,
    state: WorkflowStateRef,
    priority: Priority,
    team_id: TeamId,
    pub assignee_id: Option<UserId>,
    pub assignee_name: Option<String>,
    pub label_ids: Vec<LabelId>,
    pub due_date: Option<String>,
    pub estimate: Option<f64>,
    pub parent_id: Option<IssueId>,
    pub parent_title: Option<String>,
    pub sub_issues: Vec<SubIssueRef>,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(clippy::too_many_arguments)]
impl Issue {
    pub fn new(
        id: IssueId,
        identifier: String,
        title: String,
        description: Option<String>,
        state: WorkflowStateRef,
        priority: Priority,
        team_id: TeamId,
        assignee_id: Option<UserId>,
        assignee_name: Option<String>,
        label_ids: Vec<LabelId>,
        due_date: Option<String>,
        estimate: Option<f64>,
        parent_id: Option<IssueId>,
        parent_title: Option<String>,
        sub_issues: Vec<SubIssueRef>,
        created_at: String,
        updated_at: String,
    ) -> Result<Self, DomainError> {
        if title.is_empty() {
            return Err(DomainError::InvalidInput(
                "issue title cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            id,
            identifier,
            title,
            description,
            state,
            priority,
            team_id,
            assignee_id,
            assignee_name,
            label_ids,
            due_date,
            estimate,
            parent_id,
            parent_title,
            sub_issues,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> &IssueId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn state(&self) -> &WorkflowStateRef {
        &self.state
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn team_id(&self) -> &TeamId {
        &self.team_id
    }
}

// ---- Input structs ----

#[derive(Debug, Clone)]
pub struct ListIssuesInput {
    pub team_id: Option<TeamId>,
    pub project_id: Option<ProjectId>,
    pub state_name: Option<String>,
    pub assignee_id: Option<UserId>,
    pub priority: Option<Priority>,
    pub label_ids: Vec<LabelId>,
    pub limit: i32,
    pub cursor: Option<String>,
    pub all_pages: bool,
}

#[derive(Debug, Clone)]
pub struct ListIssuesResult {
    pub items: Vec<Issue>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(Debug, Clone)]
pub struct CreateIssueInput {
    pub title: String,
    pub team_id: TeamId,
    pub project_id: ProjectId,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub assignee_id: Option<UserId>,
    pub label_ids: Vec<LabelId>,
    pub due_date: Option<String>,
    pub estimate: Option<f64>,
    pub parent_id: Option<IssueId>,
}

#[derive(Debug, Clone)]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_id: Option<String>,
    pub priority: Option<Priority>,
    pub assignee_id: Option<UserId>,
    pub due_date: Option<String>,
    pub estimate: Option<f64>,
    pub parent_id: Option<IssueId>,
    pub no_parent: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        issue_id::IssueId, priority::Priority, team_id::TeamId,
        workflow_state_ref::WorkflowStateRef,
    };

    fn make_id() -> IssueId {
        IssueId::new("issue-1".to_string()).unwrap()
    }

    fn make_team_id() -> TeamId {
        TeamId::new("team-1".to_string()).unwrap()
    }

    fn make_state() -> WorkflowStateRef {
        WorkflowStateRef {
            id: "state-1".to_string(),
            name: "Backlog".to_string(),
            state_type: "backlog".to_string(),
        }
    }

    fn make_issue(title: &str) -> Result<Issue, DomainError> {
        Issue::new(
            make_id(),
            "ENG-1".to_string(),
            title.to_string(),
            None,
            make_state(),
            Priority::NoPriority,
            make_team_id(),
            None,
            None,
            vec![],
            None,
            None,
            None,
            None,
            vec![],
            "2026-01-01T00:00:00Z".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        )
    }

    #[test]
    fn rejects_empty_title() {
        let result = make_issue("");
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[test]
    fn accepts_valid_issue() {
        let issue = make_issue("Implement login").unwrap();
        assert_eq!(issue.title(), "Implement login");
    }
}
