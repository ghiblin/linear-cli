# Data Model: Linear API Authentication

**Phase**: 1 — Design  
**Branch**: `002-linear-api-auth`  
**Date**: 2026-04-21

---

## Domain Layer

### Value Object: `ApiKey`

**Module**: `src/domain/value_objects/api_key.rs`

| Field | Type | Notes |
|-------|------|-------|
| `0` (inner) | `String` | Raw API key string; never exposed via `Debug`/`Display` |

**Invariants**:
- Non-empty string (validated at construction via `ApiKey::new`).
- No maximum-length constraint enforced in domain (format validation is secondary per spec Assumptions).

**Methods**:
- `ApiKey::new(raw: impl Into<String>) -> Result<ApiKey, DomainError>` — validates non-empty.
- `ApiKey::as_str(&self) -> &str` — raw access; restricted to infrastructure callers.

**Trait implementations**:
- `Debug` — emits `ApiKey([REDACTED])`.
- `Display` — emits `[REDACTED]`.
- `Clone` — needed for passing into async tasks.
- `PartialEq`, `Eq` — for tests.
- NOT `Serialize`/`Deserialize` — key is never in JSON output.

---

### Entity: `Workspace`

**Module**: `src/domain/entities/workspace.rs`

| Field | Type | Notes |
|-------|------|-------|
| `id` | `String` | Linear organisation UUID |
| `name` | `String` | Display name of the organisation |
| `url_key` | `String` | URL slug (e.g. `"my-org"`) |

**Invariants**:
- `id` and `name` are non-empty (validated at construction).

**Methods**:
- `Workspace::new(id, name, url_key) -> Result<Workspace, DomainError>`
- `id(&self) -> &str`, `name(&self) -> &str`, `url_key(&self) -> &str`

**Trait implementations**: `Debug`, `Clone`, `PartialEq`, `Serialize` (for JSON output in `auth status`).

---

### Entity: `AuthSession`

**Module**: `src/domain/entities/auth_session.rs`

| Field | Type | Notes |
|-------|------|-------|
| `api_key` | `ApiKey` | Resolved credential for this invocation |
| `workspace` | `Option<Workspace>` | Present after remote validation; absent if validation is deferred |
| `source` | `CredentialSource` | Where the key came from (env var, keychain, file) |

**`CredentialSource` enum**:
```rust
pub enum CredentialSource {
    EnvVar,
    Keychain,
    File(std::path::PathBuf),
}
```

**Methods**:
- `AuthSession::new(api_key, workspace, source) -> AuthSession`
- `api_key(&self) -> &ApiKey`
- `workspace(&self) -> Option<&Workspace>`
- `source(&self) -> &CredentialSource`

**Trait implementations**: `Debug`, `Clone`. NOT `Serialize` (contains `ApiKey`).

---

### Repository Trait: `LinearApiClient`

**Module**: `src/domain/repositories/linear_api_client.rs`

```rust
#[async_trait]
pub trait LinearApiClient: Send + Sync {
    async fn validate_api_key(&self, key: &ApiKey) -> Result<Workspace, AuthError>;
}
```

- Depends only on domain types (`ApiKey`, `Workspace`, `AuthError`) — zero infrastructure imports.
- Register with `#[cfg_attr(test, mockall::automock)]` so application-layer unit tests can mock it.
- Register `pub mod linear_api_client;` in `src/domain/repositories/mod.rs`.

**Implementation** (in infrastructure layer):
- `LinearGraphqlClient` — uses `reqwest` + `cynic` with the `viewer { id name organization { name urlKey } }` query.

---

### Repository Trait: `CredentialStore`

**Module**: `src/domain/repositories/credential_store.rs`

```rust
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Store the API key. Returns Err on keychain/file I/O failure.
    async fn store(&self, key: &ApiKey) -> Result<(), AuthError>;

    /// Retrieve the stored API key. Returns Ok(None) if absent.
    async fn retrieve(&self) -> Result<Option<ApiKey>, AuthError>;

    /// Remove the stored credential. Returns Ok(()) even if no credential exists.
    async fn remove(&self) -> Result<(), AuthError>;

    /// Returns the storage kind (for display in logout --dry-run output).
    fn kind(&self) -> StorageKind;
}

pub enum StorageKind {
    Keychain,
    File(std::path::PathBuf),
}
```

**Implementations** (in infrastructure layer):
- `KeyringCredentialStore` — uses `keyring` crate; service `"linear-cli"`, username `"default"`.
- `FileCredentialStore` — writes raw key to `~/.config/linear-cli/credentials` (or configured path) at mode `0o600`.

---

### Error Extension: `DomainError` + `AuthError`

**`AuthError`** — new error type in `src/domain/errors.rs`:

