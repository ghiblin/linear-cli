# Tasks: Initial Project Structure

**Input**: Design documents from `specs/001-initial-project-structure/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/cli-schema.md ✅

**Tests**: Included — constitution mandates TDD (Principle II, NON-NEGOTIABLE). Tests MUST be written and confirmed failing before any implementation code is written.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1–US4)
- Exact file paths are included in every task description

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Repository scaffolding — no logic, no tests yet. Establishes the physical structure that all subsequent tasks build on.

- [ ] T001 Initialize Cargo.toml as a binary crate named `linear` with all dependencies declared: tokio (full features), reqwest (rustls-tls), cynic, clap (v4, derive feature), serde (derive), serde_json, thiserror, anyhow, keyring, tracing, tracing-subscriber (env-filter, json features), mockall (dev), insta (dev)
- [ ] T002 [P] Create rust-toolchain.toml at repository root pinned to channel `1.85.0` with components `["rustfmt", "clippy"]`
- [ ] T003 [P] Create schema.graphql at repository root as a placeholder file with comment: `# Linear GraphQL schema — vendor the real schema here before enabling cynic build validation`
- [ ] T004 Create the full directory skeleton: `src/domain/entities/`, `src/domain/value_objects/`, `src/domain/repositories/`, `src/application/use_cases/`, `src/infrastructure/graphql/`, `src/infrastructure/repositories/`, `src/cli/commands/`, `tests/integration/`, `tests/e2e/`
- [ ] T005 [P] Create empty `mod.rs` files for every module directory: `src/domain/mod.rs`, `src/domain/entities/mod.rs`, `src/domain/value_objects/mod.rs`, `src/domain/repositories/mod.rs`, `src/application/mod.rs`, `src/application/use_cases/mod.rs`, `src/infrastructure/mod.rs`, `src/infrastructure/graphql/mod.rs`, `src/infrastructure/repositories/mod.rs`, `src/cli/mod.rs`, `src/cli/commands/mod.rs`

**Checkpoint**: `cargo build` produces zero errors (empty crate compiles).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain error types, all value objects, domain entities, and repository traits. Must be complete before any user story can start. TDD applies from this phase onward.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

### Error Types

- [ ] T006 Write failing unit tests for `DomainError` variants (`NotFound`, `InvalidInput`, `NotImplemented`) inside a `#[cfg(test)]` block at the bottom of `src/domain/errors.rs` — confirm `cargo test` fails before proceeding
- [ ] T007 Implement `DomainError` enum using `thiserror::Error` in `src/domain/errors.rs` to make T006 tests pass
- [ ] T008 [P] Write failing unit test for `ApplicationError::Domain` variant wrapping `DomainError` in `src/application/errors.rs` — confirm failure
- [ ] T009 [P] Implement `ApplicationError` enum using `thiserror::Error` in `src/application/errors.rs` to make T008 pass

### Value Objects

- [ ] T010 [P] Write failing unit tests for `IssueId` (non-empty string invariant, equality, display) in `src/domain/value_objects/issue_id.rs` — confirm failure
- [ ] T011 [P] Write failing unit tests for `TeamId` (non-empty string invariant, equality, display) in `src/domain/value_objects/team_id.rs` — confirm failure
- [ ] T012 [P] Write failing unit tests for `Priority` enum (all 5 variants, Linear integer mapping 0–4, round-trip serde) in `src/domain/value_objects/priority.rs` — confirm failure
- [ ] T013 [P] Write failing unit tests for `WorkflowState` enum (all 5 variants, display, round-trip serde) in `src/domain/value_objects/workflow_state.rs` — confirm failure
- [ ] T014 [P] Implement `IssueId` newtype struct with invariant check returning `DomainError::InvalidInput` on empty input, plus `Display`, `PartialEq`, `serde` derives in `src/domain/value_objects/issue_id.rs`
- [ ] T015 [P] Implement `TeamId` newtype struct with the same pattern as T014 in `src/domain/value_objects/team_id.rs`
- [ ] T016 [P] Implement `Priority` enum with `TryFrom<u8>` (mapping 0→NoPriority, 1→Urgent, 2→High, 3→Medium, 4→Low) and `serde` in `src/domain/value_objects/priority.rs`
- [ ] T017 [P] Implement `WorkflowState` enum with `Display` and `serde` in `src/domain/value_objects/workflow_state.rs`

### Domain Entities

