# Feature Specification: Issue CRUD Operations

**Feature Branch**: `004-issue-crud`  
**Created**: 2026-05-05  
**Status**: Draft  
**Input**: User description: "I want to work on the issue entity, so I need to list all the existing issues, create a new one, and for an existing issue, I want to create sub issue, sub tasks, and so on."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List Issues (Priority: P1)

A user wants to see all issues they have access to in their Linear workspace. They run an issue list command and receive a structured list of issues with key attributes such as title, workflow state, priority, and assignee — enough to triage and select issues for further action.

**Why this priority**: Listing is the entry point for all issue workflows. Without it, users and agents cannot discover issues to act upon, making every other operation unreachable. An existing partial implementation (`list_issues` use case) is already in place but needs to be completed to full spec.

**Independent Test**: Can be fully tested by running `linear issue list` and confirming a paginated list of issues is returned with at minimum title, state, and priority fields — delivers a fully browsable issue directory.

**Acceptance Scenarios**:

1. **Given** the user is authenticated, **When** they run `linear issue list`, **Then** the CLI outputs accessible issues with title, state, priority, assignee, and team; results are paginated and the user is informed if more pages exist.
2. **Given** the user is authenticated and passes `--output json`, **When** they run `linear issue list`, **Then** the CLI outputs a machine-readable JSON array of issue objects with a stable schema.
3. **Given** no issues exist in the workspace, **When** they run `linear issue list`, **Then** the CLI outputs an empty result set with a user-friendly message (human mode) or an empty JSON array (JSON mode), and exits with code 0.
4. **Given** the user is not authenticated, **When** they run `linear issue list`, **Then** the CLI exits with code 3 and instructs the user to authenticate.

---

### User Story 2 - View Issue Details (Priority: P2)

A user wants to inspect the full details of a specific issue — including its title, description, state, priority, assignee, labels, due date, estimate, and child sub-issues — by providing the issue's identifier.

**Why this priority**: Detail retrieval is a prerequisite for update operations and is frequently used by AI agents to read current state before deciding what to change. It is the most commonly performed single-issue operation.

**Independent Test**: Can be fully tested by running `linear issue get ENG-123` for a known issue identifier and confirming all fields are present and accurate — independently delivers a complete issue view.

**Acceptance Scenarios**:

1. **Given** a valid issue identifier (e.g. `ENG-123`), **When** the user runs `linear issue get <id>`, **Then** the CLI outputs the full issue record including title, description, state, priority, assignee, labels, due date, estimate, and a list of any sub-issues.
2. **Given** a valid issue identifier and `--output json`, **When** the user runs `linear issue get <id>`, **Then** the CLI outputs a single issue object as stable JSON.
3. **Given** an issue identifier that does not exist or the user does not have access to, **When** the user runs `linear issue get <id>`, **Then** the CLI outputs a descriptive "not found" error on stderr and exits with code 1.

---

### User Story 3 - Create an Issue (Priority: P3)

A user wants to create a new issue in a Linear team by providing a title and a team identifier. Optional attributes such as description, priority, assignee, labels, due date, and estimate may also be supplied at creation time.

**Why this priority**: Issue creation enables new work to be tracked. It is the foundational write operation from which updates and state transitions follow. Without it, the CLI is purely read-only for issues.

**Independent Test**: Can be fully tested by running `linear issue create --title "..." --team <team-id> --project <project-id>` and confirming the new issue appears in `linear issue list --project <project-id>` — independently verifies write capability end-to-end.

**Acceptance Scenarios**:

1. **Given** the user is authenticated and provides a title, a valid team identifier, and a valid project identifier, **When** they run the issue create command, **Then** the CLI creates the issue in Linear assigned to that project, outputs the new issue's identifier and title, and exits with code 0.
2. **Given** the user provides `--dry-run`, **When** they run the issue create command, **Then** the CLI reports what would be created without performing any mutation, and exits with code 0.
3. **Given** the user omits any of the required `--title` or `--team` flags, or omits `--project` when `LINEAR_PROJECT_ID` is not set, **When** they run the issue create command, **Then** the CLI emits a usage error on stderr and exits with code 1 without contacting the API.
4. **Given** `LINEAR_PROJECT_ID` is set in the environment and the user omits `--project`, **When** they run the issue create command with `--title` and `--team`, **Then** the CLI uses the environment variable as the project identifier and creates the issue successfully.
4. **Given** the provided team identifier does not exist or the user lacks permission, **When** they run the issue create command, **Then** the CLI emits a descriptive error on stderr and exits with code 2.

