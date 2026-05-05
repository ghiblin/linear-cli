use crate::infrastructure::graphql::schema::schema;
use cynic::MutationBuilder;

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

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "IssueUpdateInput")]
pub struct IssueUpdateInput {
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
pub struct IssueUpdateVariables {
    pub id: String,
    pub input: IssueUpdateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "IssueUpdateVariables")]
pub struct IssueUpdateMutation {
    #[arguments(id: $id, input: $input)]
    pub issue_update: IssuePayload,
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
    let op = IssueUpdateMutation::build(IssueUpdateVariables { id: id.to_string(), input });
    let resp: GraphqlResponse<IssueUpdateMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
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
