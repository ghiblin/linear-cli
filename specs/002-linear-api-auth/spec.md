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
4. **Given** the user provides the API key via an environment variable instead of interactive input, **When** they run the auth login command, **Then** the CLI accepts the key from the environment variable and proceeds with validation and storage.

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

- What happens when the secure credential store (system keychain) is unavailable or access is denied?
- What happens when the Linear API is unreachable during key validation at login?
- What happens when the stored credential is corrupted or truncated?
- How does the system behave when the API key has read-only scope vs. full scope?
- What exit code is returned when the user cancels an interactive login prompt (Ctrl-C)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST provide an `auth login` command that accepts a Linear personal API key and stores it for subsequent commands.
- **FR-002**: The `auth login` command MUST validate the provided key against the Linear API before storing it, confirming the key grants access to at least one workspace.
- **FR-003**: Credentials MUST be stored in the system's secure credential store by default; plain-text storage is only permitted if the user explicitly opts in.
- **FR-004**: The CLI MUST provide an `auth status` command that reports whether valid credentials are present and identifies the connected workspace.
- **FR-005**: The CLI MUST provide an `auth logout` command that removes stored credentials from all storage locations used by the CLI.
- **FR-006**: The `auth logout` command MUST support a `--dry-run` flag that reports what would be removed without making any changes.
- **FR-007**: All commands that require authentication MUST exit with code 3 and emit a descriptive error on stderr when credentials are absent or invalid, guiding the user to run `auth login`.
- **FR-008**: Auth tokens MUST be redacted in all log output regardless of log verbosity level.
- **FR-009**: When stdout is a TTY, auth commands MUST produce human-readable output; when stdout is a pipe or `--output json` is set, output MUST be machine-parseable JSON.
- **FR-010**: The `auth login` command MUST accept the API key via an environment variable as an alternative to interactive input, to support non-interactive and agent-driven workflows.

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
- The system keychain is available on all supported platforms (macOS Keychain, Linux Secret Service / kwallet, Windows Credential Manager); graceful fallback messaging is required when it is unavailable.
- Re-authentication (login while already logged in) overwrites the existing credential after user confirmation; there is no support for multiple simultaneous credentials.
- Network connectivity is assumed to be available during `auth login` and `auth status`; offline validation is not required.
