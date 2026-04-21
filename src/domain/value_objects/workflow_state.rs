use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    Backlog,
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WorkflowState::Backlog => "Backlog",
            WorkflowState::Todo => "Todo",
            WorkflowState::InProgress => "InProgress",
            WorkflowState::Done => "Done",
            WorkflowState::Cancelled => "Cancelled",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_variants_exist() {
        let _ = WorkflowState::Backlog;
        let _ = WorkflowState::Todo;
        let _ = WorkflowState::InProgress;
        let _ = WorkflowState::Done;
        let _ = WorkflowState::Cancelled;
    }

    #[test]
    fn display_variants() {
        assert_eq!(WorkflowState::Backlog.to_string(), "Backlog");
        assert_eq!(WorkflowState::Todo.to_string(), "Todo");
        assert_eq!(WorkflowState::InProgress.to_string(), "InProgress");
        assert_eq!(WorkflowState::Done.to_string(), "Done");
        assert_eq!(WorkflowState::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn round_trip_serde() {
        let s = WorkflowState::InProgress;
        let json = serde_json::to_string(&s).unwrap();
        let back: WorkflowState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, WorkflowState::InProgress);
    }
}
