# Tasks: Project CRUD Operations

**Input**: Design documents from `/specs/003-project-crud/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/cli-commands.md ✅, quickstart.md ✅

**Tests**: Unit tests included per plan.md's Test-First constitution requirement (co-located in `#[cfg(test)]` modules per Rust convention); integration/e2e tests in `tests/`.

**Organization**: Tasks grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Exact file paths included in all descriptions

---

## Phase 1: Setup (Build Infrastructure)

**Purpose**: Wire up cynic build-time schema validation — required before any GraphQL types can be written.

- [X] T001 Vendor full Linear SDL: download the Linear GraphQL schema (via introspection or from the Linear SDK GitHub repo at `https://github.com/linear/linear`) and replace the stub `schema.graphql` at the repository root
- [X] T002 Create `build.rs` at repository root — call `cynic_codegen::register_schema("linear").from_sdl_file("schema.graphql").unwrap()` (per research.md Decision 1)
- [X] T003 Update `Cargo.toml` — add `cynic_codegen = { version = "3", features = ["rkyv"] }` to `[build-dependencies]`; add `chrono = { version = "0.4", features = ["serde"] }` to `[dependencies]`

---

## Phase 2: Foundational (Domain Primitives + Repository Interface)

**Purpose**: Core domain types and repository trait that ALL user story phases depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Create `src/domain/value_objects/project_state.rs` — `ProjectState` enum (`Planned`, `Started`, `Paused`, `Completed`, `Cancelled`); `from_str` case-insensitive (errors list valid values per FR-007); `Display` (lowercase); `Serialize`/`Deserialize` (lowercase strings); `#[cfg(test)]` unit tests for parse + display roundtrip and invalid input
- [X] T005 [P] Create `src/domain/value_objects/project_id.rs` — `ProjectId` with `Uuid(String)` and `DisplayId(String)` variants; `ProjectId::parse(s)` auto-detects format by regex (`[0-9a-f]{8}-...-[0-9a-f]{12}` vs `[A-Z]+-\d+`) per research.md Decision 3; `as_uuid()` / `as_display_id()` accessors; `DomainError::InvalidInput` on unrecognised format; unit tests for UUID, display-ID (e.g. `PRJ-1`), and invalid format
- [X] T006 [P] Create `src/domain/value_objects/user_id.rs` — `UserId(String)` newtype; `UserId::new(s)` rejects empty string with `DomainError::InvalidInput`; `Display` impl; unit tests for empty rejection and valid input
- [X] T007 Update `src/domain/value_objects/mod.rs` to `pub mod project_state; pub mod project_id; pub mod user_id;` and re-export `ProjectState`, `ProjectId`, `UserId`
- [X] T008 Create `src/domain/entities/project.rs` — `Project` struct with all fields from data-model.md (`id: ProjectId`, `name: String`, `description: Option<String>`, `state: ProjectState`, `progress: f32`, `lead_id: Option<UserId>`, `team_ids: Vec<TeamId>`, `start_date: Option<NaiveDate>`, `target_date: Option<NaiveDate>`, `updated_at: DateTime<Utc>`); `Project::new` validates name non-empty, team_ids non-empty, progress ∈ [0.0, 100.0] — `DomainError::InvalidInput` on violation; `#[cfg(test)]` unit tests for all three validation rules
- [X] T009 Update `src/domain/entities/mod.rs` to `pub mod project;` and re-export `Project`
- [X] T010 Create `src/domain/repositories/project_repository.rs` — `ProjectRepository` `#[async_trait]` trait with `list(team_id, first, after)`, `get(id)`, `create(input)`, `update(id, input)`, `archive(id)` signatures from data-model.md; `ListProjectsResult { items: Vec<Project>, page_info: PageInfo }` struct; `PageInfo { has_next_page: bool, end_cursor: Option<String> }` struct
- [X] T011 Update `src/domain/repositories/mod.rs` to `pub mod project_repository;` and re-export `ProjectRepository`, `ListProjectsResult`, `PageInfo`
- [X] T012 Create `src/infrastructure/graphql/schema.rs` — `#[cynic::schema("linear")] mod schema {}` referencing the crate name registered in `build.rs`
- [X] T013 [P] Create `src/infrastructure/graphql/queries/mod.rs` — empty module file (populated per user story phase)
- [X] T014 [P] Create `src/infrastructure/graphql/mutations/mod.rs` — empty module file (populated per user story phase)
- [X] T015 Update `src/infrastructure/graphql/mod.rs` to `pub mod schema; pub mod queries; pub mod mutations;`
- [X] T016 Create `src/infrastructure/repositories/project_repository.rs` — `LinearProjectRepository` struct with HTTP client and credential fields; `new()` constructor; empty `impl` block (methods added per story); does NOT yet implement `ProjectRepository` trait
- [X] T017 Update `src/infrastructure/repositories/mod.rs` to `pub mod project_repository;` and re-export `LinearProjectRepository`; run `cargo build` to confirm Phase 2 compiles without errors before proceeding

