use async_trait::async_trait;

use crate::domain::{
    entities::team::Team, errors::DomainError, repositories::team_repository::TeamRepository,
    value_objects::team_id::TeamId,
};

pub struct LinearTeamRepository;

#[async_trait]
impl TeamRepository for LinearTeamRepository {
    async fn list(&self) -> Result<Vec<Team>, DomainError> {
        Ok(vec![])
    }

    async fn get(&self, _id: TeamId) -> Result<Team, DomainError> {
        Err(DomainError::NotImplemented)
    }
}
