# Feature Specification: Issue Get — Optional Detail Flags

**Feature Branch**: `009-issue-get-details`  
**Created**: 2026-05-07  
**Status**: Draft  
**Input**: User description: "issue get returns basic details. Now I want to specify more, like descriptions and the list of subtasks, each of them should be triggered with a flag"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Issue Description (Priority: P1)

A user wants to read the full description of an issue from the terminal without opening the Linear web interface. They pass `--description` to `issue get` and the description text is appended to the standard output.

**Why this priority**: Description is the most informative field on an issue and the most commonly needed detail beyond the summary line.

**Independent Test**: Running `linear issue get <id> --description` prints the description field below the basic details. Running without the flag omits it.

**Acceptance Scenarios**:

1. **Given** an issue with a non-empty description, **When** the user runs `linear issue get <id> --description`, **Then** the description text is printed after the standard fields.
2. **Given** an issue with no description, **When** the user runs `linear issue get <id> --description`, **Then** the command completes successfully and no description line is printed.
3. **Given** `--description` is not passed, **When** the user runs `linear issue get <id>`, **Then** no description text appears in the output.

---

### User Story 2 - View Issue Subtasks (Priority: P2)

A user wants to see the breakdown of an issue into its child tasks directly from the CLI. They pass `--subtasks` to `issue get` and the list of sub-issues is shown below the basic details.

**Why this priority**: Seeing subtasks requires a separate look-up today; making it opt-in keeps default output concise while offering depth on demand.

**Independent Test**: Running `linear issue get <id> --subtasks` prints the sub-issue list. Running without the flag omits it.

**Acceptance Scenarios**:

1. **Given** an issue with one or more sub-issues, **When** the user runs `linear issue get <id> --subtasks`, **Then** each sub-issue identifier and title is listed.
2. **Given** an issue with no sub-issues, **When** the user runs `linear issue get <id> --subtasks`, **Then** the command completes successfully and no subtask list is printed.
3. **Given** `--subtasks` is not passed, **When** the user runs `linear issue get <id>`, **Then** no sub-issue list appears in the output.

---

### User Story 3 - Combine Both Flags (Priority: P3)

A user wants a comprehensive view of an issue in one call. They pass both `--description` and `--subtasks` together and receive the full detail output.

**Why this priority**: Common workflow; both flags must be composable.

**Independent Test**: Running `linear issue get <id> --description --subtasks` shows both the description and the sub-issue list in the same output.

**Acceptance Scenarios**:

1. **Given** an issue with both description and sub-issues, **When** the user runs `linear issue get <id> --description --subtasks`, **Then** both sections appear in the output.

---

### Edge Cases

- What happens when the issue has a very long description (multiple paragraphs)?
- What happens when `--description` or `--subtasks` is combined with `--json`?
- What happens when the issue ID is invalid or the issue does not exist?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `issue get` MUST accept a `--description` flag that, when present, prints the issue's description text in human-readable output.
- **FR-002**: `issue get` MUST accept a `--subtasks` flag that, when present, prints the list of child issues in human-readable output.
- **FR-003**: Without `--description`, the description field MUST NOT appear in human-readable output.
- **FR-004**: Without `--subtasks`, the sub-issue list MUST NOT appear in human-readable output.
- **FR-005**: Both `--description` and `--subtasks` MUST be combinable in a single invocation.
- **FR-006**: JSON output (via `--json` / `--output json`) MUST remain unchanged — description and sub-issues are already included in the JSON payload and are unaffected by these flags.
- **FR-007**: When `--description` is passed but the issue has no description, the command MUST succeed silently without printing an empty or placeholder line.
- **FR-008**: When `--subtasks` is passed but the issue has no sub-issues, the command MUST succeed silently without printing an empty list.

### Key Entities

- **Issue**: Has a title, state, priority, and optional fields: description text and a list of sub-issues.
- **Sub-issue**: A child issue with its own identifier and title.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can read the full issue description from the terminal with a single command, without visiting the web interface.
- **SC-002**: Users can see all subtasks of an issue in under 3 seconds on a normal connection.
- **SC-003**: Default `issue get` output remains unchanged — no regression for users not using the new flags.
- **SC-004**: Both flags work independently and in combination without errors.

## Assumptions

- The underlying API already returns description and sub-issues in the same query used by `issue get`; no additional network call is needed.
- Sub-issues currently shown unconditionally in human output will be made opt-in (behind `--subtasks`) as part of this change.
- JSON output shape is not modified — `--description` and `--subtasks` are human-output-only flags.
- The user is already authenticated before running the command.
