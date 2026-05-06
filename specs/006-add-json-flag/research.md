# Research: JSON Output Shorthand Flag

**Feature**: 006-add-json-flag  
**Date**: 2026-05-06

## Decision 1: Per-Command vs. Global-Only `--json` Flag

**Decision**: Add `--json` as a per-command `bool` field on each subcommand's `Args` struct.

**Rationale**: The root `Cli` struct already has a global `--json: bool` field (passed as `force_json` to all handlers). However, clap's global flag handling means it must appear *before* the subcommand in the invocation (`linear --json issue list`), not after (`linear issue list --json`). AI agents and scripts commonly place flags after the subcommand, so per-command flags are required for usability and discoverability in per-command `--help` output.

**Alternatives considered**:
- `#[arg(global = true)]` on the Cli field — rejected because clap v4 does not support placing a global flag after a positional subcommand in all shells; per-command fields are explicit and reliable.
- Replace `--output <FORMAT>` with `--json` only — rejected; `--output` remains for forward compatibility (e.g., future `--output table` or `--output csv`) and the spec requires both flags.

---

## Decision 2: Conflict Resolution — `--json` + `--output <value>`

**Decision**: Last-flag-wins (clap's default), with `--json` treated as equivalent to `--output json`. If both are provided, the effective result is JSON output regardless of order since `--json` sets a boolean and `--output json` matches the same string check. The only true conflict is `--json --output human` or `--json --output table` — in this case `--json` takes precedence because it is an explicit boolean opt-in.

**Rationale**: Implementing last-flag-wins fully in clap for two different argument types (bool + string) is complex. Instead, the merge logic will be: `use_json = args.json || args.output.as_deref() == Some("json") || should_use_json(force_json)`. This means `--json` always wins over `--output human`. This is documented in help text.

**Alternatives considered**:
- Error on conflict — rejected; too strict for AI agent use where flags may be composed automatically.
- `--output` wins over `--json` — rejected; boolean flag is more explicit and should not be silently overridden.

---

## Decision 3: Scope — Which Commands Get `--json`

**Decision**: Add `--json` to all commands that already have `--output` (issue list/get/create/update, project list/get/create/update/archive) AND to `team list` (currently has neither flag, but produces structured output). Skip `auth` commands — they do not produce machine-parseable data output.

**Rationale**: Constitution Principle IV requires all commands to support JSON output. `team list` currently outputs tabular data, so it qualifies. `auth` commands output status messages and credentials, which have no established JSON schema.

**Alternatives considered**:
- Add `--json` only where `--output` already exists — rejected; `team list` should be consistent.
- Add `--output` to auth commands — deferred; auth output format is not part of this feature scope.

---

## Decision 4: Code Structure for `use_json` Logic

**Decision**: Centralize the merge logic in a single helper in `src/cli/output.rs`:

```
fn resolve_use_json(per_command_json: bool, output_flag: Option<&str>, force_json: bool) -> bool {
    per_command_json || output_flag == Some("json") || should_use_json(force_json)
}
```

This replaces the currently duplicated inline expressions across issue.rs and project.rs.

**Rationale**: DRY principle. Currently `issue.rs` uses `output.as_deref() == Some("json") || should_use_json(force_json)` and `project.rs` uses `force_json || args.output.as_deref() == Some("json")` — inconsistent. Centralizing ensures all commands behave identically.

**Alternatives considered**:
- Leave inline expressions and just add `|| args.json` — rejected; doesn't fix the existing inconsistency.

---

## All NEEDS CLARIFICATION Items Resolved

No unresolved clarifications remain. All decisions above are justified by codebase analysis and constitution requirements.
