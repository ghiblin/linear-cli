# Feature Specification: Improve Project List Identifiers

**Feature Branch**: `005-improve-project-list`  
**Created**: 2026-05-05  
**Status**: Draft  
**Input**: User description: "when I run `linear project list` it doesn't show the project ID, so I cannot retrieve the identifier I need for the follow up requests. I need to use the `--json` flag, but that produces a lot of output and it's noisy. Furthermore, linear supports a human friendly identifier, like <name>-<short ID>. Improve the project command with these improvements."

## Clarifications

### Session 2026-05-05

- Q: Should `project create` output the new project's slug in its confirmation message? → A: Yes, show slug in create success message (e.g., `Project "Q3 Platform" created (q3-platform)`)
- Q: What should the slug field be called in JSON output? → A: `slug_id`
- Q: Should `project show` detail view display the slug? → A: Yes, include slug as a field in detail output for parity with issue detail view

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Project Slug in Table Output (Priority: P1)

A user runs `linear project list` and wants to see a short, human-readable identifier for each project alongside the name, so they can copy it and use it in follow-up commands without switching to verbose JSON output.

**Why this priority**: This is the core pain point — users cannot retrieve a usable project identifier from the default output. Every follow-up command (show, update, delete) requires an identifier, so this blocks the primary workflow.

**Independent Test**: Run `linear project list` and verify the output table includes a slug column. Can be fully tested in isolation and delivers immediate value by making follow-up commands possible.

**Acceptance Scenarios**:

1. **Given** at least one project exists, **When** the user runs `linear project list`, **Then** the table output includes a column showing the human-readable slug identifier for each project (e.g., `q3-platform`).
2. **Given** the user has copied a slug from `project list` output, **When** they use it in a follow-up command (e.g., `linear project show q3-platform`), **Then** the command resolves the project correctly.
3. **Given** a project with a multi-word name, **When** the user runs `linear project list`, **Then** the slug is displayed in full without truncation.

---

### User Story 2 - Use Slug as Identifier in All Project Commands (Priority: P2)

A user wants to reference a project by its human-readable slug (e.g., `q3-platform`) in any project subcommand, rather than having to look up and copy a UUID.

**Why this priority**: Displaying the slug is only useful if it can actually be used. Without acceptance in commands, users still need to fall back to UUIDs.

**Independent Test**: Run `linear project show <slug>`, `linear project update <slug>`, and `linear project delete <slug>` with a known slug and verify each resolves correctly.

**Acceptance Scenarios**:

1. **Given** a project with slug `q3-platform`, **When** the user runs `linear project show q3-platform`, **Then** the correct project details are displayed.
2. **Given** a project with slug `q3-platform`, **When** the user passes it to any project subcommand, **Then** it behaves identically to passing the UUID.
3. **Given** a non-existent slug, **When** the user passes it to a project subcommand, **Then** a clear "project not found" error is shown.

---

### User Story 3 - Include Slug in JSON Output (Priority: P3)

A user running `linear project list --json` or `linear project show --json` wants the slug identifier to be present in the JSON output so that scripts and integrations can use it without parsing human-readable text.

**Why this priority**: JSON output is used for scripting. Including the slug makes scripted workflows simpler and more readable than UUIDs.

**Independent Test**: Run `linear project list --json` and verify each project object in the output contains a `slug_id` field with the correct value.

**Acceptance Scenarios**:

1. **Given** at least one project exists, **When** the user runs `linear project list --json`, **Then** each project object in the output includes a `slug_id` field.
2. **Given** a project with slug `q3-platform`, **When** the user runs `linear project show q3-platform --json`, **Then** the JSON output includes `"slug_id": "q3-platform"`.

---

### Edge Cases

- What happens when a project's slug is identical to a valid UUID format? The system must resolve it unambiguously (prefer UUID interpretation).
- How does the system handle a slug that no longer exists (deleted project)? Return a clear "not found" error.
- What happens when the project list is empty? The table should show a "no projects found" message with no identifier column required.
- What if two workspaces have projects with the same slug? Slugs are scoped to a workspace; the authenticated workspace context determines resolution.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `project list` table output MUST include a column displaying each project's human-readable slug identifier.
- **FR-002**: The slug column MUST display the full slug value without truncation.
- **FR-003**: All project subcommands (show, update, delete) MUST accept a slug identifier as a valid alternative to a UUID.
- **FR-008**: The `project create` command MUST display the newly created project's slug in its success confirmation message (e.g., `Project "Q3 Platform" created (q3-platform)`).
- **FR-009**: The `project show` detail output MUST display the project's slug as a labelled field alongside other project attributes.
- **FR-004**: When a slug is provided to a project subcommand, the system MUST resolve it to the correct project without requiring the user to know the UUID.
- **FR-005**: The JSON output for project commands MUST include the `slug_id` field in every project object.
- **FR-006**: When both a UUID and a slug are valid inputs, the system MUST resolve them identically to the same project.
- **FR-007**: When a provided slug does not match any project, the system MUST return a clear error message indicating the project was not found.

### Key Entities

- **Project**: Represents a Linear project. Relevant attributes: UUID identifier, human-readable slug (e.g., `q3-platform`), name, state, target date.
- **Project Identifier**: An input value used to reference a project — either a UUID or a slug. The system resolves both to the same project.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify and copy a project's slug from `linear project list` output without using any additional flags.
- **SC-002**: 100% of project subcommands accept slug identifiers, with the same success rate as UUID identifiers.
- **SC-003**: The `project list` table remains readable with the slug column added — no columns are truncated or wrapped in standard terminal widths (80+ characters).
- **SC-004**: JSON output for all project commands includes `slug_id` in every project object, enabling scripts to use slugs without parsing human-readable output.

## Assumptions

- The `slug_id` value is already fetched from the Linear API and stored in the project domain entity; no new API calls are required to obtain it.
- Slug uniqueness is guaranteed within a workspace by the Linear platform; the CLI does not need to validate uniqueness.
- The existing `ProjectId` type already supports slug-based resolution; the work is primarily in surfacing the slug in output and ensuring all commands pass it through.
- UUID format detection (36 chars with dashes) is already implemented and will continue to take precedence over slug interpretation when input matches both patterns.
- Mobile or web UI rendering is out of scope; this feature targets the CLI only.
