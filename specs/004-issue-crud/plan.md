# Implementation Plan: Issue CRUD Operations

**Branch**: `004-issue-crud` | **Date**: 2026-05-05 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `/specs/004-issue-crud/spec.md`

## Summary

Implement full CRUD lifecycle for Linear issues: list with filters, get by ID, create (with optional parent for sub-issues), and update (including state validation and re-parenting). The existing `Issue` entity, `IssueId` value object, `IssueRepository` trait, and `list_issues` use case stubs are in place but incomplete. All four layers (domain, application, infrastructure, CLI) require extension following the established Project CRUD pattern.

## Technical Context

**Language/Version**: Rust stable (pinned in `rust-toolchain.toml`)  
**Primary Dependencies**: `cynic` (GraphQL, build-time schema validation), `clap` v4 (derive API), `reqwest` (HTTP, rustls-tls), `serde`/`serde_json`, `mockall`, `async-trait`, `thiserror`, `tracing`  
**Storage**: N/A — stateless CLI; Linear API is the authoritative store  
**Testing**: `cargo test`, `mockall` for trait mocking, `insta` for CLI snapshot tests, `#[cfg(test)]` unit modules in `src/`, integration under `tests/`  
**Target Platform**: macOS (arm64, x86_64), Linux (x86_64, arm64)  
**Project Type**: CLI tool  
**Performance Goals**: First page (≤50 issues) in <3 s; full workspace (≤500 issues) via `--all` in <10 s  
**Constraints**: HTTP 429 → retry 3× with exponential backoff (1 s, 2 s, 4 s); errors on stderr; never mix errors with stdout data  
**Scale/Scope**: Linear workspace scale (typically <10k issues per workspace)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Domain-Driven Design | ✅ PASS | Issue, IssueId, Priority already domain types. WorkflowState enum must be replaced with dynamic `WorkflowStateRef` value object (team-specific strings). CreateIssueInput, UpdateIssueInput, ListIssuesInput added as domain structs. |
| II. Test-First Development | ✅ PASS (gate on implementation) | Red-Green-Refactor enforced per task. Unit tests use mockall mocks for IssueRepository. |
| III. Linear API Compliance | ✅ PASS | schema.graphql already vendored (used by projects). cynic build-time validation applies. `execute_with_retry` handles rate limits. |
| IV. CLI Interface Design | ✅ PASS | `--output json`, TTY detection, exit codes 0/1/2/3, `--dry-run` on mutating commands, `--verbose`/`--debug`, command taxonomy `linear issue *`. |
| V. Rust Safety & Idioms | ✅ PASS | No `unsafe`. `thiserror` for domain errors, `anyhow` only in binary entry. `unwrap()`/`expect()` forbidden in production paths. |
| VI. Observability | ✅ PASS | `#[instrument]` on all use case methods. API calls at DEBUG level. `--verbose`/`--debug` flags. |

**Complexity Tracking**: No constitution violations anticipated.

## Project Structure

### Documentation (this feature)

```text
specs/004-issue-crud/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
├── contracts/           ← Phase 1 output
│   └── cli-schema.md
└── tasks.md             ← Phase 2 output (/speckit-tasks command)
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── entities/
│   │   └── issue.rs                    # Extended: add all fields, SubIssueRef, input structs
│   ├── value_objects/
│   │   ├── label_id.rs                 # New: LabelId(String) value object
│   │   ├── workflow_state_ref.rs       # New: WorkflowStateRef { id, name } replaces enum
│   │   └── workflow_state.rs           # Modified: repurposed or removed
│   └── repositories/
│       └── issue_repository.rs         # Extended: create, update, list (filters), list_workflow_states
│
├── application/
│   └── use_cases/
│       ├── list_issues.rs              # Extended: filters, pagination
│       ├── get_issue.rs                # New
│       ├── create_issue.rs             # New
│       └── update_issue.rs             # New (includes state validation pre-flight)
│
├── cli/
│   └── commands/
│       └── issue.rs                    # Extended: get, create, update subcommands
│
└── infrastructure/
    ├── graphql/
    │   ├── queries/
    │   │   └── issue_queries.rs        # New: list, get, workflow_states queries
    │   └── mutations/
    │       └── issue_mutations.rs      # New: create, update mutations
    └── repositories/
        └── issue_repository.rs         # Full implementation (currently stub)

tests/
├── integration/
│   └── issue_commands_test.rs          # New: exit code tests for issue subcommands
```

**Structure Decision**: Single-project layout, identical to existing project CRUD structure. No new top-level directories needed.

## Complexity Tracking

No constitution violations require justification.

> **Note on WorkflowState**: The existing `WorkflowState` enum (Backlog, Todo, InProgress, Done, Cancelled) is a fixed set inconsistent with Linear's actual team-specific workflow states. Replacing it with `WorkflowStateRef { id, name }` is a required correction, not a complexity violation. The stub implementation never used the enum in production paths.
