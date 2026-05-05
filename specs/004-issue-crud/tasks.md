# Tasks: Issue CRUD Operations

**Input**: Design documents from `/specs/004-issue-crud/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/cli-schema.md ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create missing file stubs and module declarations needed by all phases.

- [X] T001 Create src/infrastructure/graphql/queries/issue_queries.rs as an empty module stub and add `pub mod issue_queries;` to src/infrastructure/graphql/queries/mod.rs
- [X] T002 Create src/infrastructure/graphql/mutations/issue_mutations.rs as an empty module stub and add `pub mod issue_mutations;` to src/infrastructure/graphql/mutations/mod.rs
- [X] T003 Create src/application/use_cases/get_issue.rs, src/application/use_cases/create_issue.rs, src/application/use_cases/update_issue.rs as empty module stubs and register them in src/application/use_cases/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain model changes that every user story phase depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Create WorkflowStateRef value object in src/domain/value_objects/workflow_state_ref.rs: `pub struct WorkflowStateRef { pub id: String, pub name: String, pub state_type: String }` deriving Debug, Clone, Serialize, Deserialize, PartialEq, Eq; implement Display showing `name`
- [X] T005 [P] Create LabelId value object in src/domain/value_objects/label_id.rs: `pub struct LabelId(String)` with `new(s: String) -> Result<Self, DomainError>` (reject empty), implementing Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display — same pattern as IssueId
- [X] T006 Update src/domain/value_objects/mod.rs to add `pub mod workflow_state_ref;`, `pub mod label_id;`, and `pub use workflow_state_ref::WorkflowStateRef;`, `pub use label_id::LabelId;` (T004 and T005 must be complete first)
- [X] T007 Extend src/domain/entities/issue.rs: (a) replace `WorkflowState` with `WorkflowStateRef` on the `Issue` struct and its constructor; (b) add all new fields: `identifier: String`, `description: Option<String>`, `assignee_id: Option<UserId>`, `assignee_name: Option<String>`, `label_ids: Vec<LabelId>`, `due_date: Option<String>`, `estimate: Option<f64>`, `parent_id: Option<IssueId>`, `parent_title: Option<String>`, `sub_issues: Vec<SubIssueRef>`, `created_at: String`, `updated_at: String`; (c) add `SubIssueRef { pub id: IssueId, pub title: String, pub identifier: String }` struct deriving Debug, Clone, Serialize, Deserialize; (d) add `WorkflowStateInfo { pub id: String, pub name: String, pub state_type: String }` struct; (e) add accessors for all new fields; (f) update existing unit tests to use WorkflowStateRef instead of WorkflowState enum (T006 must be complete first)
- [X] T008 Add domain input structs to src/domain/entities/issue.rs: `ListIssuesInput { team_id: Option<TeamId>, project_id: Option<ProjectId>, state_name: Option<String>, assignee_id: Option<UserId>, priority: Option<Priority>, label_ids: Vec<LabelId>, limit: i32, cursor: Option<String>, all_pages: bool }`, `ListIssuesResult { items: Vec<Issue>, next_cursor: Option<String>, has_next_page: bool }`, `CreateIssueInput { title: String, team_id: TeamId, project_id: ProjectId, description: Option<String>, priority: Option<Priority>, assignee_id: Option<UserId>, label_ids: Vec<LabelId>, due_date: Option<String>, estimate: Option<f64>, parent_id: Option<IssueId> }`, `UpdateIssueInput { title: Option<String>, description: Option<String>, state_id: Option<String>, priority: Option<Priority>, assignee_id: Option<UserId>, due_date: Option<String>, estimate: Option<f64>, parent_id: Option<IssueId>, no_parent: bool }` — all deriving Debug, Clone
- [X] T009 Extend IssueRepository trait in src/domain/repositories/issue_repository.rs: replace `list(&self, team_id: TeamId)` with `list(&self, input: ListIssuesInput) -> Result<ListIssuesResult, DomainError>`; add `create(&self, input: CreateIssueInput) -> Result<Issue, DomainError>`; add `update(&self, id: IssueId, input: UpdateIssueInput) -> Result<Issue, DomainError>`; add `list_workflow_states(&self, team_id: TeamId) -> Result<Vec<WorkflowStateInfo>, DomainError>` (T007, T008 must be complete first)

**Checkpoint**: Domain model complete — all user story phases can now begin.

---

## Phase 3: User Story 1 - List Issues (Priority: P1) 🎯 MVP

**Goal**: Complete `linear issue list` with full filter support and pagination.

**Independent Test**: Run `linear issue list` and confirm paginated issues with title, state, priority, assignee, team; run `linear issue list --output json` and confirm JSON schema from contracts/cli-schema.md.

- [X] T010 [US1] Implement issue list GraphQL query in src/infrastructure/graphql/queries/issue_queries.rs: define `IssueFilter`, `TeamFilter`, `ProjectFilter`, `StateFilter`, `AssigneeFilter`, `PriorityComparator`, `LabelFilter` cynic InputObjects; define `WorkflowStateNode`, `UserNode`, `IssueNode`, `IssueConnection`, `PageInfoNode` cynic QueryFragments; define `PaginationOrderBy` cynic Enum; define `IssueListVariables` and `IssueListQuery` cynic QueryFragment; implement `fetch_issues(client, api_key, input: &ListIssuesInput)` async fn returning `(Vec<IssueNode>, PageInfoNode)`
- [X] T011 [US1] Implement LinearIssueRepository::list in src/infrastructure/repositories/issue_repository.rs: map ListIssuesInput fields to IssueFilter variables; call `fetch_issues`; convert IssueNode → Issue (mapping all fields including WorkflowStateRef from WorkflowStateNode); return ListIssuesResult (T010 must be complete first)
- [X] T012 [US1] Rewrite ListIssues use case in src/application/use_cases/list_issues.rs: change signature to `execute(&self, input: ListIssuesInput) -> Result<ListIssuesResult, ApplicationError>`; when `input.all_pages` is true, loop fetching pages using cursor until `has_next_page` is false, accumulating all issues; update mock in `#[cfg(test)]` to match the new IssueRepository trait signature (T009 must be complete first)
- [X] T013 [US1] Update IssueSubcommand::List in src/cli/commands/issue.rs: add args `--team`, `--project`, `--state`, `--assignee`, `--priority` (parse to Priority), `--label` (Vec<String>, repeatable), `--all`, `--limit` (default 50), `--cursor`, `--output`; build ListIssuesInput; call ListIssues use case; human output: tabular rows of `identifier | title | state.name | priority | assignee_name | team_id` with footer "Showing N of M issues…"; JSON output: serialize ListIssuesResult per contracts/cli-schema.md schema (T012 must be complete first)