**Checkpoint**: Domain types, repository trait, build infrastructure, and infra scaffolding ready — user story phases can begin.

---

## Phase 3: User Story 1 — List Projects (Priority: P1) 🎯 MVP

**Goal**: Users can run `linear project list` and receive a paginated list of projects with name, state, and target date.

**Independent Test**: Run `linear project list --output json` against a real Linear workspace and confirm the JSON structure matches `contracts/cli-commands.md`; run `linear project list --all` for a workspace with >50 projects and confirm all pages are fetched.

- [X] T018 [P] [US1] Create `src/infrastructure/graphql/queries/project_queries.rs` — `ProjectsQuery` `#[derive(QueryFragment)]` with connection nodes (id, name, description, state, progress, leadId, teams.nodes.id, startDate, targetDate, updatedAt) and `PageInfo` (hasNextPage, endCursor); `ProjectsQueryVariables` (first: `i32`, after: `Option<String>`, teamId filter); confirm `cargo build` passes cynic compile-time validation
- [X] T019 [P] [US1] Create `src/application/use_cases/list_projects.rs` — `ListProjects` struct holding `Arc<dyn ProjectRepository>`; `execute(team_id: Option<TeamId>, first: u32, after: Option<String>, all: bool) -> Result<ListProjectsResult, DomainError>`; when `all=true` drives pagination loop until `page_info.has_next_page == false`, collecting all items; `#[cfg(test)]` unit tests with `mockall` mock for `ProjectRepository` covering single-page, multi-page `--all`, and empty-result paths
- [X] T020 [US1] Update `src/infrastructure/graphql/queries/mod.rs` to `pub mod project_queries;`
- [X] T021 [US1] Add `list()` method to `LinearProjectRepository` in `src/infrastructure/repositories/project_repository.rs` — executes `ProjectsQuery` via HTTP; implements rate-limit detection: checks `errors[].extensions.type == "RATELIMITED"` first, HTTP 429 as secondary signal (per research.md Decision 2); retries up to 3× with exponential backoff (1 s, 2 s, 4 s); logs `X-RateLimit-*` headers at DEBUG level; maps response nodes to `Vec<Project>` and `PageInfo`; add `impl ProjectRepository for LinearProjectRepository` block
- [X] T022 [US1] Update `src/application/use_cases/mod.rs` to `pub mod list_projects;`
- [X] T023 [US1] Create `src/cli/commands/project.rs` — `ProjectCommand` enum with `List(ListArgs)` variant; `ListArgs` struct (`#[derive(clap::Args)]`) with `--team <UUID>`, `--limit <n>` (default 50), `--cursor <token>`, `--all`, `--output <json|human>`, `--verbose`/`-v`, `--debug`; `run_project()` dispatch; human-readable table output (TTY) and JSON output per `contracts/cli-commands.md`; exit code 3 on auth error, 2 on API/network error, 1 on input error, 0 on success
- [X] T024 [US1] Update `src/cli/commands/mod.rs` to `pub mod project;` and add `"project"` subcommand routing to `run_project()`

**Checkpoint**: `linear project list` fully functional with pagination, `--all`, `--team` filter, human and JSON output modes.

---

## Phase 4: User Story 2 — View Project Details (Priority: P2)

**Goal**: Users can run `linear project get <id>` with either a UUID or display ID (e.g. `PRJ-1`) and see all project fields.

**Independent Test**: Run `linear project get PRJ-1 --output json` for a known project and confirm all fields from `data-model.md` JSON schema are present; run `linear project get nonexistent-id` and confirm exit code 1 with error on stderr.

