# Data Model: Issue CRUD Operations

**Branch**: `004-issue-crud` | **Date**: 2026-05-05

## Domain Entities

### `Issue` (modified)

Primary aggregate for all issue operations. Extends the existing stub entity.

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | `IssueId` | ✅ | UUID or display ID (e.g. `ENG-123`) |
| `identifier` | `String` | ✅ | Human-readable display ID (`ENG-123`) |
| `title` | `String` | ✅ | Non-empty, validated in constructor |
| `description` | `Option<String>` | — | Markdown |
| `state` | `WorkflowStateRef` | ✅ | Replaces fixed `WorkflowState` enum |
| `priority` | `Priority` | ✅ | Existing enum (NoPriority/Urgent/High/Medium/Low) |
| `team_id` | `TeamId` | ✅ | Owning team |
| `assignee_id` | `Option<UserId>` | — | Assigned user UUID |
| `assignee_name` | `Option<String>` | — | Denormalized display name |
| `label_ids` | `Vec<LabelId>` | — | Applied label UUIDs |
| `due_date` | `Option<String>` | — | ISO 8601 date string (`YYYY-MM-DD`) |
| `estimate` | `Option<f64>` | — | Story points or time (team-configured) |
| `parent_id` | `Option<IssueId>` | — | UUID of parent issue |
| `parent_title` | `Option<String>` | — | Denormalized parent title for display |
| `sub_issues` | `Vec<SubIssueRef>` | — | Empty in list context; populated in get context |
| `created_at` | `String` | ✅ | ISO 8601 timestamp |
| `updated_at` | `String` | ✅ | ISO 8601 timestamp |

**Validation rules**:
- `title` must be non-empty → `DomainError::InvalidInput`

**State transitions**: Linear does not enforce workflow state transitions via its API. Any → any transition is accepted. The CLI validates only that the target state name exists in the team's workflow states.

---

### `SubIssueRef` (new)

Lightweight reference used in `Issue.sub_issues` for display purposes.

| Field | Type | Notes |
|-------|------|-------|
| `id` | `IssueId` | Child issue identifier |
| `title` | `String` | Child issue title |
| `identifier` | `String` | Display ID (e.g. `ENG-456`) |

---

### `WorkflowStateInfo` (new)

Used by `IssueRepository::list_workflow_states` for `--state` validation in `issue update`.

| Field | Type | Notes |
|-------|------|-------|
| `id` | `String` | Linear UUID |
| `name` | `String` | Human-readable (e.g., "In Progress") |
| `state_type` | `String` | One of: triage, backlog, unstarted, started, completed, canceled, duplicate |

---

## Value Objects

### `WorkflowStateRef` (new — replaces `WorkflowState` enum)

Replaces the fixed `WorkflowState` enum. Linear workflow states are team-specific strings, not a global enum.

```rust
pub struct WorkflowStateRef {
    pub id: String,
    pub name: String,
    pub state_type: String,
}
```

**Implements**: `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`  
**Display**: shows `name`  
**Note**: The existing `WorkflowState` enum (`src/domain/value_objects/workflow_state.rs`) is replaced. All existing usages (only in the stub `Issue` entity) are updated.

---

### `LabelId` (new)

```rust
pub struct LabelId(String);
```

Wraps a Linear label UUID. Same pattern as `IssueId`, `TeamId`, etc.

**Validation**: non-empty string  
**Implements**: `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`, `Display`

---

### Existing value objects (unchanged)

- `IssueId` — type-safe issue identifier (UUID or display ID string)
- `TeamId` — team UUID
- `UserId` — user UUID
- `ProjectId` — project identifier (Uuid | Slug variants)
- `Priority` — NoPriority/Urgent/High/Medium/Low (integer mapping 0–4)

---

## Input Structs

### `CreateIssueInput`

