# Cynic QueryFragment/InputObject Migration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace raw serde + const query-string GraphQL with cynic QueryFragment/QueryVariables/InputObject types so `cargo build` exercises compile-time schema validation.

**Architecture:** `execute_with_retry` stays unchanged; callers build a `cynic::Operation` via `QueryType::build(&vars)`, destructure `.query` and `.variables`, then call `execute_with_retry(client, api_key, &query, variables)`. The HTTP layer is untouched. GraphQL response wrapper types (`GraphqlResponse`, `GraphqlError`) stay as plain serde types.

**Tech Stack:** Rust · cynic v3 · cynic-codegen v3 (rkyv feature) · reqwest · serde

---

## Files to modify

| File | Change |
|---|---|
| `src/infrastructure/graphql/schema.rs` | Add `impl_scalar!` for custom scalars (`ID`, `DateTime`, `TimelessDate`) |
| `src/infrastructure/graphql/queries/project_queries.rs` | Replace serde structs + const strings with cynic derives |
| `src/infrastructure/graphql/mutations/project_mutations.rs` | Replace serde input/response structs with cynic derives |

---

## Task 1: Add scalar mappings to schema.rs

The cynic-registered schema does not know how to map custom scalars to Rust types until we declare them. `ID`, `DateTime`, and `TimelessDate` all need explicit `impl_scalar!` registrations; `Float` and `String` are built-in.

**Files:**
- Modify: `src/infrastructure/graphql/schema.rs`

- [ ] **Step 1: Replace schema.rs contents**

```rust
#[cynic::schema("linear")]
pub mod schema {}

cynic::impl_scalar!(String, schema::DateTime);
cynic::impl_scalar!(String, schema::TimelessDate);
cynic::impl_scalar!(String, schema::ID);
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | head -30
```

Expected: No errors (or only pre-existing warnings).

- [ ] **Step 3: Commit**

```bash
git add src/infrastructure/graphql/schema.rs
git commit -m "feat(graphql): register scalar mappings for DateTime, TimelessDate, ID"
```

---

## Task 2: Migrate shared data types to cynic QueryFragment

Convert the response-side node types. These are used by both queries and mutations. Keep `GraphqlResponse<T>`, `GraphqlError`, `GraphqlErrorExtension`, `execute_with_retry`, and `try_execute` exactly as-is — only replace the domain node structs.

**Files:**
- Modify: `src/infrastructure/graphql/queries/project_queries.rs`

- [ ] **Step 1: Replace the node types**

Delete `LeadNode`, `TeamNode`, `TeamConnection`, `ProjectConnection`, `PageInfoNode`, and `ProjectNode` (lines 70–106) and replace with cynic-derived versions:

```rust
use cynic::QueryBuilder;

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "User")]
pub struct LeadNode {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Team")]
pub struct TeamNode {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "TeamConnection")]
pub struct TeamConnection {
    pub nodes: Vec<TeamNode>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "PageInfo")]
pub struct PageInfoNode {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Project")]
pub struct ProjectNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slug_id: String,
    pub progress: f64,
    pub state: String,
    pub lead: Option<LeadNode>,
    #[arguments(first = 50)]
    pub teams: TeamConnection,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub updated_at: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectConnection")]
pub struct ProjectConnection {
    pub nodes: Vec<ProjectNode>,
    pub page_info: PageInfoNode,
}
```

Also remove the old `#[serde(rename_all = "camelCase")]` attributes from the old structs (all replaced above).

- [ ] **Step 2: Remove the old `ProjectsVariables` serde struct** (lines 41–46)

It will be replaced in Task 3.

- [ ] **Step 3: Remove `ProjectsData` and `TeamProjectsData` structs** (lines 49–61)

They will be replaced by root QueryFragment types in Task 3.

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1 | head -50
```

Expected: Compiler confirms cynic field names match the schema. If it reports a field mismatch (e.g. `field 'state' is deprecated`), that is expected — add `#[allow(deprecated)]` to the struct or accept the warning.

