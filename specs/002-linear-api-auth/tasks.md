# Tasks: Linear API Authentication

**Input**: Design documents from `specs/002-linear-api-auth/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-auth.md

**Tests**: Included per constitution Principle II (TDD is NON-NEGOTIABLE). Write each test task first, confirm RED, then implement.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no unresolved dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

---

## Phase 1: Setup

**Purpose**: Scaffold new module files so subsequent tasks have concrete target files.

- [X] T001 Create `src/infrastructure/auth/` directory with empty `mod.rs`, `keyring_store.rs`, and `file_store.rs`; add `pub mod auth;` to `src/infrastructure/mod.rs`
- [X] T002 [P] Create stub `tests/integration/auth_integration.rs` and `tests/e2e/auth_e2e.rs` with `// TODO` placeholders; add `[[test]]` entries to `Cargo.toml` for both files if not already present

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain types, error variants, GraphQL client, and exit-code mapping that all three user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T003 Write failing unit tests for `ApiKey` (non-empty invariant, redacted `Debug`/`Display`, `as_str()` raw access) inside `#[cfg(test)]` in `src/domain/value_objects/api_key.rs`; run `cargo test` and confirm RED
- [X] T004 Implement `ApiKey` newtype in `src/domain/value_objects/api_key.rs`; register `pub mod api_key;` in `src/domain/value_objects/mod.rs`; confirm T003 GREEN
- [X] T005 [P] Write failing unit tests for `AuthError` variants (`NotAuthenticated`, `InvalidKey`, `ValidationFailed`, `NetworkError`, `KeychainUnavailable`, `FileError`) inside `#[cfg(test)]` in `src/domain/errors.rs`; confirm RED
- [X] T006 [P] Add `AuthError` enum to `src/domain/errors.rs` (using `thiserror`); confirm T005 GREEN
- [X] T007 Write failing unit tests for `LinearApiClient` trait mock — add `#[cfg_attr(test, mockall::automock)]` stub and confirm the mock compiles — in `src/domain/repositories/linear_api_client.rs`; confirm RED
- [X] T008 Define `LinearApiClient` trait (`validate_api_key(&self, key: &ApiKey) -> Result<Workspace, AuthError>`) with `#[cfg_attr(test, mockall::automock)]` in `src/domain/repositories/linear_api_client.rs`; register in `src/domain/repositories/mod.rs`; implement `LinearGraphqlClient` using reqwest + serde with `viewer { id name organization { name urlKey } }` query and `Authorization: Bearer` header in `src/infrastructure/graphql/client.rs`; update `src/infrastructure/graphql/mod.rs`; confirm T007 GREEN
- [X] T009 [P] Add `ApplicationError::Auth(#[from] AuthError)` variant to `src/application/errors.rs`; confirm existing tests still GREEN
- [X] T010 [P] Extend `run()` in `src/main.rs` to map `AuthError::NetworkError` / `ValidationFailed` → `process::exit(2)` and `AuthError::NotAuthenticated` / `InvalidKey` / `KeychainUnavailable` → `process::exit(3)`

**Checkpoint**: `cargo test` passes — domain types, error variants, GraphQL client, and exit code mapping are ready.

---

## Phase 3: User Story 1 — Set Up Authentication (Priority: P1) 🎯 MVP

**Goal**: User runs `linear auth login`, provides a Linear API key, the CLI validates it, stores it in the system keychain, and confirms the connected workspace name.

**Independent Test**: Run `linear auth login` → enter a valid key → CLI outputs authenticated workspace name and exits 0.

### Tests for User Story 1

> **Write tests FIRST — confirm they FAIL before implementing**

- [X] T011 [P] [US1] Write failing unit tests for `Workspace` entity (non-empty invariants for `id`/`name`, field accessors) inside `#[cfg(test)]` in `src/domain/entities/workspace.rs`; confirm RED
- [X] T012 [P] [US1] Write failing unit tests for `AuthSession` and `CredentialSource` enum (construction, field accessors) inside `#[cfg(test)]` in `src/domain/entities/auth_session.rs`; confirm RED

### Implementation for User Story 1

