use async_trait::async_trait;

use crate::domain::{entities::team::Team, errors::DomainError, value_objects::team_id::TeamId};

#[async_trait]
pub trait TeamRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Team>, DomainError>;
    #[allow(dead_code)]
    async fn get(&self, id: TeamId) -> Result<Team, DomainError>;
}
