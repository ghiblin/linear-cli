use std::sync::Arc;

use tracing::instrument;

use crate::domain::{
    entities::project::Project,
    errors::DomainError,
    repositories::project_repository::{ListProjectsResult, ProjectRepository},
    value_objects::team_id::TeamId,
};

pub struct ListProjects {
    repo: Arc<dyn ProjectRepository>,
}

impl ListProjects {
    pub fn new(repo: Arc<dyn ProjectRepository>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        team_id: Option<TeamId>,
        first: u32,
        after: Option<String>,
        all: bool,
    ) -> Result<ListProjectsResult, DomainError> {
        if !all {
            return self.repo.list(team_id, first, after).await;
        }

        let mut items: Vec<Project> = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let page = self.repo.list(team_id.clone(), first, cursor).await?;
            items.extend(page.items);
            if !page.page_info.has_next_page {
                return Ok(ListProjectsResult {
                    items,
                    page_info: page.page_info,
                });
            }
            cursor = match page.page_info.end_cursor {
                Some(c) => Some(c),
                None => {
                    return Ok(ListProjectsResult {
                        items,
                        page_info: page.page_info,
                    });
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    use crate::domain::{
        entities::project::{CreateProjectInput, UpdateProjectInput},
        repositories::project_repository::PageInfo,
        value_objects::ProjectId,
    };

    mock! {
        TestRepo {}
        #[async_trait]
        impl ProjectRepository for TestRepo {
            async fn list(
                &self,
                team_id: Option<TeamId>,
                first: u32,
                after: Option<String>,
            ) -> Result<ListProjectsResult, DomainError>;
            async fn get(&self, id: ProjectId) -> Result<Project, DomainError>;
            async fn create(&self, input: CreateProjectInput) -> Result<Project, DomainError>;
            async fn update(
                &self,
                id: ProjectId,
                input: UpdateProjectInput,
            ) -> Result<Project, DomainError>;
            async fn archive(&self, id: ProjectId) -> Result<(), DomainError>;
        }
    }

    fn empty_result(has_next: bool, cursor: Option<&str>) -> ListProjectsResult {
        ListProjectsResult {
            items: vec![],
            page_info: PageInfo {
                has_next_page: has_next,
                end_cursor: cursor.map(String::from),
            },
        }
    }

    #[tokio::test]
    async fn single_page_returns_immediately() {
        let mut mock = MockTestRepo::new();
        mock.expect_list()
            .times(1)
            .returning(|_, _, _| Ok(empty_result(false, None)));

        let uc = ListProjects::new(Arc::new(mock));
        let result = uc.execute(None, 50, None, false).await.unwrap();
        assert!(!result.page_info.has_next_page);
    }

    #[tokio::test]
    async fn all_flag_collects_all_pages() {
        let mut mock = MockTestRepo::new();
        mock.expect_list()
            .times(1)
            .withf(|_, _, after| after.is_none())
            .returning(|_, _, _| Ok(empty_result(true, Some("cursor1"))));
        mock.expect_list()
            .times(1)
            .withf(|_, _, after| after.as_deref() == Some("cursor1"))
            .returning(|_, _, _| Ok(empty_result(false, None)));

        let uc = ListProjects::new(Arc::new(mock));
        let result = uc.execute(None, 50, None, true).await.unwrap();
        assert!(!result.page_info.has_next_page);
    }

    #[tokio::test]
    async fn empty_result_returns_ok() {
        let mut mock = MockTestRepo::new();
        mock.expect_list()
            .times(1)
            .returning(|_, _, _| Ok(empty_result(false, None)));

        let uc = ListProjects::new(Arc::new(mock));
        let result = uc.execute(None, 50, None, false).await.unwrap();
        assert!(result.items.is_empty());
    }
}
