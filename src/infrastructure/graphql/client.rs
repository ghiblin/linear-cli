use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::domain::{
    entities::workspace::Workspace, errors::AuthError,
    repositories::linear_api_client::LinearApiClient, value_objects::api_key::ApiKey,
};

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

pub struct LinearGraphqlClient {
    http: Client,
}

#[allow(dead_code)]
impl LinearGraphqlClient {
    pub fn new() -> Self {
        LinearGraphqlClient {
            http: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GraphqlRequest {
    query: &'static str,
}

#[derive(Deserialize)]
struct GraphqlResponse {
    data: Option<ViewerData>,
    errors: Option<Vec<GraphqlError>>,
}

#[derive(Deserialize)]
struct ViewerData {
    viewer: ViewerPayload,
}

#[derive(Deserialize)]
struct ViewerPayload {
    id: String,
    #[allow(dead_code)]
    name: String,
    organization: OrganizationPayload,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationPayload {
    name: String,
    url_key: String,
}

#[derive(Deserialize)]
struct GraphqlError {
    message: String,
    extensions: Option<GraphqlErrorExtensions>,
}

#[derive(Deserialize)]
struct GraphqlErrorExtensions {
    #[serde(rename = "type")]
    error_type: Option<String>,
}

const VIEWER_QUERY: &str =
    "query ValidateApiKey { viewer { id name organization { name urlKey } } }";

#[allow(dead_code)]
#[async_trait]
impl LinearApiClient for LinearGraphqlClient {
    #[instrument(skip(self, key), fields(redacted_key = %key))]
    async fn validate_api_key(&self, key: &ApiKey) -> Result<Workspace, AuthError> {
        let response = self
            .http
            .post(LINEAR_API_URL)
            .header("Authorization", key.as_str())
            .header("Content-Type", "application/json")
            .json(&GraphqlRequest {
                query: VIEWER_QUERY,
            })
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AuthError::InvalidKey);
        }

        let body: GraphqlResponse = response
            .json()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))?;

        if let Some(errors) = body.errors {
            if let Some(first) = errors.first() {
                let is_unauth = first
                    .extensions
                    .as_ref()
                    .and_then(|ext| ext.error_type.as_deref())
                    .map(|t| t == "UNAUTHENTICATED")
                    .unwrap_or(false);
                if is_unauth {
                    return Err(AuthError::InvalidKey);
                }
                return Err(AuthError::ValidationFailed(first.message.clone()));
            }
        }

        let viewer = body
            .data
            .ok_or_else(|| AuthError::ValidationFailed("empty response from Linear API".into()))?
            .viewer;

        Workspace::new(
            viewer.id,
            viewer.organization.name,
            viewer.organization.url_key,
        )
        .map_err(|e| AuthError::ValidationFailed(e.to_string()))
    }
}
