# Research: Improve Project List Identifiers

**Branch**: `005-improve-project-list` | **Date**: 2026-05-05

## Resolved Questions

### 1. Is `slug_id` already available throughout the stack?

**Decision**: Yes — no new API queries or schema changes are required.

**Rationale**: The `ProjectNode` GraphQL fragment already fetches `slug_id` (project_queries.rs:75). The `node_to_project` mapping in `project_repository.rs:125` passes it into `Project::new(...)`. The `Project` domain entity stores it as `pub slug_id: String` (project.rs:21). The value is fetched, mapped, and stored — it is simply never surfaced in output.

**Alternatives considered**: Fetching slug via a separate API call was never needed.

---

### 2. Do mutation responses (create/update) include `slug_id`?

**Decision**: Yes — create and update both return a full `ProjectNode` which includes `slug_id`.

**Rationale**: `create_project()` and `update_project()` in project_mutations.rs both return `ProjectNode` (the same fragment used for queries), which includes `slug_id`. The `project create` command's current `MutationResultDto` simply discards `slug_id` when building its output. This field is already in the domain `Project` returned by the use cases.

**Alternatives considered**: N/A.

---

### 3. Does slug resolution (`ProjectId::Slug` → UUID) already work?

**Decision**: Yes — slug-based resolution is fully implemented for all commands that accept a project identifier.

**Rationale**: `ProjectId::parse()` correctly identifies slugs (value_objects/project_id.rs:43-59). `LinearProjectRepository::resolve_id()` calls `resolve_slug_to_uuid()` for `ProjectId::Slug` variants (project_repository.rs:39-44). The `resolve_slug_to_uuid()` function issues a filtered GraphQL query (`projects(filter: {slugId: {eq: $slug}}, first: 1)`) to look up the UUID. All commands that accept a project identifier — `get`, `update`, `archive` — already go through `resolve_id()`.

**Alternatives considered**: Client-side slug mapping (caching the slug→UUID mapping from `list`) was considered but rejected — the existing API-based lookup is simpler, correct, and consistent.

---

### 4. What are the terminal width constraints for the new slug column?

**Decision**: Use column widths `name: 35`, `slug: 22`, `state: 12`, leaving target date unrestricted. Total fixed columns = 71 chars, comfortably within 80-column terminals.

**Rationale**: Current layout is `{:<40} {:<12} date`. Linear slugs average 10-20 characters; a 22-char column fits all but pathological cases without wrapping. Reducing name column from 40 → 35 recovers 5 chars. Column order: name → slug → state → target date (slug after name aligns with user scanning pattern: identify project by name, copy slug).

**Alternatives considered**:
- Slug as first column: less intuitive since users scan by name.
- Truncating slug: rejected per FR-002 (no truncation).
- Keeping name at 40: total fixed = 76 chars, still fine but tighter.

---

### 5. Where does slug appear in `project get` detail output?

**Decision**: Slug appears on the second line, immediately after `Name:`, before `ID:`.

**Rationale**: The slug is the user-facing identifier; UUID (`ID:`) is secondary. Placing slug prominently makes it easy to spot for copy-paste. The current `project get` output order is `Name → ID → State → Progress → Description → Lead → Teams → Start date → Target date → Updated`.

**Alternatives considered**: Placing slug after ID was considered but rejected — users reaching `project get` typically want the slug, not the UUID.

---

### 6. What changes are needed in `MutationResultDto`?

**Decision**: Add `slug_id: String` to `MutationResultDto`. Update create/update human-readable messages to show slug instead of UUID in the parenthetical.

**Rationale**: `MutationResultDto` is used by both `create` and `update` success JSON output. Adding `slug_id` there covers FR-005 and FR-008 for both commands. Human-readable create message changes from `Created project: "Name" (uuid)` to `Created project: "Name" (slug)`. Update message changes from `Updated project uuid: state → X` to `Updated project slug: state → X`.

**Alternatives considered**: Separate DTOs for create vs update were considered but rejected — unnecessary complexity for a field addition.
