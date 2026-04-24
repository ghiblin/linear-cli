# Research: Project CRUD Operations

**Branch**: `004-project-crud` | **Date**: 2026-04-23

## Decision 1: Adopt cynic v3 for all new GraphQL operations

**Decision**: Implement all project queries and mutations using `cynic` v3 derive macros with compile-time schema validation.

**Rationale**: Constitution Principle III mandates "All query and mutation definitions MUST be validated against the pinned schema at build time." The existing `validate_api_key` raw HTTP approach pre-dates the constitution and is tech debt. Cynic v3 uses `build.rs` + `cynic_codegen` to register the schema, and proc macros validate struct fields against the schema at compile time — field typos, missing types, and API schema drift all become compile errors.

**Alternatives considered**:
- `graphql-client`: Similar build-time validation but generates structs from `.graphql` files (the opposite direction). Cynic is already in `Cargo.toml` and better suited to Rust-first design.
- Continue raw HTTP for project operations: Violates Principle III and creates two divergent patterns in the same infra layer.

**Action items**:
1. Vendor full Linear SDL schema into `schema.graphql` (replace the current stub) via introspection query or from the Linear SDK GitHub repository.
2. Create `build.rs` with `cynic_codegen::register_schema("linear").from_sdl_file("schema.graphql")`.
3. Add `cynic_codegen = { version = "3", features = ["rkyv"] }` to `[build-dependencies]` in `Cargo.toml`.
4. Add `schema_path` module in `src/infrastructure/graphql/schema.rs`.

---

## Decision 2: Rate-limit detection — GraphQL error extension takes precedence over HTTP 429

**Decision**: Detect rate limiting by checking `errors[].extensions.type == "RATELIMITED"` in the GraphQL response body first; fall back to HTTP status 429 as a secondary signal. Retry up to 3 times with exponential backoff (1 s, 2 s, 4 s).

**Rationale**: Linear's GraphQL API returns rate-limit errors as HTTP 400 with `RATELIMITED` in `errors[].extensions.type`, not HTTP 429 as is common in REST APIs. The feature spec calls out "HTTP 429" (reasonable assumption), but the actual error surface is the GraphQL error body. To satisfy the spec's intent and remain robust against CDN-level rate limiting (which does use HTTP 429), the implementation checks both.

**Rate limit headers** (for `--debug` / `--verbose` output):
- `X-RateLimit-Requests-Limit` / `X-RateLimit-Requests-Remaining`
- `X-RateLimit-Complexity-Limit` / `X-RateLimit-Complexity-Remaining`
- `X-RateLimit-Requests-Reset` (UTC epoch milliseconds)

**Alternatives considered**:
- HTTP 429 only: Misses Linear's primary error signalling mechanism.
- RATELIMITED extension only: May miss proxy/CDN level rate limiting.

---

## Decision 3: Project identifier format — `KEY-N` display ID with UUID fallback

**Decision**: `ProjectId` value object accepts both UUID (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`) and display ID (`[A-Z]+-\d+`, e.g., `PRJ-1`). Auto-detection by regex: UUID pattern → direct API lookup; display ID pattern → resolve to UUID via a `projects(filter: {identifier: {eq: "PRJ-1"}})` query.

**Rationale**: Linear projects have an `identifier` field in `<key>-<N>` format (similar to issue identifiers like `ENG-123`). The feature spec explicitly requires accepting display IDs (FR-004). Display ID resolution requires one additional API round-trip on first lookup; this is acceptable given the CLI's human-interactive use case.

**Alternatives considered**:
- UUID only: Breaks FR-004; forces users to find UUID via another tool.
- Project key only (no number): Ambiguous if a workspace has multiple projects with the same key prefix.

---

## Decision 4: Pagination — cursor-based relay model; `--all` collects pages sequentially

**Decision**: Use Linear's Relay-style cursor pagination (`first`/`after` on `ProjectConnection`). Default page size: 50 (per FR-003 and Linear API default). `--all` flag drives a sequential loop: fetch → check `pageInfo.hasNextPage` → if true, fetch next page with `after: endCursor` → repeat. Each page fetch honours the rate-limit retry policy.

**Rationale**: Linear uses Relay cursor pagination across all list queries. Sequential (not parallel) page fetching avoids triggering rate limits for large workspaces.

**Alternatives considered**:
- Always return all results: Too slow and expensive for large workspaces on a simple `project list` call.
- Parallel page fetching: Higher throughput but increases complexity-point consumption and risks RATELIMITED errors.

---

## Decision 5: State representation — domain `ProjectState` maps to Linear API string values

**Decision**: `ProjectState` domain enum has five variants: `Planned`, `Started`, `Paused`, `Completed`, `Cancelled`. These map to the Linear API's `state` field string values `"planned"`, `"started"`, `"paused"`, `"completed"`, `"cancelled"` in the infrastructure layer (not exposed to domain). CLI accepts lowercase strings matching FR-007.

**Rationale**: Linear's project state values are lowercase strings in the GraphQL API. Representing them as a typed enum in the domain layer prevents invalid states from entering business logic.

**Note on schema verification**: The exact GraphQL type name for project state (enum vs string field) and its member values should be verified against the vendored schema during implementation. The research indicates `planned/started/paused/completed/cancelled` are the operative values.

---

## Decision 6: Cynic schema module architecture

**Decision**: Single `#[cynic::schema("linear")]` module at `src/infrastructure/graphql/schema.rs`. All `QueryFragment` and `InputObject` derives in `src/infrastructure/graphql/queries/` and `src/infrastructure/graphql/mutations/` reference this schema module. The existing `validate_api_key` raw HTTP implementation is left unchanged (to be migrated in a future feature).

**Rationale**: Clean infrastructure separation; one schema registration per crate. Migrating `validate_api_key` in this feature would be scope creep.

**Alternatives considered**:
- Separate schema modules per feature: Over-engineering for a single-crate CLI.
- Full migration of existing raw HTTP code in this feature: Out of scope; addressed as separate tech-debt item.