- [ ] **Step 5: Commit**

```bash
git add src/infrastructure/graphql/queries/project_queries.rs
git commit -m "feat(graphql): migrate ProjectNode and related types to cynic QueryFragment"
```

---

## Task 3: Migrate list-projects queries

Replace `PROJECTS_QUERY`, `TEAM_PROJECTS_QUERY`, and their variable + root types.

**Files:**
- Modify: `src/infrastructure/graphql/queries/project_queries.rs`

- [ ] **Step 1: Add ordering enum and list-query types**

Add after the node types:

```rust
#[derive(cynic::Enum, Debug, Clone, Copy)]
#[cynic(graphql_type = "PaginationOrderBy")]
pub enum PaginationOrderBy {
    UpdatedAt,
    CreatedAt,
}

// ---- List Projects ----

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectsVariables {
    pub first: i32,
    pub after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ProjectsVariables")]
pub struct ProjectsQuery {
    #[arguments(first = $first, after = $after, order_by = PaginationOrderBy::UpdatedAt)]
    pub projects: ProjectConnection,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct TeamProjectsVariables {
    pub team_id: String,
    pub first: i32,
    pub after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TeamProjectsVariables")]
pub struct TeamProjectsQuery {
    #[arguments(id = $team_id)]
    pub team: TeamWithProjects,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Team", variables = "TeamProjectsVariables")]
pub struct TeamWithProjects {
    #[arguments(first = $first, after = $after, order_by = PaginationOrderBy::UpdatedAt)]
    pub projects: ProjectConnection,
}
```

- [ ] **Step 2: Delete `PROJECTS_QUERY` and `TEAM_PROJECTS_QUERY` constants**

- [ ] **Step 3: Rewrite `fetch_projects` to use cynic build**

```rust
pub async fn fetch_projects(
    client: &reqwest::Client,
    api_key: &str,
    first: i32,
    after: Option<String>,
    team_id: Option<&str>,
) -> Result<(Vec<ProjectNode>, PageInfoNode), crate::domain::errors::DomainError> {
    if let Some(tid) = team_id {
        let op = TeamProjectsQuery::build(TeamProjectsVariables {
            team_id: tid.to_string(),
            first,
            after,
        });
        let resp: GraphqlResponse<TeamProjectsQuery> =
            execute_with_retry(client, api_key, &op.query, op.variables).await?;
        if let Some(errors) = resp.errors {
            return Err(map_errors(errors));
        }
        let data = resp.data.ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "empty response from Linear API".to_string(),
            )
        })?;
        Ok((data.team.projects.nodes, data.team.projects.page_info))
    } else {
        let op = ProjectsQuery::build(ProjectsVariables { first, after });
        let resp: GraphqlResponse<ProjectsQuery> =
            execute_with_retry(client, api_key, &op.query, op.variables).await?;
        if let Some(errors) = resp.errors {
            return Err(map_errors(errors));
        }
        let data = resp.data.ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(
                "empty response from Linear API".to_string(),
            )
        })?;
        Ok((data.projects.nodes, data.projects.page_info))
    }
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1 | head -50
```

- [ ] **Step 5: Commit**

```bash
git add src/infrastructure/graphql/queries/project_queries.rs
git commit -m "feat(graphql): migrate list-projects queries to cynic QueryFragment/QueryVariables"
```

---

## Task 4: Migrate get-project and slug-lookup queries

**Files:**
- Modify: `src/infrastructure/graphql/queries/project_queries.rs`

- [ ] **Step 1: Add get-project and slug-lookup types**

Delete `GetProjectData`, `SlugLookupData`, `SlugProjectConnection`, `SlugProjectNode` structs and the `GET_PROJECT_QUERY`, `SLUG_LOOKUP_QUERY` constants. Replace with:

