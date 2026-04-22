# Research: Linear API Authentication

**Phase**: 0 — Research  
**Branch**: `002-linear-api-auth`  
**Date**: 2026-04-21

---

## Decision 1: Keychain Storage via `keyring` Crate

**Decision**: Use `keyring` v3 (already declared in `Cargo.toml`) with a fixed service name `"linear-cli"` and username `"default"`.

**Rationale**: Single account per machine (per constitution / spec Assumptions). A fixed username keeps the API minimal; no user enumeration needed. `keyring` abstracts macOS Keychain, Secret Service (Linux), and Windows Credential Manager behind one trait.

**API contract**:
```rust
let entry = keyring::Entry::new("linear-cli", "default")?;
entry.set_password(api_key_str)?;       // store
let key = entry.get_password()?;        // retrieve → Err(NoEntry) if absent
entry.delete_credential()?;             // remove
```

Error variants of interest:
- `keyring::Error::NoEntry` — no stored credential (→ return `None`, not an error).
- `keyring::Error::PlatformFailure(_)` — keychain unavailable (→ map to `AuthError::KeychainUnavailable`).

**Alternatives considered**:
- `secret-service` crate: Linux-only, no macOS support.
- Direct OS APIs (`Security.framework`, `libsecret`): platform-specific, defeats cross-platform goal.

---

## Decision 2: Linear API Key Validation Query

**Decision**: Use the `viewer { id name organization { name urlKey } }` GraphQL query.

**Rationale**: `viewer` is the canonical "who am I" query on the Linear API. A successful response confirms the key is valid and grants access; an error response (HTTP 401 or GraphQL `UNAUTHENTICATED` error) indicates an invalid or expired key. The `organization` block provides the workspace name displayed in `auth login` / `auth status` output.

**Query**:
```graphql
query ValidateApiKey {
  viewer {
    id
    name
    organization {
      name
      urlKey
    }
  }
}
```

**Error mapping**:
- HTTP 401 or GraphQL error code `UNAUTHENTICATED` → `AuthError::InvalidKey`.
- Network error / timeout → `AuthError::NetworkError(msg)` → exit code 2.
- HTTP 200 with valid data → extract workspace name; construct `Workspace` entity.

**Alternatives considered**:
- `{ teams { nodes { id } } }`: works but returns no user context; rejected for insufficient workspace info.
- Parsing the key format (prefix check): useful as a secondary guard but not a substitute for remote validation.

---

## Decision 3: Credential Resolution Architecture

**Decision**: Implement a `ResolveAuth` application-layer function (not a use case struct) that checks sources in order and returns an `AuthSession`.

**Resolution order** (per FR-011):
1. `LINEAR_API_KEY` environment variable — if set, validate remotely and return; never store.
2. Keychain store — if credential exists, validate remotely and return.
3. File store — if `--store-file` was used at login and file exists, validate remotely and return.
4. If all sources empty or invalid → return `Err(AuthError::NotAuthenticated)` → exit code 3.

**Rationale**: Env-var first honours the "ephemeral override" contract (FR-010). File store is checked last as it is opt-in and lower-trust. Remote validation on every `status` call satisfies SC-002 and the "revoked key" scenario (Story 2, AC-3).

**Note on performance**: `auth status` must return in < 2 s (SC-002). The `viewer` query is a small, indexed query on Linear's side. Acceptable on standard connections.

**Alternatives considered**:
- Cache-based validation (store timestamp + skip remote check): rejected because it cannot detect revoked keys (Story 2, AC-3).
- Offline-capable validation (key format check only): rejected by spec Assumptions ("offline validation is not required").

---

## Decision 4: `ApiKey` Redaction Strategy

**Decision**: Implement a newtype `ApiKey(String)` with a custom `Debug` and `Display` that emits `[REDACTED]`. Derive `Clone` but not `Serialize` / `Deserialize` (the key is never in JSON output).

```rust
pub struct ApiKey(String);

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}
```

`ApiKey::as_str(&self) -> &str` is the only way to access the raw value, restricted to infrastructure layer callers that need to pass it to the HTTP client or keychain.

**Rationale**: Constitution Principle VI requires redaction at all log levels. Custom `Debug` is the only reliable way to prevent the raw key from appearing in `tracing` spans (which use `{:?}` formatting).

**Alternatives considered**:
- Regex-based log scrubbing in `tracing-subscriber`: fragile; misses new log sites.
- Zeroing memory on drop (`zeroize` crate): good hygiene but orthogonal to redaction; deferred to v2.

---

## Decision 5: Exit Code Propagation

**Decision**: Extend `main.rs` to map domain-level errors to exit codes before calling `process::exit`. Define `CliExit` as a private enum in `main.rs` with `Success`, `UserError(1)`, `ApiError(2)`, `AuthError(3)`. The `run()` function returns `Result<(), CliExit>`.

**Exit code table** (from constitution Principle IV and spec FR-002, FR-003, FR-007):

| Code | Meaning | Examples |
|------|---------|---------|
| 0 | Success | Normal completion |
| 1 | User / input error | Bad flags, unknown command |
| 2 | API / network error | Linear unreachable during login |
| 3 | Auth error | No credentials, invalid key, keychain unavailable |

**Rationale**: `anyhow::Error` in `run()` does not carry exit codes. A dedicated `CliExit` type makes the mapping explicit and testable. The existing pattern of `process::exit(1)` in `main.rs` is extended, not replaced.

**Alternatives considered**:
- `anyhow` error downcasting in `main.rs`: works but brittle; types must be checked at runtime.
- Single exit code 1 for all errors: breaks the machine-readable contract (Constitution IV).

---

## Decision 6: File Store Location and Permissions

**Decision**: Default path for `--store-file` (no explicit PATH): `~/.config/linear-cli/credentials`. Permissions: `0o600`. Warn on write.

**Rationale**: `~/.config` is the XDG base directory standard on Linux and is conventional on macOS. Restrictive permissions reduce exposure if the file is accidentally visible. The warning satisfies FR-003a.

**Alternatives considered**:
- `~/.linear-cli` (dot-file in home): less conventional; clutters home directory.
- `XDG_CONFIG_HOME` env var: more correct on Linux; deferred to v2 to keep scope tight.

---

## Resolved Clarifications

All spec ambiguities have been resolved in the spec's Clarifications section (Session 2026-04-21):

| Question | Resolution |
|----------|-----------|
| Keychain unavailable | Fail with exit 3; user must re-run with `--store-file` |
| `LINEAR_API_KEY` persistence | Never stored; per-invocation only |
| `LINEAR_API_KEY` without `auth login` | Sufficient; no prior login required |
| API unreachable during login | Refuse to store key; exit code 2 |

No NEEDS CLARIFICATION items remain.
