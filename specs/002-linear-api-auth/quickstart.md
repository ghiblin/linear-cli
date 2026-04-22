# Quickstart: Linear API Authentication

**Branch**: `002-linear-api-auth`

---

## Prerequisites

- Rust 1.85.0 toolchain (`rustup show` to confirm)
- A Linear Personal API Key: Settings → API → Personal API Keys → Create Key

---

## Authenticate (default — system keychain)

```sh
linear auth login
# Prompted: Enter your Linear API key: ••••••••
# Output:   ✓ Authenticated as Jane Smith (workspace: My Org)
```

## Authenticate (file fallback — shared machines or CI)

```sh
linear auth login --store-file
# Warning: credentials will be stored unencrypted at ~/.config/linear-cli/credentials
# Output:  ✓ Authenticated as Jane Smith (workspace: My Org)

# Or specify a path:
linear auth login --store-file /path/to/credentials
```

## Use environment variable (CI / ephemeral)

```sh
export LINEAR_API_KEY=lin_api_xxxxxxxxxxxx
linear auth status    # uses env var, no login required
linear issue list     # env var credential is used automatically
```

The env var is never written to disk and takes precedence over any stored credential.

---

## Check authentication status

```sh
linear auth status
# ✓ Authenticated (workspace: My Org, source: keychain)

# Machine-readable:
linear auth status --output json
```

---

## Sign out

```sh
linear auth logout
# ✓ Signed out. Credentials removed from keychain.

# Preview without deleting:
linear auth logout --dry-run
# Would remove: keychain entry (service: linear-cli, account: default)
# No changes made (--dry-run).
```

---

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 2 | Network / API error (Linear unreachable) |
| 3 | Auth error (no credentials, invalid key, keychain unavailable) |

---

## Running tests

```sh
# Unit + integration tests (no network required)
cargo test

# End-to-end tests (requires a real Linear API key)
LINEAR_TEST_API_KEY=lin_api_xxxxxxxxxxxx cargo test --test auth_e2e
```
