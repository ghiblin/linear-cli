# CLI Contract: Team Commands

**Feature**: 006-add-json-flag  
**Scope**: `linear team` subcommands

## Flag Contract

`team list` currently relies on the global `--json` flag (placed before the subcommand). This feature adds per-command `--json` and `--output` flags so the flag can appear after the subcommand.

| Flag | Type | Description |
|------|------|-------------|
| `--output <FORMAT>` | `json` \| `human` | Explicit output format selection (new) |
| `--json` | boolean | Shorthand for `--output json` (moved to per-command) |

**Precedence rule**: Same as issue/project commands.

## Command Signatures

```
linear team list [--output <FORMAT>] [--json]
```

## JSON Output Schema

No schema changes. `--json` and `--output json` produce the same JSON as the existing global `--json` flag behavior.
