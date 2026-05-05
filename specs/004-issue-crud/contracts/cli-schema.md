# CLI Contract: Issue Commands

**Branch**: `004-issue-crud` | **Date**: 2026-05-05

This document defines the stable CLI interface for issue commands. Command signatures follow semver: additions are minor, removals/renames are major.

---

## `linear issue list`

List issues accessible to the authenticated user.

```
linear issue list [OPTIONS]

Options:
  --team <id>           Filter by team UUID or identifier
  --project <id>        Filter by project UUID or slug
  --state <name>        Filter by workflow state name (e.g. "In Progress")
  --assignee <user-id>  Filter by assignee UUID
  --priority <level>    Filter by priority: no_priority | urgent | high | medium | low
  --label <id>          Filter by label UUID (repeatable; issues must carry ALL specified labels)
  --all                 Auto-paginate and return all matching issues
  --limit <n>           Max results per page (default: 50, max: 250)
  --cursor <token>      Pagination cursor for manual paging
  --output <format>     Output format: json (default: human when TTY, json when pipe)
  --verbose / -v        Increase log verbosity on stderr
  --debug               Raw request/response on stderr (implies --verbose)
```

**Exit codes**: 0 success, 1 input error, 2 API/network error, 3 auth error

**Human output** (TTY):
```
ENG-123  Fix login bug         In Progress  High     alice   Platform Team
ENG-124  Update dependencies   Backlog      Medium   —       Platform Team

Showing 2 of 47 issues. Use --all to retrieve all, or --cursor <token> to page.
```

**JSON output** (`--output json` or piped):
```json
{
  "items": [
    {
      "id": "uuid",
      "identifier": "ENG-123",
      "title": "Fix login bug",
      "state": { "id": "uuid", "name": "In Progress", "type": "started" },
      "priority": "high",
      "assignee_id": "uuid",
      "assignee_name": "alice",
      "team_id": "uuid",
      "created_at": "2026-05-01T10:00:00Z",
      "updated_at": "2026-05-04T14:30:00Z"
    }
  ],
  "next_cursor": "cursor-token",
  "has_next_page": true
}
```

---

## `linear issue get <id>`

Retrieve the full record of a single issue.

```
linear issue get <id> [OPTIONS]

Arguments:
  <id>  Issue identifier: display ID (e.g. ENG-123) or Linear UUID

Options:
  --output <format>  Output format: json (default: human when TTY, json when pipe)
  --verbose / -v     Increase log verbosity on stderr
  --debug            Raw request/response on stderr
```

**Exit codes**: 0 success, 1 not found / input error, 2 API/network error, 3 auth error

**JSON output** (`--output json` or piped):
```json
{
  "id": "uuid",
  "identifier": "ENG-123",
  "title": "Fix login bug",
  "description": "Steps to reproduce...",
  "state": { "id": "uuid", "name": "In Progress", "type": "started" },
  "priority": "high",
  "assignee_id": "uuid",
  "assignee_name": "alice",
  "team_id": "uuid",
  "label_ids": ["uuid1", "uuid2"],
  "due_date": "2026-05-15",
  "estimate": 3.0,
  "parent_id": null,
  "parent_title": null,
  "sub_issues": [
    { "id": "uuid", "identifier": "ENG-456", "title": "Write tests" }
  ],
  "created_at": "2026-05-01T10:00:00Z",
  "updated_at": "2026-05-04T14:30:00Z"
}
```

---

## `linear issue create`

Create a new issue (or sub-issue when `--parent` is supplied).

