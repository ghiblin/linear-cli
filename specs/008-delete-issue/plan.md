# Implementation Plan: Delete Issue

**Branch**: `008-delete-issue` | **Date**: 2026-05-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/008-delete-issue/spec.md`

## Summary

Add a `linear issue delete <id>` CLI command that calls the Linear `issueDelete` GraphQL mutation, following the same DDD-layered pattern already used by `archive_project`. The feature touches four layers: domain (trait), application (use case), infrastructure (mutation + repo impl), and CLI (subcommand).

## Technical Context

**Language/Version**: Rust stable (pinned in `rust-toolchain.toml`)
**Primary Dependencies**: cynic 3 (GraphQL/build-time schema validation), clap 4 (CLI), tokio (async), thiserror/anyhow (errors), mockall (test doubles)
**Storage**: N/A — stateless CLI, Linear API is the store
**Testing**: `cargo test` with `mockall` for unit tests; `insta` for CLI snapshot tests; `tests/e2e.rs` for live API tests
**Target Platform**: macOS arm64/x86_64 and Linux x86_64/arm64
**Project Type**: CLI tool
**Performance Goals**: Command completes in under 3 seconds on normal connection
**Constraints**: Binary under 20 MB; `clippy -D warnings` must pass; no `unsafe`
**Scale/Scope**: Single-user CLI; one issue deleted per invocation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. DDD | ✅ Pass | `delete` added to `IssueRepository` trait (domain); implementation in `LinearIssueRepository` (infra) |
| II. TDD | ✅ Pass | Unit tests written first for `DeleteIssue` use case using `mockall`; integration/e2e tests follow |
| III. Linear API Compliance | ✅ Pass | `issueDelete` mutation exists in vendored `schema.graphql`; cynic validates at build time |
| IV. CLI Interface Design | ✅ Pass | `--dry-run` required (mutating command); `--json`/`--output json` required; errors on stderr; exit codes enforced |
| V. Rust Safety | ✅ Pass | `Result<T, E>` throughout; no `unwrap`/`expect` in production paths |
| VI. Observability | ✅ Pass | `#[instrument(skip(self))]` on repository method; tracing spans on use case |

**No violations. Complexity Tracking table omitted.**

## Project Structure

### Documentation (this feature)

```text
specs/008-delete-issue/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── cli/commands/
│   └── issue.rs                         # add Delete subcommand variant + run_issue handler arm
├── application/use_cases/
│   ├── delete_issue.rs                  # NEW: DeleteIssue use case + DeleteOutcome enum
│   └── mod.rs                           # register delete_issue module
├── domain/repositories/
│   └── issue_repository.rs              # add delete() to IssueRepository trait
└── infrastructure/
    ├── graphql/mutations/
    │   └── issue_mutations.rs           # add IssueDeleteMutation + delete_issue() fn
    └── repositories/
        └── issue_repository.rs          # impl delete() on LinearIssueRepository

tests/
└── e2e.rs                               # optional: delete e2e test behind LINEAR_TEST_API_KEY
```

**Structure Decision**: Single-project layout (existing). No new directories needed; all additions slot into the established layer hierarchy.