- [X] T025 [P] [US2] Add to `src/infrastructure/graphql/queries/project_queries.rs` — `GetProjectQuery` `#[derive(QueryFragment)]` fetching a single project by UUID (all fields per data-model.md); `DisplayIdLookupQuery` fragment for resolving display IDs via `projects(filter: {identifier: {eq: "..."}})` returning only the UUID (per research.md Decision 3)
- [X] T026 [P] [US2] Create `src/application/use_cases/get_project.rs` — `GetProject` struct; `execute(id: ProjectId) -> Result<Project, DomainError>` delegates to `repository.get(id)`; `#[cfg(test)]` unit tests with `mockall` covering found, not-found (`DomainError::NotFound`), and auth-error paths
- [X] T027 [US2] Add `get()` method to `LinearProjectRepository` in `src/infrastructure/repositories/project_repository.rs` — resolves `ProjectId::DisplayId` via `DisplayIdLookupQuery` before calling `GetProjectQuery`; maps GraphQL null / empty result to `DomainError::NotFound`; rate-limit retry policy; extend `impl ProjectRepository for LinearProjectRepository`
- [X] T028 [US2] Update `src/application/use_cases/mod.rs` to `pub mod get_project;`
- [X] T029 [US2] Add `Get(GetArgs)` variant to `ProjectCommand` in `src/cli/commands/project.rs` — `GetArgs` with `id: String` positional arg, `--output`, `--verbose`, `--debug`; dispatch to `GetProject` use case; human multi-field output and JSON object output per `contracts/cli-commands.md`; not-found error → stderr + exit 1

**Checkpoint**: `linear project get <id>` fully functional with UUID and display-ID support; not-found case handled correctly.

---

## Phase 5: User Story 3 — Create a Project (Priority: P3)

**Goal**: Users can run `linear project create --name "..." --team <id>` to create a new project in Linear.

**Independent Test**: Run `linear project create --name "CLI Test" --team <team-uuid> --output json` and confirm the project appears in `linear project list --output json`; run with `--dry-run` and confirm no API mutation occurs.

- [X] T030 [P] [US3] Create `src/infrastructure/graphql/mutations/project_mutations.rs` — `ProjectCreateMutation` `#[derive(MutationFragment)]` returning created project (id, name, state); `ProjectCreateInput` `#[derive(InputObject)]` with `name`, `teamIds`, `description`, `leadId`, `startDate`, `targetDate`; confirm `cargo build` passes cynic compile-time validation
- [X] T031 [P] [US3] Create `src/application/use_cases/create_project.rs` — `CreateProject` struct; `execute(input: CreateProjectInput, dry_run: bool) -> Result<Project, DomainError>`; validates `input.name` non-empty and `input.team_ids` non-empty before any repo call (exit 1 per SC-004); dry-run path returns structured dry-run info without calling repository; `#[cfg(test)]` unit tests covering valid creation, dry-run, missing-name, and missing-team paths
- [X] T032 [US3] Update `src/infrastructure/graphql/mutations/mod.rs` to `pub mod project_mutations;`
- [X] T033 [US3] Add `create()` method to `LinearProjectRepository` in `src/infrastructure/repositories/project_repository.rs` — maps `CreateProjectInput` fields to cynic `ProjectCreateInput`; formats `NaiveDate` as ISO 8601 string; rate-limit retry policy; maps GraphQL permission/team-not-found errors to `DomainError`; extend `impl ProjectRepository for LinearProjectRepository`
- [X] T034 [US3] Update `src/application/use_cases/mod.rs` to `pub mod create_project;`
- [X] T035 [US3] Add `Create(CreateArgs)` variant to `ProjectCommand` in `src/cli/commands/project.rs` — `CreateArgs` with `--name` (required), `--team` (required, `Vec<String>` for multiple teams), `--description`, `--lead`, `--start-date`, `--target-date`, `--dry-run`, `--output`, `--verbose`, `--debug`; local validation before dispatch; human, JSON, and dry-run output per `contracts/cli-commands.md`; missing required flag → exit 1 before any API call

**Checkpoint**: `linear project create` fully functional with dry-run, JSON output, and local validation.

---

## Phase 6: User Story 4 — Update a Project (Priority: P4)

**Goal**: Users can run `linear project update <id> --state started` (or any combination of update flags) to modify project attributes.

**Independent Test**: Run `linear project update PRJ-1 --state started` and confirm state change via `linear project get PRJ-1 --output json | jq .state`; run with no update flags and confirm exit code 1.

