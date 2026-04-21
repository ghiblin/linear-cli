<!--
SYNC IMPACT REPORT
==================
Version change: N/A → 1.0.0 (initial ratification)

Modified principles: none (initial creation)

Added sections:
  - Core Principles (6 principles)
  - Technology Stack & Constraints
  - AI Agent Interaction Contract
  - Governance

Removed sections: none (initial creation)

Templates reviewed:
  - .specify/templates/plan-template.md          ✅ compatible (Constitution Check section is generic)
  - .specify/templates/spec-template.md          ✅ compatible (FR/SC format maps to Linear domain)
  - .specify/templates/tasks-template.md         ✅ compatible (TDD test-first task ordering aligns with Principle II)

Follow-up TODOs:
  - TODO(RATIFICATION_DATE): Confirm exact ratification date if different from 2026-04-21
-->

# Linear CLI Constitution

## Core Principles

### I. Domain-Driven Design (NON-NEGOTIABLE)

The codebase MUST be structured around the Linear domain model:

- **Domain layer** (entities, value objects, aggregates, domain events) MUST have zero dependencies on
  infrastructure or application layers.
- Every concept that exists in Linear's domain (Issue, Project, Team, Cycle, WorkflowState, etc.) MUST be
  represented as an explicit domain type — no raw strings or primitive obsession.
- Repositories MUST be defined as traits in the domain layer; implementations live in the infrastructure layer.
- Domain services encapsulate business rules that span multiple aggregates.
- Application services orchestrate use cases; they depend on domain traits, never on concrete infrastructure.
- The ubiquitous language of the Linear API specification MUST be used consistently across all layers:
  code names, doc strings, error messages, and CLI command names.

**Rationale**: DDD enforces a clean boundary between Linear's business rules and technical concerns (HTTP,
serialization, storage), making the codebase testable and maintainable as the Linear API evolves.

### II. Test-First Development (NON-NEGOTIABLE)

All features MUST follow the Red-Green-Refactor cycle:

1. Write a failing test that captures the desired behavior (Red).
2. Obtain explicit confirmation that the test fails before writing any implementation code.
3. Write the minimal implementation to make the test pass (Green).
4. Refactor while keeping tests green.

- Unit tests target domain logic exclusively (no I/O, no network).
- Integration tests cover repository implementations and API client behavior using recorded fixtures or a
  test Linear workspace.
- End-to-end tests validate CLI commands against the real Linear API (opt-in, require `LINEAR_TEST_API_KEY`).
- Test coverage for domain and application layers MUST remain at or above 80%.
- Tests MUST be co-located with source in `src/` using Rust's `#[cfg(test)]` modules, except integration
  and e2e tests which live under `tests/`.

**Rationale**: TDD provides a living specification of domain behavior and prevents regression as new Linear
API features are integrated.

### III. Linear API Compliance

All interactions with Linear MUST conform to the official Linear GraphQL API specification:

- The GraphQL schema (`schema.graphql`) MUST be vendored and version-pinned in the repository.
- All query and mutation definitions MUST be validated against the pinned schema at build time.
- API response types MUST be deserialized into domain types through explicit mapping — no raw JSON leakage
  into the application or domain layers.
- Rate limits and pagination MUST be handled transparently by the API client (infrastructure layer).
- Breaking API changes MUST trigger a MAJOR version bump of the CLI.
- The Linear `apiKey` MUST be stored in the system keychain or an explicitly user-configured path; it MUST
  NOT be stored in plain-text files without user consent.

**Rationale**: The CLI is a contract with the Linear service. Schema pinning and build-time validation
prevent silent breakage when the API evolves.

### IV. CLI Interface Design for AI Agent Interaction

The CLI MUST be designed as a first-class interface for AI coding agents:

- Every command MUST support a `--output json` flag (or default to JSON when `--json` is set) producing
  deterministic, schema-stable output on stdout.
- Human-readable output is the default when stdout is a TTY; JSON is the default when stdout is a pipe.
- Exit codes MUST be meaningful: 0 = success, 1 = user/input error, 2 = API/network error, 3 = auth error.
- Commands MUST be composable: output of one command MUST be usable as input to another via pipes.
- Errors MUST be emitted on stderr; never mix error text with data output on stdout.
- Every command MUST have a `--dry-run` flag where the command would mutate state.
- The command taxonomy MUST mirror Linear's domain (e.g., `linear issue list`, `linear project create`).

