# Implementation Plan: JSON Output Shorthand Flag

**Branch**: `006-add-json-flag` | **Date**: 2026-05-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-add-json-flag/spec.md`

## Summary

Add `--json` as a per-command shorthand flag that is functionally identical to `--output json`. Every command that currently accepts `--output <format>` will also accept `--json` as a boolean flag. The global `--json` flag on the root `Cli` struct already exists; this feature makes `--json` available directly on each subcommand's `Args` struct so it appears in per-command help text and can be placed after the subcommand name (e.g., `linear issue list --json`).

## Technical Context

**Language/Version**: Rust (stable, pinned in `rust-toolchain.toml`)
**Primary Dependencies**: `clap` v4 (derive API), `serde` + `serde_json`, `thiserror`
**Storage**: N/A
**Testing**: `cargo test`, `insta` for snapshot tests of CLI output
**Target Platform**: macOS (arm64, x86_64), Linux (x86_64, arm64)
**Project Type**: CLI
**Performance Goals**: N/A (flag parsing has no measurable performance impact)
**Constraints**: No `unsafe`, `clippy` must pass at `deny(warnings)`, `cargo fmt` clean
**Scale/Scope**: All commands — `issue` (list, get, create, update), `project` (list, get, create, update, archive), `team` (list), `auth` (not applicable — no data output)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Domain-Driven Design | ✅ Pass | Flag parsing lives in CLI layer (application/CLI boundary); no domain changes |
| II. Test-First Development | ✅ Pass | Tests will be written before implementation (see tasks) |
| III. Linear API Compliance | ✅ Pass | No API changes |
| IV. CLI Interface Design | ✅ Pass | Constitution **explicitly mandates** both `--output json` and `--json`; this feature fulfills the requirement |
| V. Rust Safety and Idioms | ✅ Pass | No `unsafe`, clap derive API, clippy-clean |
| VI. Observability | ✅ Pass | No tracing changes needed |

**No violations. Feature is constitutionally mandated by Principle IV.**

## Project Structure

### Documentation (this feature)

```text
specs/006-add-json-flag/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── contracts/           # Phase 1 output
│   ├── issue-commands.md
│   ├── project-commands.md
│   └── team-commands.md
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (repository root)

```text
src/
├── main.rs                        # No changes expected
├── cli/
│   ├── output.rs                  # Extend should_use_json or add merge helper
│   └── commands/
│       ├── issue.rs               # Add --json bool field to List/Get/Create/Update Args
│       ├── project.rs             # Add --json bool field to all 5 Args structs
│       └── team.rs                # Add --json bool field to ListArgs (and --output if absent)

tests/
└── integration/                   # New/updated snapshot tests per command
```

**Structure Decision**: Single project, minimal changes confined to CLI layer. No new files in `src/`; all changes are additive fields on existing Args structs and updated `use_json` logic.

## Complexity Tracking

No violations. No complexity tracking required.
