# Feature Specification: Linear API Authentication

**Feature Branch**: `002-linear-api-auth`  
**Created**: 2026-04-21  
**Status**: Draft  
**Input**: User description: "implement authentication against the Linear API"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Set Up Authentication (Priority: P1)

A new user runs the CLI for the first time and wants to connect it to their Linear account. They run an auth login command, provide their personal API key when prompted, and the CLI verifies the key is valid and stores it securely. On success, the CLI confirms authentication and displays the connected workspace name.

**Why this priority**: Without authentication, no other CLI command is usable. This is the foundational capability the entire tool depends on.

**Independent Test**: Can be fully tested by running `linear auth login`, entering a valid API key, and confirming the CLI reports the connected workspace — delivers a fully authenticated CLI session.

**Acceptance Scenarios**:

1. **Given** the user has not previously authenticated, **When** they run the auth login command and enter a valid Linear API key, **Then** the CLI validates the key against Linear, stores it securely, and confirms the connected workspace name.
2. **Given** the user has not previously authenticated, **When** they run the auth login command and enter an invalid or expired API key, **Then** the CLI reports that the key could not be verified and exits with auth error code (3), without storing the key.
3. **Given** the user is already authenticated, **When** they run the auth login command, **Then** the CLI warns that an existing session is active and prompts for confirmation before overwriting.
4. **Given** `LINEAR_API_KEY` is set in the environment and no stored credential exists, **When** the user runs any command that requires authentication, **Then** the CLI uses the env-var key for that invocation, validates it, and proceeds without requiring `auth login`.

---

### User Story 2 - Check Authentication Status (Priority: P2)

A user wants to know whether the CLI is currently authenticated and which Linear workspace it is connected to, without performing any destructive operation.

**Why this priority**: Agents and users routinely need to verify auth state before running commands. Without this, diagnosing "why is command X failing" is harder.

**Independent Test**: Can be fully tested by running `linear auth status` after completing Story 1 — independently confirms that stored credentials are readable and valid.

**Acceptance Scenarios**:

1. **Given** the user is authenticated, **When** they run `linear auth status`, **Then** the CLI outputs the connected workspace name and indicates the session is active.
2. **Given** the user is not authenticated, **When** they run `linear auth status`, **Then** the CLI outputs that no credentials are stored and exits with auth error code (3).
3. **Given** the stored credentials have become invalid (e.g., key revoked in Linear), **When** they run `linear auth status`, **Then** the CLI reports the key is no longer valid and exits with auth error code (3).
4. **Given** `--output json` is passed, **When** they run `linear auth status`, **Then** the CLI outputs a structured JSON object with `authenticated`, `workspace`, and `error` fields.

---

### User Story 3 - Remove Stored Credentials (Priority: P3)

A user wants to sign out of the CLI — either to revoke access on a shared machine or to switch to a different Linear account.

**Why this priority**: Credential hygiene and the ability to switch accounts are secondary but essential for security and multi-account workflows.

**Independent Test**: Can be fully tested by running `linear auth logout` after completing Story 1 and confirming that subsequent `linear auth status` reports unauthenticated — independently verifies credential removal.

**Acceptance Scenarios**:

1. **Given** the user is authenticated, **When** they run the auth logout command, **Then** the CLI removes the stored credentials and confirms sign-out.
2. **Given** the user is not authenticated, **When** they run the auth logout command, **Then** the CLI informs the user that no credentials were found and exits cleanly (code 0).
3. **Given** `--dry-run` is passed, **When** they run the auth logout command, **Then** the CLI reports what would be removed without actually deleting anything.

---

### Edge Cases

