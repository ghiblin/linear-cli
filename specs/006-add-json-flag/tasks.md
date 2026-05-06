# Tasks: JSON Output Shorthand Flag

**Input**: Design documents from `/specs/006-add-json-flag/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: Included — Constitution Principle II mandates TDD (Red-Green-Refactor) for all features.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)

## Path Conventions

Single project: `src/`, `tests/` at repository root.

---

## Phase 1: Foundational (Blocking Prerequisite)

**Purpose**: Add the shared `resolve_use_json()` helper that all command implementations depend on. Must be complete before any per-command flag work begins.

**⚠️ CRITICAL**: All US1 and US2 tasks depend on this phase.

- [X] T001 Write unit tests for `resolve_use_json()` in `src/cli/output.rs` — cover: `(true, None, false)` → true; `(false, Some("json"), false)` → true; `(false, Some("human"), false)` → false; `(true, Some("human"), false)` → true; `(false, None, false)` → false
- [X] T002 Confirm T001 tests fail (function does not exist yet — Red phase confirmed)
- [X] T003 Implement `pub fn resolve_use_json(per_cmd_json: bool, output: Option<&str>, force_json: bool) -> bool` in `src/cli/output.rs` and confirm T001 tests pass (Green)

**Checkpoint**: `resolve_use_json()` is tested, passing, and ready for use.

---

## Phase 2: User Story 1 — `--json` Shorthand Flag (Priority: P1) 🎯 MVP

**Goal**: Every command that produces structured output accepts `--json` as a per-command boolean flag equivalent to `--output json`.

**Independent Test**: Run `linear issue list --json`, `linear project list --json`, and `linear team list --json` — each must produce valid JSON output without error, and `--help` on each must list the `--json` flag.

### Tests for User Story 1

> **Write these tests FIRST — they MUST FAIL before implementation.**

- [X] T004 [P] [US1] Write snapshot tests for `--json` flag acceptance on issue commands in `tests/integration/issue_json_flag.rs` — test that clap accepts `--json` on `issue list`, `issue get`, `issue create`, `issue update` (parse-only, no API call required)
- [X] T005 [P] [US1] Write snapshot tests for `--json` flag acceptance on project commands in `tests/integration/project_json_flag.rs` — cover `project list`, `project get`, `project create`, `project update`, `project archive`
- [X] T006 [P] [US1] Write snapshot test for `--json` flag acceptance on `team list` in `tests/integration/team_json_flag.rs`
- [X] T007 [US1] Write snapshot tests for `--help` output of `issue list`, `project list`, and `team list` in `tests/integration/help_text.rs` — assert `--json` appears in each help string
- [X] T008 [US1] Confirm T004–T007 all fail (no `--json` field on Args structs yet — Red phase confirmed)

### Implementation for User Story 1

- [X] T009 [US1] Add `/// Use JSON output format (alias for --output json)\n#[arg(long)]` `json: bool` field to `IssueListArgs`, `IssueGetArgs`, `IssueCreateArgs`, `IssueUpdateArgs` in `src/cli/commands/issue.rs`
- [X] T010 [US1] Replace all `let use_json = ...` expressions in issue command handlers with `resolve_use_json(args.json, args.output.as_deref(), force_json)` in `src/cli/commands/issue.rs` — import `resolve_use_json` from `crate::cli::output`
- [X] T011 [US1] Add `json: bool` field (same annotation as T009) to all five project Args structs (`ListArgs`, `GetArgs`, `CreateArgs`, `UpdateArgs`, `ArchiveArgs`) in `src/cli/commands/project.rs`
- [X] T012 [US1] Replace all `let use_json = ...` expressions in project command handlers with `resolve_use_json(args.json, args.output.as_deref(), force_json)` in `src/cli/commands/project.rs`
- [X] T013 [US1] Add `json: bool` field to `team::ListArgs` AND add `output: Option<String>` field (if absent) to `team::ListArgs` in `src/cli/commands/team.rs`
- [X] T014 [US1] Update team list handler to use `resolve_use_json(args.json, args.output.as_deref(), force_json)` in `src/cli/commands/team.rs` — replace the existing `should_use_json(force_json)` call
- [X] T015 [US1] Run T004–T007 tests and confirm they now pass (Green phase)

