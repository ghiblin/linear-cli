# Feature Specification: Project CRUD Operations

**Feature Branch**: `004-project-crud`  
**Created**: 2026-04-23  
**Status**: Draft  
**Input**: User description: "Implement all the read and write operation related to the project entity"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List Projects (Priority: P1)

A user wants to see all projects they have access to in their Linear workspace. They run a project list command and receive a structured list of projects with key attributes such as name, state, and target date — enough to identify and select a project for further operations.

**Why this priority**: Listing is the entry point for all project workflows. Without it, users and agents cannot discover projects to act upon, making every other operation unreachable.

**Independent Test**: Can be fully tested by running `linear project list` and confirming a paginated list of projects is returned with at minimum name and state fields — delivers a fully browsable project directory.

**Acceptance Scenarios**:

1. **Given** the user is authenticated, **When** they run `linear project list`, **Then** the CLI outputs all accessible projects with name, state, and target date; results are paginated and the user is informed if more pages exist.
2. **Given** the user is authenticated and passes `--output json`, **When** they run `linear project list`, **Then** the CLI outputs a machine-readable JSON array of project objects with a stable schema.
3. **Given** no projects exist in the workspace, **When** they run `linear project list`, **Then** the CLI outputs an empty result set with a user-friendly message (human mode) or an empty JSON array (JSON mode), and exits with code 0.
4. **Given** the user is not authenticated, **When** they run `linear project list`, **Then** the CLI exits with code 3 and instructs the user to authenticate.

---

### User Story 2 - View Project Details (Priority: P2)

A user wants to inspect the full details of a specific project — including its description, current state, associated team(s), lead, target date, and progress — by providing the project's identifier.

**Why this priority**: Detail retrieval is a prerequisite for update operations and is frequently used by AI agents to read current state before deciding what to change.

**Independent Test**: Can be fully tested by running `linear project get <id>` for a known project identifier and confirming all fields are present and accurate — independently delivers a complete project view.

**Acceptance Scenarios**:

1. **Given** a valid project identifier, **When** the user runs `linear project get <id>`, **Then** the CLI outputs the full project record including name, description, state, team(s), lead, target date, start date, and progress percentage.
2. **Given** a valid project identifier and `--output json`, **When** the user runs `linear project get <id>`, **Then** the CLI outputs a single project object as stable JSON.
3. **Given** a project identifier that does not exist or the user does not have access to, **When** the user runs `linear project get <id>`, **Then** the CLI outputs a descriptive "not found" error on stderr and exits with code 1.

---

### User Story 3 - Create a Project (Priority: P3)

A user wants to create a new project in their Linear workspace by providing a name and associating it with one or more teams. Optional fields such as description, target date, and lead may also be supplied at creation time.

**Why this priority**: Project creation enables new workflows to begin. Without it, the tool is purely read-only. Creation is the foundational write operation from which updates and archival follow.

**Independent Test**: Can be fully tested by running `linear project create --name "..." --team <team-id>` and confirming the new project appears in `linear project list` — independently verifies write capability end-to-end.

**Acceptance Scenarios**:

1. **Given** the user is authenticated and provides a name and at least one valid team identifier, **When** they run the project create command, **Then** the CLI creates the project in Linear, outputs the new project's identifier and name, and exits with code 0.
2. **Given** the user provides `--dry-run`, **When** they run the project create command, **Then** the CLI reports what would be created without performing any mutation, and exits with code 0.
3. **Given** the user omits the required `--name` or `--team` flags, **When** they run the project create command, **Then** the CLI emits a usage error on stderr and exits with code 1 without contacting the API.
4. **Given** the provided team identifier does not exist or the user lacks permission, **When** they run the project create command, **Then** the CLI emits a descriptive error on stderr and exits with code 2.

---

### User Story 4 - Update a Project (Priority: P4)

A user wants to modify one or more attributes of an existing project — such as changing its name, description, state, lead, target date, or associated teams — without recreating it.

**Why this priority**: Projects evolve over time; state transitions (planned → started → completed) and metadata updates are routine operations for both human users and AI agents managing Linear workflows.

**Independent Test**: Can be fully tested by running `linear project update <id> --state started` and confirming the state change is reflected in a subsequent `linear project get <id>` call — independently verifies the full update cycle.

**Acceptance Scenarios**:

1. **Given** a valid project identifier and at least one update flag (e.g., `--name`, `--state`, `--description`, `--lead`, `--target-date`), **When** the user runs the project update command, **Then** the CLI applies the change to Linear and outputs the updated project's identifier and changed fields.
2. **Given** a valid state transition (e.g., `planned` → `started`), **When** the user runs `linear project update <id> --state started`, **Then** the CLI updates the state and confirms the new state in output.
3. **Given** the user provides `--dry-run`, **When** they run the project update command, **Then** the CLI reports the intended changes without applying them and exits with code 0.
4. **Given** the user provides no update flags, **When** they run the project update command, **Then** the CLI emits a usage error indicating at least one field must be specified, and exits with code 1.
5. **Given** the project does not exist, **When** the user runs the project update command, **Then** the CLI exits with code 1 and a descriptive not-found error on stderr.

---

### User Story 5 - Archive a Project (Priority: P5)

A user wants to archive (soft-close) a project that is no longer active, removing it from active views without permanently deleting its data.

**Why this priority**: Archival is the standard project lifecycle end state in Linear. It is lower priority than CRUD operations but is required for complete lifecycle management.

