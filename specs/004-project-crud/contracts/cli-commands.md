# CLI Command Contracts: Project CRUD

**Branch**: `004-project-crud` | **Date**: 2026-04-23

These contracts define the stable interface for all `linear project` subcommands. Changes to arguments, flags, or output schemas are **breaking** and require a major version bump per the constitution's AI Agent Interaction Contract.

---

## `linear project list`

**Purpose**: List projects accessible to the authenticated user.

### Arguments & Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--team <id>` | `String` (UUID) | — | Filter to projects belonging to this team |
| `--limit <n>` | `u32` | 50 | Max results per page |
| `--cursor <token>` | `String` | — | Pagination cursor from a prior response |
| `--all` | `bool` | false | Auto-fetch all pages (ignores `--limit`/`--cursor`) |
| `--output json` | `bool` | false | Force JSON output |
| `--verbose` / `-v` | `bool` | false | Emit step-by-step progress to stderr |
| `--debug` | `bool` | false | Emit raw GraphQL request+response to stderr; implies `--verbose` |

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success (including empty result set) |
| 1 | User/input error (invalid flag values) |
| 2 | API/network error (after retries exhausted) |
| 3 | Authentication error |

### Stdout (human mode, TTY)

```
Projects (3):
  PRJ-1  Alpha Initiative        started    2026-06-01
  PRJ-2  Beta Platform           planned    —
  PRJ-3  Gamma Research          completed  2026-03-15

Page 1 of 2 — run with --cursor <token> for next page
```

### Stdout (JSON mode)

```json
{
  "projects": [
    {
      "id": "9cfb482a-81e3-4154-b5b9-2c805e70a02d",
      "name": "Alpha Initiative",
      "description": null,
      "state": "started",
      "progress": 45.0,
      "lead_id": null,
      "team_ids": ["team-uuid-1"],
      "start_date": null,
      "target_date": "2026-06-01",
      "updated_at": "2026-04-20T10:00:00Z"
    }
  ],
  "page_info": {
    "has_next_page": true,
    "end_cursor": "cursor-token-xyz"
  }
}
```

### Empty result (JSON)

```json
{
  "projects": [],
  "page_info": {
    "has_next_page": false,
    "end_cursor": null
  }
}
```

---

## `linear project get <id>`

**Purpose**: Retrieve full details of a single project.

### Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `<id>` | `String` | Project UUID or display ID (e.g., `PRJ-1`) |

### Flags

| Flag | Description |
|------|-------------|
| `--output json` | Force JSON output |
| `--verbose` / `-v` | Emit progress to stderr |
| `--debug` | Emit raw GraphQL to stderr |

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success |
| 1 | Project not found or no access |
| 2 | API/network error |
| 3 | Authentication error |

### Stdout (JSON)

```json
{
  "id": "9cfb482a-81e3-4154-b5b9-2c805e70a02d",
  "name": "Alpha Initiative",
  "description": "Strategic initiative for Q2",
  "state": "started",
  "progress": 45.0,
  "lead_id": "user-uuid-abc",
  "team_ids": ["team-uuid-1", "team-uuid-2"],
  "start_date": "2026-01-15",
  "target_date": "2026-06-01",
  "updated_at": "2026-04-20T10:00:00Z"
}
```

---

## `linear project create`

**Purpose**: Create a new project in the Linear workspace.

### Flags

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--name <string>` | `String` | Yes | Project name |
| `--team <id>` | `String` (UUID) | Yes (repeatable) | Team to associate; use multiple times for multiple teams |
| `--description <string>` | `String` | No | Project description |
| `--lead <user-id>` | `String` (UUID) | No | Project lead user ID |
| `--start-date <YYYY-MM-DD>` | `String` | No | ISO 8601 start date |
| `--target-date <YYYY-MM-DD>` | `String` | No | ISO 8601 target date |
| `--dry-run` | `bool` | No | Report intended action without API call |
| `--output json` | `bool` | No | Force JSON output |
| `--verbose` / `-v` | `bool` | No | Emit progress to stderr |
| `--debug` | `bool` | No | Emit raw GraphQL to stderr |

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success |
| 1 | Missing required flags, invalid input (caught locally) |
| 2 | API/network error (team not found, permission denied, retries exhausted) |
| 3 | Authentication error |

### Stdout (human mode)

```
Created project: PRJ-5 "My New Project"
```

### Stdout (JSON)

```json
{
  "id": "new-project-uuid",
  "name": "My New Project",
  "state": "planned"
}
```

### Dry-run output (human)

```
[dry-run] Would create project:
  name:        My New Project
  team(s):     team-uuid-1
  description: (none)
  lead:        (none)
  start date:  (none)
  target date: 2026-12-31
```

### Dry-run output (JSON)

```json
{
  "dry_run": true,
  "action": "create",
  "input": {
    "name": "My New Project",
    "team_ids": ["team-uuid-1"],
    "description": null,
    "lead_id": null,
    "start_date": null,
    "target_date": "2026-12-31"
  }
}
```

---

## `linear project update <id>`

**Purpose**: Modify one or more attributes of an existing project.

### Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `<id>` | `String` | Project UUID or display ID |

### Flags

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--name <string>` | `String` | No* | New project name |
| `--description <string>` | `String` | No* | New description |
| `--state <value>` | `String` | No* | One of: `planned`, `started`, `paused`, `completed`, `cancelled` |
| `--lead <user-id>` | `String` | No* | New lead user ID |
| `--start-date <YYYY-MM-DD>` | `String` | No* | New start date |
| `--target-date <YYYY-MM-DD>` | `String` | No* | New target date |
| `--dry-run` | `bool` | No | Report without applying |
| `--output json` | `bool` | No | Force JSON output |
| `--verbose` / `-v` | `bool` | No | Emit progress to stderr |
| `--debug` | `bool` | No | Emit raw GraphQL to stderr |

*At least one update flag must be provided; CLI rejects the call with exit code 1 if none are given.

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success |
| 1 | No update flags provided; invalid state value; project not found |
| 2 | API/network error |
| 3 | Authentication error |

### Stdout (human)

```
Updated project PRJ-1: state → started
```

### Stdout (JSON)

```json
{
  "id": "project-uuid",
  "name": "Alpha Initiative",
  "state": "started"
}
```

---

## `linear project archive <id>`

**Purpose**: Archive a project (soft-close; removes from active views).

### Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `<id>` | `String` | Project UUID or display ID |

### Flags

| Flag | Description |
|------|-------------|
| `--dry-run` | Report without applying |
| `--output json` | Force JSON output |
| `--verbose` / `-v` | Emit progress to stderr |
| `--debug` | Emit raw GraphQL to stderr |

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success (including already-archived case) |
| 1 | Project not found |
| 2 | API/network error |
| 3 | Authentication error |

### Stdout (human)

```
Archived project PRJ-1 "Alpha Initiative"
```

(If already archived):

```
Project PRJ-1 is already archived.
```

### Stdout (JSON)

```json
{
  "success": true,
  "id": "project-uuid"
}
```

### Dry-run output

```
[dry-run] Would archive project PRJ-1 "Alpha Initiative"
```
