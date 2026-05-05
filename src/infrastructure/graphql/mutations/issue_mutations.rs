use crate::infrastructure::graphql::schema::schema;
use cynic::MutationBuilder;
use serde::Serialize;

use crate::infrastructure::graphql::queries::project_queries::{
    GraphqlResponse, execute_with_retry, map_errors,
};
use crate::infrastructure::graphql::queries::issue_queries::IssueDetailNode;

// ---- Create ----

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueCreateInput")]
pub struct IssueCreateInput {
    pub title: String,
    pub team_id: String,
    pub project_id: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub assignee_id: Option<String>,
    pub label_ids: Option<Vec<String>>,
    pub due_date: Option<String>,
    pub estimate: Option<i32>,
    pub parent_id: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "IssuePayload")]
pub struct IssuePayload {
    pub success: bool,
    pub issue: Option<IssueDetailNode>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueCreateVariables {
    pub input: IssueCreateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "IssueCreateVariables")]
pub struct IssueCreateMutation {
    #[arguments(input: $input)]
    pub issue_create: IssuePayload,
}

// ---- Update ----

/// Cynic-derived struct used solely to generate the mutation query string.
/// `parent_id` is `Option<String>` to satisfy cynic's schema validation.
#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueUpdateInput")]
pub(crate) struct IssueUpdateInputCynic {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_id: Option<String>,
    pub priority: Option<i32>,
    pub assignee_id: Option<String>,
    pub due_date: Option<String>,
    pub estimate: Option<i32>,
    pub parent_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub(crate) struct IssueUpdateVariablesCynic {
    pub id: String,
    pub input: IssueUpdateInputCynic,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "IssueUpdateVariablesCynic")]
pub(crate) struct IssueUpdateMutationCynic {
    #[arguments(id: $id, input: $input)]
    pub issue_update: IssuePayload,
}

// Alias for the response deserialization type, which shares the same structure.
pub type IssueUpdateMutation = IssueUpdateMutationCynic;

/// Plain-serde input struct for the update mutation.
///
/// `parent_id` is `serde_json::Value` so the caller can pass `Value::Null` to
/// send an explicit `null` (detach parent) or `Value::String(id)` to set one.
/// All other optional fields use `#[serde(skip_serializing_if)]` so they are
/// omitted from the JSON when `None`, matching the behaviour of the cynic derive.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate: Option<i32>,
    /// `None`  → field omitted (no change to parent)
    /// `Some(Value::Null)` → sends `null` (detaches parent)
    /// `Some(Value::String(id))` → sets parent to the given id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct IssueUpdateVariables {
    pub id: String,
    pub input: IssueUpdateInput,
}

// ---- Fetch functions ----

pub async fn create_issue(
    client: &reqwest::Client,
    api_key: &str,
    input: IssueCreateInput,
) -> Result<IssueDetailNode, crate::domain::errors::DomainError> {
    let op = IssueCreateMutation::build(IssueCreateVariables { input });
    let resp: GraphqlResponse<IssueCreateMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let payload = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .issue_create;
    if !payload.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "issue create reported failure".to_string(),
        ));
    }
    payload.issue.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput(
            "issue not returned in create response".to_string(),
        )
    })
}

pub async fn update_issue(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
    input: IssueUpdateInput,
) -> Result<IssueDetailNode, crate::domain::errors::DomainError> {
    // Use the cynic-derived dummy struct only to obtain the validated query string.
    // The actual variables are serialised from `IssueUpdateVariables` (plain serde)
    // which supports sending an explicit `null` for `parentId`.
    let query_string = {
        let op = IssueUpdateMutationCynic::build(IssueUpdateVariablesCynic {
            id: String::new(),
            input: IssueUpdateInputCynic {
                title: None,
                description: None,
                state_id: None,
                priority: None,
                assignee_id: None,
                due_date: None,
                estimate: None,
                parent_id: None,
            },
        });
        op.query
    };
    let variables = IssueUpdateVariables { id: id.to_string(), input };
    let resp: GraphqlResponse<IssueUpdateMutation> =
        execute_with_retry(client, api_key, &query_string, variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let payload = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .issue_update;
    if !payload.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "issue update reported failure".to_string(),
        ));
    }
    payload.issue.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput(
            "issue not returned in update response".to_string(),
        )
    })
}