```rust
pub struct CreateIssueInput {
    pub title: String,                    // required
    pub team_id: TeamId,                  // required
    pub project_id: ProjectId,            // required (or from LINEAR_PROJECT_ID)
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub assignee_id: Option<UserId>,
    pub label_ids: Vec<LabelId>,
    pub due_date: Option<String>,         // YYYY-MM-DD
    pub estimate: Option<f64>,
    pub parent_id: Option<IssueId>,       // set for sub-issue creation
}
```

**Validation** (enforced in `CreateIssue` use case before any API call):
- `title` non-empty → exit code 1
- `team_id` non-empty → exit code 1
- `project_id` set (either from flag or `LINEAR_PROJECT_ID`) → exit code 1 if missing
- `due_date` matches `YYYY-MM-DD` format if supplied → exit code 1
- `estimate` non-negative if supplied → exit code 1

---

### `UpdateIssueInput`

All fields optional. At least one must be Some/true (enforced by use case).

```rust
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_id: Option<String>,         // resolved UUID (validated against team's states)
    pub priority: Option<Priority>,
    pub assignee_id: Option<UserId>,      // None = no change; explicit "unassign" not in scope
    pub due_date: Option<String>,         // YYYY-MM-DD
    pub estimate: Option<f64>,
    pub parent_id: Option<IssueId>,       // set or change parent
    pub no_parent: bool,                  // detach from parent (mutually exclusive with parent_id)
}
```

**Validation** (enforced in `UpdateIssue` use case):
- At least one field set → exit code 1 if all None/false
- `parent_id` and `no_parent` are mutually exclusive → exit code 1
- `state_id` is resolved (not the raw name): the use case fetches team's workflow states, validates the supplied `--state` name (case-insensitive), extracts the UUID, and stores it in `state_id`

---

### `ListIssuesInput`

```rust
pub struct ListIssuesInput {
    pub team_id: Option<TeamId>,
    pub project_id: Option<ProjectId>,
    pub state_name: Option<String>,
    pub assignee_id: Option<UserId>,
    pub priority: Option<Priority>,
    pub label_ids: Vec<LabelId>,
    pub limit: i32,                       // default 50, max 250
    pub cursor: Option<String>,
    pub all_pages: bool,
}
```

---

## Repository Trait (extended)

```rust
#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn list(&self, input: ListIssuesInput) -> Result<ListIssuesResult, DomainError>;
    async fn get(&self, id: IssueId) -> Result<Issue, DomainError>;
    async fn create(&self, input: CreateIssueInput) -> Result<Issue, DomainError>;
    async fn update(&self, id: IssueId, input: UpdateIssueInput) -> Result<Issue, DomainError>;
    async fn list_workflow_states(&self, team_id: TeamId) -> Result<Vec<WorkflowStateInfo>, DomainError>;
}

pub struct ListIssuesResult {
    pub items: Vec<Issue>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}
```

---

## GraphQL Mapping

### Query: List Issues

Linear API: `issues(filter: IssueFilter, first: Int, after: String, orderBy: PaginationOrderBy)`

Key filter fields used:
- `team: { id: { eq: $team_id } }`
- `project: { id: { eq: $project_id } }`
- `state: { name: { eq: $state_name } }`
- `assignee: { id: { eq: $assignee_id } }`
- `priority: { eq: $priority_int }`
- `labels: { id: { in: [$label_ids] } }`

Default sort: `orderBy: updatedAt` (descending)

### Query: Get Issue by UUID

`issue(id: ID!)`

### Query: Get Issue by Display ID

`issues(filter: { identifier: { eq: "ENG-123" } }, first: 1)`

### Query: List Workflow States

`workflowStates(filter: { team: { id: { eq: $team_id } } })`

### Mutation: Create Issue

`issueCreate(input: IssueCreateInput!)`

Key input fields: `title`, `teamId`, `projectId`, `description`, `priority` (Int 0–4), `assigneeId`, `labelIds`, `dueDate`, `estimate`, `parentId`

### Mutation: Update Issue

`issueUpdate(id: ID!, input: IssueUpdateInput!)`

Key input fields: `title`, `description`, `stateId`, `priority` (Int 0–4), `assigneeId`, `dueDate`, `estimate`, `parentId` (`null` = detach parent)