**Independent Test**: Can be fully tested by running `linear project archive <id>` and confirming the project no longer appears in the default `linear project list` output (which excludes archived projects) — independently verifies archival.

**Acceptance Scenarios**:

1. **Given** a valid project identifier, **When** the user runs `linear project archive <id>`, **Then** the CLI archives the project in Linear and confirms the action.
2. **Given** the user provides `--dry-run`, **When** they run the project archive command, **Then** the CLI reports what would be archived without making any change.
3. **Given** the project is already archived, **When** the user runs the project archive command, **Then** the CLI informs the user it is already archived and exits with code 0.
4. **Given** the project does not exist, **When** the user runs the project archive command, **Then** the CLI exits with code 1 and a descriptive not-found error.

---

### Edge Cases

- When the API is unreachable during any project command, the CLI MUST exit with code 2 and a network error on stderr.
- When the project list spans multiple pages, the CLI MUST either paginate automatically (returning all results) or support `--limit` and `--cursor` flags for manual pagination.
- When a team filter is applied to `project list` (e.g., `--team <id>`), only projects belonging to that team are returned.
- When an update command supplies an invalid state value (not one of the recognised states), the CLI MUST reject it locally before calling the API, exit with code 1, and list valid states in the error message.
- When the user's API key has read-only permissions and a write command is attempted, the CLI MUST surface the GraphQL permission error on stderr and exit with code 2.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST provide a `project list` command that returns all projects accessible to the authenticated user, including name, state, team(s), lead, target date, and progress.
- **FR-002**: The `project list` command MUST support a `--team <id>` filter to restrict results to a specific team.
- **FR-003**: The `project list` command MUST handle pagination transparently, returning all results by default; it MUST also support `--limit <n>` and `--cursor <token>` flags for consumers that prefer manual pagination.
- **FR-004**: The CLI MUST provide a `project get <id>` command that returns the full record for a single project identified by its unique identifier.
- **FR-005**: The CLI MUST provide a `project create` command accepting `--name <string>` (required) and `--team <id>` (required, repeatable for multiple teams), with optional `--description <string>`, `--lead <user-id>`, `--start-date <YYYY-MM-DD>`, and `--target-date <YYYY-MM-DD>` flags.
- **FR-006**: The CLI MUST provide a `project update <id>` command accepting any combination of `--name`, `--description`, `--state`, `--lead`, `--start-date`, and `--target-date` flags; at least one flag MUST be required.
- **FR-007**: The `--state` flag on `project update` MUST only accept the values `planned`, `started`, `paused`, `completed`, and `cancelled`; invalid values MUST be rejected locally before any API call.
- **FR-008**: The CLI MUST provide a `project archive <id>` command that transitions the project to archived state in Linear.
- **FR-009**: Every mutating command (`create`, `update`, `archive`) MUST support a `--dry-run` flag that reports the intended operation without executing it.
- **FR-010**: All `project` commands MUST produce human-readable output when stdout is a TTY and machine-readable JSON when stdout is a pipe or `--output json` is passed.
- **FR-011**: All `project` commands MUST emit errors on stderr and never mix error text with data output on stdout.
- **FR-012**: All `project` commands MUST exit with code 0 on success, code 1 on user/input error, code 2 on API/network error, and code 3 on authentication error.

### Key Entities

- **Project**: A Linear project representing a goal-oriented container for issues; key attributes include identifier, name, description, state, color, icon, start date, target date, progress percentage, lead (user), and associated teams.
- **ProjectState**: The lifecycle state of a project — one of `planned`, `started`, `paused`, `completed`, or `cancelled`.
- **Team**: A Linear team that owns or participates in a project; a project must belong to at least one team.
- **User**: A Linear workspace member who may be assigned as a project lead.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can retrieve a full list of their workspace projects in under 5 seconds on a standard internet connection, regardless of project count (up to Linear API pagination limits).
- **SC-002**: A user can complete the full create-update-archive lifecycle for a single project in under 60 seconds using only CLI commands.
- **SC-003**: All five project commands (`list`, `get`, `create`, `update`, `archive`) produce valid, schema-stable JSON output when `--output json` is passed — verifiable by schema validation in CI.
- **SC-004**: 100% of user-input errors (missing required flags, invalid state values) are caught and reported locally before any API call is made — verifiable by running commands without network access.
- **SC-005**: Dry-run mode for all mutating commands completes without making any state change in Linear — verifiable by comparing Linear project state before and after a `--dry-run` invocation.
- **SC-006**: All project commands correctly report authentication errors (exit code 3) when credentials are absent — verifiable without a Linear account.

## Assumptions

- The authenticated user has at least read access to the projects returned by `project list`; projects outside the user's permissions are not returned by the Linear API and do not require explicit filtering by the CLI.
- Project deletion (permanent, non-recoverable removal) is out of scope for this feature; `archive` is the only end-of-life operation.
- The `project list` command defaults to returning only non-archived projects; a `--include-archived` flag may be added in a future feature but is out of scope here.
- Team identifiers used in `project create` and `project list --team` are Linear team IDs obtainable from a (future) `linear team list` command or via Linear's UI.
- The CLI does not manage project membership (adding/removing members) in this feature; that is a separate concern.
- Progress percentage on a project is computed by Linear and is read-only from the CLI's perspective.
- Start date and target date accept ISO 8601 date format (`YYYY-MM-DD`); time-zone handling follows Linear's API defaults.