**Checkpoint**: `linear issue list` fully functional with filters and pagination.

---

## Phase 4: User Story 2 - View Issue Details (Priority: P2)

**Goal**: Complete `linear issue get <id>` returning all fields including sub-issues.

**Independent Test**: Run `linear issue get ENG-123` for a known issue; confirm all fields present; run with `--output json` and confirm full schema from contracts/cli-schema.md.

- [X] T014 [US2] Implement issue get GraphQL queries in src/infrastructure/graphql/queries/issue_queries.rs: define `SubIssueNode` QueryFragment (id, title, identifier); define `IssueDetailNode` QueryFragment (all Issue fields + `children` as SubIssueConnection); define `GetIssueByIdVariables` and `GetIssueByIdQuery` (uses `issue(id: $id)`); define `IdentifierFilter`, `GetIssueByIdentifierVariables`, `GetIssueByIdentifierQuery` (uses `issues(filter: { identifier: { eq: $id } }, first: 1)`); implement `fetch_issue(client, api_key, id: &str, is_display_id: bool)` routing to the correct query
- [X] T015 [US2] Create GetIssue use case in src/application/use_cases/get_issue.rs: `pub struct GetIssue { repo: Box<dyn IssueRepository> }`; `execute(&self, id: String) -> Result<Issue, ApplicationError>`; auto-detect display ID with regex `^[A-Z]+-\d+$`; pass resolved `IssueId` to `repo.get()`; include `#[instrument]` tracing
- [X] T016 [US2] Implement LinearIssueRepository::get in src/infrastructure/repositories/issue_repository.rs: detect display ID vs UUID format; call `fetch_issue` with correct flag; convert IssueDetailNode → Issue including sub_issues: Vec<SubIssueRef> (T014 must be complete first)
- [X] T017 [US2] Implement IssueSubcommand::Get in src/cli/commands/issue.rs: call GetIssue use case; human output: labelled fields block (identifier, title, state, priority, assignee, labels, due_date, estimate, parent, sub_issues list); JSON output: serialize full Issue per contracts/cli-schema.md schema; exit code 1 on NotFound (T015, T016 must be complete first)

**Checkpoint**: `linear issue get <id>` returns all fields and sub-issues list.

---

## Phase 5: User Story 3 - Create an Issue (Priority: P3)

**Goal**: Implement `linear issue create` with all required and optional flags, dry-run, and LINEAR_PROJECT_ID support.

**Independent Test**: Run `linear issue create --title "Test" --team <id> --project <id>` and confirm new issue appears in `linear issue list --project <id>`; run with `--dry-run` and confirm no mutation occurs.

