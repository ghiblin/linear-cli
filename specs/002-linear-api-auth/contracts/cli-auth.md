# CLI Contract: `linear auth`

**Phase**: 1 — Design  
**Branch**: `002-linear-api-auth`  
**Date**: 2026-04-21  
**Schema version**: 1.0.0

This document is the stable contract for the `linear auth` command group. It defines command signatures, flags, exit codes, and JSON output schemas that AI agents and scripts may depend on.

---

## Command Group

```
linear auth <SUBCOMMAND>
```

**Subcommands**: `login`, `status`, `logout`

---

## `linear auth login`

Store a Linear personal API key after remote validation.

### Signature

```
linear auth login [OPTIONS]
```

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--store-file [PATH]` | optional path | `~/.config/linear-cli/credentials` | Store credential in plain-text file instead of keychain |

### Global flags inherited

| Flag | Description |
|------|-------------|
| `--output json` / `--json` | Force JSON output |
| `-v` / `--verbose` | Increase log verbosity |

### Behaviour

1. Prompts for an API key on stdin (TTY) or reads from stdin pipe.
2. Validates the key against `viewer { id name organization { name urlKey } }`.
3. Stores the key in the selected store with a confirmation message.
4. If a credential already exists, warns and prompts for confirmation before overwriting.
5. If `--store-file` is used, prints a warning to stderr that credentials are stored unencrypted.

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Key valid and stored successfully |
| 2 | Linear API unreachable during validation |
| 3 | Invalid/expired key; keychain unavailable; user cancelled |

### Output (TTY, success)

```
✓ Authenticated as <user-name> (workspace: <org-name>)
```

### Output (JSON, `--output json`)

**Success schema** (`$schema` version `1.0.0`):
```json
{
  "authenticated": true,
  "user": {
    "id": "<string>",
    "name": "<string>"
  },
  "workspace": {
    "id": "<string>",
    "name": "<string>",
    "url_key": "<string>"
  },
  "storage": "keychain" | "file",
  "storage_path": "<string | null>"
}
```

**Error schema** (exit code ≠ 0):
```json
{
  "authenticated": false,
  "error": "<string>",
  "error_code": "invalid_key" | "network_error" | "keychain_unavailable" | "cancelled"
}
```

---

## `linear auth status`

Report whether valid credentials are present and identify the connected workspace.

### Signature

```
linear auth status [OPTIONS]
```

### Options

No command-specific options. Inherits global flags.

### Behaviour

1. Resolves credentials in order: `LINEAR_API_KEY` env var → keychain → file.
2. Validates the resolved key remotely (`viewer` query).
3. Outputs workspace name and auth state.

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Authenticated and credential valid |
| 2 | Credential present but Linear API unreachable |
| 3 | No credential found; credential present but invalid/revoked |

### Output (TTY, authenticated)

```
✓ Authenticated (workspace: <org-name>, source: keychain)
```

### Output (TTY, not authenticated)

```
✗ Not authenticated. Run `linear auth login` or set LINEAR_API_KEY.
```

### Output (JSON, `--output json`)

**Authenticated schema**:
```json
{
  "authenticated": true,
  "workspace": {
    "id": "<string>",
    "name": "<string>",
    "url_key": "<string>"
  },
  "source": "env_var" | "keychain" | "file",
  "source_path": "<string | null>"
}
```

**Not authenticated schema**:
```json
{
  "authenticated": false,
  "error": "no credentials found; run `linear auth login` or set LINEAR_API_KEY",
  "error_code": "not_authenticated" | "invalid_key" | "network_error"
}
```

---

## `linear auth logout`

Remove stored credentials from all storage locations.

### Signature

```
linear auth logout [OPTIONS]
```

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--dry-run` | bool | false | Report what would be removed without deleting |

### Behaviour

1. Checks all known storage locations (keychain, configured file path).
2. If `--dry-run`: lists what would be removed and exits 0 without deleting.
3. Otherwise: removes credentials from all locations and confirms.
4. If no credentials found: exits 0 with informational message.

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Credentials removed (or none found); dry-run completed |
| 3 | Storage access error (keychain permission denied, file locked) |

### Output (TTY, credentials removed)

```
✓ Signed out. Credentials removed from keychain.
```

### Output (TTY, dry-run)

```
Would remove: keychain entry (service: linear-cli, account: default)
No changes made (--dry-run).
```

### Output (TTY, not authenticated)

```
No credentials found. Nothing to remove.
```

### Output (JSON, `--output json`)

**Success schema**:
```json
{
  "logged_out": true,
  "removed": [
    {
      "storage": "keychain" | "file",
      "path": "<string | null>"
    }
  ],
  "dry_run": false
}
```

**Dry-run schema**:
```json
{
  "logged_out": false,
  "would_remove": [
    {
      "storage": "keychain" | "file",
      "path": "<string | null>"
    }
  ],
  "dry_run": true
}
```

**Nothing to remove**:
```json
{
  "logged_out": false,
  "removed": [],
  "dry_run": false
}
```

---

## Environment Variables

| Variable | Effect |
|----------|--------|
| `LINEAR_API_KEY` | Per-invocation credential override; takes precedence over stored credentials; never stored |

---

## Exit Code Reference

| Code | Meaning | Auth context |
|------|---------|-------------|
| 0 | Success | — |
| 1 | User / input error | Bad flags, missing required argument |
| 2 | API / network error | Linear API unreachable |
| 3 | Auth error | No credential, invalid key, keychain unavailable |

---

## Versioning Policy

- **MINOR**: New optional output fields, new optional flags.
- **MAJOR**: Removed fields, renamed commands, changed exit code semantics.
- Schema version bumps are tracked in this document header.