- [X] T036 [P] [US4] Add `ProjectUpdateMutation` `#[derive(MutationFragment)]` and `ProjectUpdateInput` `#[derive(InputObject)]` to `src/infrastructure/graphql/mutations/project_mutations.rs` — all fields optional (`name`, `description`, `state`, `leadId`, `startDate`, `targetDate`); mutation returns updated project (id, name, state)
- [X] T037 [P] [US4] Create `src/application/use_cases/update_project.rs` — `UpdateProject` struct; `execute(id: ProjectId, input: UpdateProjectInput, dry_run: bool) -> Result<Project, DomainError>`; validates at least one `UpdateProjectInput` field is `Some` before any repo call (exit 1 per FR-006); dry-run path; `#[cfg(test)]` unit tests covering valid update, no-fields-provided error, dry-run, and not-found paths
- [X] T038 [US4] Add `update()` method to `LinearProjectRepository` in `src/infrastructure/repositories/project_repository.rs` — resolves `ProjectId::DisplayId` via lookup query; maps `UpdateProjectInput` to cynic `ProjectUpdateInput` (only `Some` fields serialised); rate-limit retry policy; extend `impl ProjectRepository for LinearProjectRepository`
- [X] T039 [US4] Update `src/application/use_cases/mod.rs` to `pub mod update_project;`
- [X] T040 [US4] Add `Update(UpdateArgs)` variant to `ProjectCommand` in `src/cli/commands/project.rs` — `UpdateArgs` with `id: String` positional; `--name`, `--description`, `--state`, `--lead`, `--start-date`, `--target-date` all optional; `--state` value validated against `ProjectState::from_str` locally before any API call (exit 1 + list valid values on invalid input per FR-007); at-least-one-flag check before dispatch; `--dry-run`, `--output`, `--verbose`, `--debug`; human, JSON, and dry-run output per `contracts/cli-commands.md`

**Checkpoint**: `linear project update` fully functional with `--state` validation, dry-run, and JSON output.

---

## Phase 7: User Story 5 — Archive a Project (Priority: P5)

**Goal**: Users can run `linear project archive <id>` to soft-archive a project, removing it from active views.

**Independent Test**: Run `linear project archive PRJ-1` and confirm the project is absent from `linear project list` output (which excludes archived projects by default); run `linear project archive PRJ-1` a second time and confirm exit 0 with already-archived message.

- [X] T041 [P] [US5] Add `ProjectArchiveMutation` `#[derive(MutationFragment)]` to `src/infrastructure/graphql/mutations/project_mutations.rs` — mutation archives a project by ID and returns success boolean and archived project `id`
- [X] T042 [P] [US5] Create `src/application/use_cases/archive_project.rs` — `ArchiveProject` struct; `execute(id: ProjectId, dry_run: bool) -> Result<(), DomainError>`; dry-run path; handles already-archived as non-error (returns `Ok(())`); `#[cfg(test)]` unit tests covering success, already-archived, not-found, and dry-run paths
- [X] T043 [US5] Add `archive()` method to `LinearProjectRepository` in `src/infrastructure/repositories/project_repository.rs` — resolves `ProjectId::DisplayId` via lookup; detects already-archived from GraphQL response and returns `Ok(())` rather than an error; rate-limit retry policy; extend `impl ProjectRepository for LinearProjectRepository`
- [X] T044 [US5] Update `src/application/use_cases/mod.rs` to `pub mod archive_project;`
- [X] T045 [US5] Add `Archive(ArchiveArgs)` variant to `ProjectCommand` in `src/cli/commands/project.rs` — `ArchiveArgs` with `id: String` positional, `--dry-run`, `--output`, `--verbose`, `--debug`; human output distinguishes success from already-archived message; JSON output per `contracts/cli-commands.md`; not-found → exit 1; dry-run output per contracts

**Checkpoint**: Full project lifecycle (create → update → archive) testable end-to-end.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Observability wiring, e2e tests, and CI compliance across all stories.

