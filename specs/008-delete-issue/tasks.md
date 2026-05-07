# Tasks: Delete Issue

**Input**: Design documents from `specs/008-delete-issue/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)

---

## Phase 1: Setup

No project setup needed — all additions slot into the existing single-project layout.

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: Extend the domain repository trait to declare `delete`. This is the single blocker for all user story work.

**⚠️ CRITICAL**: No user story work can begin until T001 is complete.

- [ ] T001 Add `delete(id: IssueId) -> Result<(), DomainError>` method to the `IssueRepository` trait in `src/domain/repositories/issue_repository.rs`

**Checkpoint**: Trait extended. All downstream layers can now be compiled against it.

---

## Phase 3: User Story 1 - Delete an Issue by ID (Priority: P1) 🎯 MVP

**Goal**: `linear issue delete <id>` calls the Linear API and removes the issue. Exit 0 on success, non-zero on error. Human and JSON output supported.

**Independent Test**: Run `linear issue delete <valid-id>`, verify success message printed and the issue no longer appears in `linear issue list`.

### TDD — Write Failing Tests First ⚠️

> **Write these tests FIRST and confirm they FAIL (Red) before writing implementation code.**

- [ ] T002 [US1] Write failing unit tests for `DeleteIssue` use case (success path + not_found error path using `mockall`) inside `#[cfg(test)]` module in `src/application/use_cases/delete_issue.rs`

### Implementation for User Story 1

- [ ] T003 [P] [US1] Add `IssueDeleteVariables`, `IssueArchivePayload`, `IssueDeleteMutation` cynic types and `delete_issue()` async fn to `src/infrastructure/graphql/mutations/issue_mutations.rs` (can run in parallel with T002 — different file)
- [ ] T004 [US1] Create `src/application/use_cases/delete_issue.rs` with `DeleteOutcome` enum and `DeleteIssue` use case struct; implement `execute(id, dry_run=false)` to make T002 tests green
- [ ] T005 [US1] Register `pub mod delete_issue;` in `src/application/use_cases/mod.rs`
- [ ] T006 [US1] Implement `delete()` on `LinearIssueRepository` in `src/infrastructure/repositories/issue_repository.rs` using the `delete_issue()` fn from T003 (depends on T003)
- [ ] T007 [US1] Add `Delete { id, output, json }` variant to `IssueSubcommand` and live-path handler arm in `src/cli/commands/issue.rs`; print `"Deleted issue {id}"` on success, JSON `{"deleted":true,"id":"..."}` when `--json` (depends on T004, T005, T006)

**Checkpoint**: `cargo build` and `cargo test` pass. `linear issue delete <id>` works end-to-end.

---

## Phase 4: User Story 2 - Dry-Run Delete (Priority: P2)

**Goal**: `linear issue delete <id> --dry-run` previews what would be deleted without making any API call. Instant response.

**Independent Test**: Run `linear issue delete <id> --dry-run`; confirm preview output appears and no issue is deleted (verify via `linear issue list`).

### TDD — Write Failing Tests First ⚠️

> **Write this test FIRST and confirm it FAILS (Red) before writing implementation.**

- [ ] T008 [US2] Add failing unit test for dry-run path in the `#[cfg(test)]` module of `src/application/use_cases/delete_issue.rs` (dry_run=true must return `Ok(Deleted)` without calling repo)

### Implementation for User Story 2

- [ ] T009 [US2] Extend `DeleteIssue::execute()` to short-circuit when `dry_run=true` (return early without calling repo) in `src/application/use_cases/delete_issue.rs` — makes T008 green
- [ ] T010 [US2] Add `--dry-run` flag to `Delete` subcommand variant; add dry-run output `"[dry-run] Would delete issue: {id}"` (human) and `{"dry_run":true,"id":"..."}` (JSON) in `src/cli/commands/issue.rs`

**Checkpoint**: `cargo test` passes. Dry-run exits instantly, no API call made.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T011 [P] Run `cargo clippy -- -D warnings` from repo root; fix any new warnings introduced by this feature
- [ ] T012 [P] Run `cargo fmt --check`; fix any formatting issues
- [ ] T013 Run `cargo test` (full suite); confirm all tests pass including pre-existing ones

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 2)**: No dependencies — start immediately
- **User Story 1 (Phase 3)**: Depends on T001 completion
- **User Story 2 (Phase 4)**: Depends on Phase 3 (US1) completion
- **Polish (Phase 5)**: Depends on all user stories complete

### User Story Dependencies

- **T002**: Depends on T001 (trait must exist to write the mock impl)
- **T003**: Depends on T001 (parallel with T002 — different file)
- **T004**: Depends on T001, T002 (implement to make tests green)
- **T005**: Depends on T004
- **T006**: Depends on T001, T003
- **T007**: Depends on T004, T005, T006
- **T008**: Depends on T007 (US1 complete)
- **T009**: Depends on T008
- **T010**: Depends on T009

### Parallel Opportunities

- T002 and T003 can run concurrently (different files, same phase)
- T011 and T012 can run concurrently (independent lint/fmt checks)

---

## Parallel Example: User Story 1

```bash
# After T001 completes, launch T002 and T003 concurrently:
Task: "Write failing unit tests for DeleteIssue use case in src/application/use_cases/delete_issue.rs"
Task: "Add IssueDeleteMutation cynic types + delete_issue() fn in src/infrastructure/graphql/mutations/issue_mutations.rs"
```

---

## Implementation Strategy

### MVP (User Story 1 Only)

1. Complete T001 (Foundational)
2. Complete T002–T007 (User Story 1)
3. **STOP and VALIDATE**: `cargo test` passes, `linear issue delete <id>` works
4. Ship MVP

### Incremental Delivery

1. T001 → foundation
2. T002–T007 → live delete working (MVP)
3. T008–T010 → dry-run working
4. T011–T013 → polish complete

---

## Notes

- Constitution Principle II (TDD): T002 and T008 MUST be written and confirmed failing before their respective implementation tasks
- `IssueArchivePayload` in `issue_mutations.rs` follows the same shape as `ProjectArchivePayload` in `project_mutations.rs`
- `DeleteIssue` use case follows the same shape as `ArchiveProject` in `src/application/use_cases/archive_project.rs`
- The `permanentlyDelete` mutation argument is intentionally out of scope
