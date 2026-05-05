# Tasks: Improve Project List Identifiers

**Input**: Design documents from `specs/005-improve-project-list/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: TDD is **required** per Constitution Principle II and quickstart.md. Write tests first; ensure they fail before implementing.

**Organization**: All changes are confined to `src/cli/commands/project.rs`. Tasks are grouped by user story to enable independent delivery and validation of each increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (independent test functions or read-only checks)
- **[Story]**: Which user story this task belongs to
- All tasks target `src/cli/commands/project.rs` unless stated otherwise

---

## Phase 1: Setup

**Purpose**: Verify baseline before any changes

- [ ] T001 Run `cargo test` to confirm all existing tests pass in `src/cli/commands/project.rs` before any modifications

---

## Phase 2: Foundational (No Blocking Prerequisites)

**Purpose**: No new infrastructure required. `slug_id` is already present in the `Project` domain entity, DTO types exist in `src/cli/commands/project.rs`, and slug-based resolution already works via `ProjectId::parse()` and `LinearProjectRepository::resolve_id()` (see research.md §3). User stories can start immediately after Phase 1.

*(No tasks — proceed to Phase 3)*

---

## Phase 3: User Story 1 — View Slug in Table Output (Priority: P1) 🎯 MVP

**Goal**: `project list` displays a slug column; `project get` displays a `Slug:` line immediately after `Name:`.

**Independent Test**: `cargo test` passes with snapshot tests for `project list` and `project get` human output asserting slug column/line present. Verifiable in isolation; delivers the core value of making slugs visible.

### Tests for User Story 1 (TDD — write first, ensure they FAIL before T004)

- [ ] T002 [US1] Write failing insta snapshot test for `project list` human output asserting slug column (`{:<22}`) appears between name and state columns in `src/cli/commands/project.rs`
- [ ] T003 [US1] Write failing insta snapshot test for `project get` human output asserting `Slug:` line appears immediately after `Name:` line in `src/cli/commands/project.rs`

### Implementation for User Story 1

- [ ] T004 [US1] Update `project list` format string: change name column from `{:<40}` to `{:<35}` and insert slug column `{:<22}` between name and state in `src/cli/commands/project.rs`
- [ ] T005 [US1] Add `println!("Slug:        {}", project.slug_id)` immediately after the `Name:` println in `project get` human output in `src/cli/commands/project.rs`
- [ ] T006 [US1] Run `cargo insta review` to accept updated snapshots; confirm T002 and T003 pass with `cargo test`

**Checkpoint**: `cargo test` passes; `project list` shows slug column per `specs/005-improve-project-list/contracts/project-commands.md`; `project get` shows `Slug:` line

---

## Phase 4: User Story 2 — Slug in Create/Update Messages (Priority: P2)

**Goal**: `project create` and `project update` human success messages display the human-readable slug instead of the UUID.

**Independent Test**: `cargo test` passes with snapshot tests for create and update success messages showing slug (not UUID) in the output strings. Independent of US1 — different println sites.

### Tests for User Story 2 (TDD — write first, ensure they FAIL before T009)

- [ ] T007 [US2] Write failing insta snapshot test for `project create` human success message asserting slug appears in parentheses (e.g. `Created project: "Q3 Platform" (q3-platform)`) in `src/cli/commands/project.rs`
- [ ] T008 [US2] Write failing insta snapshot test for `project update` human success message asserting slug used as identifier (e.g. `Updated project q3-platform: state → started`) in `src/cli/commands/project.rs`

### Implementation for User Story 2

- [ ] T009 [US2] Change `project create` success println from `project.id` to `project.slug_id` in the parenthetical (before: `(uuid)`, after: `(slug)`) in `src/cli/commands/project.rs`
- [ ] T010 [US2] Change `project update` success println from `project.id` to `project.slug_id` as the project identifier in `src/cli/commands/project.rs`
- [ ] T011 [US2] Run `cargo insta review` to accept updated snapshots; confirm T007 and T008 pass with `cargo test`

**Checkpoint**: `cargo test` passes; create message shows slug; update message shows slug per contracts

---

## Phase 5: User Story 3 — Include Slug in JSON Output (Priority: P3)

**Goal**: `ProjectDto` gains `slug_id: String`; `MutationResultDto` gains `slug_id: String`. Both are serialized in JSON output for list, get, create, and update commands.

**Independent Test**: `cargo test` passes with unit tests `project_dto_includes_slug_id` and `mutation_result_dto_includes_slug_id`. Independent of US1/US2 — DTO struct changes do not affect human output paths.

### Tests for User Story 3 (TDD — write first, ensure they FAIL before T014)

- [ ] T012 [P] [US3] Write failing unit test `project_dto_includes_slug_id`: construct a `Project` fixture with a known `slug_id`, call `ProjectDto::from(&project)`, assert `dto.slug_id` equals the fixture value in `src/cli/commands/project.rs`
- [ ] T013 [P] [US3] Write failing unit test `mutation_result_dto_includes_slug_id`: construct a `MutationResultDto` with a `slug_id` value, call `serde_json::to_string`, assert the output string contains `"slug_id"` in `src/cli/commands/project.rs`

### Implementation for User Story 3

- [ ] T014 [US3] Add `slug_id: String` field to `ProjectDto` struct in `src/cli/commands/project.rs`
- [ ] T015 [US3] Add `slug_id: p.slug_id.clone()` to `From<&Project> for ProjectDto` implementation in `src/cli/commands/project.rs`
- [ ] T016 [US3] Add `slug_id: String` field to `MutationResultDto` struct in `src/cli/commands/project.rs`
- [ ] T017 [US3] Add `slug_id: project.slug_id.clone()` to `MutationResultDto` construction in both `project create` and `project update` command handlers in `src/cli/commands/project.rs`

**Checkpoint**: `cargo test` passes; `project list --json` and `project get --json` include `"slug_id"`; create/update JSON include `"slug_id"` per contracts

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories per quickstart.md verification steps

- [ ] T018 [P] Run `cargo test` and confirm all tests pass (T002–T013 all green)
- [ ] T019 [P] Run `cargo clippy -- -D warnings` and resolve any new warnings in `src/cli/commands/project.rs`
- [ ] T020 [P] Run `cargo fmt --check` and apply `cargo fmt` if formatting drift exists
- [ ] T021 Manually verify all four command outputs match `specs/005-improve-project-list/contracts/project-commands.md` (list, get, create, update — both human and JSON)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Skipped — no blocking prerequisites
- **US1 (Phase 3)**: Depends on Phase 1 only — human output format string changes
- **US2 (Phase 4)**: Depends on Phase 1 only — independent of US1 (different println sites)
- **US3 (Phase 5)**: Depends on Phase 1 only — DTO struct additions, independent of US1/US2
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: No dependency on US2 or US3 — touches only list/get format strings
- **US2 (P2)**: No dependency on US1 or US3 — touches only create/update format strings
- **US3 (P3)**: No dependency on US1 or US2 — touches only DTO structs and their construction sites

### Within Each User Story

- Tests MUST be written first and MUST fail before implementation
- Snapshot tests (T002, T003, T007, T008): run `cargo insta review` after implementation to accept new baselines
- Unit tests (T012, T013): pure `cargo test` red → green cycle

### Parallel Opportunities

- T012 and T013 (US3 tests for different structs) can be written in parallel by separate developers
- T018, T019, T020 (Polish checks) are read-only and can run in parallel
- US1, US2, and US3 can be worked in parallel by different developers (same file — coordinate on merge)

---

## Parallel Example: User Story 3

```bash
# Write both DTO unit tests simultaneously (different test functions, no conflict):
Task T012: project_dto_includes_slug_id test
Task T013: mutation_result_dto_includes_slug_id test
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 3: User Story 1 (T002–T006)
3. **STOP and VALIDATE**: `cargo test` passes; slug column visible in `project list`
4. Ship/demo — users can copy slugs without `--json` flag

### Incremental Delivery

1. Phase 1 + Phase 3 (US1) → Slug visible in list and detail output ✅
2. Phase 4 (US2) → Create/update messages use slug instead of UUID ✅
3. Phase 5 (US3) → JSON output includes `slug_id` for scripting ✅
4. Phase 6 (Polish) → Clean build, clippy, fmt ✅

### Suggested MVP Scope

**Phase 3 (US1) alone** is a complete, shippable increment:
- Users can see and copy slugs from `project list` and `project get`
- No other changes required
- Total: 5 tasks (T002–T006), all in one file

---

## Notes

- [P] tasks = independent test functions or read-only commands — safe to dispatch in parallel
- All source changes confined to `src/cli/commands/project.rs` — no other files modified
- Slug resolution via `ProjectId::Slug` is already fully implemented (research.md §3) — no changes needed to accept slugs in subcommands
- Insta snapshot tests require `cargo insta review` to accept new baselines on first run
- Verify final output against `specs/005-improve-project-list/contracts/project-commands.md` for all commands
