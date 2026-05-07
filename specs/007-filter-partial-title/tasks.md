# Tasks: Filter by Partial Title

**Input**: Design documents from `/specs/007-filter-partial-title/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Included — the constitution (Principle II) mandates TDD (Red-Green-Refactor) as non-negotiable. Tests must be written and confirmed failing before each implementation task.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- TDD tasks marked with 🔴 must fail before the paired implementation task

---

## Phase 1: Setup

No new dependencies or project structure required. The existing `Cargo.toml`, `schema.graphql`, and build pipeline support all changes.

---

## Phase 2: Foundational (Blocking Prerequisites)

No cross-story prerequisites. US1 (issues) and US2 (projects) touch entirely separate files and can proceed in parallel once implementation begins. Proceed directly to user story phases.

---

## Phase 3: User Story 1 — Find Issues by Keyword (Priority: P1) 🎯 MVP

**Goal**: `linear issue list --title <keyword>` returns only issues whose title contains the keyword (case-insensitive). Empty or omitted flag is a no-op.

**Independent Test**: `linear issue list --title "login"` returns issues containing "login"; `linear issue list --title "LOGIN"` returns the same set; `linear issue list --title "xyznonexistent"` returns zero items with exit code 0.

### Tests for User Story 1 🔴

> **Write these tests FIRST. Confirm they FAIL before writing any implementation.**

- [x] T001 [US1] 🔴 Add failing unit test `title_contains_is_threaded_to_filter` in `src/infrastructure/graphql/queries/issue_queries.rs` — construct a `ListIssuesInput` with `title_contains: Some("login")` and assert `build_issue_filter` returns a filter with `title.contains_ignore_case == Some("login")`
- [x] T002 [US1] 🔴 Add failing unit test `empty_title_is_treated_as_no_filter` in `src/infrastructure/graphql/queries/issue_queries.rs` — assert `build_issue_filter` with `title_contains: Some("")` returns the same result as `title_contains: None` (no title key in filter)
- [x] T003 [US1] 🔴 Add failing unit test `list_issues_use_case_passes_title_contains` in `src/application/use_cases/list_issues.rs` — mock repo expects `list` called with an input where `title_contains == Some("fix")`; assert execute passes it through unchanged

### Implementation for User Story 1

- [x] T004 [US1] Add `title_contains: Option<String>` to `ListIssuesInput` in `src/domain/entities/issue.rs`; update `default_input()` in the existing test helper to include `title_contains: None`
- [x] T005 [US1] Add `#[cynic(rename = "containsIgnoreCase", skip_serializing_if = "Option::is_none")] pub contains_ignore_case: Option<String>` to `StringComparatorInput` in `src/infrastructure/graphql/queries/issue_queries.rs`
- [x] T006 [US1] Add `#[cynic(skip_serializing_if = "Option::is_none")] pub title: Option<StringComparatorInput>` to `IssueFilterInput` in `src/infrastructure/graphql/queries/issue_queries.rs`; update `build_issue_filter` to set `title` from `input.title_contains` (skip when `None` or `""`); update the early-return guard to include the title check
- [x] T007 [US1] Add `--title <SUBSTRING>` flag (`#[arg(long)] title: Option<String>`) to `IssueSubcommand::List` in `src/cli/commands/issue.rs`; thread it into `ListIssuesInput { title_contains: title, .. }` in the List arm handler

**Checkpoint**: Run `cargo test` — all three T001–T003 tests must now pass. Run `cargo clippy -- -D warnings` and `cargo fmt --check`.

---

## Phase 4: User Story 2 — Find Projects by Partial Name (Priority: P2)

**Goal**: `linear project list --name <keyword>` returns only projects whose name contains the keyword (case-insensitive). Composes correctly with `--team`.

**Independent Test**: `linear project list --name "Platform"` returns only projects containing "Platform"; `linear project list --name "platform"` returns the same set; combining with `--team <id>` returns projects matching both criteria.

### Tests for User Story 2 🔴

> **Write these tests FIRST. Confirm they FAIL before writing any implementation.**

- [x] T008 [P] [US2] 🔴 Add failing unit test `name_contains_filters_to_matching_projects` in `src/application/use_cases/list_projects.rs` — mock `ProjectRepository::list` expects a call with `name_contains: Some("Platform")`; assert `execute` passes it through
- [x] T009 [P] [US2] 🔴 Add failing unit test `empty_name_is_treated_as_no_filter` in `src/application/use_cases/list_projects.rs` — assert `execute` with `name_contains: Some("")` calls repo with `name_contains: None` (or `Some("")` treated as no-op in fetch, per research decision — whichever is implemented)

### Implementation for User Story 2

