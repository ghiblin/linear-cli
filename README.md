# linear-cli

A command-line interface for [Linear](https://linear.app), built in Rust.

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| Rust toolchain | 1.85.0 | Managed automatically by `rust-toolchain.toml` via `rustup` |
| `rustup` | Any recent | Install from https://rustup.rs |
| Git | Any recent | Required for cloning |

> `rustup` automatically downloads and activates the pinned toolchain on first `cargo` invocation.

## Installation

```sh
git clone <repository-url>
cd linear-cli
cargo build
```

Expected outcome: zero errors, zero warnings, binary at `target/debug/linear`.

## Quick Start

```sh
# Version info
./target/debug/linear --version
# {"version":"0.1.0","api_schema":"2026-04-21"}

# Help
./target/debug/linear --help

# List issues (stub — returns empty list)
./target/debug/linear issue list --json
# []

# List teams (stub — returns empty list)
./target/debug/linear team list --json
# []
```

## Project Layout

```
linear-cli/
├── Cargo.toml                  # Single crate manifest
├── rust-toolchain.toml         # Pinned toolchain (1.85.0)
├── schema.graphql              # Vendored Linear GraphQL schema (placeholder)
├── src/
│   ├── main.rs                 # Binary entry point — wires DI, dispatches commands
│   ├── domain/                 # Pure business logic: entities, value objects, repo traits
│   ├── application/            # Use-case orchestrators
│   ├── infrastructure/         # GraphQL client, repository implementations, auth
│   └── cli/                    # clap commands, output formatting
└── tests/
    ├── integration/            # Integration tests against the compiled binary
    └── e2e/                    # End-to-end tests (require LINEAR_TEST_API_KEY)
```

**Dependency direction** (enforced by module visibility):

```
cli → application → domain
infrastructure → domain
main → cli, infrastructure  (wires DI)
```

## Development

### Quality checks (matches CI)

```sh
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Layer contribution guide

| Layer | Path | Add here when… |
|-------|------|----------------|
| Domain entity | `src/domain/entities/` | Adding a new Linear resource (issue, project, cycle…) |
| Value object | `src/domain/value_objects/` | Wrapping a primitive with domain invariants |
| Repository trait | `src/domain/repositories/` | Defining the persistence contract for a new entity |
| Use case | `src/application/use_cases/` | Orchestrating domain logic for a user action |
| API repository | `src/infrastructure/repositories/` | Implementing a repo trait against the Linear API |
| CLI command | `src/cli/commands/` | Exposing a use case as a CLI subcommand |

### Running specific test suites

```sh
# Unit tests
cargo test --bin linear

# Integration tests
cargo test --test integration

# End-to-end tests (requires env var)
LINEAR_TEST_API_KEY=<key> cargo test --test e2e
```

## CI

GitHub Actions runs on every push and pull request:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. Cross-platform build matrix: macOS arm64, macOS x86_64, Linux x86_64, Linux arm64

A pull request **must** pass all steps before merging.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User or input error (bad arguments) |
| 2 | API or network error |
| 3 | Authentication error |