- [ ] T018 Write failing unit test for `Issue` invariant: constructing an `Issue` with an empty title MUST return `Err(DomainError::InvalidInput)` in `src/domain/entities/issue.rs` — confirm failure
- [ ] T019 [P] Write failing unit test for `Team` invariant: constructing a `Team` with an empty `key` MUST return `Err(DomainError::InvalidInput)` in `src/domain/entities/team.rs` — confirm failure
- [ ] T020 Implement `Issue` struct with fields `id: IssueId`, `title: String`, `state: WorkflowState`, `priority: Priority`, `team_id: TeamId`; constructor enforces non-empty title invariant in `src/domain/entities/issue.rs`
- [ ] T021 [P] Implement `Team` struct with fields `id: TeamId`, `name: String`, `key: String`; constructor enforces non-empty key and validates 1–5 uppercase-letter format in `src/domain/entities/team.rs`

### Repository Traits

- [ ] T022 [P] Define `IssueRepository` trait with async methods `list(team_id: TeamId) → Result<Vec<Issue>, DomainError>` and `get(id: IssueId) → Result<Issue, DomainError>` in `src/domain/repositories/issue_repository.rs`
- [ ] T023 [P] Define `TeamRepository` trait with async method `list() → Result<Vec<Team>, DomainError>` and `get(id: TeamId) → Result<Team, DomainError>` in `src/domain/repositories/team_repository.rs`

**Checkpoint**: `cargo test --lib` passes all domain unit tests. Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 — Developer Bootstraps the Project (Priority: P1) 🎯 MVP

**Goal**: A developer clones the repo, runs one build command, and gets a working binary that responds to `--version`, `--help`, `linear issue list`, and `linear team list` with correct exit codes and output mode (TTY vs JSON).

**Independent Test**: On a fresh clone, run `cargo build && ./target/debug/linear --version` and verify the output matches the JSON schema in `contracts/cli-schema.md`. Then run `linear issue list --json` and verify `[]` is returned on stdout with exit code 0.

### Tests for User Story 1

> **Write these tests FIRST — confirm they FAIL before writing any implementation.**

- [ ] T024 [US1] Write failing integration test in `tests/integration/version_test.rs`: spawn the binary, capture stdout, assert output parses as JSON with `version` and `api_schema` fields — confirm failure
- [ ] T025 [P] [US1] Write failing integration test in `tests/integration/exit_codes_test.rs`: assert `linear --bad-flag` exits with code 1; assert `linear --version` exits with code 0 — confirm failure
- [ ] T026 [P] [US1] Write failing unit tests for `output::format_json` and `output::is_tty` in `src/cli/output.rs` — confirm failure

### Implementation for User Story 1

- [ ] T027 [US1] Implement `src/cli/output.rs`: `format_json<T: Serialize>(value: &T) → String` and `should_use_json(force_json: bool) → bool` (TTY detection via `atty` or `is-terminal` crate) — make T026 pass
- [ ] T028 [P] [US1] Implement `LinearIssueRepository` stub in `src/infrastructure/repositories/issue_repository.rs` implementing `IssueRepository` with all methods returning `Err(DomainError::NotImplemented)`
- [ ] T029 [P] [US1] Implement `LinearTeamRepository` stub in `src/infrastructure/repositories/team_repository.rs` implementing `TeamRepository` with all methods returning `Err(DomainError::NotImplemented)`
- [ ] T030 [US1] Write failing mockall-based unit test for `ListIssues` use case in `src/application/use_cases/list_issues.rs`: mock `IssueRepository`, call use case, assert `Ok(vec![])` is returned when repo returns empty — confirm failure
- [ ] T031 [P] [US1] Write failing mockall-based unit test for `ListTeams` use case in `src/application/use_cases/list_teams.rs` — confirm failure
- [ ] T032 [US1] Implement `ListIssues` use case struct holding a boxed `IssueRepository` trait object with async `execute(team_id: Option<TeamId>) → Result<Vec<Issue>, ApplicationError>` in `src/application/use_cases/list_issues.rs` — make T030 pass
- [ ] T033 [P] [US1] Implement `ListTeams` use case in `src/application/use_cases/list_teams.rs` with the same pattern — make T031 pass
- [ ] T034 [US1] Implement clap `Cli` root struct in `src/cli/commands/mod.rs`: `--json` global flag, `--verbose` / `-v` global flag (repeatable), `--version` flag, subcommand enum `Commands`
- [ ] T035 [US1] Implement `linear issue` subcommands (`list --team <id>`, `get <id>`) using clap derive in `src/cli/commands/issue.rs`
- [ ] T036 [P] [US1] Implement `linear team` subcommands (`list`) using clap derive in `src/cli/commands/team.rs`
- [ ] T037 [US1] Implement `src/main.rs`: `#[tokio::main]`, initialise `tracing_subscriber` (log level from `--verbose` count, JSON format when `LINEAR_LOG_FORMAT=json`), construct repository stubs, dispatch commands, map errors to exit codes (0/1/2/3)
- [ ] T038 [US1] Implement `--version` handler returning `{"version":"0.1.0","api_schema":"YYYY-MM-DD"}` in `src/main.rs` (read date from a compile-time constant) — make T024 and T025 pass

