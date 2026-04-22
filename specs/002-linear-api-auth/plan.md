# Implementation Plan: Linear API Authentication

**Branch**: `002-linear-api-auth` | **Date**: 2026-04-21 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/002-linear-api-auth/spec.md`

## Summary

Implement the `linear auth` command group (`login`, `status`, `logout`) that resolves, validates, stores, and removes Linear API keys using the system keychain by default, with an opt-in file-based fallback. The implementation follows the existing DDD layered architecture, adding `ApiKey`, `Workspace`, and `AuthSession` domain types, a `CredentialStore` trait, keychain and file infrastructure implementations, and three application use cases wired to the CLI.

## Technical Context

**Language/Version**: Rust 1.85.0 (pinned in `rust-toolchain.toml`)  
**Primary Dependencies**: `keyring` v3, `cynic` (GraphQL), `reqwest` + `rustls`, `clap` v4, `thiserror`, `tokio`  
**Storage**: System keychain via `keyring` crate (default); plain-text file at `~/.config/linear-cli/credentials` (opt-in via `--store-file`)  
**Testing**: `cargo test`, `mockall` for trait mocking, `insta` for snapshot tests  
**Target Platform**: macOS (arm64, x86_64), Linux (x86_64, arm64)  
**Project Type**: CLI (single binary)  
**Performance Goals**: `auth login` < 30 s (SC-001); `auth status` < 2 s (SC-002)  
**Constraints**: API keys redacted from all log output (SC-005); keys never in plain-text files by default (SC-004)  
**Scale/Scope**: Single Linear account per machine; per-invocation credential resolution

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. DDD — Domain types | `ApiKey`, `Workspace`, `AuthSession` as explicit domain types | ✅ PASS |
| I. DDD — No primitive obsession | No raw `String` for API key in application/domain layers | ✅ PASS |
| I. DDD — Repository trait | `CredentialStore` defined in domain; implementations in infra | ✅ PASS |
| I. DDD — Repository trait | `LinearApiClient` defined in domain (`src/domain/repositories/`); `LinearGraphqlClient` impl in infra | ✅ PASS |
| I. DDD — Ubiquitous language | `AuthSession`, `Workspace`, `CredentialStore` match Linear domain | ✅ PASS |
| II. TDD — Red-Green-Refactor | Task ordering is test-first (failing test before each implementation task) | ✅ PASS |
| II. TDD — Coverage ≥ 80% | Unit tests in every new module via `#[cfg(test)]` | ✅ PASS |
| II. TDD — Test placement | Unit tests co-located in `src/`; integration/e2e under `tests/` | ✅ PASS |
| III. Linear API Compliance | `viewer` query validates key before any storage occurs | ✅ PASS |
| III. Linear API Compliance | API key not stored in plain text by default | ✅ PASS |
| IV. CLI Design — JSON output | All three auth commands support `--output json` / `--json` | ✅ PASS |
| IV. CLI Design — Exit codes | Exit 2 = network error; exit 3 = auth error; per FR-002, FR-003, FR-007 | ✅ PASS |
| IV. CLI Design — `--dry-run` | `auth logout` supports `--dry-run` per FR-006 | ✅ PASS |
| V. Rust Safety — No `unsafe` | No `unsafe` blocks; all errors via `Result` | ✅ PASS |
| V. Rust Safety — Error types | `AuthError` via `thiserror`; no `unwrap()` in production paths | ✅ PASS |
| VI. Observability — Redaction | Custom `Debug`/`Display` on `ApiKey` emits `[REDACTED]` | ✅ PASS |
| VI. Observability — Tracing | All infra operations instrumented with `tracing` spans | ✅ PASS |

**Post-Phase-1 Re-check**: Constitution Check re-run after data-model.md completed. All 16 gates pass. No violations to track in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/002-linear-api-auth/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── cli-auth.md      # Phase 1 output — CLI command contract
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code

```text
src/
├── domain/
│   ├── entities/
│   │   ├── auth_session.rs  [NEW]   — AuthSession, CredentialSource
│   │   ├── workspace.rs     [NEW]   — Workspace entity
│   │   ├── issue.rs
│   │   └── team.rs
│   ├── value_objects/
│   │   ├── api_key.rs       [NEW]   — ApiKey value object (redacted Debug)
│   │   └── ...existing...
│   ├── repositories/
│   │   ├── credential_store.rs    [NEW] — CredentialStore trait, StorageKind
│   │   ├── linear_api_client.rs   [NEW] — LinearApiClient trait
│   │   └── ...existing...
│   └── errors.rs            [EXTEND] — add AuthError variants
├── application/
│   ├── use_cases/
│   │   ├── login.rs         [NEW]   — LoginUseCase
│   │   ├── auth_status.rs   [NEW]   — AuthStatusUseCase
│   │   ├── logout.rs        [NEW]   — LogoutUseCase
│   │   ├── resolve_auth.rs  [NEW]   — resolve_auth() helper
│   │   └── ...existing...
│   └── errors.rs            [EXTEND] — ApplicationError wraps AuthError
├── infrastructure/
│   ├── auth/
│   │   ├── mod.rs           [NEW]
│   │   ├── keyring_store.rs [NEW]   — KeyringCredentialStore
│   │   └── file_store.rs    [NEW]   — FileCredentialStore
│   ├── graphql/
│   │   ├── client.rs        [NEW]   — LinearGraphqlClient
│   │   └── mod.rs           [EXTEND]
│   └── repositories/
│       └── ...existing...
├── cli/
│   ├── commands/
│   │   ├── auth.rs          [NEW]   — AuthCommand (login/status/logout subcommands)
│   │   ├── mod.rs           [EXTEND] — add Commands::Auth
│   │   └── ...existing...
│   └── output.rs
└── main.rs                  [EXTEND] — handle Auth commands; add exit code 2/3 mapping

tests/
├── integration/
│   └── auth_integration.rs  [NEW]   — keyring/file store round-trip tests
└── e2e/
    └── auth_e2e.rs          [NEW]   — opt-in via LINEAR_TEST_API_KEY
```

**Structure Decision**: Single Rust binary with DDD layers as modules. Auth is a vertical slice across all four layers. No new crates required beyond the already-declared `keyring` dependency.

## Complexity Tracking

> No constitution violations. No entries required.