- [X] T013 [P] [US1] Implement `Workspace` entity in `src/domain/entities/workspace.rs`; register in `src/domain/entities/mod.rs`; confirm T011 GREEN
- [X] T014 [P] [US1] Implement `AuthSession` entity and `CredentialSource` enum in `src/domain/entities/auth_session.rs`; register in `src/domain/entities/mod.rs`; confirm T012 GREEN
- [X] T015 [US1] Define `CredentialStore` trait and `StorageKind` enum in `src/domain/repositories/credential_store.rs`; register in `src/domain/repositories/mod.rs`
- [X] T016 [US1] Write failing integration tests for `KeyringCredentialStore` round-trip (store → retrieve → remove; `NoEntry` maps to `Ok(None)`; `PlatformFailure` maps to `AuthError::KeychainUnavailable`) in `tests/integration/auth_integration.rs`; confirm RED
- [X] T017 [US1] Implement `KeyringCredentialStore` in `src/infrastructure/auth/keyring_store.rs` (service `"linear-cli"`, username `"default"`); register in `src/infrastructure/auth/mod.rs`; confirm T016 GREEN
- [X] T018 [P] [US1] Write failing integration tests for `FileCredentialStore` round-trip (store with warning → retrieve → remove; file permissions `0o600`; default path `~/.config/linear-cli/credentials`) in `tests/integration/auth_integration.rs`; confirm RED
- [X] T019 [P] [US1] Implement `FileCredentialStore` in `src/infrastructure/auth/file_store.rs`; confirm T018 GREEN
- [X] T020 [US1] Write failing unit tests for `LoginUseCase` (valid key → validate → store → `Ok(Workspace)`; invalid key → `Err(AuthError::InvalidKey)`; network error → `Err(AuthError::NetworkError)`; credential present + `overwrite=false` → early return) inside `#[cfg(test)]` in `src/application/use_cases/login.rs` using `mockall` mocks for `MockCredentialStore` and `MockLinearApiClient`; confirm RED
- [X] T021 [US1] Implement `LoginUseCase` in `src/application/use_cases/login.rs`; register in `src/application/use_cases/mod.rs`; confirm T020 GREEN
- [X] T022 [US1] Implement `linear auth login [--store-file [PATH]]` subcommand in `src/cli/commands/auth.rs` — prompt for key on TTY or read from stdin pipe, build correct store based on flag, call `LoginUseCase`, handle overwrite confirmation prompt, output human text or JSON per `contracts/cli-auth.md`; handle Ctrl-C / SIGINT during the prompt by exiting cleanly with code 0 and no error output
- [X] T023 [US1] Register `Commands::Auth(AuthCommand)` in `src/cli/commands/mod.rs`; add `Commands::Auth` match arm in `src/main.rs`

**Checkpoint**: `cargo test` all green. Run `linear auth login` manually per `quickstart.md` to confirm end-to-end.

---

## Phase 4: User Story 2 — Check Authentication Status (Priority: P2)

**Goal**: User runs `linear auth status` to see whether the CLI is authenticated and which workspace is connected, including `--output json` and revoked-key detection.

**Independent Test**: After Phase 3, run `linear auth status` → outputs connected workspace name and source.

### Tests for User Story 2

> **Write tests FIRST — confirm they FAIL before implementing**

- [X] T024 [US2] Write failing unit tests for `resolve_auth` function (env var checked first; keychain second; `NotAuthenticated` when all empty; revoked key → `Err(InvalidKey)`; corrupted/invalid stored credential → store `remove()` called then `Err(NotAuthenticated)`) inside `#[cfg(test)]` in `src/application/use_cases/resolve_auth.rs` using `mockall` mocks; confirm RED
- [X] T025 [P] [US2] Write failing unit tests for `AuthStatusUseCase` (authenticated → `Ok(AuthSession)`; revoked key → `Err(AuthError::InvalidKey)`; no credential → `Err(AuthError::NotAuthenticated)`) inside `#[cfg(test)]` in `src/application/use_cases/auth_status.rs`; confirm RED

### Implementation for User Story 2

- [X] T026 [US2] Implement `resolve_auth` function in `src/application/use_cases/resolve_auth.rs`; register in `src/application/use_cases/mod.rs`; confirm T024 GREEN
- [X] T027 [US2] Implement `AuthStatusUseCase` in `src/application/use_cases/auth_status.rs`; confirm T025 GREEN
- [X] T028 [US2] Implement `linear auth status` subcommand in `src/cli/commands/auth.rs` (append to existing `AuthCommand`) — call `AuthStatusUseCase`, output human text or JSON per `contracts/cli-auth.md`; handle exit codes 2 and 3
- [X] T029 [P] [US2] Write failing integration test for `auth status` (authenticated → workspace output; no credential → exit 3; `LINEAR_API_KEY` env var takes precedence) in `tests/integration/auth_integration.rs`; confirm RED; then make GREEN

