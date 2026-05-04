# Implementation Plan: Project CRUD Operations

**Branch**: `003-project-crud` | **Date**: 2026-04-23 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-project-crud/spec.md`

## Summary

Implement the full CRUD lifecycle for Linear projects — list, get, create, update, archive — as a `linear project <subcommand>` CLI command group. Uses `cynic` v3 for type-safe GraphQL queries validated at compile-time against the vendored Linear schema, following the established DDD layered architecture (domain → application → infrastructure → CLI). Adds build-time schema validation infrastructure (`build.rs`, `cynic_codegen`) as a prerequisite step, unblocking the entire codebase to meet Constitution Principle III.

## Technical Context

**Language/Version**: Rust (stable, version pinned in `rust-toolchain.toml`)
**Primary Dependencies**: `clap` v4 (CLI derive), `tokio` (async), `reqwest` 0.12 + rustls (HTTP), `cynic` v3 (GraphQL, build-time schema validation), `serde`/`serde_json` (serialization), `thiserror` (domain/infra errors), `anyhow` (binary entry point only), `tracing` (observability), `mockall` (test mocks), `insta` (snapshot tests), `chrono` (date types — to be added for `NaiveDate`)
**Storage**: N/A — stateless CLI; credentials via `keyring` crate
**Testing**: `cargo test`; `mockall` for trait mocks; `insta` for snapshot tests; unit tests co-located with source in `#[cfg(test)]` modules; integration tests under `tests/`
**Target Platform**: macOS (arm64, x86_64), Linux (x86_64, arm64)
**Project Type**: CLI tool
**Performance Goals**: `project list` (50 projects) < 3 s; `--all` (≤ 500 projects) < 5 s; `project get` < 2 s
**Constraints**: Binary < 20 MB release; no `unsafe`; `clippy` at `deny(warnings)`; 80%+ test coverage on domain + application layers; no `unwrap()`/`expect()` in production code paths
**Scale/Scope**: Typical Linear workspace; up to 500+ projects; cursor-based pagination

## Constitution Check

*GATE: Must pass before implementation. Checked post-design: ✅ all gates clear.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. DDD | ✅ | `Project` entity, `ProjectState`/`ProjectId`/`UserId` value objects, `ProjectRepository` trait in domain layer; `LinearProjectRepository` impl in infrastructure |
| II. Test-First | ✅ | All domain entities and use cases written test-first (red-green-refactor); integration tests for infra repository |
| III. Linear API Compliance | ✅ | Full Linear SDL schema vendored to `schema.graphql`; cynic validates queries at build time; rate-limit retry + cursor pagination handled in infra layer |
| IV. CLI for AI Agents | ✅ | `--output json`, `--dry-run`, exit codes 0/1/2/3, `--verbose`/`--debug`, composable stdout |
| V. Rust Safety | ✅ | No `unsafe`; `Result<T,E>` throughout; `thiserror` for domain/infra; `anyhow` only in `main.rs` |
| VI. Observability | ✅ | `tracing` spans on all use cases and infra calls; `--verbose` maps to `INFO`; `--debug` maps to `DEBUG` and logs raw GraphQL payloads |

## Project Structure

### Documentation (this feature)

```text
specs/003-project-crud/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── cli-commands.md  # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit-tasks — NOT created here)
```

### Source Code (repository root)

```text
schema.graphql                        # Replace stub with full Linear SDL (update)
build.rs                              # cynic_codegen schema registration (new)
Cargo.toml                            # add cynic_codegen + chrono build/deps (update)

src/
├── domain/
│   ├── entities/
│   │   ├── project.rs                # Project entity (new)
│   │   └── mod.rs                    # register project module (update)
│   ├── value_objects/
│   │   ├── project_id.rs             # ProjectId; UUID + display-ID variants (new)
│   │   ├── project_state.rs          # ProjectState enum + from_str/display (new)
│   │   ├── user_id.rs                # UserId for project lead (new)
│   │   └── mod.rs                    # register new modules (update)
│   ├── repositories/
│   │   ├── project_repository.rs     # ProjectRepository trait (new)
│   │   └── mod.rs                    # register project_repository (update)
│   └── mod.rs                        # no change
│
├── application/
│   ├── use_cases/
│   │   ├── list_projects.rs          # ListProjects use case (new)
│   │   ├── get_project.rs            # GetProject use case (new)
│   │   ├── create_project.rs         # CreateProject use case + CreateProjectInput (new)
│   │   ├── update_project.rs         # UpdateProject use case + UpdateProjectInput (new)
│   │   ├── archive_project.rs        # ArchiveProject use case (new)
│   │   └── mod.rs                    # register new modules (update)
│   └── mod.rs                        # no change
│
├── cli/
│   ├── commands/
│   │   ├── project.rs                # ProjectCommand + run_project (new)
│   │   └── mod.rs                    # register + route project command (update)
│   └── mod.rs                        # no change
│
└── infrastructure/
    ├── graphql/
    │   ├── schema.rs                  # #[cynic::schema("linear")] module (new)
    │   ├── queries/
    │   │   ├── mod.rs                 # new
    │   │   └── project_queries.rs     # cynic QueryFragment types for list/get (new)
    │   ├── mutations/
    │   │   ├── mod.rs                 # new
    │   │   └── project_mutations.rs   # cynic InputObject + MutationFragment types (new)
    │   └── mod.rs                     # register schema + queries + mutations (update)
    ├── repositories/
    │   ├── project_repository.rs      # LinearProjectRepository impl (new)
    │   └── mod.rs                     # register project module (update)
    └── mod.rs                         # no change

tests/
└── e2e.rs                             # add project e2e test cases (update)
```

**Structure Decision**: Single-project CLI following the established DDD layered structure. New files are additive; no new workspaces, packages, or architectural layers. The cynic integration adds `build.rs` and `src/infrastructure/graphql/schema.rs` as the only structural additions to the build pipeline. The `validate_api_key` raw HTTP implementation is left unchanged (tech debt tracked separately).

## Complexity Tracking

No constitution violations.
