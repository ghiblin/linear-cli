# Implementation Plan: Filter by Partial Title

**Branch**: `007-filter-partial-title` | **Date**: 2026-05-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/007-filter-partial-title/spec.md`

## Summary

Add `--title` to `issue list` and `--name` to `project list` for server-side case-insensitive substring filtering, using the Linear API's existing `IssueFilter.title` and `ProjectFilter.name` `StringComparator` fields (`containsIgnoreCase`). Changes cascade through four layers: domain input structs → GraphQL filter builders → application use cases → CLI flags.

## Technical Context

**Language/Version**: Rust (stable, edition 2024; see `rust-toolchain.toml`)
**Primary Dependencies**: `clap` v4 (derive), `cynic` v3 (GraphQL client, build-time schema validation), `serde`/`serde_json`, `tokio`, `reqwest`
**Storage**: N/A
**Testing**: `cargo test`; unit tests via `#[cfg(test)]` + `mockall`; integration/e2e via `tests/e2e.rs`
**Target Platform**: macOS arm64/x86_64, Linux x86_64/arm64
**Project Type**: CLI binary
**Performance Goals**: Filtered results in under 3 seconds for ≤500 items (SC-003)
**Constraints**: Binary under 20 MB; no `unsafe`; clippy clean at `deny(warnings)`
**Scale/Scope**: Single-workspace, up to ~500 issues or projects per invocation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. DDD — domain layer has zero infra deps | ✅ PASS | `title_contains` added only to domain input struct; no raw GraphQL types leak into domain |
| II. TDD — Red-Green-Refactor | ✅ PASS | Tasks must be sequenced: failing tests first, then implementation |
| III. Linear API Compliance — schema validated at build time | ✅ PASS | Adding fields that exist in vendored `schema.graphql` (`IssueFilter.title`, `ProjectFilter.name`, `StringComparator.containsIgnoreCase`) |
| IV. CLI Interface — composable, JSON-stable | ✅ PASS | New flags compose with all existing flags; JSON schema extended (new filter fields never appear in output) |
| V. Rust Safety — no `unsafe`, `Result`-based errors | ✅ PASS | All new code follows existing patterns |
| VI. Observability — tracing instrumented | ✅ PASS | `#[instrument]` already applied to use case `execute` methods; no new spans required |

**No violations.** No Complexity Tracking entries needed.

## Project Structure

### Documentation (this feature)

```text
specs/007-filter-partial-title/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (affected files only)

```text
src/
├── domain/
│   ├── entities/issue.rs                         # Add title_contains: Option<String> to ListIssuesInput
│   └── repositories/project_repository.rs        # Add name_contains param to list() trait method
├── application/
│   └── use_cases/
│       ├── list_issues.rs                        # Thread title_contains; update test default_input()
│       └── list_projects.rs                      # Add name_contains param to execute(); update tests
├── infrastructure/
│   ├── graphql/
│   │   └── queries/
│   │       ├── issue_queries.rs                  # Add containsIgnoreCase to StringComparatorInput, title to IssueFilterInput, update build_issue_filter
│   │       └── project_queries.rs                # Add containsIgnoreCase to StringComparator, name to ProjectFilter; add filter vars to ProjectsVariables and TeamProjectsVariables; update fetch_projects
│   └── repositories/
│       ├── issue_repository.rs                   # Pass title_contains to build_issue_filter (already done via ListIssuesInput)
│       └── project_repository.rs                 # Pass name_contains to fetch_projects
└── cli/
    └── commands/
        ├── issue.rs                              # Add --title flag to List subcommand
        └── project.rs                            # Add --name flag to ListArgs
```

**Structure Decision**: Single-project layout. No new files required — all changes extend existing types and functions.

## Complexity Tracking

> No constitution violations to justify.
