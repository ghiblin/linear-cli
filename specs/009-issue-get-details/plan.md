# Implementation Plan: Issue Get — Optional Detail Flags

**Branch**: `009-issue-get-details` | **Date**: 2026-05-07 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/009-issue-get-details/spec.md`

## Summary

Extend `issue get` with two opt-in flags, `--description` and `--subtasks`, that print the issue's description text and child issue list respectively in human-readable output. JSON output is unchanged. The change is confined to the CLI presentation layer — no domain, application, or infrastructure code requires modification.

## Technical Context

**Language/Version**: Rust (stable, `rust-toolchain.toml` pin)  
**Primary Dependencies**: `clap` v4 (derive API), `serde` + `serde_json`, `tokio`, `cynic`, `anyhow`, `thiserror`, `tracing`  
**Storage**: N/A  
**Testing**: `cargo test`, `mockall`, `insta`  
**Target Platform**: macOS (arm64, x86_64) + Linux (x86_64, arm64)  
**Project Type**: CLI  
**Performance Goals**: Commands respond in under 3 seconds on a normal connection  
**Constraints**: No `unsafe`, `clippy -D warnings` clean, JSON output schema must not break  
**Scale/Scope**: Single binary; additive flag additions with no breaking changes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Notes |
|-----------|-------|-------|
| I. Domain-Driven Design | ✅ PASS | `Issue` entity already models `description` and `sub_issues`. No domain layer changes. |
| II. Test-First Development | ✅ PASS | Unit tests for display flag behaviour must be written before implementation (see tasks). |
| III. Linear API Compliance | ✅ PASS | `IssueDetailNode` fragment already fetches `description` and `children`. No GraphQL changes. |
| IV. CLI Interface Design | ✅ PASS | Additive flags; JSON unchanged; non-mutating (no `--dry-run` needed); exit codes unchanged. |
| V. Rust Safety | ✅ PASS | Standard `clap` derive pattern; no `unsafe`. |
| VI. Observability | ✅ PASS | No new spans required for display-only flags. |

**Post-Phase 1 re-check**: All gates still pass. No new complexity introduced.

## Project Structure

### Documentation (this feature)

```text
specs/009-issue-get-details/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── contracts/
│   └── cli-contract.md  # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (files touched by this feature)

```text
src/
└── cli/
    └── commands/
        └── issue.rs     # Only file that changes
            ├── IssueSubcommand::Get — add --description and --subtasks flags
            └── format_issue_human  — accept display flags, make sub-issues opt-in

tests/                   # No new integration test files; unit tests live in issue.rs
```

## Complexity Tracking

No constitution violations. No complexity tracking required.
