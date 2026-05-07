# Feature Specification: Delete Issue

**Feature Branch**: `008-delete-issue`  
**Created**: 2026-05-07  
**Status**: Draft  
**Input**: User description: "Is it possible to delete an issue?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Delete an Issue by ID (Priority: P1)

A user wants to permanently remove an issue from their Linear workspace via the CLI. They provide the issue identifier and the issue is deleted, disappearing from their workspace after the grace period.

**Why this priority**: Core feature — without this, delete is unusable.

**Independent Test**: Can be fully tested by running `linear issue delete <id>` and verifying the issue no longer appears in `linear issue list`.

**Acceptance Scenarios**:

1. **Given** a valid issue ID, **When** the user runs `linear issue delete <id>`, **Then** the issue is deleted and a success confirmation is printed.
2. **Given** a valid issue ID, **When** the user runs `linear issue delete <id> --json`, **Then** the result is printed as JSON.
3. **Given** an invalid or non-existent issue ID, **When** the user runs `linear issue delete <id>`, **Then** an error message is printed and the command exits with a non-zero code.

---

### User Story 2 - Dry-Run Delete (Priority: P2)

A user wants to preview what would be deleted before committing, using a `--dry-run` flag consistent with other mutating commands in the CLI.

**Why this priority**: Reduces risk of accidental deletion; consistent with `issue create` and `issue update` patterns already in the CLI.

**Independent Test**: Running `linear issue delete <id> --dry-run` prints a preview and exits without contacting the API.

**Acceptance Scenarios**:

1. **Given** a valid issue ID, **When** the user runs `linear issue delete <id> --dry-run`, **Then** the command prints what would be deleted without making any changes.
2. **Given** `--dry-run --json`, **When** the user runs the command, **Then** the dry-run result is printed as JSON.

---

### Edge Cases

- What happens when the issue ID is syntactically invalid?
- What happens when the authenticated user lacks permission to delete the issue?
- What happens when the issue has sub-issues — are they also deleted?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST expose a `delete` subcommand under `issue` (e.g., `linear issue delete <id>`).
- **FR-002**: The command MUST accept a required positional argument: the issue ID.
- **FR-003**: The command MUST support a `--dry-run` flag that previews the action without modifying data.
- **FR-004**: The command MUST support `--json` / `--output json` flags consistent with other issue commands.
- **FR-005**: On success, the command MUST print a human-readable confirmation (or JSON result when requested).
- **FR-006**: On error (invalid ID, permission denied, network failure), the command MUST print a clear error message and exit with a non-zero status code.

### Key Entities

- **Issue**: Identified by its ID; deleted via the Linear platform's delete operation, which enters a grace period before permanent removal.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can delete an issue with a single command in under 3 seconds on a normal connection.
- **SC-002**: Dry-run mode completes instantly without network calls.
- **SC-003**: Error messages are clear enough that users can self-correct without consulting documentation.
- **SC-004**: JSON output is machine-parseable and consistent in structure with other issue commands.

## Assumptions

- The underlying platform supports issue deletion with a grace period — confirmed via schema (`issueDelete` mutation exists).
- The `permanently_delete` admin-only option is out of scope for this feature; only standard deletion is supported.
- Sub-issue behavior on parent deletion is governed by the platform and not controlled by this CLI.
- The user is already authenticated before running the command (consistent with all other commands).
- Issue ID format validation follows the same rules as other issue commands in the codebase.
