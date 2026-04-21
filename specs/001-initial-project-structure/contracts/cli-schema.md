# CLI Contract: Initial Command Schema

**Phase**: 1 — Design  
**Feature**: 001-initial-project-structure  
**Date**: 2026-04-21  
**Version**: 0.1.0 (skeleton)

## Overview

This document defines the stable contract for the `linear` CLI binary. All commands, flags, exit codes, and output schemas defined here are versioned under semver. Additions are minor bumps; removals or renames are major bumps.

---

## Global Flags

These flags are available on all commands.

| Flag | Short | Description |
| --- | --- | --- |
| `--json` | — | Force JSON output regardless of TTY detection |
| `--verbose` | `-v` | Increase log verbosity (repeatable: `-v`, `-vv`, `-vvv`) |
| `--version` | — | Output version information as JSON and exit |
| `--help` | `-h` | Display help for the current command |

---

## Output Mode

- **TTY detected** (interactive terminal): human-readable table/text output on stdout.
- **No TTY** (pipe or redirect): JSON output on stdout by default.
- `--json` flag overrides TTY detection and always produces JSON.
- Errors and diagnostic messages are always written to **stderr**, never stdout.

---

## Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | User or input error (bad arguments, missing required flag) |
| `2` | API or network error (Linear API unreachable, unexpected response) |
| `3` | Authentication error (missing or invalid API key) |

---

## `linear --version`

Outputs version information and exits with code 0.

**JSON schema** (stdout):

```json
{
  "version": "0.1.0",
  "api_schema": "YYYY-MM-DD"
}
```

| Field | Type | Description |
| --- | --- | --- |
| `version` | string | Semver version of the CLI binary |
| `api_schema` | string | ISO-8601 date of the vendored Linear GraphQL schema |

---

## `linear --schema`

Outputs the JSON output schema for all commands and exits with code 0.

**JSON schema** (stdout):

```json
{
  "schema_version": "1",
  "commands": {
    "<command>": {
      "description": "string",
      "output_schema": {}
    }
  }
}
```

---

## `linear issue` — Issue Commands

### `linear issue list`

List issues, optionally filtered by team.

**Flags**:

| Flag | Type | Required | Description |
| --- | --- | --- | --- |
| `--team` | string | No | Filter by team key or ID |
| `--json` | — | No | Force JSON output |

**JSON output schema** (stdout, array):

```json
[
  {
    "id": "string",
    "title": "string",
    "state": "string",
    "priority": "string",
    "team_id": "string"
  }
]
```

**Exit codes**: 0 (success), 1 (bad arguments), 2 (API error), 3 (auth error).

---

### `linear issue get <id>`

Fetch a single issue by identifier.

**Arguments**:

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `id` | string | Yes | Issue identifier (e.g., `ENG-123`) |

**JSON output schema** (stdout):

```json
{
  "id": "string",
  "title": "string",
  "state": "string",
  "priority": "string",
  "team_id": "string"
}
```

**Exit codes**: 0 (found), 1 (bad arguments), 2 (API error), 3 (auth error).

---

## `linear team` — Team Commands

### `linear team list`

List all accessible teams.

**JSON output schema** (stdout, array):

```json
[
  {
    "id": "string",
    "name": "string",
    "key": "string"
  }
]
```

---

## Composability

All commands produce newline-delimited JSON objects when piped. Example composition:

```sh
linear team list | jq '.[0].id' | xargs linear issue list --team
```

---

## Stability Guarantees (Skeleton)

Fields listed in this document will not be removed without a major version bump. Additional fields may be added in minor versions. All commands listed are stubs in the initial skeleton and return placeholder responses; real implementations will be added in subsequent features without changing this contract.

---

## Future Commands (Out of Scope for Skeleton)

The following are planned but not included in the skeleton:

- `linear project list` / `linear project get`
- `linear cycle list`
- `linear issue create` / `linear issue update` (with `--dry-run`)
- `linear auth login` / `linear auth logout`