- [X] T018 [US3] Implement IssueCreateMutation in src/infrastructure/graphql/mutations/issue_mutations.rs: define `IssueCreateInput` cynic InputObject (title, teamId, projectId, description, priority, assigneeId, labelIds, dueDate, estimate, parentId — all optional except title/teamId/projectId); define `IssuePayload` QueryFragment (success, issue: Option<IssueDetailNode>); define `IssueCreateVariables` and `IssueCreateMutation`; implement `create_issue(client, api_key, input) -> Result<IssueDetailNode, DomainError>`; import `IssueDetailNode` from issue_queries
- [X] T019 [US3] Create CreateIssue use case in src/application/use_cases/create_issue.rs: `execute(&self, input: CreateIssueInput, dry_run: bool) -> Result<Issue, ApplicationError>`; validate: title non-empty → ApplicationError::Validation; team_id non-empty → Validation; project_id resolved (provided or from LINEAR_PROJECT_ID env var at CLI boundary — input already has it resolved); due_date matches YYYY-MM-DD if Some; estimate >= 0.0 if Some; if dry_run → return early with Err describing what would happen OR handle at CLI layer; call `repo.create(input)`; include `#[instrument]` tracing
- [X] T020 [US3] Implement LinearIssueRepository::create in src/infrastructure/repositories/issue_repository.rs: build IssueCreateInput for cynic from domain CreateIssueInput (map LabelId → String vec, Priority → i32, etc.); call `create_issue` from issue_mutations; convert returned IssueDetailNode → Issue (T018 must be complete first)
- [X] T021 [US3] Add IssueSubcommand::Create to src/cli/commands/issue.rs: args `--title` (required), `--team` (required), `--project` (optional, falls back to `std::env::var("LINEAR_PROJECT_ID")`; exit 1 if both absent before any API call), `--description`, `--priority`, `--assignee`, `--label` (repeatable), `--due-date`, `--estimate`, `--parent`, `--dry-run`, `--output`; build CreateIssueInput; handle dry-run with human output `[dry-run] Would create issue: …`; human success output: `Created issue ENG-125: <title>`; JSON success: full Issue object; exit 1 on validation error (T019, T020 must be complete first)

**Checkpoint**: `linear issue create` creates issues and sub-issues with full validation and dry-run.

---

## Phase 6: User Story 4 - Update an Issue (Priority: P4)

**Goal**: Implement `linear issue update <id>` with state validation, parent reparenting, and dry-run.

**Independent Test**: Run `linear issue update ENG-123 --state "In Progress"` then `linear issue get ENG-123 --output json` and confirm state change; run with `--dry-run` and confirm no change.

- [X] T022 [US4] Implement WorkflowStatesQuery and IssueUpdateMutation in src/infrastructure/graphql: (a) in issue_queries.rs: define `WorkflowStateNode` (id, name, type field as `state_type`), `WorkflowStateConnection`, `WorkflowStateFilter`, `TeamIdFilter`, `WorkflowStatesVariables`, `WorkflowStatesQuery`; implement `fetch_workflow_states(client, api_key, team_id: &str) -> Result<Vec<WorkflowStateNode>, DomainError>`; (b) in issue_mutations.rs: define `IssueUpdateInput` cynic InputObject (all Optional fields + parentId as nullable String); define `IssueUpdateVariables` and `IssueUpdateMutation`; implement `update_issue(client, api_key, id: &str, input) -> Result<IssueDetailNode, DomainError>`
- [X] T023 [US4] Create UpdateIssue use case in src/application/use_cases/update_issue.rs: `execute(&self, id: String, input: UpdateIssueInput, dry_run: bool) -> Result<Issue, ApplicationError>`; validate: at least one field set (all None and no_parent false) → exit 1; parent_id.is_some() && no_parent → exit 1 with "mutually exclusive"; if `input.state_id` is a state name (any non-empty): resolve via `repo.list_workflow_states(team.id)`, case-insensitive match, error with valid names if not found; auto-detect display ID for `id` arg; resolve issue's team_id for state validation; if dry_run → return early (handled at CLI layer); call `repo.update(issue_id, input)`; include `#[instrument]` tracing
- [X] T024 [US4] Implement LinearIssueRepository::update and list_workflow_states in src/infrastructure/repositories/issue_repository.rs: `list_workflow_states(team_id)` calls `fetch_workflow_states` and maps to `Vec<WorkflowStateInfo>`; `update(id, input)` calls `update_issue` mutation mapping domain UpdateIssueInput to cynic IssueUpdateInput (no_parent=true → parentId: Some(null-equivalent empty or Option<Option<String>>)); convert returned IssueDetailNode → Issue (T022 must be complete first)
- [X] T025 [US4] Add IssueSubcommand::Update to src/cli/commands/issue.rs: args `<id>`, `--title`, `--description`, `--state`, `--priority`, `--assignee`, `--due-date`, `--estimate`, `--parent`, `--no-parent`, `--dry-run`, `--output`; validate `--parent` and `--no-parent` mutually exclusive at CLI before use case call (exit 1); build UpdateIssueInput with state name in state_id field (use case resolves to UUID); dry-run human output `[dry-run] Would update ENG-123: …`; success human output `Updated ENG-123: state → "In Progress"`; JSON: full Issue object (T023, T024 must be complete first)