- [X] T046 Add `tracing` instrumentation to all use cases in `src/application/use_cases/` (list_projects, get_project, create_project, update_project, archive_project) and all `LinearProjectRepository` methods in `src/infrastructure/repositories/project_repository.rs` — `#[tracing::instrument]` on `execute()` methods; `tracing::debug!` for raw GraphQL request and response bodies (activated by `--debug` subscriber level); `--verbose` subscriber maps to `INFO`, `--debug` to `DEBUG` (and implies `--verbose`) per FR-014/FR-015
- [X] T047 Add project command e2e test cases to `tests/e2e.rs` — one test per key acceptance scenario from `spec.md`: list happy path, list JSON schema validation, empty list (exit 0), get by UUID, get by display-ID, get not-found (exit 1), create minimal, create `--dry-run`, create missing required flag (exit 1 before API call), update `--state`, update `--dry-run`, update no flags (exit 1), archive, archive `--dry-run`, archive not-found; verify exit codes and JSON output schema stability (SC-003)
- [X] T048 [P] Run `cargo clippy -- -D warnings` and fix all warnings; run `cargo test` and confirm all unit + e2e tests pass; run `cargo build --release && du -sh target/release/linear` to verify binary < 20 MB (Constraint from plan.md)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **User Stories (Phases 3–7)**: All depend on Phase 2; proceed in priority order P1 → P2 → P3 → P4 → P5 (or in parallel if staffed)
- **Polish (Phase 8)**: Depends on all user story phases being complete

### User Story Dependencies

- **US1 (List)**: Starts after Phase 2; no dependency on other stories
- **US2 (Get)**: Starts after Phase 2; display-ID resolution pattern established here reused by US4/US5
- **US3 (Create)**: Starts after Phase 2; independently testable
- **US4 (Update)**: Starts after Phase 2; verifiable against US2 (get after update) but independently testable
- **US5 (Archive)**: Starts after Phase 2; verifiable against US1 (archived absent from list) but independently testable

### Within Each User Story

- `[P]` tasks (different files) can start concurrently
- GraphQL fragment (infra layer) and use case (application layer) can be implemented in parallel
- Repository method implementation depends on the GraphQL fragment compiling successfully
- CLI command variant depends on the use case being implemented
- `mod.rs` update tasks follow the corresponding file creation

---

## Parallel Opportunities by Story

### Phase 3 (US1 — List)
```
Parallel: T018 (graphql fragment) + T019 (use case + tests)
Then:     T020 (queries/mod.rs) → T021 (repo list() method)
Then:     T022 → T023 → T024 (use case mod + CLI command + routing)
```

### Phase 4 (US2 — Get)
```
Parallel: T025 (graphql queries) + T026 (use case + tests)
Then:     T027 (repo get() method) → T028 → T029 (use case mod + CLI Get variant)
```

### Phase 5 (US3 — Create)
```
Parallel: T030 (graphql mutation) + T031 (use case + tests)
Then:     T032 (mutations/mod.rs) + T033 (repo create() method)
Then:     T034 → T035 (use case mod + CLI Create variant)
```

### Phase 6 (US4 — Update)
```
Parallel: T036 (graphql mutation) + T037 (use case + tests)
Then:     T038 (repo update() method) → T039 → T040 (use case mod + CLI Update variant)
```

### Phase 7 (US5 — Archive)
```
Parallel: T041 (graphql mutation) + T042 (use case + tests)
Then:     T043 (repo archive() method) → T044 → T045 (use case mod + CLI Archive variant)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T017)
3. Complete Phase 3: US1 — List Projects (T018–T024)
4. **STOP and VALIDATE**: Run `linear project list` and `linear project list --output json`
5. Confirm pagination, `--all`, `--team` filter, exit codes, and human/JSON modes

### Incremental Delivery

- Phase 3 complete → `linear project list` ✅
- Phase 4 complete → `linear project get` ✅
- Phase 5 complete → `linear project create` ✅ (first write operation)
- Phase 6 complete → `linear project update` ✅
- Phase 7 complete → `linear project archive` ✅ (full lifecycle)
- Phase 8 complete → Observability + e2e tests + CI compliance ✅

---

## Notes

- `[P]` tasks touch different files with no cross-dependencies — safe to run concurrently
- `[USn]` label maps each task to its user story for traceability
- Unit tests are co-located in `#[cfg(test)]` blocks; write tests FIRST per plan.md Test-First requirement (red-green-refactor)
- Display-ID resolution (`PRJ-1` → UUID) is first established in T027 (US2); extract to a shared helper in `LinearProjectRepository` before implementing US4/US5 to avoid duplication
- Rate-limit retry logic first implemented in T021 (US1); extract to a shared `execute_with_retry()` helper in `src/infrastructure/` before US2 to avoid repetition
- Confirm `cargo build` after each story's GraphQL tasks to catch schema drift early (cynic compile-time validation)