```
linear issue create [OPTIONS]

Required:
  --title <string>        Issue title
  --team <id>             Team UUID or identifier

Required unless LINEAR_PROJECT_ID is set:
  --project <id>          Project UUID or slug

Optional:
  --description <string>  Issue description (Markdown)
  --priority <level>      no_priority | urgent | high | medium | low
  --assignee <user-id>    Assignee UUID
  --label <id>            Label UUID (repeatable)
  --due-date <YYYY-MM-DD> Due date
  --estimate <number>     Story point or time estimate (non-negative)
  --parent <issue-id>     Parent issue ID (UUID or display ID) — creates a sub-issue
  --dry-run               Report intended operation without executing; exit 0
  --output <format>       Output format: json (default: human when TTY, json when pipe)
  --verbose / -v          Increase log verbosity on stderr
  --debug                 Raw request/response on stderr

Environment:
  LINEAR_PROJECT_ID       Default project ID when --project is not supplied
```

**Exit codes**: 0 success, 1 input/validation error, 2 API/network/permission error, 3 auth error

**Human output** (success):
```
Created issue ENG-125: Fix login bug
```

**Human output** (dry-run):
```
[dry-run] Would create issue:
  Title:    Fix login bug
  Team:     Platform Team (ENG)
  Project:  Q2 Roadmap
  Priority: high
  Parent:   ENG-123 (Fix login bug)
```

**JSON output** (`--output json` or piped) — success:
```json
{
  "id": "uuid",
  "identifier": "ENG-125",
  "title": "Fix login bug",
  "state": { "id": "uuid", "name": "Backlog", "type": "backlog" },
  "priority": "high",
  "team_id": "uuid",
  "parent_id": "uuid",
  "created_at": "2026-05-05T09:00:00Z",
  "updated_at": "2026-05-05T09:00:00Z"
}
```

---

## `linear issue update <id>`

Modify one or more attributes of an existing issue. At least one update flag required.

```
linear issue update <id> [OPTIONS]

Arguments:
  <id>  Issue identifier: display ID (e.g. ENG-123) or Linear UUID

Options:
  --title <string>        New title
  --description <string>  New description (Markdown)
  --state <name>          New workflow state name (validated against team's states)
  --priority <level>      no_priority | urgent | high | medium | low
  --assignee <user-id>    New assignee UUID
  --due-date <YYYY-MM-DD> New due date
  --estimate <number>     New estimate (non-negative)
  --parent <issue-id>     Set or change parent issue (UUID or display ID)
  --no-parent             Detach from parent, promoting to top-level issue
  --dry-run               Report intended changes without applying; exit 0
  --output <format>       Output format: json (default: human when TTY, json when pipe)
  --verbose / -v          Increase log verbosity on stderr
  --debug                 Raw request/response on stderr
```

**Mutual exclusion**: `--parent` and `--no-parent` cannot be used together → exit code 1

**State validation**: The CLI fetches the issue's team's workflow states, validates the supplied `--state` value (case-insensitive), then submits the mutation with the state UUID. Invalid state → exit code 1, lists valid state names in error.

**Exit codes**: 0 success, 1 input/validation error, 2 API/network/permission error, 3 auth error

**Human output** (success):
```
Updated ENG-123: state → "In Progress"
```

**Human output** (dry-run):
```
[dry-run] Would update ENG-123:
  state:    Backlog → In Progress
  priority: medium → high
```

**JSON output** (`--output json` or piped) — success: same schema as `issue get`

---

## Error Output Contract

All errors are emitted on **stderr**. stdout remains clean for data.

```
Error: issue "ENG-999" not found
Error: --state "Shipped" is not a valid state for team ENG; valid states: Backlog, Todo, In Progress, In Review, Done, Cancelled
Error: --parent and --no-parent are mutually exclusive
Error: --title is required
Error: network error: connection refused
Error: not authenticated; run `linear auth login`
```

---

## `LINEAR_PROJECT_ID` Environment Variable

When set, serves as the default `--project` value for `linear issue create`. Explicit `--project` flag always takes precedence.

```bash
export LINEAR_PROJECT_ID="proj-uuid"
linear issue create --title "New task" --team ENG   # uses LINEAR_PROJECT_ID
linear issue create --title "New task" --team ENG --project other-uuid  # overrides
```