### FR-007 / SC-003 — Auth Guard on Existing Commands

**Purpose**: Wire `resolve_auth` into all existing auth-requiring commands so FR-007 ("all commands that require auth MUST exit 3 with a guiding error") and SC-003 ("100% coverage") are achievable.

- [X] T030 [US2] Write failing unit tests for auth-guard behaviour in `src/cli/commands/issue.rs` — mock `resolve_auth` to return `Err(AuthError::NotAuthenticated)` and assert `run_issue` propagates exit 3; confirm RED
- [X] T031 [P] [US2] Write failing unit tests for auth-guard behaviour in `src/cli/commands/team.rs` — same pattern; confirm RED
- [X] T032 [US2] Wire `resolve_auth` into `run_issue` in `src/cli/commands/issue.rs` — call at function entry, propagate `AuthError` → exit 3 via `main.rs` mapping; confirm T030 GREEN
- [X] T033 [P] [US2] Wire `resolve_auth` into `run_team` in `src/cli/commands/team.rs`; confirm T031 GREEN
- [X] T034 [P] [US2] Write failing integration tests in `tests/integration/auth_integration.rs` confirming `linear issue list` and `linear team list` each exit 3 with a guiding error when no credential is present (FR-007, SC-003); confirm RED; then make GREEN
- [X] T035 [P] [US2] Write integration test in `tests/integration/auth_integration.rs` confirming `LINEAR_API_KEY` env var allows `issue list` and `team list` without prior `auth login` (US1-AC4, FR-010, FR-011); confirm RED; then make GREEN

**Checkpoint**: `linear auth status` works independently. All existing commands guard auth correctly. `cargo test` all green.

---

## Phase 5: User Story 3 — Remove Stored Credentials (Priority: P3)

**Goal**: User runs `linear auth logout` to remove stored credentials from all locations; `--dry-run` reports what would be removed without deleting.

**Independent Test**: After Phase 3, run `linear auth logout` → then `linear auth status` returns exit code 3.

### Tests for User Story 3

> **Write tests FIRST — confirm they FAIL before implementing**

- [X] T036 [US3] Write failing unit tests for `LogoutUseCase` (credential present → removes from all stores → `Ok(removed_list)`; no credential → `Ok(empty)`; `dry_run=true` → returns what-would-be-removed without calling `remove()`) inside `#[cfg(test)]` in `src/application/use_cases/logout.rs` using `mockall` mocks; confirm RED

### Implementation for User Story 3

- [X] T037 [US3] Implement `LogoutUseCase` in `src/application/use_cases/logout.rs`; register in `src/application/use_cases/mod.rs`; confirm T036 GREEN
- [X] T038 [US3] Implement `linear auth logout [--dry-run]` subcommand in `src/cli/commands/auth.rs` (append to existing `AuthCommand`) — collect configured stores, call `LogoutUseCase`, output human text or JSON per `contracts/cli-auth.md`
- [X] T039 [P] [US3] Write failing integration test for `auth logout` (credential present → removed; dry-run → no deletion; no credential → exit 0 with informational message) in `tests/integration/auth_integration.rs`; confirm RED; then make GREEN

**Checkpoint**: All three auth commands (`login`, `status`, `logout`) fully functional. All existing commands guard auth correctly. `cargo test --all` green.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Observability, contract validation, and CI readiness.