**Rationale**: AI agents depend on stable, machine-parseable contracts. Mixing human UX concerns with
agent-facing I/O creates fragile integrations.

### V. Rust Safety and Idioms

The implementation MUST adhere to idiomatic, safe Rust:

- `unsafe` code is FORBIDDEN unless explicitly justified with a documented safety invariant.
- Error handling MUST use `Result<T, E>` with structured error types (via `thiserror`); `unwrap()`/`expect()`
  are forbidden in production code paths (allowed only in tests with explanatory messages).
- Panics MUST NOT cross library/API boundaries; all fallible operations MUST return `Result`.
- Dependencies MUST be minimal and auditable; every new crate dependency requires documented justification.
- Async code MUST use `tokio` as the runtime; blocking calls inside async contexts are forbidden.
- The `clippy` linter MUST pass at `deny(warnings)` level in CI.

**Rationale**: Rust's safety guarantees are only as strong as the discipline around `unsafe` and error
handling. Idiomatic code reduces the cognitive load for AI agents reading and generating code in this repo.

### VI. Observability and Debuggability

- Structured tracing MUST be instrumented using `tracing` crate with span-level context.
- All API requests and responses MUST be logged at `DEBUG` level with redacted auth tokens.
- The `--verbose` / `-v` flag MUST increase log verbosity (errors → info → debug → trace).
- Performance-sensitive operations (bulk queries, sync) MUST emit duration spans.
- Log output MUST be machine-parseable JSON when `LINEAR_LOG_FORMAT=json` is set.

**Rationale**: AI agents operating autonomously need full observability to diagnose failures without
human intervention.

## Technology Stack & Constraints

- **Language**: Rust (stable toolchain, minimum version pinned in `rust-toolchain.toml`)
- **Async runtime**: `tokio` (multi-thread flavor)
- **HTTP client**: `reqwest` with `rustls` (no OpenSSL dependency)
- **GraphQL client**: `cynic` or `graphql-client` (build-time schema validation required)
- **CLI framework**: `clap` v4 (derive API)
- **Serialization**: `serde` + `serde_json`
- **Error types**: `thiserror` for library errors, `anyhow` permitted only in binary entry points
- **Testing**: `cargo test`, `mockall` for trait mocking, `insta` for snapshot tests of CLI output
- **Credentials**: `keyring` crate for system keychain integration
- **Logging/Tracing**: `tracing` + `tracing-subscriber`
- **CI**: GitHub Actions; MUST run `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`

Constraints:
- Binary MUST compile and run on macOS (arm64, x86_64) and Linux (x86_64, arm64).
- Windows support is aspirational; MUST NOT introduce Windows-only breakage.
- Binary size target: under 20 MB release build.

## AI Agent Interaction Contract

This section defines the stable contract AI coding agents MUST rely on:

- Command signatures follow semver: additions are minor, removals/renames are major.
- JSON output schemas are versioned; fields are never removed without a major bump.
- The `linear --schema` command MUST output the current JSON output schema for all commands.
- All commands MUST be idempotent where possible; non-idempotent commands MUST warn via stderr.
- The `linear --version` command MUST output `{"version": "X.Y.Z", "api_schema": "YYYY-MM-DD"}`.

## Governance

- This constitution supersedes all other practices, guidelines, and conventions in this repository.
- Amendments MUST be proposed as a pull request modifying this file with an updated version and
  Sync Impact Report.
- MAJOR amendments (principle removal/redefinition) require documented rationale and a migration plan.
- MINOR amendments (new principle or material expansion) require rationale only.
- PATCH amendments (clarifications, wording) may be merged without formal review if uncontroversial.
- All implementation PRs MUST reference the principle(s) governing the changed code in the PR description.
- Complexity violations (e.g., `unsafe` usage, skipped tests) MUST be documented in the plan's
  Complexity Tracking table with justification.
- CI MUST enforce: tests green, clippy clean, fmt clean, schema validation passing.

**Version**: 1.0.0 | **Ratified**: 2026-04-21 | **Last Amended**: 2026-04-21
