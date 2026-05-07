# Tasks: Issue Get — Optional Detail Flags

**Input**: Design documents from `specs/009-issue-get-details/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included — Constitution Principle II (Test-First) is NON-NEGOTIABLE.

**Organization**: Tasks are grouped by user story. All changes are confined to a single file: `src/cli/commands/issue.rs`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story this task belongs to
- All tasks touch `src/cli/commands/issue.rs` unless noted

---

## Phase 1: Foundational (TDD Scaffold)

**Purpose**: Refactor the display function to be testable and write failing tests that capture all three user stories. No user story work begins until the test scaffold exists.

**⚠️ CRITICAL**: Tests must be written first and confirmed failing before any implementation.

- [X] T001 Write failing unit tests for `render_issue_human` in `src/cli/commands/issue.rs` — cover: (a) description shown when `show_description=true`, (b) description hidden when false, (c) sub-issues shown when `show_subtasks=true`, (d) sub-issues hidden when false, (e) both flags true shows both sections. Reference the function before it exists to cause compile failure (Red).
- [X] T002 Extract a `render_issue_human(issue: &Issue, show_description: bool, show_subtasks: bool) -> String` helper from `format_issue_human` in `src/cli/commands/issue.rs`; update `format_issue_human` to call it and print the result. Tests from T001 now compile but description and subtasks cases still fail.

**Checkpoint**: `cargo test` compiles; description and subtasks test cases are failing (Red confirmed).

---

## Phase 2: User Story 1 — View Issue Description (Priority: P1) 🎯 MVP

**Goal**: `issue get <id> --description` prints the issue description below basic fields.

**Independent Test**: `linear issue get ENG-X --description` prints the `Description:` block when the issue has a description; omits it when absent.

- [X] T003 [US1] Implement `show_description` branch in `render_issue_human` in `src/cli/commands/issue.rs`: when `show_description=true` and `issue.description` is `Some`, append a `Description:\n  <text>` section to the output. US1 unit tests from T001 now pass (Green).
- [X] T004 [US1] Add `#[arg(long)] description: bool` to `IssueSubcommand::Get` and destructure it in the `Get` match arm of `run_issue`; pass it as `show_description` to `format_issue_human` in `src/cli/commands/issue.rs`.

**Checkpoint**: `cargo test` green; `linear issue get <id> --description` prints the description; default output unchanged.

---

## Phase 3: User Story 2 — View Issue Subtasks (Priority: P2)

**Goal**: `issue get <id> --subtasks` prints the sub-issue list; default output omits it.

**Independent Test**: `linear issue get ENG-X --subtasks` prints the `Sub-issues:` block when children exist; default `issue get ENG-X` no longer shows sub-issues unconditionally.

- [X] T005 [US2] Implement `show_subtasks` branch in `render_issue_human` in `src/cli/commands/issue.rs`: gate the sub-issue block behind `show_subtasks=true` and remove the existing unconditional sub-issue display. US2 unit tests from T001 now pass (Green).
- [X] T006 [US2] Add `#[arg(long)] subtasks: bool` to `IssueSubcommand::Get` and destructure it in the `Get` match arm; pass it as `show_subtasks` to `format_issue_human` in `src/cli/commands/issue.rs`.

**Checkpoint**: `cargo test` green; `linear issue get <id> --subtasks` shows sub-issues; default output no longer includes sub-issues.

---

## Phase 4: User Story 3 — Combine Both Flags (Priority: P3)

**Goal**: `issue get <id> --description --subtasks` shows both sections in one call.

**Independent Test**: `linear issue get ENG-X --description --subtasks` prints both the `Description:` block and the `Sub-issues:` block.

- [X] T007 [US3] Run `cargo test` and verify the combined-flags unit test (scenario e from T001) passes. No additional code changes needed — T003 + T005 already implement independent branches that compose correctly.

**Checkpoint**: All unit tests green including combined-flags case.

---

## Phase 5: Polish & Validation

**Purpose**: Confirm the implementation is clean and consistent before completion.

- [X] T008 [P] Run `cargo clippy -- -D warnings` from repo root and resolve any warnings in `src/cli/commands/issue.rs`.
- [X] T009 [P] Run `cargo fmt --check` from repo root and apply `cargo fmt` if needed.
- [X] T010 Run `cargo test` from repo root and confirm all tests pass (unit + any existing integration tests).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — start immediately
- **US1 (Phase 2)**: Depends on Foundational complete (T001 + T002 done)
- **US2 (Phase 3)**: Depends on Foundational complete; independent of US1
- **US3 (Phase 4)**: Depends on US1 + US2 complete
- **Polish (Phase 5)**: Depends on all user stories complete

### Within Each Phase

- Tests (T001) MUST be written and confirmed failing before T002
- T002 (extract helper) must complete before T003 or T005
- T003 before T004 (implement flag before wiring to CLI)
- T005 before T006 (implement flag before wiring to CLI)
- T007 is a verification step, not a code change
- T008 and T009 can run in parallel (different concerns)

### All Tasks Are Sequential Within a File

All tasks modify `src/cli/commands/issue.rs`. No intra-file parallelism is possible.

---

## Implementation Strategy

### MVP (User Story 1 Only)

1. Complete Phase 1: Foundational (T001–T002)
2. Complete Phase 2: User Story 1 (T003–T004)
3. **STOP and VALIDATE**: `linear issue get <id> --description` works
4. Proceed to US2 and US3

### Full Delivery

1. Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5
2. Each story adds one flag; Polish confirms CI gates pass

---

## Notes

- No domain, application, or infrastructure files change — only `src/cli/commands/issue.rs`
- JSON output (`IssueDto`) is not modified; `--description` and `--subtasks` are human-output-only
- Sub-issues are currently shown unconditionally in `format_issue_human`; T005 removes that behavior
- `render_issue_human` returning `String` enables unit tests without stdout capture
