# Feature Specification: Filter by Partial Title

**Feature Branch**: `007-filter-partial-title`  
**Created**: 2026-05-06  
**Status**: Draft  
**Input**: User description: "filter query by partial title"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find Issues by Keyword (Priority: P1)

A user wants to narrow down a long list of issues to those whose title contains a specific keyword or phrase. Instead of scrolling through dozens of results, they provide a partial title string and only matching issues are shown.

**Why this priority**: Workspaces accumulate hundreds of issues; filtering by keyword is the most natural and frequent way users locate specific work items from the command line.

**Independent Test**: Can be fully tested by running `linear issue list --title <keyword>` and verifying that only issues whose title contains `<keyword>` (case-insensitively) appear in the output.

**Acceptance Scenarios**:

1. **Given** a workspace with issues titled "Fix login bug", "Add login page", and "Update footer", **When** the user runs `issue list --title "login"`, **Then** only the two issues containing "login" are returned.
2. **Given** a workspace with issues, **When** the user runs `issue list --title "LOGIN"` (uppercase), **Then** the same issues matching "login" (case-insensitively) are returned.
3. **Given** a workspace with no issues matching the supplied string, **When** the user runs `issue list --title "xyznonexistent"`, **Then** an empty result set is returned with a message indicating zero issues found.

---

### User Story 2 - Find Projects by Partial Name (Priority: P2)

A user wants to filter the project list by a partial project name, following the same pattern as issue title filtering.

**Why this priority**: Projects accumulate over time; users scripting against the CLI often need to locate a project by a known substring of its name rather than its exact ID or slug.

**Independent Test**: Can be fully tested by running `linear project list --name <keyword>` and verifying only matching projects appear.

**Acceptance Scenarios**:

1. **Given** projects named "Platform Migration", "Platform Audit", and "Frontend Redesign", **When** the user runs `project list --name "Platform"`, **Then** only the two "Platform" projects are returned.
2. **Given** a project list, **When** `--name` is combined with `--team`, **Then** the results are filtered by both criteria simultaneously.

---

### User Story 3 - Combine Title Filter with Existing Filters (Priority: P3)

A user wants to combine the new partial-title filter with the existing filter flags (team, state, assignee, priority, labels) to achieve precise results.

**Why this priority**: Compound filtering is a natural progression once single-field filtering exists, and is essential for scripting workflows.

**Independent Test**: Can be fully tested by combining `--title` with `--state` or `--team` and verifying only issues matching all supplied criteria appear.

**Acceptance Scenarios**:

1. **Given** issues across multiple states, **When** the user runs `issue list --title "auth" --state "in_progress"`, **Then** only in-progress issues whose title contains "auth" are returned.
2. **Given** issues across multiple teams, **When** the user runs `issue list --title "deploy" --team <team-id>`, **Then** only issues on that team whose title contains "deploy" are returned.

---

### Edge Cases

- What happens when `--title ""` (empty string) is supplied? The filter is treated as unset (no title filtering applied) and all results matching other filters are returned.
- How does the system handle titles containing special characters (parentheses, slashes)? The substring match is performed on the literal characters supplied.
- What if `--title` is supplied alongside `--all`? All pages are fetched and the title filter is applied to the full result set.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `issue list` command MUST accept a `--title` flag that accepts a string value.
- **FR-002**: When `--title` is supplied, the command MUST return only issues whose title contains the supplied string.
- **FR-003**: Title matching MUST be case-insensitive.
- **FR-004**: The `project list` command MUST accept a `--name` flag that accepts a string value.
- **FR-005**: When `--name` is supplied, the command MUST return only projects whose name contains the supplied string.
- **FR-006**: Project name matching MUST be case-insensitive.
- **FR-007**: Both `--title` and `--name` flags MUST compose correctly with all existing filter flags.
- **FR-008**: When no results match the supplied substring, the command MUST produce an empty result set (zero items) with a user-readable message, not an error.
- **FR-009**: When `--title` or `--name` is omitted or given an empty value, the filter MUST have no effect on results.
- **FR-010**: The `--json` output for filtered results MUST follow the same schema as unfiltered results.

### Key Entities

- **Issue**: Has a `title` field that is the searchable text for issue filtering.
- **Project**: Has a `name` field that is the searchable text for project filtering.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can locate issues by partial title using a single command invocation without post-processing shell pipelines.
- **SC-002**: Users can locate projects by partial name using a single command invocation.
- **SC-003**: Filtered results appear in under 3 seconds for typical workspaces (up to 500 issues/projects).
- **SC-004**: Combining `--title` with any existing issue filter flag produces correct compound-filtered results in 100% of tested cases.
- **SC-005**: Zero issues matching the substring produces a clear, non-error output in both human and JSON modes.

## Assumptions

- The matching behaviour (case-insensitive substring/contains) is implemented consistently whether filtering happens at the API level or client-side; the user experience is identical either way.
- The `--name` flag name for `project list` aligns with the project's existing use of "name" (not "title") for project identifiers.
- Team `list` is out of scope for this feature; team names are short and the team list is typically small.
- Filtering is purely read-only; no mutations are involved.
- Users are expected to supply non-empty, meaningful substrings; very short strings (e.g., one character) that match hundreds of results are valid and handled without error.
