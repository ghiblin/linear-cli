# CLI Contract: Project Commands

**Branch**: `005-improve-project-list` | **Date**: 2026-05-05

This document defines the stable output contracts for all `linear project` subcommands after this feature is implemented.

---

## `linear project list`

### Human output (TTY)

```
Projects (N):
  <name:35>                  <slug:22>              <state:12>    <target_date>
```

Example:
```
Projects (2):
  Q3 Platform                q3-platform            started       2026-09-30
  Infrastructure Hardening   infra-hardening        planned       —
```

Column widths: name `{:<35}`, slug `{:<22}`, state `{:<12}`, target date unrestricted.
Slug displayed in full — never truncated.

### JSON output (`--output json` or `--json`)

```json
{
  "projects": [
    {
      "id": "9cfb482a-81e3-4154-b5b9-2c805e70a02d",
      "slug_id": "q3-platform",
      "name": "Q3 Platform",
      "description": "Q3 objectives",
      "state": "started",
      "progress": 42.0,
      "lead_id": null,
      "team_ids": ["team-uuid"],
      "start_date": "2026-07-01",
      "target_date": "2026-09-30",
      "updated_at": "2026-05-01T10:00:00Z"
    }
  ],
  "page_info": {
    "has_next_page": false,
    "end_cursor": null
  }
}
```

---

## `linear project get <id>`

Accepts: UUID or slug (e.g. `q3-platform`).

### Human output (TTY)

```
Name:        Q3 Platform
Slug:        q3-platform
ID:          9cfb482a-81e3-4154-b5b9-2c805e70a02d
State:       started
Progress:    42.0%
Description: Q3 objectives
Lead:        user-uuid
Teams:       team-uuid
Target date: 2026-09-30
Updated:     2026-05-01T10:00:00Z
```

`Slug:` appears as the second line, immediately after `Name:`.

### JSON output

```json
{
  "id": "9cfb482a-81e3-4154-b5b9-2c805e70a02d",
  "slug_id": "q3-platform",
  "name": "Q3 Platform",
  "description": "Q3 objectives",
  "state": "started",
  "progress": 42.0,
  "lead_id": "user-uuid",
  "team_ids": ["team-uuid"],
  "start_date": null,
  "target_date": "2026-09-30",
  "updated_at": "2026-05-01T10:00:00Z"
}
```

---

## `linear project create --name <name> --team <team-id>`

### Human output (TTY) — success

```
Created project: "Q3 Platform" (q3-platform)
```

Slug replaces the UUID that was previously shown in parentheses.

### JSON output — success

```json
{
  "id": "9cfb482a-81e3-4154-b5b9-2c805e70a02d",
  "slug_id": "q3-platform",
  "name": "Q3 Platform",
  "state": "planned"
}
```

---

## `linear project update <id> [flags]`

Accepts: UUID or slug.

### Human output (TTY) — success

```
Updated project q3-platform: state → started
```

Slug replaces the UUID previously shown.

### JSON output — success

```json
{
  "id": "9cfb482a-81e3-4154-b5b9-2c805e70a02d",
  "slug_id": "q3-platform",
  "name": "Q3 Platform",
  "state": "started"
}
```

---

## `linear project archive <id>`

Accepts: UUID or slug. Output unchanged (no slug displayed — archive is a terminal operation).

---

## Error contract (unchanged)

- Project not found (slug or UUID): `error: project 'q3-platform' not found` on stderr, exit 1.
- Invalid identifier format: `error: unrecognised project id '...'; expected a UUID or a slug (e.g. 'q3-platform')` on stderr, exit 1.
