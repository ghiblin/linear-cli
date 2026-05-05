# Implementation Plan: Improve Project List Identifiers

**Branch**: `005-improve-project-list` | **Date**: 2026-05-05 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/005-improve-project-list/spec.md`

## Summary

Surface the `slug_id` field (already fetched from the Linear API and stored in the `Project` domain entity) in all project command outputs: the `project list` table, the `project get` detail view, the `project create` success message, and the `project update` success message. Add `slug_id` to the `ProjectDto` and `MutationResultDto` JSON output structs. All changes are confined to `src/cli/commands/project.rs`.

## Technical Context

**Language/Version**: Rust stable (toolchain pinned in `rust-toolchain.toml`)  
**Primary Dependencies**: clap v4 (CLI), cynic (GraphQL), serde/serde_json, tokio, reqwest  
**Storage**: N/A — no local persistence; all data from Linear API  
**Testing**: cargo test, insta (snapshot), mockall (trait mocking)  
**Target Platform**: macOS (arm64, x86_64) + Linux (x86_64, arm64)  
**Project Type**: CLI tool  
**Performance Goals**: No new API calls introduced; slug is already in fetched data  
**Constraints**: Binary size <20 MB; no new crate dependencies  
**Scale/Scope**: Single workspace per invocation; changes are output-layer only

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Domain-Driven Design | ✅ Pass | `slug_id` already in domain entity. Changes are in CLI/output layer only. No domain logic modified. |
| II. Test-First Development | ✅ Must comply | TDD order specified in quickstart.md. Tests for DTO serialization and human output snapshots written before implementation. |
| III. Linear API Compliance | ✅ Pass | No new API queries. `slug_id` already in `ProjectNode` fragment. |
| IV. CLI Interface Design | ✅ Pass | JSON output gains `slug_id`. Human output gains slug column/field. Exit codes unchanged. |
| V. Rust Safety & Idioms | ✅ Pass | No unsafe code. Only struct field additions and format string changes. |
| VI. Observability | ✅ Pass | No new performance-sensitive code paths. Existing tracing spans unchanged. |

**Post-design re-check**: All principles remain satisfied. No violations. Complexity Tracking table not required.

## Project Structure

### Documentation (this feature)

```text
specs/005-improve-project-list/
├── plan.md              ← this file
├── research.md          ← Phase 0: all unknowns resolved
├── data-model.md        ← Phase 1: DTO changes
├── quickstart.md        ← Phase 1: implementation guide
├── contracts/
│   └── project-commands.md   ← Phase 1: CLI output contracts
├── checklists/
│   └── requirements.md
└── tasks.md             ← Phase 2 output (/speckit.tasks — not yet created)
```

### Source Code (repository root)

```text
src/
└── cli/
    └── commands/
        └── project.rs    ← all changes in this file
```

No other source files require modification.

## Implementation Scope

### Changes in `src/cli/commands/project.rs`

**1. `ProjectDto` struct** — add field:
```
slug_id: String
```

**2. `From<&Project> for ProjectDto`** — add mapping:
```
slug_id: p.slug_id.clone(),
```

**3. `MutationResultDto` struct** — add field:
```
slug_id: String
```

**4. `project list` human output** — change format string:
- Before: `println!("  {:<40} {:<12} {}", p.name, p.state.to_string(), date)`
- After: `println!("  {:<35} {:<22} {:<12} {}", p.name, p.slug_id, p.state.to_string(), date)`

**5. `project get` human output** — add line after `Name:`:
- `println!("Slug:        {}", project.slug_id)`

**6. `project create` human output** — change parenthetical:
- Before: `println!("Created project: \"{}\" ({})", project.name, project.id)`
- After: `println!("Created project: \"{}\" ({})", project.name, project.slug_id)`

**7. `project create` JSON `MutationResultDto`** — add `slug_id: project.slug_id.clone()`

**8. `project update` human output** — change identifier shown:
- Before: `println!("Updated project {}: state → {}", project.id, project.state)`
- After: `println!("Updated project {}: state → {}", project.slug_id, project.state)`

**9. `project update` JSON `MutationResultDto`** — add `slug_id: project.slug_id.clone()`

### Tests to write (TDD — write first)

All tests live in the `#[cfg(test)] mod tests` block within `project.rs` or as integration snapshots:

1. `project_dto_includes_slug_id` — unit test: build a `ProjectDto` from a `Project`, assert `slug_id` field is populated.
2. `mutation_result_dto_includes_slug_id` — unit test: build a `MutationResultDto`, assert JSON serialization includes `slug_id`.
3. Snapshot tests (insta) for list/get/create/update human output formats (if insta is in use for CLI output tests).

## Complexity Tracking

*(No violations — table omitted as per constitution governance rules.)*
