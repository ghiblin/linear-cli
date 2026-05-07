# CLI Contract: `issue get` Detail Flags

## Command Signature

```
linear issue get <id> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Issue UUID or display identifier (e.g. `ENG-42`) |

### Options (all existing, unchanged)

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--output <format>` | String | human | Output format (`json` or omit for human) |
| `--json` | bool | false | Alias for `--output json` |

### New Options (this feature)

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--description` | bool | false | Print the issue description below basic fields |
| `--subtasks` | bool | false | Print the list of child issues below basic fields |

## Human Output

### Default (no flags)

```
Identifier: ENG-42
Title:      Implement login
State:      In Progress
Priority:   high
Assignee:   Alice
Due date:   2026-06-01
Estimate:   3
Parent:     uuid-of-parent (Implement auth)
```

### With `--description`

```
Identifier:   ENG-42
Title:        Implement login
State:        In Progress
Priority:     high
Description:
  Implement the login flow using the existing auth service.
  Supports email/password only in v1.
```

### With `--subtasks`

```
Identifier: ENG-42
Title:      Implement login
State:      In Progress
Priority:   high
Sub-issues:
  ENG-43 — Add login form
  ENG-44 — Wire up auth service
```

### With both flags

```
Identifier:   ENG-42
Title:        Implement login
State:        In Progress
Priority:     high
Description:
  Implement the login flow using the existing auth service.
Sub-issues:
  ENG-43 — Add login form
  ENG-44 — Wire up auth service
```

## JSON Output (unchanged)

`--json` output is unaffected by `--description` and `--subtasks`. The JSON payload already includes `description` and `sub_issues` unconditionally.

```json
{
  "id": "uuid",
  "identifier": "ENG-42",
  "title": "Implement login",
  "description": "...",
  "state": { "id": "...", "name": "In Progress", "type": "started" },
  "priority": "high",
  "sub_issues": [
    { "id": "uuid", "identifier": "ENG-43", "title": "Add login form" }
  ],
  ...
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User/input error (invalid ID format) |
| 2 | API/network error |
| 3 | Auth error |

Exit codes are unchanged by this feature.
