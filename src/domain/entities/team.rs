use crate::domain::{errors::DomainError, value_objects::team_id::TeamId};

#[derive(Debug, Clone)]
pub struct Team {
    id: TeamId,
    name: String,
    key: String,
}

#[allow(dead_code)]
impl Team {
    pub fn new(id: TeamId, name: String, key: String) -> Result<Self, DomainError> {
        if key.is_empty() {
            return Err(DomainError::InvalidInput(
                "team key cannot be empty".to_string(),
            ));
        }
        let valid_key = key.len() <= 5 && key.chars().all(|c| c.is_ascii_uppercase());
        if !valid_key {
            return Err(DomainError::InvalidInput(
                "team key must be 1–5 uppercase ASCII letters".to_string(),
            ));
        }
        Ok(Self { id, name, key })
    }

    pub fn id(&self) -> &TeamId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{errors::DomainError, value_objects::team_id::TeamId};

    fn make_id() -> TeamId {
        TeamId::new("team-abc".to_string()).unwrap()
    }

    #[test]
    fn rejects_empty_key() {
        let result = Team::new(make_id(), "Engineering".to_string(), "".to_string());
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[test]
    fn rejects_invalid_key_format() {
        let result = Team::new(make_id(), "Engineering".to_string(), "toolong".to_string());
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[test]
    fn accepts_valid_team() {
        let team = Team::new(make_id(), "Engineering".to_string(), "ENG".to_string()).unwrap();
        assert_eq!(team.key(), "ENG");
        assert_eq!(team.name(), "Engineering");
    }
}