**Checkpoint**: User Story 1 complete. `cargo build && ./target/debug/linear --version` outputs the version JSON. `linear issue list --json` returns `[]`. All exit codes match the contract.

---

## Phase 4: User Story 2 — Developer Verifies Code Quality Gates (Priority: P2)

**Goal**: Every quality check (`fmt`, `clippy`, `test`) passes locally in a single command invocation, with no false positives and no unchecked warnings.

**Independent Test**: Run `cargo fmt --check && cargo clippy -- -D warnings && cargo test` on a clean checkout and confirm all three steps exit with code 0 and produce no output to stderr.

- [ ] T039 [US2] Create `.cargo/config.toml` with `[target.'cfg(all())'] rustflags = ["-D", "warnings"]` so `deny(warnings)` is enforced by the build tool rather than requiring per-file annotations
- [ ] T040 [US2] Create `rustfmt.toml` at repository root with project formatting preferences (edition = "2024", max_width = 100)
- [ ] T041 [US2] Run `cargo fmt` to auto-format all source files, then verify `cargo fmt --check` exits cleanly
- [ ] T042 [US2] Fix all remaining `cargo clippy -- -D warnings` violations across `src/` and `tests/`
- [ ] T043 [P] [US2] Add insta snapshot test for `linear --version` JSON output in `tests/integration/snapshots/` — run `cargo insta review` to accept the initial snapshot
- [ ] T044 [P] [US2] Add insta snapshot test for `linear issue list --json` empty-list output in `tests/integration/snapshots/`
- [ ] T045 [US2] Confirm `cargo test` passes all tests including snapshot tests; fix any regressions

**Checkpoint**: User Story 2 complete. The single-command quality check (`cargo fmt --check && cargo clippy -- -D warnings && cargo test`) exits 0 with no output to stderr.

---

## Phase 5: User Story 3 — CI Pipeline Validates Contributions (Priority: P3)

**Goal**: Every pull request is automatically validated by CI across all four target platforms; CI blocks merging on any quality failure.

**Independent Test**: Open a draft PR with an intentionally unformatted line; verify the CI `fmt` step fails and the merge button is blocked. Restore the formatting; verify all CI steps pass.

- [ ] T046 [US3] Create `.github/workflows/ci.yml` with three jobs: `fmt` (`cargo fmt --check`), `clippy` (`cargo clippy -- -D warnings`), `test` (`cargo test`), all running on `ubuntu-latest` with the pinned toolchain via `dtolnay/rust-toolchain@stable` (channel overridden by `rust-toolchain.toml`)
- [ ] T047 [US3] Add `Swatinem/rust-cache@v2` action to each CI job to cache `.cargo/registry` and `target/` between runs
- [ ] T048 [US3] Add a `build` job to `.github/workflows/ci.yml` with a 4-platform matrix: `aarch64-apple-darwin` on `macos-latest`, `x86_64-apple-darwin` on `macos-13`, `x86_64-unknown-linux-gnu` on `ubuntu-latest` (native), `aarch64-unknown-linux-gnu` on `ubuntu-latest` using `houseabsolute/actions-rust-cross@v0`
- [ ] T049 [P] [US3] Set `on: [push, pull_request]` triggers and configure branch protection rules documentation in the repository README so contributors know CI is required

**Checkpoint**: User Story 3 complete. CI runs on every PR and blocks merge on any failure.

---

## Phase 6: User Story 4 — Developer Navigates Layered Architecture (Priority: P3)

**Goal**: A contributor new to the codebase can identify the correct layer for any new piece of code within 2 minutes of reading the project structure.

**Independent Test**: Ask a developer who has not seen the codebase to locate where they would add (1) a new domain entity, (2) a new use case, (3) a new API repository implementation, and (4) a new CLI command — all four answers must be correct.

- [ ] T050 [US4] Create `README.md` at repository root with sections: Prerequisites, Installation, Quick Start (showing `--version` and `--help` output), Project Layout diagram (matching `quickstart.md`), and Development guide (quality-check command, layer contribution guide)
- [ ] T051 [P] [US4] Add Rust doc comments (`//!`) to `src/domain/mod.rs` explaining: "Pure business logic. Zero dependencies on infrastructure or application. All types in this module are independent of any I/O concern."
- [ ] T052 [P] [US4] Add doc comments to `src/application/mod.rs`: "Use-case orchestrators. Depends on domain traits only. Never imports from infrastructure."
- [ ] T053 [P] [US4] Add doc comments to `src/infrastructure/mod.rs`: "Concrete implementations of domain repository traits. The only layer that may import external crates for I/O (HTTP, keychain, filesystem)."
- [ ] T054 [P] [US4] Add doc comments to `src/cli/mod.rs`: "Thin command dispatch layer. Maps clap arguments to application use cases and formats results for output."

