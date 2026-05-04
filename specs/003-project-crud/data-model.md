# Data Model: Project CRUD Operations

**Branch**: `003-project-crud` | **Date**: 2026-04-23

## Domain Entities

### `Project` (entity)

The root aggregate for project operations. Immutable after construction (all fields set via `Project::new`; mutations go through use cases that call the repository).

| Field | Rust Type | Notes |
|-------|-----------|-------|
| `id` | `ProjectId` | Primary identifier (UUID or resolved from display ID) |
| `name` | `String` | Non-empty; max length enforced by domain rule (validation in `Project::new`) |
| `description` | `Option<String>` | Free-form text |
| `state` | `ProjectState` | Lifecycle state enum |
| `progress` | `f32` | 0.0–100.0 percentage; read-only from CLI (computed by Linear) |
| `lead_id` | `Option<UserId>` | Assigned project lead |
| `team_ids` | `Vec<TeamId>` | Must have at least one team (invariant enforced in `Project::new`) |
| `start_date` | `Option<NaiveDate>` | ISO 8601 date |
| `target_date` | `Option<NaiveDate>` | ISO 8601 date |
| `updated_at` | `DateTime<Utc>` | Last modification timestamp (from Linear API) |

**Validation rules (enforced in `Project::new`)**:
- `name` must not be empty
- `team_ids` must not be empty
- `progress` must be in `[0.0, 100.0]`

---

## Value Objects

### `ProjectId`

Wraps a Linear project identifier. Accepts two formats, auto-detected at parse time.

| Variant | Pattern | Example |
|---------|---------|---------|
| UUID | `[0-9a-f]{8}-[0-9a-f]{4}-...-[0-9a-f]{12}` | `9cfb482a-81e3-4154-b5b9-2c805e70a02d` |
| DisplayId | `[A-Z]+-\d+` | `PRJ-1`, `ENG-42` |

- `ProjectId::parse(s: &str) -> Result<ProjectId, DomainError>`: fails if neither pattern matches.
- `ProjectId::as_uuid(&self) -> Option<&str>`: `Some` if UUID variant.
- `ProjectId::as_display_id(&self) -> Option<&str>`: `Some` if display-ID variant.
- Infrastructure layer is responsible for resolving display IDs to UUIDs before calling mutations.

### `ProjectState`

```
ProjectState {
    Planned,    → "planned"  (maps from/to Linear API string)
    Started,    → "started"
    Paused,     → "paused"
    Completed,  → "completed"
    Cancelled,  → "cancelled"
}
```

- `ProjectState::from_str(s: &str) -> Result<ProjectState, DomainError>`: case-insensitive parse; errors list valid values.
- Implements `Display` (lowercase output) and `Serialize`/`Deserialize` (lowercase strings for JSON output).

### `UserId`

New value object (parallel to `TeamId`). Wraps a Linear user UUID string. Used for project lead field.

| Rule | Detail |
|------|--------|
| Non-empty | `UserId::new` rejects empty string |
| Format | UUID string; no pattern validation at domain level |

### `PageInfo` (value object / result wrapper)

Used by `ListProjectsResult` to carry pagination state.

| Field | Rust Type |
|-------|-----------|
| `has_next_page` | `bool` |
| `end_cursor` | `Option<String>` |

---

## Application-Layer Inputs

These are plain Rust structs used as use-case inputs (not domain entities).

### `CreateProjectInput`

| Field | Type | Required |
|-------|------|----------|
| `name` | `String` | Yes |
| `team_ids` | `Vec<TeamId>` | Yes (≥ 1) |
| `description` | `Option<String>` | No |
| `lead_id` | `Option<UserId>` | No |
| `start_date` | `Option<NaiveDate>` | No |
| `target_date` | `Option<NaiveDate>` | No |

Validation: `name` non-empty, `team_ids` non-empty — checked in `CreateProject` use case before calling repository.

### `UpdateProjectInput`

| Field | Type | Required |
|-------|------|----------|
| `name` | `Option<String>` | No (but ≥ 1 field required overall) |
| `description` | `Option<String>` | No |
| `state` | `Option<ProjectState>` | No |
| `lead_id` | `Option<UserId>` | No |
| `start_date` | `Option<NaiveDate>` | No |
| `target_date` | `Option<NaiveDate>` | No |

Validation: At least one field must be `Some` — checked in `UpdateProject` use case (FR-006).

---

## Repository Traits (domain layer)

### `ProjectRepository`

```rust
#[async_trait]
trait ProjectRepository: Send + Sync {
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
```

`ListProjectsResult`:
```rust
struct ListProjectsResult {
    items: Vec<Project>,
    page_info: PageInfo,
}
```

---

## State Transitions

```
Planned ──► Started ──► Completed
    │           │
    └──► Paused ◄┘
    │
    └──► Cancelled
Started ──► Cancelled
```

State transition validation is NOT enforced at the domain level (Linear enforces validity server-side). Invalid transitions return a `DomainError::InvalidInput` from the infrastructure layer after receiving a GraphQL error.

---

## JSON Output Schema (stable contract, per SC-003)

### Project object (used in `project list` and `project get --output json`)

```json
{
  "id": "string (UUID)",
  "name": "string",
  "description": "string | null",
  "state": "planned | started | paused | completed | cancelled",
  "progress": "number (0.0–100.0)",
  "lead_id": "string (UUID) | null",
  "team_ids": ["string (UUID)"],
  "start_date": "string (YYYY-MM-DD) | null",
  "target_date": "string (YYYY-MM-DD) | null",
  "updated_at": "string (ISO 8601)"
}
```

### List response

```json
{
  "projects": [...],
  "page_info": {
    "has_next_page": "boolean",
    "end_cursor": "string | null"
  }
}
```

### Mutation response (create / update)

```json
{
  "id": "string (UUID)",
  "name": "string",
  "state": "string"
}
```

### Archive response

```json
{
  "success": true,
  "id": "string (UUID)"
}
```
