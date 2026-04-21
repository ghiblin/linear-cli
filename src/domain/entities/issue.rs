use crate::domain::{
    errors::DomainError,
    value_objects::{
        issue_id::IssueId, priority::Priority, team_id::TeamId, workflow_state::WorkflowState,
    },
};

#[derive(Debug, Clone)]
pub struct Issue {
    id: IssueId,
    title: String,
    state: WorkflowState,
    priority: Priority,
    team_id: TeamId,
}

#[allow(dead_code)]
impl Issue {
    pub fn new(
        id: IssueId,
        title: String,
        state: WorkflowState,
        priority: Priority,
        team_id: TeamId,
    ) -> Result<Self, DomainError> {
        if title.is_empty() {
            return Err(DomainError::InvalidInput(
                "issue title cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            id,
            title,
            state,
            priority,
            team_id,
        })
    }

    pub fn id(&self) -> &IssueId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn state(&self) -> WorkflowState {
        self.state
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn team_id(&self) -> &TeamId {
        &self.team_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        errors::DomainError,
        value_objects::{
            issue_id::IssueId, priority::Priority, team_id::TeamId, workflow_state::WorkflowState,
        },
    };

    fn make_id() -> IssueId {
        IssueId::new("issue-1".to_string()).unwrap()
    }

    fn make_team_id() -> TeamId {
        TeamId::new("team-1".to_string()).unwrap()
    }

    #[test]
    fn rejects_empty_title() {
        let result = Issue::new(
            make_id(),
            "".to_string(),
            WorkflowState::Backlog,
            Priority::NoPriority,
            make_team_id(),
        );
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[test]
    fn accepts_valid_issue() {
        let issue = Issue::new(
            make_id(),
            "Implement login".to_string(),
            WorkflowState::Todo,
            Priority::High,
            make_team_id(),
        )
        .unwrap();
        assert_eq!(issue.title(), "Implement login");
    }
}
