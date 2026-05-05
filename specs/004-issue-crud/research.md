# Research: Issue CRUD Operations

**Branch**: `004-issue-crud` | **Date**: 2026-05-05

## Decision 1: WorkflowState representation — dynamic, not a fixed enum

**Decision**: Replace the fixed `WorkflowState` enum (Backlog/Todo/InProgress/Done/Cancelled) with a `WorkflowStateRef { id: String, name: String, state_type: String }` value object.

**Why**: Linear workflow states are team-specific strings — not a global enum. Real teams may have states named "Triage", "In Review", "Accepted", etc. The `type` field on `WorkflowState` (one of `triage`, `backlog`, `unstarted`, `started`, `completed`, `canceled`, `duplicate`) is a semantic category for progress display, not a fixed state machine definition.

**Investigation — can we validate state transitions?**: The Linear GraphQL `WorkflowState` type has: `id`, `name`, `type`, `position`, `color`, `inheritedFrom`, `team`, `issues`. There are **no** `allowedNextStates`, `transitions`, or FSM edge fields. Linear's API accepts any state → any state transition without enforcement; the web UI behaves the same way. CLI-side FSM validation based on `type` would be more restrictive than Linear itself and could block legitimate workflows. Therefore, validation is limited to: **does the named state exist in the team's states list?** (as specified by FR-009).

**Alternatives considered**:
- Keep fixed enum, use raw string only in mutations → rejected: wrong abstraction; hides the fact that state names are team-defined.
- Infer allowed transitions from `type` field (e.g., block `completed → completed`) → rejected: Linear does not enforce this; CLI should not be more restrictive than the service it wraps.
- Use raw `String` field on `Issue` → rejected: violates DDD (no raw strings for domain concepts).

---

## Decision 2: Issue identifier resolution (display ID vs. UUID)

**Decision**: `issue get` and `issue update` accept either the human-readable display ID (e.g. `ENG-123`) or a Linear UUID. Auto-detect format using regex `^[A-Z]+-\d+$` = display ID, else UUID. For display IDs, use `issues(filter: { identifier: { eq: "ENG-123" } })` query; for UUIDs, use `issue(id: "uuid")`.

**Why**: FR-004 requires both formats. Using the `identifier` filter avoids an extra resolution round-trip.

**Alternatives considered**:
- Resolve display ID → UUID upfront in all cases → rejected: adds a round-trip.
- Accept only UUIDs → rejected: spec requires display IDs.

---

## Decision 3: Sub-issues via `--parent` on `issue create`

**Decision**: Sub-issue creation is handled by `issue create --parent <issue-id>`. No separate `sub-issue create` command. The Linear API's `IssueCreateInput` accepts a `parentId` field.

**Why**: FR-007, US5 confirm this. The parent linkage is one additional field on the same mutation, not a distinct operation.

---

## Decision 4: `LINEAR_PROJECT_ID` environment variable as default project

**Decision**: Read `std::env::var("LINEAR_PROJECT_ID")` in the CLI layer. An explicit `--project` flag always takes precedence. Omitting `--project` when `LINEAR_PROJECT_ID` is unset → exit code 1 before any API call.

**Why**: FR-006 and Clarifications section explicitly require this. The env var is resolved to a `ProjectId` value object at the CLI boundary; it never leaks into domain/application layers as a raw string.

---

## Decision 5: `list_workflow_states` placement in `IssueRepository`

**Decision**: Add `list_workflow_states(team_id: TeamId) -> Result<Vec<WorkflowStateInfo>, DomainError>` to `IssueRepository` trait.

**Why**: Only needed for `issue update --state` validation. Keeping it in `IssueRepository` means `UpdateIssue` use case has one repository dependency. `WorkflowStateInfo { id: String, name: String, state_type: String }` is a new lightweight domain struct (not a repository-level entity).

**Alternatives considered**:
- Add to `TeamRepository` → rejected: forces `UpdateIssue` to depend on two repos.
- Validate in CLI layer → rejected: business validation belongs in application/domain layer per DDD.
- Create `WorkflowStateRepository` → rejected: over-engineering for a single use case.

**Future refactor**: If `linear state list` command is added, extract to `TeamRepository`.

---

## Decision 6: Issue list filters and pagination via `ListIssuesInput`

**Decision**: Introduce `ListIssuesInput` domain struct:
```
team_id: Option<TeamId>
project_id: Option<ProjectId>
state_name: Option<String>
assignee_id: Option<UserId>
priority: Option<Priority>
label_ids: Vec<LabelId>
limit: i32          // default 50
cursor: Option<String>
all_pages: bool     // triggers auto-pagination in use case
```

**Why**: Fulfills FR-002/FR-003. A struct is cleaner than a method with 8+ parameters and easier to extend.

---

## Decision 7: `--no-parent` on `issue update`

**Decision**: `UpdateIssueInput` includes `no_parent: bool`. When `true`, the mutation sets `parentId: null` in `IssueUpdateInput`. `--parent` and `--no-parent` are mutually exclusive; the CLI validates before constructing the input.

**Why**: FR-008 requires detaching a sub-issue from its parent. Linear API accepts `null` for `parentId`.

---

## Decision 8: `Issue` entity fields

**Decision**: Extend `Issue` entity with all fields needed for `issue get` output:
```
description: Option<String>
assignee_id: Option<UserId>
assignee_name: Option<String>   // denormalized for display
label_ids: Vec<LabelId>
due_date: Option<String>        // ISO 8601, stored as String
estimate: Option<f64>
parent_id: Option<IssueId>
parent_title: Option<String>    // denormalized for display
sub_issues: Vec<SubIssueRef>    // empty Vec in list context; populated in get context
```

`SubIssueRef { id: IssueId, title: String }` is a new lightweight domain struct.

**Why**: FR-005 requires all these fields in `issue get` output. Using `sub_issues: Vec<SubIssueRef>` (empty in list, populated in get) avoids parallel entity hierarchies while remaining honest about what each operation fetches. The project pattern uses one entity for all operations.

---

## All NEEDS CLARIFICATION resolved

No open questions remain.