```rust
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("no credentials found; run `linear auth login` or set LINEAR_API_KEY")]
    NotAuthenticated,

    #[error("invalid or expired API key")]
    InvalidKey,

    #[error("API key validation failed: {0}")]
    ValidationFailed(String),

    #[error("could not reach Linear API: {0}")]
    NetworkError(String),

    #[error("system keychain is unavailable: {0}; re-run with --store-file to use file storage")]
    KeychainUnavailable(String),

    #[error("credential file error: {0}")]
    FileError(String),
}
```

Exit code mapping (in `main.rs`):
- `NotAuthenticated`, `InvalidKey`, `KeychainUnavailable` → exit 3
- `NetworkError`, `ValidationFailed` → exit 2

---

## Application Layer

### Use Case: `LoginUseCase`

**Module**: `src/application/use_cases/login.rs`

**Inputs**:
- `api_key: ApiKey`
- `store: Box<dyn CredentialStore>` (keychain or file, based on CLI flags)

**Flow**:
1. Validate `api_key` via `LinearGraphqlClient::validate_api_key(key)`.
2. On network error → return `Err(AuthError::NetworkError(...))`.
3. On invalid key → return `Err(AuthError::InvalidKey)`.
4. On existing credential → check for confirmation flag (passed as `overwrite: bool`); if `false` → return early with `AuthError::NotAuthenticated` (caller prompts user).
5. `store.store(&api_key)` → propagate errors.
6. Return `Ok(Workspace)`.

---

### Use Case: `AuthStatusUseCase`

**Module**: `src/application/use_cases/auth_status.rs`

**Inputs**:
- `env_key: Option<ApiKey>` (from `LINEAR_API_KEY`)
- `stores: Vec<Box<dyn CredentialStore>>` (keychain first, then file)
- `client: Arc<dyn LinearApiClient>`

**Flow**:
1. Check `env_key` first; if present, validate and return `AuthSession`.
2. For each store, call `retrieve()`; on `Some(key)`, validate and return `AuthSession`.
3. If all sources exhausted → return `Err(AuthError::NotAuthenticated)`.

**Output**: `AuthSession` (caller serialises to JSON or human text).

---

### Use Case: `LogoutUseCase`

**Module**: `src/application/use_cases/logout.rs`

**Inputs**:
- `stores: Vec<Box<dyn CredentialStore>>`
- `dry_run: bool`

**Flow**:
1. Collect all stores where `retrieve()` returns `Some`.
2. If `dry_run` → report what would be removed; return `Ok(vec![StorageKind])`.
3. For each store with credential → `remove()`; propagate errors.
4. Return `Ok(())`.

---

### Helper: `resolve_auth`

**Module**: `src/application/use_cases/resolve_auth.rs`

**Inputs**:
- `env_key: Option<ApiKey>`
- `stores: Vec<Box<dyn CredentialStore>>`
- `client: Arc<dyn LinearApiClient>`

**Purpose**: Called by non-auth commands to obtain a validated `AuthSession` before proceeding. Encapsulates FR-011 resolution order.

**Output**: `Result<AuthSession, AuthError>`

---

## Infrastructure Layer

### `KeyringCredentialStore`

**Module**: `src/infrastructure/auth/keyring_store.rs`

| Service name | `"linear-cli"` |
| Username | `"default"` |
| Errors | `keyring::Error::NoEntry` → `Ok(None)`; `PlatformFailure` → `AuthError::KeychainUnavailable` |

### `FileCredentialStore`

**Module**: `src/infrastructure/auth/file_store.rs`

| Default path | `~/.config/linear-cli/credentials` |
| Permissions | `0o600` |
| Format | Single-line raw key string (no JSON wrapper) |
| Warning | Printed to stderr on every `store()` call |

### `LinearGraphqlClient`

**Module**: `src/infrastructure/graphql/client.rs`

**Implements** the domain trait `LinearApiClient` (defined in `src/domain/repositories/linear_api_client.rs`).

Uses `reqwest` + `cynic` to send the `ValidateApiKey` query (validated against `schema.graphql` at build time via `#[cynic::schema_for_derives(file = "schema.graphql", module = "schema")]`). Sets `Authorization: Bearer {key}` header. Redacts key from `DEBUG`-level request logs via `ApiKey`'s custom `Display`.

---

## State Transitions

```
[No credential]
      │
      │  auth login (valid key + keychain available)
      ▼
[Authenticated — keychain]
      │
      │  auth logout
      ▼
[No credential]

[No credential]
      │
      │  auth login --store-file
      ▼
[Authenticated — file]

[Any state]
      │
      │  LINEAR_API_KEY set in env
      ▼
[Ephemeral AuthSession — not stored]
```

---

## Validation Rules

| Field | Rule | Error |
|-------|------|-------|
| `ApiKey` raw string | Non-empty | `DomainError::InvalidInput("api key cannot be empty")` |
| `Workspace.id` | Non-empty | `DomainError::InvalidInput("workspace id cannot be empty")` |
| `Workspace.name` | Non-empty | `DomainError::InvalidInput("workspace name cannot be empty")` |
| File path (--store-file) | Valid UTF-8, writable parent dir | `AuthError::FileError(...)` |
