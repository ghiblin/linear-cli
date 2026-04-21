# Developer Quickstart: Initial Project Structure

**Feature**: 001-initial-project-structure  
**Date**: 2026-04-21

## Prerequisites

| Requirement | Version | Notes |
| --- | --- | --- |
| Rust toolchain | 1.85.0 | Managed automatically by `rust-toolchain.toml` via `rustup` |
| `rustup` | Any recent | Install from https://rustup.rs |
| Git | Any recent | Required for cloning |

> `rustup` will automatically download and activate the pinned toolchain the first time you run any `cargo` command in the project directory.

---

## Setup

```sh
git clone <repository-url>
cd linear-cli
cargo build
```

Expected outcome: zero errors, zero warnings, binary produced at `target/debug/linear`.

Release binary size: **2.4 MB** (well under the 20 MB limit). Build with `cargo build --release`.

---

## Run Quality Checks

Run all checks in one pass (matches what CI enforces):

```sh
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

Each command:
- `cargo fmt --check` — verifies formatting without modifying files
- `cargo clippy -- -D warnings` — lints at deny-warnings level
- `cargo test` — runs all unit and integration tests

---

## Verify the Binary

```sh
cargo run -- --version
# Output: {"version":"0.1.0","api_schema":"YYYY-MM-DD"}

cargo run -- --help
# Output: human-readable command listing

cargo run -- issue list
# Output: [] (stub; no real API connection)
```

---

## Project Layout

```
linear-cli/
├── Cargo.toml                  # Single crate manifest
├── rust-toolchain.toml         # Pinned toolchain (1.85.0)
├── schema.graphql              # Vendored Linear GraphQL schema (placeholder)
├── src/
│   ├── main.rs                 # Binary entry point
│   ├── domain/                 # Domain entities, value objects, repository traits
│   ├── application/            # Use case orchestrators
│   ├── infrastructure/         # GraphQL client, repository implementations, auth
│   └── cli/                    # clap commands, output formatting
└── tests/
    ├── integration/            # Tests against real or recorded API responses
    └── e2e/                    # End-to-end tests (require LINEAR_TEST_API_KEY)
```

**Dependency direction** (enforced by module visibility):
```
cli → application → domain
infrastructure → domain
main → cli, infrastructure  (wires DI)
```

---

## Running Specific Test Suites

```sh
# Unit tests only (embedded in src/ with #[cfg(test)])
cargo test --lib

# Integration tests
cargo test --test '*'

# End-to-end tests (requires env var)
LINEAR_TEST_API_KEY=<key> cargo test --test e2e
```

---

## Adding a New Domain Entity

1. Create `src/domain/entities/<entity>.rs`
2. Define the entity struct with typed fields (no primitive obsession)
3. Define value objects in `src/domain/value_objects/<vo>.rs`
4. Define a repository trait in `src/domain/repositories/<entity>_repository.rs`
5. Write a failing unit test first (`#[cfg(test)]` block in the same file)
6. Confirm the test fails, then implement the minimal logic to pass
7. Add a stub implementation in `src/infrastructure/repositories/<entity>_repository_impl.rs`

---

## CI Pipeline

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs on every pull request:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. Build matrix across all four target platforms

A pull request MUST pass all four steps before it can be merged.