---

### User Story 4 - Update an Issue (Priority: P4)

A user wants to modify one or more attributes of an existing issue — such as changing its title, description, state, priority, assignee, due date, or estimate — without recreating it.

**Why this priority**: Issues are the primary unit of work in Linear and routinely change state and ownership. Updating issues is a core daily action for both human users and AI agents automating workflows.

**Independent Test**: Can be fully tested by running `linear issue update ENG-123 --state in_progress` and confirming the state change is reflected in a subsequent `linear issue get ENG-123 --output json` call — independently verifies the full update cycle.

**Acceptance Scenarios**:

1. **Given** a valid issue identifier and at least one update flag, **When** the user runs the issue update command, **Then** the CLI applies the change to Linear and outputs the updated issue's identifier and changed fields.
2. **Given** a valid state value, **When** the user runs `linear issue update <id> --state <state>`, **Then** the CLI updates the workflow state and confirms the new state in output.
3. **Given** the user provides `--dry-run`, **When** they run the issue update command, **Then** the CLI reports the intended changes without applying them and exits with code 0.
4. **Given** the user provides no update flags, **When** they run the issue update command, **Then** the CLI emits a usage error indicating at least one field must be specified, and exits with code 1.
5. **Given** the issue does not exist, **When** the user runs the issue update command, **Then** the CLI exits with code 1 and a descriptive not-found error on stderr.

---

### User Story 5 - Create a Sub-Issue (Priority: P5)

A user wants to break a large issue into smaller pieces by creating a child issue (sub-issue) nested under an existing parent issue. The sub-issue behaves as a full issue in its own right but is linked to and displayed under its parent.

**Why this priority**: Sub-issues are the primary decomposition mechanism in Linear. They are required for sprint planning and tracking hierarchical work. This is a write operation that depends on US3 (create) being in place.

**Independent Test**: Can be fully tested by running `linear issue create --title "Sub-task" --team <id> --project <project-id> --parent ENG-123` and confirming the new issue appears as a child of ENG-123 in `linear issue get ENG-123` output — independently verifies parent-child linking.

**Acceptance Scenarios**:

1. **Given** a valid parent issue identifier, a valid team identifier, and a valid project identifier, **When** the user runs `linear issue create --title "..." --team <id> --project <project-id> --parent <parent-id>`, **Then** the CLI creates a child issue linked to the parent and outputs the new sub-issue's identifier.
2. **Given** a `--dry-run` flag, **When** the user runs the create sub-issue command, **Then** the CLI reports what would be created, including the parent linkage, without performing any mutation.
3. **Given** a parent identifier that does not exist, **When** the user runs the create sub-issue command, **Then** the CLI emits a descriptive error and exits with code 1 or 2 as appropriate.
4. **Given** a valid parent issue, **When** the user runs `linear issue get <parent-id>`, **Then** the output includes a list of sub-issues with their identifiers and titles.

---

### Edge Cases

