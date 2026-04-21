# Research: Initial Project Structure

**Phase**: 0 — Research  
**Feature**: 001-initial-project-structure  
**Date**: 2026-04-21

## Decision 1: GraphQL Client — cynic

**Decision**: Use `cynic` as the GraphQL client library.

**Rationale**: cynic follows a struct-first model — Rust types are defined first and GraphQL queries are generated from them. This aligns directly with the constitution's DDD mandate: domain entities remain the source of truth, and the GraphQL layer is an infrastructure concern that adapts to those types. `graphql-client` follows a query-first model that auto-generates structs, which would leak GraphQL schema details into domain types.

**Alternatives considered**:
- `graphql-client`: Faster to bootstrap, but auto-generated types create tight coupling between the GraphQL schema and the domain model — violates Principle I (DDD).

---

## Decision 2: Single Binary Crate (Initial)

**Decision**: Start with a single binary crate (`linear`) using module-based DDD layering. Migrate to a Cargo workspace when/if reusable library extraction is needed.

**Rationale**: A single crate with clear module boundaries (`src/domain/`, `src/application/`, `src/infrastructure/`, `src/cli/`) enforces DDD separations via Rust's visibility system without the operational overhead of a workspace. Module-level visibility rules (`pub(crate)`, `pub(super)`) are sufficient to prevent layer leakage during early development. Workspace migration is a clean refactor once boundaries are proven stable.

**Alternatives considered**:
- Cargo workspace from day one: Stronger compile-time boundary enforcement via crate APIs, but adds friction (multiple `Cargo.toml` files, cross-crate dependency wiring) that slows initial development without proportional benefit at this scale.

---

## Decision 3: Rust Version — 1.85.0

**Decision**: Pin `rust-toolchain.toml` to channel `1.85.0`.

**Rationale**: Rust 1.85.0 (February 2025) is the first stable release to include the Rust 2024 Edition, providing a solid stability checkpoint. Pinning ensures all contributors and CI pipelines use identical compiler behavior, making `clippy -D warnings` deterministic across environments.

**Alternatives considered**:
- `channel = "stable"` (unpinned): Simpler but non-deterministic; a compiler upgrade can silently introduce new clippy lints that break CI.
- Newer stable (1.86+): No features required beyond 1.85.0 for this project; upgrading should be a deliberate decision.

---

## Decision 4: DDD Module Layout

**Decision**: Organize `src/` into four top-level modules with strict dependency direction: `domain` ← `application` ← `infrastructure`, with `cli` depending only on `application`.

**Rationale**: Mirrors the constitution's layer requirements. Rust's module system enforces one-way dependencies if infrastructure types are never imported in domain or application modules. Each layer exposes a minimal public surface area.

**Module structure**:
```
src/
├── main.rs                   # tokio::main entry point; wires DI
├── domain/
│   ├── mod.rs
│   ├── entities/             # Linear domain types (Issue, Project, Team, …)
│   ├── value_objects/        # Typed wrappers (IssueId, Priority, …)
│   ├── repositories/         # Abstract traits only (no implementations)
│   └── errors.rs
├── application/
│   ├── mod.rs
│   ├── use_cases/            # One file per use case
│   └── errors.rs
├── infrastructure/
│   ├── mod.rs
│   ├── graphql/              # cynic client, schema bindings
│   ├── repositories/         # Trait implementations
│   └── auth.rs               # keyring integration
└── cli/
    ├── mod.rs
    ├── commands/             # clap subcommand structs
    └── output.rs             # TTY/JSON output formatting
```

**Alternatives considered**:
- Flat `src/models/`, `src/services/` layout: Common in smaller projects but conflates all concerns and provides no layer boundary enforcement.

---

## Decision 5: GitHub Actions CI Matrix

**Decision**: Use a build matrix with native runners for macOS (both architectures) and cross-compilation for Linux arm64.

**Rationale**: GitHub Actions provides native macOS arm64 runners (`macos-latest`), avoiding cross-compilation complexity for Apple Silicon. Linux arm64 has no native public runner, so `cross-rs` via `ubuntu-latest` is the standard approach.

**Matrix**:
| Target | Runner | Strategy |
| --- | --- | --- |
| `aarch64-apple-darwin` | `macos-latest` | Native |
| `x86_64-apple-darwin` | `macos-13` | Native |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | cross-rs |

**CI gates (all pull requests)**:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. Schema validation (build-time via cynic, implicit in step 3)

**Alternatives considered**:
- QEMU emulation for Linux arm64: Slower than cross-rs, no advantage for compile-and-test workflows.
- Single-platform CI initially: Violates the constitution's platform support mandate.

---

## Resolved: All NEEDS CLARIFICATION Items

No NEEDS CLARIFICATION markers were present in the spec. All technical decisions are derived from the constitution and confirmed by research above.