```rust
// ---- Get Project ----

#[derive(cynic::QueryVariables, Debug)]
pub struct GetProjectVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProjectVariables")]
pub struct GetProjectQuery {
    #[arguments(id = $id)]
    pub project: Option<ProjectNode>,
}

// ---- Slug Lookup ----

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Project")]
pub struct SlugProjectNode {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectConnection")]
pub struct SlugProjectConnection {
    pub nodes: Vec<SlugProjectNode>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SlugLookupVariables {
    pub slug_id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SlugLookupVariables")]
pub struct SlugLookupQuery {
    #[arguments(filter = { slug_id: { eq: $slug_id } }, first = 1)]
    pub projects: SlugProjectConnection,
}
```

> **Note:** The `filter` argument syntax for `SlugLookupQuery` uses cynic's inline object syntax. If the compiler rejects it, use a `ProjectFilter` input object instead — define `#[derive(cynic::InputObject)] struct ProjectFilter { slug_id: Option<SlugIdFilter> }` and `struct SlugIdFilter { eq: Option<String> }`, then pass `filter = $filter` as a variable.

- [ ] **Step 2: Rewrite `fetch_project_by_id` and `resolve_slug_to_uuid`**

```rust
pub async fn fetch_project_by_id(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    let op = GetProjectQuery::build(GetProjectVariables { id: id.to_string() });
    let resp: GraphqlResponse<GetProjectQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project
        .ok_or_else(|| crate::domain::errors::DomainError::NotFound(id.to_string()))
}

pub async fn resolve_slug_to_uuid(
    client: &reqwest::Client,
    api_key: &str,
    slug: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let op = SlugLookupQuery::build(SlugLookupVariables { slug_id: slug.to_string() });
    let resp: GraphqlResponse<SlugLookupQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    resp.data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .projects
        .nodes
        .into_iter()
        .next()
        .map(|n| n.id)
        .ok_or_else(|| crate::domain::errors::DomainError::NotFound(slug.to_string()))
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1 | head -50
```

- [ ] **Step 4: Commit**

```bash
git add src/infrastructure/graphql/queries/project_queries.rs
git commit -m "feat(graphql): migrate get-project and slug-lookup queries to cynic"
```

---

## Task 5: Migrate org-status query

**Files:**
- Modify: `src/infrastructure/graphql/queries/project_queries.rs`

- [ ] **Step 1: Add status types and query**

Delete `OrgStatusData`, `OrgWithStatuses`, `ProjectStatusNode` structs and `ORG_PROJECT_STATUSES_QUERY` constant. Replace with:

```rust
// ---- Workspace project statuses ----

#[derive(cynic::Enum, Debug, PartialEq, Clone, Copy)]
#[cynic(graphql_type = "ProjectStatusType")]
pub enum ProjectStatusType {
    Backlog,
    Canceled,
    Completed,
    Paused,
    Planned,
    Started,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectStatus")]
pub struct ProjectStatusNode {
    pub id: String,
    #[cynic(rename = "type")]
    pub status_type: ProjectStatusType,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Organization")]
pub struct OrgWithStatuses {
    pub project_statuses: Vec<ProjectStatusNode>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct OrgStatusQuery {
    pub organization: OrgWithStatuses,
}
```

- [ ] **Step 2: Rewrite `fetch_status_id_for_type`**

```rust
pub async fn fetch_status_id_for_type(
    client: &reqwest::Client,
    api_key: &str,
    state_type: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let target = match state_type {
        "cancelled" | "canceled" => ProjectStatusType::Canceled,
        "backlog" => ProjectStatusType::Backlog,
        "completed" => ProjectStatusType::Completed,
        "paused" => ProjectStatusType::Paused,
        "planned" => ProjectStatusType::Planned,
        "started" => ProjectStatusType::Started,
        other => {
            return Err(crate::domain::errors::DomainError::InvalidInput(format!(
                "unknown project status type: '{}'",
                other
            )))
        }
    };
    let op = OrgStatusQuery::build(());
    let resp: GraphqlResponse<OrgStatusQuery> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let statuses = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .organization
        .project_statuses;
    statuses
        .into_iter()
        .find(|s| s.status_type == target)
        .map(|s| s.id)
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput(format!(
                "no project status of type '{}' found in workspace",
                state_type
            ))
        })
}
```