- When the API is unreachable during any issue command, the CLI MUST exit with code 2 and a network error on stderr.
- When the Linear API responds with HTTP 429 (rate limited), the CLI MUST retry the request up to 3 times using exponential backoff (1 s, 2 s, 4 s). If all retries are exhausted, it MUST exit with code 2 and a "rate limited" error on stderr.
- When the issue list spans multiple pages, the CLI MUST support `--all` for auto-pagination and `--limit`/`--cursor` for manual pagination.
- When filtering by team (`--team`), only issues belonging to that team are returned.
- When an update command supplies an invalid state value, the CLI MUST reject it locally before any API call and list valid states in the error message.
- When the user's API key has read-only permissions and a write command is attempted, the CLI MUST surface the permission error on stderr and exit with code 2.
- When a sub-issue is created under a parent, the parent's own team membership is used as the default team; an explicit `--team` flag overrides this.
- When `--parent` and `--no-parent` are both supplied on `issue update`, the CLI MUST exit with code 1 and a usage error before contacting the API.
- Concurrent edits to the same issue are not detected; the CLI applies a last-write-wins strategy, consistent with Linear's own web UI behaviour.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST provide an `issue list` command that returns issues accessible to the authenticated user, including title, workflow state, priority, assignee, team, and issue identifier.
- **FR-002**: The `issue list` command MUST support the following filters, all optional and combinable: `--team <id>` (restrict to a team), `--project <id>` (restrict to a project), `--state <name>` (restrict to a workflow state), `--assignee <user-id>` (restrict to an assignee), `--priority <level>` (one of `no_priority`, `urgent`, `high`, `medium`, `low`), and `--label <id>` (repeatable; restrict to issues carrying all specified labels).
- **FR-003**: The `issue list` command MUST default to returning the first 50 issues, sorted by `updatedAt` descending. It MUST support `--all` for auto-pagination, and `--limit <n>` and `--cursor <token>` for manual pagination.
- **FR-004**: The CLI MUST provide an `issue get <id>` command that returns the full record for a single issue. The `<id>` argument MUST accept the human-readable display ID (e.g. `ENG-123`) as well as the Linear UUID; the CLI auto-detects the format.
- **FR-005**: The `issue get` output MUST include all issue fields: title, description, workflow state, priority, assignee, labels, due date, estimate, parent issue (if any), and a list of direct sub-issues (identifier and title only).
- **FR-006**: The CLI MUST provide an `issue create` command accepting `--title <string>` (required), `--team <id>` (required), and `--project <id>` (required unless the `LINEAR_PROJECT_ID` environment variable is set, in which case it is used as the default and `--project` may be omitted), with optional `--description <string>`, `--priority <level>`, `--assignee <user-id>`, `--label <id>` (repeatable), `--due-date <YYYY-MM-DD>`, `--estimate <number>`, and `--parent <issue-id>` flags. An explicit `--project` flag always takes precedence over `LINEAR_PROJECT_ID`. Omitting `--project` when `LINEAR_PROJECT_ID` is not set MUST produce a usage error on stderr and exit with code 1 before any API call.
- **FR-007**: When `--parent <issue-id>` is provided on `issue create`, the new issue is linked as a child of the specified parent issue; the parent issue MUST exist and be accessible to the user.
- **FR-008**: The CLI MUST provide an `issue update <id>` command accepting any combination of `--title`, `--description`, `--state`, `--priority`, `--assignee`, `--due-date`, `--estimate`, `--parent <id>` (set or change parent issue), and `--no-parent` (detach from parent, promoting the issue to top-level) flags; at least one flag MUST be required. `--parent` and `--no-parent` are mutually exclusive.
- **FR-009**: The `--state` flag on `issue update` MUST validate the provided value against the team's workflow states fetched from Linear at command time. The CLI MUST first retrieve the list of workflow states for the target issue's team, check that the supplied value matches a state name (case-insensitive), and only then submit the mutation. If the value does not match any state, the CLI MUST exit with code 1 and list the valid state names in the error message. This pre-flight lookup counts as one of the two API calls for this command.
- **FR-010**: Every mutating command (`create`, `update`) MUST support a `--dry-run` flag that reports the intended operation without executing it.
- **FR-011**: All `issue` commands MUST produce human-readable output when stdout is a TTY and machine-readable JSON when stdout is a pipe or `--output json` is passed.
- **FR-012**: All `issue` commands MUST emit errors on stderr and never mix error text with data output on stdout.
- **FR-013**: All `issue` commands MUST exit with code 0 on success, code 1 on user/input error, code 2 on API/network error, and code 3 on authentication error.
- **FR-014**: All `issue` commands MUST retry HTTP 429 responses up to 3 times with exponential backoff (1 s, 2 s, 4 s).
- **FR-015**: All `issue` commands MUST support `--verbose` (human-readable step progress on stderr) and `--debug` (raw request/response on stderr; implies `--verbose`).

### Key Entities

