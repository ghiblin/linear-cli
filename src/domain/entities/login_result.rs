use crate::domain::entities::workspace::Workspace;

#[derive(Debug, Clone)]
pub struct LoginResult {
    user_id: String,
    user_name: String,
    workspace: Workspace,
}

impl LoginResult {
    pub fn new(
        user_id: impl Into<String>,
        user_name: impl Into<String>,
        workspace: Workspace,
    ) -> Self {
        LoginResult {
            user_id: user_id.into(),
            user_name: user_name.into(),
            workspace,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }
}
