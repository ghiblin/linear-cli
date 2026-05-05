# Data Model: Improve Project List Identifiers

**Branch**: `005-improve-project-list` | **Date**: 2026-05-05

## Changed Types

### `ProjectDto` (CLI output DTO — `src/cli/commands/project.rs`)

Used by `project list` (JSON) and `project get` (JSON).

| Field | Type | Change | Notes |
|-------|------|--------|-------|
| `id` | `String` | Existing | UUID |
| `slug_id` | `String` | **Added** | Human-readable slug (e.g. `q3-platform`) |
| `name` | `String` | Existing | |
| `description` | `Option<String>` | Existing | |
| `state` | `String` | Existing | |
| `progress` | `f64` | Existing | |
| `lead_id` | `Option<String>` | Existing | |
| `team_ids` | `Vec<String>` | Existing | |
| `start_date` | `Option<String>` | Existing | |
| `target_date` | `Option<String>` | Existing | |
| `updated_at` | `String` | Existing | RFC 3339 |

**Mapping change**: `From<&Project> for ProjectDto` adds `slug_id: p.slug_id.clone()`.

---

### `MutationResultDto` (CLI output DTO — `src/cli/commands/project.rs`)

Used by `project create` and `project update` (JSON).

| Field | Type | Change | Notes |
|-------|------|--------|-------|
| `id` | `String` | Existing | UUID |
| `slug_id` | `String` | **Added** | Human-readable slug |
| `name` | `String` | Existing | |
| `state` | `String` | Existing | |

---

## Unchanged Types

These types require no changes for this feature:

- `Project` (domain entity) — `slug_id` already present
- `ProjectId` (value object) — `Uuid` and `Slug` variants already implemented
- `ProjectNode` (GraphQL fragment) — `slug_id` already fetched
- `ProjectListDto` — wraps `Vec<ProjectDto>`, inherits the change

## No New Entities

This feature introduces no new domain entities, value objects, or repository traits. All changes are within the CLI output layer (DTOs and formatting).
