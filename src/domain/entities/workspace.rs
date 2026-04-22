use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Workspace {
    id: String,
    name: String,
    url_key: String,
}

#[allow(dead_code)]
impl Workspace {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        url_key: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let id = id.into();
        let name = name.into();
        if id.is_empty() {
            return Err(DomainError::InvalidInput(
                "workspace id cannot be empty".into(),
            ));
        }
        if name.is_empty() {
            return Err(DomainError::InvalidInput(
                "workspace name cannot be empty".into(),
            ));
        }
        Ok(Workspace {
            id,
            name,
            url_key: url_key.into(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url_key(&self) -> &str {
        &self.url_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty_id() {
        let err = Workspace::new("", "Acme Corp", "acme").unwrap_err();
        assert!(err.to_string().contains("workspace id cannot be empty"));
    }

    #[test]
    fn new_rejects_empty_name() {
        let err = Workspace::new("org-123", "", "acme").unwrap_err();
        assert!(err.to_string().contains("workspace name cannot be empty"));
    }

    #[test]
    fn new_accepts_empty_url_key() {
        let ws = Workspace::new("org-123", "Acme Corp", "").unwrap();
        assert_eq!(ws.url_key(), "");
    }

    #[test]
    fn new_creates_workspace_with_correct_fields() {
        let ws = Workspace::new("org-123", "Acme Corp", "acme").unwrap();
        assert_eq!(ws.id(), "org-123");
        assert_eq!(ws.name(), "Acme Corp");
        assert_eq!(ws.url_key(), "acme");
    }

    #[test]
    fn clone_produces_equal_workspace() {
        let ws = Workspace::new("org-123", "Acme Corp", "acme").unwrap();
        assert_eq!(ws, ws.clone());
    }
}