- When the system keychain is unavailable or access is denied, `auth login` MUST fail with a clear error message and exit code 3; the user must re-run with `--store-file` to opt into config-file storage.
- When the Linear API is unreachable during `auth login`, the CLI MUST refuse to store the key, emit a "could not reach Linear API" error on stderr, and exit with code 2.
- When the stored credential is corrupted or truncated (a non-empty string that fails remote validation), the CLI MUST silently remove the entry from the store and continue as if unauthenticated, surfacing the normal "no valid credential" error via exit code 3.
- API key scope (read-only vs. full) is NOT validated at `auth login` in v1; the only check is that the key grants API access. Write commands that receive a GraphQL permission error MUST surface it as an API error (exit code 2).
- When the user cancels an interactive `auth login` prompt (Ctrl-C / SIGINT), the CLI MUST exit cleanly with code 0 and emit no error output.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST provide an `auth login` command that accepts a Linear personal API key and stores it for subsequent commands.
- **FR-002**: The `auth login` command MUST validate the provided key against the Linear API before storing it, confirming the key grants access to at least one workspace; if the API is unreachable, the command MUST exit with code 2 and refuse to store the key.
- **FR-003**: Credentials MUST be stored in the system's secure credential store by default; if the keychain is unavailable, `auth login` MUST fail with exit code 3 and instruct the user to re-run with `--store-file` to opt into config-file storage.
- **FR-003a**: The `auth login` command MUST support a `--store-file [PATH]` flag that stores credentials in a plain-text file at the specified path (or a default config-file location if no path is given); use of this flag MUST display a warning that credentials will be stored unencrypted.
- **FR-004**: The CLI MUST provide an `auth status` command that reports whether valid credentials are present and identifies the connected workspace.
- **FR-005**: The CLI MUST provide an `auth logout` command that removes stored credentials from all storage locations used by the CLI.
- **FR-006**: The `auth logout` command MUST support a `--dry-run` flag that reports what would be removed without making any changes.
- **FR-007**: All commands that require authentication MUST exit with code 3 and emit a descriptive error on stderr when both `LINEAR_API_KEY` and the stored credential are absent or invalid; the error MUST guide the user to either set `LINEAR_API_KEY` or run `auth login`.
- **FR-008**: Auth tokens MUST be redacted in all log output regardless of log verbosity level.
- **FR-009**: When stdout is a TTY, auth commands MUST produce human-readable output; when stdout is a pipe or `--output json` is set, output MUST be machine-parseable JSON.
- **FR-010**: Any CLI command MUST accept the `LINEAR_API_KEY` environment variable as the sole credential source; when set, it takes precedence over any stored credential, requires no prior `auth login`, and is never written to disk.
- **FR-011**: The credential resolution order MUST be: (1) `LINEAR_API_KEY` env var, (2) stored credential. A command MUST proceed if either source yields a valid key; exit code 3 is only emitted when both sources are absent or invalid.

### Key Entities

- **Credential**: A Linear personal API key that grants access to a workspace; has an associated workspace identity used for display and validation.
- **CredentialStore**: The system-level secure storage where the credential is persisted; has a defined location and access policy.
- **AuthSession**: The in-memory resolved credential available to all commands during a CLI invocation; derived from CredentialStore at startup.
- **Workspace**: The Linear organisation context the credential is scoped to; represented by a name and identifier used in status output.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user with a valid Linear API key can complete the full `auth login` flow in under 30 seconds on a standard internet connection.
- **SC-002**: `linear auth status` returns a result in under 2 seconds, including a remote key validation check.
- **SC-003**: 100% of CLI commands that require auth emit a clear, actionable error message (pointing to `auth login`) when credentials are absent — no command exits silently or with a generic error.
- **SC-004**: Credentials are never written to plain-text files in the default configuration — verifiable by inspecting the filesystem after `auth login`.
- **SC-005**: Auth tokens never appear in log output at any verbosity level — verifiable by running with maximum verbosity and scanning stdout/stderr.
- **SC-006**: All three auth commands (`login`, `status`, `logout`) produce valid, schema-stable JSON output when `--output json` is passed.

## Assumptions

- The CLI targets a single Linear account per machine in v1; multi-profile support is out of scope.
- The user has already generated a Personal API Key in their Linear account settings; the CLI does not manage key creation within Linear.
- The API key format is a stable string token as defined by Linear's API documentation; format validation (length, prefix) is a reasonable secondary check but is not the primary validation mechanism.
- When the system keychain is unavailable, the CLI fails with a clear error and directs the user to `--store-file`; there is no silent fallback.
- Re-authentication (login while already logged in) overwrites the existing credential after user confirmation; there is no support for multiple simultaneous credentials.
- Network connectivity is assumed to be available during `auth login` and `auth status`; offline validation is not required.

## Clarifications

### Session 2026-04-21

- Q: When `auth login` is run and the system keychain is unavailable, what should the CLI do? → A: Fail with exit code 3 and a clear error message; user must re-run with `--store-file` to opt into config-file storage.
- Q: When the API key is provided via environment variable, is it persisted? → A: No — `LINEAR_API_KEY` is a per-invocation override on any command; it is never written to the credential store and does not interact with `auth login`.
- Q: Does `LINEAR_API_KEY` require a prior `auth login`? → A: No — `LINEAR_API_KEY` alone is sufficient; credential resolution checks env var first, then stored credential.
- Q: When the Linear API is unreachable during `auth login`, should the CLI store the key anyway? → A: No — refuse to store, emit a network error on stderr, and exit with code 2.
- Q: What happens when the stored credential is corrupted or truncated? → A: Silently remove the entry and proceed as unauthenticated (exit code 3 via the normal `NotAuthenticated` path).
- Q: How does the system behave when the API key has read-only scope vs. full scope? → A: Scope is not validated at login in v1; write-command failures surface as exit code 2 (API/GraphQL error).
- Q: What exit code is returned when the user cancels an interactive login prompt (Ctrl-C)? → A: Exit code 0 — clean cancellation, no error output.