- [x] T010 [US2] Add `name_contains: Option<String>` parameter to `ProjectRepository::list()` in `src/domain/repositories/project_repository.rs`; update the `#[allow(dead_code)]` mock impl in `list_projects.rs` tests to add the new parameter
- [x] T011 [P] [US2] Add `#[cynic(rename = "containsIgnoreCase")] pub contains_ignore_case: Option<String>` to `StringComparator` in `src/infrastructure/graphql/queries/project_queries.rs`; add `pub name: Option<StringComparator>` to `ProjectFilter`
- [x] T012 [P] [US2] Add `pub filter: Option<ProjectFilter>` to both `ProjectsVariables` and `TeamProjectsVariables` in `src/infrastructure/graphql/queries/project_queries.rs`; update `#[arguments]` on `ProjectsQuery` and `TeamWithProjects` to include `filter: $filter`; update `fetch_projects` signature to accept `name_contains: Option<&str>` and construct a `ProjectFilter` with `name.contains_ignore_case` when non-empty
- [x] T013 [US2] Update `LinearProjectRepository::list()` in `src/infrastructure/repositories/project_repository.rs` to accept and forward `name_contains` to `fetch_projects`
- [x] T014 [US2] Update `ListProjects::execute()` in `src/application/use_cases/list_projects.rs` to accept `name_contains: Option<String>` and forward to `self.repo.list()`; update all three existing unit tests to pass `None` as the new argument
- [x] T015 [US2] Add `#[arg(long, help = "Filter by partial name (case-insensitive)")] pub name: Option<String>` to `ListArgs` in `src/cli/commands/project.rs`; thread it into `uc.execute(team_id, first, after, all, name)` in the List handler

**Checkpoint**: Run `cargo test` — all T008–T009 tests must now pass. Run `cargo clippy -- -D warnings` and `cargo fmt --check`.

---

## Phase 5: User Story 3 — Combine Title Filter with Existing Filters (Priority: P3)

**Goal**: Verify `--title` composes correctly with `--state`, `--team`, and that `--name` composes with `--team`. No new implementation code is required — composition falls out of the existing filter builder and query design.

**Independent Test**: `linear issue list --title "auth" --state "in_progress"` returns only in-progress issues whose title contains "auth"; `linear project list --name "Platform" --team <id>` returns only that team's projects matching "Platform".

### Tests for User Story 3 🔴

> **Write these tests FIRST. Confirm they FAIL before writing any implementation.**

- [x] T016 [P] [US3] 🔴 Add unit test `title_and_state_filter_compose` in `src/infrastructure/graphql/queries/issue_queries.rs` — call `build_issue_filter` with both `title_contains: Some("auth")` and `state_name: Some("in_progress")`; assert the returned `IssueFilterInput` has both `title` and `state` set
- [x] T017 [P] [US3] 🔴 Add unit test `title_and_team_filter_compose` in `src/infrastructure/graphql/queries/issue_queries.rs` — call `build_issue_filter` with both `title_contains: Some("deploy")` and a non-None `team_id`; assert both `title` and `team` fields are set in the filter

### Implementation for User Story 3

No new implementation tasks. T016 and T017 should pass once Phase 3 is complete — if they don't, fix `build_issue_filter` to ensure the early-return guard does not skip title when other filters are also set.

**Checkpoint**: `cargo test` green. Both composition tests pass. Manual spot-check: `linear issue list --title "x" --state "y"` produces a valid (possibly empty) result.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T018 [P] Update `--help` text in `src/cli/commands/issue.rs` — add `help = "Filter by partial title (case-insensitive)"` to the `--title` arg annotation
- [x] T019 [P] Update `--help` text in `src/cli/commands/project.rs` — confirm `--name` help text is accurate (already set in T015)
- [x] T020 Run full `cargo test && cargo clippy -- -D warnings && cargo fmt --check` and resolve any warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1–2**: No-op — start immediately
- **Phase 3 (US1)** and **Phase 4 (US2)**: Independent — can run in parallel (no shared files)
- **Phase 5 (US3)**: Depends on Phase 3 (US1) complete; T017 composition test also passes after Phase 3
- **Phase 6 (Polish)**: Depends on Phases 3–5

### User Story Dependencies

- **US1 (P1)**: No dependencies — start immediately
- **US2 (P2)**: No dependencies — can start in parallel with US1
- **US3 (P3)**: Depends on US1 complete; no additional implementation

### Within Each User Story

1. 🔴 Write tests (must fail)
2. Run `cargo test` — confirm failures
3. Implement (domain → infra → app → cli)
4. Run `cargo test` — confirm green
5. Refactor if needed

### Parallel Opportunities Within US1

```
T001, T002, T003 — write all three failing tests simultaneously (different test functions)
T004 — add domain field (unblocks T005, T006 which can then run in parallel)
T005, T006 — GraphQL type changes can be made simultaneously
T007 — CLI flag (after T004 and T006 compile)
```

### Parallel Opportunities Within US2

```
T008, T009 — write both failing tests simultaneously
T010 — update domain trait (unblocks T011–T015)
T011, T012 — GraphQL changes can be made simultaneously
T013 — repo impl (after T012)
T014 — use case (after T010, T013)
T015 — CLI flag (after T014)
```

---

## Parallel Example: Running US1 and US2 Concurrently

Since all file paths are disjoint, a team can work on both stories simultaneously:

```
Agent A (US1):            Agent B (US2):
T001–T003 (tests)         T008–T009 (tests)
T004–T006 (domain+infra)  T010–T012 (domain+infra)
T007 (CLI)                T013–T015 (repo+app+CLI)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 3 (US1) — `issue list --title` working
2. **STOP and VALIDATE**: `linear issue list --title "auth"` returns matching issues
3. Merge as MVP; ship if desired

### Incremental Delivery

1. Phase 3 → `issue list --title` (MVP)
2. Phase 4 → `project list --name`
3. Phase 5 → Composition validation (tests only, no new code)
4. Phase 6 → Polish

---

## Notes

- [P] tasks touch different files — no merge conflicts expected
- TDD is non-negotiable (Constitution Principle II): confirm 🔴 test fails before each implementation step
- `cargo build` validates GraphQL input types against `schema.graphql` at build time — any typo in cynic field names will produce a build error, not a runtime error
- Commit after each phase checkpoint