- **Issue**: A Linear work item representing a unit of work; key attributes include identifier (e.g. `ENG-123`), title, description, workflow state, priority (no priority / urgent / high / medium / low), assignee (user), team, labels, due date, estimate, parent issue (optional), and child sub-issues.
- **WorkflowState**: The lifecycle state of an issue as defined by the team's workflow (e.g. Backlog, Todo, In Progress, Done, Cancelled); state names are team-specific and not a fixed enum.
- **Priority**: A fixed 5-level scale: `no_priority`, `urgent`, `high`, `medium`, `low`.
- **Team**: A Linear team that owns the issue; each issue belongs to exactly one team.
- **User**: A Linear workspace member who may be assigned to an issue.
- **Label**: A free-form tag that can be applied to an issue for categorisation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can retrieve the default first page (up to 50 issues) in under 3 seconds on a standard internet connection. Full workspace retrieval via `--all` completes in under 10 seconds for workspaces with up to 500 issues.
- **SC-002**: A user can complete the full create-update lifecycle for a single issue in under 60 seconds using only CLI commands.
- **SC-003**: All issue commands (`list`, `get`, `create`, `update`) produce valid, schema-stable JSON output when `--output json` is passed — verifiable by schema validation in CI.
- **SC-004**: 100% of user-input errors (missing required flags, invalid state/priority values) are caught and reported locally before any API call is made — verifiable by running commands without network access.
- **SC-005**: Dry-run mode for all mutating commands completes without making any state change in Linear — verifiable by comparing Linear issue state before and after a `--dry-run` invocation.
- **SC-006**: Sub-issue creation correctly links the child to the parent, verifiable by retrieving the parent issue and confirming the child appears in its sub-issues list.
- **SC-007**: All issue commands correctly report authentication errors (exit code 3) when credentials are absent — verifiable without a Linear account.

## Assumptions

- Workflow state names (e.g. "In Progress", "Done") are team-specific strings fetched from Linear at command time; the CLI validates the supplied `--state` value against this live list before executing the mutation (see FR-009).
- Issue deletion (permanent removal) is out of scope; Linear does not expose a delete mutation for issues in its public API.
- Issue archival (`linear issue archive <id>`) is out of scope for this feature and may be addressed in a subsequent spec if needed.
- The `issue list` command defaults to returning only non-archived, non-cancelled issues; a `--include-archived` flag is out of scope here.
- Labels are identified by their ID (obtainable from a future `linear label list` command or from Linear's UI); free-text label creation is out of scope.
- Assignee, label, and parent identifiers used in `issue create` and `issue update` are Linear UUIDs or display IDs; the CLI resolves display IDs to UUIDs as needed.
- Estimate values are numeric (story points or time) as configured by the team's estimation settings in Linear; the CLI accepts any non-negative number.
- Sub-issues are a first-class feature of this spec; sub-tasks (checklist items within an issue) are a distinct Linear concept and are out of scope for this feature.
- Progress on an issue is computed by Linear from sub-issue completion and is read-only from the CLI's perspective.

## Clarifications

### Session 2026-05-05

- Q: How should `--state` on `issue update` resolve and validate workflow state values? → A: Fetch the team's workflow states from Linear at command time, validate the supplied value locally against that list (case-insensitive), then execute the mutation — two API calls; invalid values exit code 1 with valid states listed.
- Q: Should `issue list` support additional filters beyond `--team`? → A: Yes — add `--team`, `--project`, `--state`, `--assignee`, `--priority`, and `--label` filters, all optional and combinable.
- Q: Should `issue update` support reparenting (changing or removing a parent)? → A: Yes — add `--parent <id>` to set/change parent and `--no-parent` to detach; the two flags are mutually exclusive.
- Q: How should the CLI handle concurrent edits to the same issue? → A: Last-write-wins — apply the update without conflict detection, consistent with Linear's web UI behaviour.
- Q: Should `issue create` support assigning a project, and if so is it optional or required? → A: Required — `--project <id>` is a mandatory flag on `issue create` alongside `--title` and `--team`.
- Q: Is there a way to avoid supplying `--project` on every `issue create` invocation? → A: Yes — if the `LINEAR_PROJECT_ID` environment variable is set, it serves as the default project and `--project` may be omitted; an explicit `--project` flag always takes precedence.