**Checkpoint**: `linear <cmd> --json` is accepted by all commands; help text lists `--json`; all snapshot tests pass.

---

## Phase 3: User Story 2 — Interoperability (Priority: P2)

**Goal**: Verify that `--json` and `--output json` produce byte-identical output, and that `--json` wins over `--output human` when both are supplied.

**Independent Test**: Unit-level assertion: `resolve_use_json(true, Some("human"), false)` returns `true`, and the inverse `resolve_use_json(false, Some("human"), false)` returns `false`.

### Tests for User Story 2

> **These build on Phase 1 (T001), which already covers the core logic. These add CLI-level precedence verification.**

- [X] T016 [US2] Write unit test in `src/cli/output.rs` for conflict case: `resolve_use_json(true, Some("human"), false)` → true (--json beats --output human)
- [X] T017 [US2] Write unit test in `src/cli/output.rs`: `resolve_use_json(true, Some("json"), false)` → true (both agree, no conflict)
- [X] T018 [US2] Write snapshot test in `tests/integration/issue_json_flag.rs` verifying `--json` and `--output json` produce the same JSON structure for `issue list` (use recorded fixture or mock)
- [X] T019 [US2] Confirm T016–T018 pass without additional implementation (logic already correct from T003)

**Checkpoint**: Conflict resolution is tested and documented. `--json` and `--output json` are verifiably interchangeable.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: CI compliance and final validation.

- [X] T020 [P] Run `cargo clippy -- -D warnings` and resolve any new warnings introduced by added fields or imports in `src/cli/commands/issue.rs`, `src/cli/commands/project.rs`, `src/cli/commands/team.rs`, `src/cli/output.rs`
- [X] T021 [P] Run `cargo fmt --check` and apply `cargo fmt` to all modified files
- [X] T022 Run `cargo test` (full suite) and confirm all tests pass — no regressions in existing `--output json` behavior
- [X] T023 Manually run `linear issue list --json` and `linear issue list --output json` and verify outputs are identical (quickstart.md validation)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — start immediately
- **User Story 1 (Phase 2)**: Depends on Phase 1 completion (needs `resolve_use_json`)
- **User Story 2 (Phase 3)**: Depends on Phase 1 completion; can run in parallel with Phase 2 after T003 (unit tests only build on T001)
- **Polish (Phase 4)**: Depends on Phase 2 and Phase 3 completion

### Within Each Phase

- Tests MUST be written and confirmed failing before implementation (T002, T008, T019)
- `resolve_use_json()` (T003) must be complete before any Args struct changes
- All five Args struct additions in project.rs (T011) can be done in a single pass
- Team command changes (T013–T014) are independent of issue/project changes

### Parallel Opportunities

- T004, T005, T006, T007 — all snapshot test scaffolding can be written in parallel
- T009 and T011 — issue and project Args changes can be done in parallel (different files)
- T020 and T021 — clippy and fmt checks are independent

---

## Parallel Example: User Story 1

```
After T003 is complete, launch in parallel:

Agent A: T004 + T009 + T010   (issue command tests + implementation)
Agent B: T005 + T011 + T012   (project command tests + implementation)
Agent C: T006 + T007 + T013 + T014  (team command test + help text test + implementation)

Then reconvene for T008 (confirm red) → T015 (confirm green)
```

---

## Implementation Strategy

### MVP (User Story 1 Only)

1. Complete Phase 1: Foundational (T001–T003)
2. Complete Phase 2: User Story 1 (T004–T015)
3. **STOP and VALIDATE**: `linear issue list --json` and `linear project list --json` work; help text is correct
4. Ship MVP — US2 is verification of existing behavior, not new behavior

### Incremental Delivery

1. Foundation (T001–T003) → `resolve_use_json()` available
2. US1 (T004–T015) → All commands accept `--json` ← MVP
3. US2 (T016–T019) → Interoperability verified
4. Polish (T020–T023) → CI-clean, ready to merge

---

## Notes

- [P] tasks = different files, no shared state
- TDD is non-negotiable (Constitution Principle II): confirm Red before Green at each checkpoint
- `resolve_use_json()` centralizes all output-mode logic — do not inline new logic in command files
- The global `--json` on `Cli` (passed as `force_json`) is preserved; per-command `--json` is additive
- Snapshot updates (`insta` crate): run `cargo insta review` after implementing to accept new snapshots
