# Data Model: Filter by Partial Title

## Entities (unchanged)

**Issue** and **Project** entities are unchanged. Filtering is a query parameter; no entity fields are added or mutated.

## Input Type Changes

### `ListIssuesInput` (domain/entities/issue.rs)

Add field:

```rust
pub title_contains: Option<String>,  // None or Some("") → no filter applied
```

All existing fields unchanged. Default/empty value: `None`.

### `ProjectRepository::list` signature (domain/repositories/project_repository.rs)

Add parameter:

```rust
async fn list(
    &self,
    team_id: Option<TeamId>,
    first: u32,
    after: Option<String>,
    name_contains: Option<String>,   // NEW — None or Some("") → no filter
) -> Result<ListProjectsResult, DomainError>;
```

## GraphQL Type Changes

### `StringComparatorInput` (infrastructure/graphql/queries/issue_queries.rs)

Add field:

```rust
#[cynic(rename = "containsIgnoreCase", skip_serializing_if = "Option::is_none")]
pub contains_ignore_case: Option<String>,
```

### `IssueFilterInput` (infrastructure/graphql/queries/issue_queries.rs)

Add field:

```rust
#[cynic(skip_serializing_if = "Option::is_none")]
pub title: Option<StringComparatorInput>,
```

### `StringComparator` (infrastructure/graphql/queries/project_queries.rs)

Add field:

```rust
#[cynic(rename = "containsIgnoreCase")]
pub contains_ignore_case: Option<String>,
```

### `ProjectFilter` (infrastructure/graphql/queries/project_queries.rs)

Add field:

```rust
pub name: Option<StringComparator>,
```

### `ProjectsVariables` / `TeamProjectsVariables` (project_queries.rs)

Add field to each:

```rust
pub filter: Option<ProjectFilter>,
```

And update the corresponding `#[arguments]` annotations to include `filter: $filter`.

## Validation Rules

- `title_contains = Some("")` → treat as `None` (no filter). Enforced in `build_issue_filter`.
- `name_contains = Some("")` → treat as `None` (no filter). Enforced in `fetch_projects`.
- Non-empty substring is passed as-is to `containsIgnoreCase`; no further sanitization needed (Linear API handles escaping).

## State Transitions

No state transitions involved. This feature is read-only.