> **Note on `OrgStatusQuery::build(())`:** Queries with no variables use the unit type `()` as the variables argument. If cynic requires a named empty struct, define `#[derive(cynic::QueryVariables)] pub struct NoVariables {}` and use `OrgStatusQuery::build(NoVariables {})`.

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1 | head -50
```

- [ ] **Step 4: Commit**

```bash
git add src/infrastructure/graphql/queries/project_queries.rs
git commit -m "feat(graphql): migrate org-status query to cynic; use typed ProjectStatusType enum"
```

---

## Task 6: Migrate mutation types and functions

**Files:**
- Modify: `src/infrastructure/graphql/mutations/project_mutations.rs`

- [ ] **Step 1: Replace mutation input and response types**

Replace the entire file with:

```rust
use cynic::MutationBuilder;

use crate::infrastructure::graphql::queries::project_queries::{
    GraphqlResponse, ProjectNode, execute_with_retry, map_errors,
};

// ---- Shared response types ----

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectPayload")]
pub struct ProjectPayload {
    pub project: Option<ProjectNode>,
    pub success: bool,
    pub last_sync_id: f64,
}

// ---- Create ----

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "ProjectCreateInput")]
pub struct ProjectCreateInput {
    pub name: String,
    pub team_ids: Vec<String>,
    pub description: Option<String>,
    pub lead_id: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub status_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectCreateVariables {
    pub input: ProjectCreateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ProjectCreateVariables")]
pub struct ProjectCreateMutation {
    #[arguments(input = $input)]
    pub project_create: ProjectPayload,
}

// ---- Update ----

#[derive(cynic::InputObject, Debug)]
#[cynic(graphql_type = "ProjectUpdateInput")]
pub struct ProjectUpdateInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub lead_id: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub status_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectUpdateVariables {
    pub id: String,
    pub input: ProjectUpdateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ProjectUpdateVariables")]
pub struct ProjectUpdateMutation {
    #[arguments(id = $id, input = $input)]
    pub project_update: ProjectPayload,
}

// ---- Archive ----

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Project")]
pub struct ArchivedProjectEntity {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProjectArchivePayload")]
pub struct ProjectArchivePayload {
    pub success: bool,
    pub entity: Option<ArchivedProjectEntity>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ProjectArchiveVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ProjectArchiveVariables")]
pub struct ProjectArchiveMutation {
    #[arguments(id = $id)]
    pub project_archive: ProjectArchivePayload,
}
```

> **Note on InputObject optional fields:** cynic's `#[derive(cynic::InputObject)]` omits `None`-valued `Option<T>` fields from the serialized JSON, matching the existing `#[serde(skip_serializing_if = "Option::is_none")]` behavior.

- [ ] **Step 2: Rewrite all three mutation functions**

```rust
pub async fn create_project(
    client: &reqwest::Client,
    api_key: &str,
    input: ProjectCreateInput,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    let op = ProjectCreateMutation::build(ProjectCreateVariables { input });
    let resp: GraphqlResponse<ProjectCreateMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let payload = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project_create;
    if !payload.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "create operation reported failure".to_string(),
        ));
    }
    payload.project.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput(
            "project not returned in create response".to_string(),
        )
    })
}

pub async fn update_project(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
    input: ProjectUpdateInput,
) -> Result<ProjectNode, crate::domain::errors::DomainError> {
    let op =
        ProjectUpdateMutation::build(ProjectUpdateVariables { id: id.to_string(), input });
    let resp: GraphqlResponse<ProjectUpdateMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let payload = resp
        .data
        .ok_or_else(|| {
            crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
        })?
        .project_update;
    if !payload.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "update operation reported failure".to_string(),
        ));
    }
    payload.project.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput(
            "project not returned in update response".to_string(),
        )
    })
}

pub async fn archive_project(
    client: &reqwest::Client,
    api_key: &str,
    id: &str,
) -> Result<String, crate::domain::errors::DomainError> {
    let op = ProjectArchiveMutation::build(ProjectArchiveVariables { id: id.to_string() });
    let resp: GraphqlResponse<ProjectArchiveMutation> =
        execute_with_retry(client, api_key, &op.query, op.variables).await?;
    if let Some(errors) = resp.errors {
        return Err(map_errors(errors));
    }
    let data = resp.data.ok_or_else(|| {
        crate::domain::errors::DomainError::InvalidInput("empty API response".to_string())
    })?;
    if !data.project_archive.success {
        return Err(crate::domain::errors::DomainError::InvalidInput(
            "archive operation reported failure".to_string(),
        ));
    }
    Ok(data
        .project_archive
        .entity
        .map(|e| e.id)
        .unwrap_or_else(|| id.to_string()))
}
```

- [ ] **Step 3: Fix callers of renamed input types**

The input structs were renamed (`ProjectCreateInputVars` → `ProjectCreateInput`, `ProjectUpdateInputVars` → `ProjectUpdateInput`). Update all callers:

```bash
grep -rn "ProjectCreateInputVars\|ProjectUpdateInputVars" src/
```

For each caller, update the import and usage:
- `ProjectCreateInputVars` → `ProjectCreateInput`
- `ProjectUpdateInputVars` → `ProjectUpdateInput`

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: Clean build with only deprecation warnings for `state` field.

- [ ] **Step 5: Commit**

```bash
git add src/infrastructure/graphql/mutations/project_mutations.rs src/
git commit -m "feat(graphql): migrate mutation types to cynic InputObject/QueryFragment"
```

---

## Task 7: Final verification

- [ ] **Step 1: Confirm compile-time schema validation is live**

Introduce a deliberate typo in `ProjectNode`:
```rust
pub slug_id_typo: String,  // temporary — rename slug_id to this
```

Run:
```bash
cargo build 2>&1 | grep "slug_id_typo\|schema"
```

Expected: A compile error naming the unknown field. Revert the typo.

- [ ] **Step 2: Run all tests**

```bash
cargo test 2>&1
```

Expected: All existing unit tests pass.

- [ ] **Step 3: Run integration tests (if LINEAR_TEST_API_KEY is available)**

```bash
LINEAR_TEST_API_KEY=<key> cargo test --test integration 2>&1
LINEAR_TEST_API_KEY=<key> cargo test --test e2e 2>&1
```

Expected: All tests pass; no functional regression.

- [ ] **Step 4: Manually exercise the CLI for all project commands**

```bash
cargo run -- project list
cargo run -- project get <id-or-slug>
cargo run -- project create --name "Test Cynic Migration" --team <team-id>
cargo run -- project update <id> --name "Updated Name"
cargo run -- project archive <id>
```

Expected: Each command returns the same data/output as before the refactor.

- [ ] **Step 5: Final commit**

```bash
git add -u
git commit -m "test(graphql): verify cynic compile-time schema validation active"
```

---

## Known edge cases and risks

| Risk | Mitigation |
|---|---|
| `PaginationOrderBy::UpdatedAt` argument syntax may not compile | Check cynic docs for enum literal in `#[arguments]`; fallback: add `order_by` to variables struct and pass from caller |
| `SlugLookupQuery` inline filter object syntax may be rejected | Define `ProjectFilter { slug_id: Option<SlugIdFilter> }` and `SlugIdFilter { eq: Option<String> }` as InputObjects and pass as a variable |
| `state` field is `@deprecated` — cynic may emit a warning or error | Add `#[allow(deprecated)]` to `ProjectNode` |
| `OrgStatusQuery::build(())` — cynic may not accept unit type as empty variables | Use `#[derive(cynic::QueryVariables)] pub struct NoVariables {}` and call `build(NoVariables {})` |
| InputObject `Option<T>` fields — cynic may serialize `None` as `null` instead of omitting | Verify with a test by checking the generated query; cynic v3 omits `None` fields by default |
