# CLI Contract: Issue Commands

**Feature**: 006-add-json-flag  
**Scope**: `linear issue` subcommands

## Flag Contract

All four issue subcommands (`list`, `get`, `create`, `update`) accept the following output flags:

| Flag | Type | Description |
|------|------|-------------|
| `--output <FORMAT>` | `json` \| `human` | Explicit output format selection |
| `--json` | boolean | Shorthand for `--output json`; takes precedence over `--output human` |

**Precedence rule**: `--json` OR `--output json` → JSON output. If both conflict (`--json --output human`), `--json` wins.

**Default behavior**: JSON when stdout is not a TTY (piped); human-readable otherwise.

## Command Signatures

```
linear issue list [--output <FORMAT>] [--json] [--team <ID>] [--state <STATE>] [--limit <N>] [--dry-run]
linear issue get <ID> [--output <FORMAT>] [--json]
linear issue create --title <TITLE> --team <TEAM> [--output <FORMAT>] [--json] [--dry-run] ...
linear issue update <ID> [--output <FORMAT>] [--json] [--dry-run] ...
```

## JSON Output Schemas

### `issue list` / `issue get`

Both flags produce the same schema as the existing `--output json` output (no schema change).

### `issue create` / `issue update`

Same schema as existing `--output json`. Dry-run also respects `--json`.
