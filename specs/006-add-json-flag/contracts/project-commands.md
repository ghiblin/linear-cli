# CLI Contract: Project Commands

**Feature**: 006-add-json-flag  
**Scope**: `linear project` subcommands

## Flag Contract

All five project subcommands (`list`, `get`, `create`, `update`, `archive`) accept the following output flags:

| Flag | Type | Description |
|------|------|-------------|
| `--output <FORMAT>` | `json` \| `human` | Explicit output format selection |
| `--json` | boolean | Shorthand for `--output json`; takes precedence over `--output human` |

**Precedence rule**: `--json` OR `--output json` → JSON output. If both conflict (`--json --output human`), `--json` wins.

**Default behavior**: JSON when stdout is not a TTY (piped); human-readable otherwise.

## Command Signatures

```
linear project list [--output <FORMAT>] [--json] [--team <ID>] [--limit <N>]
linear project get <ID> [--output <FORMAT>] [--json]
linear project create --name <NAME> --team <TEAM> [--output <FORMAT>] [--json] [--dry-run] ...
linear project update <ID> [--output <FORMAT>] [--json] [--dry-run] ...
linear project archive <ID> [--output <FORMAT>] [--json] [--dry-run]
```

## JSON Output Schemas

No schema changes. Both `--json` and `--output json` produce the same JSON as the existing `--output json` flag.