**Checkpoint**: `linear issue update` applies validated changes including state transitions.

---

## Phase 7: User Story 5 - Create a Sub-Issue (Priority: P5)

**Goal**: Verify the `--parent` flag on `issue create` correctly links child to parent, and parent's `issue get` output lists the new sub-issue.

**Independent Test**: Run `linear issue create --title "Sub-task" --team <id> --project <id> --parent ENG-123` then `linear issue get ENG-123 --output json` and confirm sub_issues contains the new issue.

- [X] T026 [US5] Ensure display ID resolution for `--parent` in src/application/use_cases/create_issue.rs: before submitting mutation, if `parent_id` is a display ID (matches `^[A-Z]+-\d+$`), resolve it to a UUID via `repo.get(IssueId)` and replace in CreateIssueInput; on parent not found → ApplicationError with clear message; when `dry_run`, skip resolution (display ID shown as-is in dry-run output)
- [X] T027 [US5] Verify sub_issues field is fully populated in LinearIssueRepository::get (src/infrastructure/repositories/issue_repository.rs): confirm IssueDetailNode includes children connection with SubIssueNode fields; confirm Issue.sub_issues is mapped from children.nodes; verify CLI `issue get` human output includes sub-issues section listing identifier and title for each child (src/cli/commands/issue.rs)

**Checkpoint**: Sub-issue creation and parent-child relationship fully verified end-to-end.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, compile verification, and edge case coverage.

- [X] T028 [P] Add issue command integration tests in tests/integration/exit_codes_test.rs: `issue get <nonexistent> --json` exits 1; `issue create` with missing --title exits 1; `issue update <id>` with no update flags exits 1; `issue update <id> --parent x --no-parent` exits 1; `issue list`, `issue get`, `issue create`, `issue update` all exit 3 when unauthenticated (may exit 0 or 2 in CI with credentials present, follow existing pattern)
- [X] T029 Run `cargo build` from repo root and fix all compilation errors introduced by the WorkflowState → WorkflowStateRef migration and new trait signatures
- [X] T030 Run `cargo test` from repo root and fix any failing tests; run `cargo clippy -- -D warnings` and resolve all lints

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately; T001–T003 can run in parallel
- **Foundational (Phase 2)**: Depends on Phase 1; T004 and T005 run in parallel; T006 depends on T004+T005; T007 depends on T006; T008 depends on T006; T009 depends on T007+T008
- **User Story Phases (3–7)**: All depend on Phase 2 completion; phases can proceed in priority order (P1→P2→P3→P4→P5) since each story builds on the previous infrastructure
- **Polish (Phase 8)**: Depends on all user story phases complete

### Within Each User Story

- GraphQL query/mutation before repository implementation
- Repository implementation before use case
- Use case before CLI handler
- Complete one story before starting next (single implementor)

### Parallel Opportunities

- T001, T002, T003 in Phase 1
- T004, T005 in Phase 2
- T028 can run while T029/T030 are in progress (different files)

---

## Parallel Example: User Story 1

```bash
# Phase 2 parallel start:
Task T004: Create workflow_state_ref.rs
Task T005: Create label_id.rs

# After T004+T005 done:
Task T006: Update value_objects/mod.rs
# Then T007, T008, T009 sequentially

# US1 sequential:
Task T010 → Task T011 → Task T012 → Task T013
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T009) — CRITICAL
3. Complete Phase 3: User Story 1 (T010–T013)
4. **STOP and VALIDATE**: `linear issue list` works with filters
5. Proceed to US2 if validated

### Incremental Delivery

1. Setup + Foundational → T001–T009
2. US1 list → validate → T010–T013
3. US2 get → validate → T014–T017
4. US3 create → validate → T018–T021
5. US4 update → validate → T022–T025
6. US5 sub-issue → validate → T026–T027
7. Polish → T028–T030

---

## Notes

- [P] tasks = different files, no dependencies between them
- All GraphQL types must be validated against schema.graphql at compile time via cynic
- Import `execute_with_retry`, `map_errors`, `GraphqlResponse` from `project_queries` (do not duplicate)
- WorkflowState enum in src/domain/value_objects/workflow_state.rs becomes unused after T007; remove it or `#[allow(dead_code)]` pending T029 compile pass
- `--json` flag already exists as `force_json` in CLI infrastructure — reuse it for `--output json`
- Exit codes: 0 success, 1 input/validation/not-found, 2 API/network, 3 auth — enforce via `ApplicationError` → `anyhow::Error` mapping in main.rs
- Avoid `unwrap()`/`expect()` in production paths; use `?` with `DomainError::InvalidInput` or `ApplicationError`