- [X] T040 [P] Add `#[tracing::instrument(skip(key))]` / `tracing::debug!` to all infrastructure auth operations in `src/infrastructure/auth/keyring_store.rs`, `src/infrastructure/auth/file_store.rs`, and `src/infrastructure/graphql/client.rs`; run with `-vvv` and confirm tokens show as `[REDACTED]` in log output
- [X] T041 [P] Write e2e test scaffold in `tests/e2e/auth_e2e.rs` — full login/status/logout cycle guarded by `std::env::var("LINEAR_TEST_API_KEY")` check at test start; skip gracefully if var absent
- [X] T042 Add `insta` snapshot tests for JSON output of all three auth commands in `tests/integration/auth_integration.rs`; validate schemas match `contracts/cli-auth.md`
- [X] T043 Run `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test --all`; resolve all issues
- [X] T044 Walk through success criteria SC-001 through SC-006 from `specs/002-linear-api-auth/spec.md` and add assertions or manual verification notes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Foundational — no inter-story dependencies
- **US2 (Phase 4)**: Depends on Foundational + US1 (`CredentialStore`, `AuthSession`, stores from US1); T030–T035 also depend on T026 (`resolve_auth`)
- **US3 (Phase 5)**: Depends on Foundational + US1; independent of US2
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: Foundational only
- **US2 (P2)**: Uses `CredentialStore`, `AuthSession`, stores, and `resolve_auth` from US1/Phase 4
- **US3 (P3)**: Uses `CredentialStore` and stores from US1; independent of US2

### Within Each Phase

- Tests MUST be written first and confirmed FAILING before implementation (Constitution Principle II)
- Domain types before infrastructure implementations
- Infrastructure implementations before use cases
- Use cases before CLI commands
- Tasks marked [P] within a phase can be dispatched concurrently

### Parallel Opportunities

- T003 / T005: Write ApiKey and AuthError tests concurrently
- T004 / T006: Implement ApiKey and AuthError concurrently (after T003/T005 RED)
- T009 / T010: ApplicationError extension and main.rs exit codes concurrently (after T006)
- T011 / T012: Write Workspace and AuthSession tests concurrently
- T013 / T014: Implement Workspace and AuthSession concurrently (after T011/T012 RED)
- T016 / T018: Write Keyring and File store tests concurrently (after T015)
- T017 / T019: Implement Keyring and File stores concurrently (after T016/T018 RED)
- T030 / T031: Write auth-guard unit tests for issue and team concurrently
- T032 / T033: Wire auth guard into issue and team concurrently (after T030/T031 RED)
- T034 / T035: Integration tests for auth guard and env-var path concurrently (after T032/T033)

---

## Parallel Example: Phase 2 Foundational

```
# Round 1 — parallel:
T003  Write ApiKey unit tests (src/domain/value_objects/api_key.rs)
T005  Write AuthError unit tests (src/domain/errors.rs)

# Round 2 — parallel (after RED confirmed):
T004  Implement ApiKey
T006  Implement AuthError

# Round 3:
T007  Write LinearApiClient trait tests (src/domain/repositories/linear_api_client.rs)
  →   T008  Define trait in domain; implement LinearGraphqlClient in infra
T009  Extend ApplicationError  [P with T010]
T010  Extend main.rs exit codes
```

## Parallel Example: Phase 4 US2 (auth guard wiring)

```
# Round 1 — parallel:
T030  Write auth-guard tests for issue.rs
T031  Write auth-guard tests for team.rs

# Round 2 — parallel (after RED confirmed):
T032  Wire resolve_auth into run_issue
T033  Wire resolve_auth into run_team

# Round 3 — parallel:
T034  Integration tests: issue/team exit 3 without credential
T035  Integration test: LINEAR_API_KEY allows issue/team
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (auth login)
4. **STOP and VALIDATE**: `cargo test --all` green; `linear auth login` smoke test passes
5. Proceed to US2 (which includes the FR-007 auth-guard wiring)

### Incremental Delivery

1. Setup + Foundational → types and client ready
2. US1 → `linear auth login` working → **MVP shippable**
3. US2 → `linear auth status` + auth guard on all commands → FR-007/SC-003 met
4. US3 → `linear auth logout` working → full auth lifecycle
5. Polish → observability, snapshots, CI clean

---

## Notes

- TDD is NON-NEGOTIABLE per constitution Principle II — never skip Red phase
- `ApiKey` custom `Debug`/`Display` is the sole redaction mechanism — validate it at T003/T004
- `LinearApiClient` trait lives in `src/domain/repositories/` — no infrastructure imports allowed in domain
- `CredentialStore` trait lives in `src/domain/repositories/` — same constraint
- Keychain tests may need a `SKIP_KEYCHAIN_TESTS=1` guard for headless CI environments
- `LINEAR_TEST_API_KEY` required for e2e tests — always opt-in, never mandatory
- [P] tasks = different files, no unresolved dependencies on in-progress work