**Checkpoint**: User Story 4 complete. A new contributor can navigate the architecture without reading extended documentation.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Whole-project validation and binary size check.

- [ ] T055 Run `cargo build --release` and verify the binary at `target/release/linear` is under 20 MB; document the size in `quickstart.md`
- [ ] T056 [P] Execute the full quickstart.md validation end-to-end: fresh directory, run prerequisites check, `cargo build`, `./target/debug/linear --version`, `./target/debug/linear --help`, `./target/debug/linear issue list --json`, `./target/debug/linear team list --json`
- [ ] T057 [P] Verify `cargo test` coverage for `src/domain/` and `src/application/` reaches ≥ 80% using `cargo llvm-cov` or `cargo tarpaulin --include-files "src/domain/*,src/application/*"`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — no dependency on US2/US3/US4
- **US2 (Phase 4)**: Depends on Phase 3 (needs the full binary to test quality gates)
- **US3 (Phase 5)**: Depends on Phase 4 (CI must test what quality gates enforce)
- **US4 (Phase 6)**: Depends on Phase 3 (documentation references the working skeleton)
- **Polish (Final)**: Depends on all user story phases

### User Story Dependencies

- **US1 (P1)**: Start after Phase 2 — no story-level dependency
- **US2 (P2)**: Start after US1 — quality gates require the binary to exist
- **US3 (P3)**: Start after US2 — CI enforces what quality gates define
- **US4 (P3)**: Can start after US1 — documentation and CI are independent, may overlap with US2/US3

### Within Each Phase

1. Tests MUST be written first and MUST fail before any implementation is written
2. Value objects before entities (entities depend on value objects)
3. Domain traits before application use cases
4. Use cases before CLI commands
5. CLI commands before `main.rs` wiring

---

## Parallel Example: Phase 2 (Foundational)

```bash
# After T006–T009 (error types), launch all value object pairs in parallel:
Task T010: "Write failing tests for IssueId in src/domain/value_objects/issue_id.rs"
Task T011: "Write failing tests for TeamId in src/domain/value_objects/team_id.rs"
Task T012: "Write failing tests for Priority in src/domain/value_objects/priority.rs"
Task T013: "Write failing tests for WorkflowState in src/domain/value_objects/workflow_state.rs"

# Once all failing, implement in parallel:
Task T014: "Implement IssueId in src/domain/value_objects/issue_id.rs"
Task T015: "Implement TeamId in src/domain/value_objects/team_id.rs"
Task T016: "Implement Priority in src/domain/value_objects/priority.rs"
Task T017: "Implement WorkflowState in src/domain/value_objects/workflow_state.rs"
```

## Parallel Example: User Story 1 (Phase 3)

```bash
# After T024 (version integration test) — launch stub implementations in parallel:
Task T028: "LinearIssueRepository stub in src/infrastructure/repositories/issue_repository.rs"
Task T029: "LinearTeamRepository stub in src/infrastructure/repositories/team_repository.rs"

# After T030–T031 (use case tests fail) — implement use cases in parallel:
Task T032: "ListIssues use case in src/application/use_cases/list_issues.rs"
Task T033: "ListTeams use case in src/application/use_cases/list_teams.rs"

# After T034 (root CLI) — implement subcommand handlers in parallel:
Task T035: "linear issue commands in src/cli/commands/issue.rs"
Task T036: "linear team commands in src/cli/commands/team.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T005)
2. Complete Phase 2: Foundational (T006–T023) — **CRITICAL, blocks everything**
3. Complete Phase 3: User Story 1 (T024–T038)
4. **STOP and VALIDATE**: `cargo build && linear --version && linear issue list --json`
5. Binary is functional — sufficient to demo the architecture

### Incremental Delivery

1. **Foundation** (Phase 1 + 2) → skeleton compiles, domain tests pass
2. **US1** (Phase 3) → working binary with stub commands — MVP demo
3. **US2** (Phase 4) → quality gates enforced locally
4. **US3** (Phase 5) → CI enforces quality on every PR
5. **US4** (Phase 6) → onboarding documentation complete
6. **Polish** → binary size validated, quickstart verified end-to-end

---

## Notes

- [P] tasks operate on different files — safe to dispatch as parallel agents
- [USn] label maps every task to its user story for traceability
- TDD is non-negotiable per the constitution: every failing test must be confirmed failing before its paired implementation task begins
- Exit codes (0/1/2/3) must be wired from `main.rs` from the very first working binary
- `unsafe` code is forbidden in all tasks — if a library requires it, choose an alternative
- Commit after each checkpoint or logical group of related tasks
