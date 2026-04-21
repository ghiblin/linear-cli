# Data Model: Initial Project Structure

**Phase**: 1 — Design  
**Feature**: 001-initial-project-structure  
**Date**: 2026-04-21

## Overview

The initial project skeleton scaffolds the structural foundation of the Linear CLI domain. No real Linear API integration is included — domain entities are placeholder stubs that establish the correct module hierarchy and dependency direction. The data model below represents the types to be introduced in the skeleton, not the full Linear domain model.

---

## Domain Layer Entities (Stubs)

These types exist in `src/domain/` and have **zero dependencies** on infrastructure or application layers.

### `Issue` (Entity)

Represents a work item in Linear. Scaffold with minimal fields sufficient to demonstrate the entity pattern.

| Field | Type | Validation |
| --- | --- | --- |
| `id` | `IssueId` (value object) | Non-empty, UUID format |
| `title` | `String` | Non-empty, max 255 chars |
| `state` | `WorkflowState` (value object) | One of the defined state variants |
| `priority` | `Priority` (value object) | One of: No Priority, Urgent, High, Medium, Low |

**Relationships**: Belongs to exactly one `Team`.

**Domain invariant**: A newly created `Issue` MUST have a non-empty `title`.

---

### `Team` (Entity)

Represents an organisational unit in Linear.

| Field | Type | Validation |
| --- | --- | --- |
| `id` | `TeamId` (value object) | Non-empty, UUID format |
| `name` | `String` | Non-empty |
| `key` | `String` | 1–5 uppercase letters |

---

### Value Objects

| Type | Description | Invariants |
| --- | --- | --- |
| `IssueId` | Typed wrapper around a Linear issue UUID | Non-empty string; does not validate UUID format at domain level |
| `TeamId` | Typed wrapper around a Linear team UUID | Non-empty string |
| `Priority` | Enum: `NoPriority`, `Urgent`, `High`, `Medium`, `Low` | Exhaustive; maps to Linear integer values 0–4 |
| `WorkflowState` | Enum: `Backlog`, `Todo`, `InProgress`, `Done`, `Cancelled` | Exhaustive at domain level; real states fetched from API |

---

### Repository Traits (Domain Layer)

Defined in `src/domain/repositories/`. These are **traits only** — no implementations in the domain layer.

```
IssueRepository
  - list(team_id: TeamId) → Result<Vec<Issue>, DomainError>
  - get(id: IssueId) → Result<Issue, DomainError>

TeamRepository
  - list() → Result<Vec<Team>, DomainError>
  - get(id: TeamId) → Result<Team, DomainError>
```

---

## Application Layer (Use Case Stubs)

Defined in `src/application/use_cases/`. Each use case depends on domain repository traits, not concrete implementations.

| Use Case | Input | Output | Notes |
| --- | --- | --- | --- |
| `ListIssues` | `team_id: TeamId` | `Vec<Issue>` | Delegates to `IssueRepository::list` |
| `GetIssue` | `id: IssueId` | `Issue` | Delegates to `IssueRepository::get` |

---

## Infrastructure Layer (Stub Implementations)

Defined in `src/infrastructure/repositories/`. Each struct implements the corresponding domain repository trait.

| Struct | Implements | Dependency |
| --- | --- | --- |
| `LinearIssueRepository` | `IssueRepository` | cynic GraphQL client |
| `LinearTeamRepository` | `TeamRepository` | cynic GraphQL client |

The skeleton implementations return `Err(DomainError::NotImplemented)` — sufficient to wire up the dependency graph and confirm compilation.

---

## Error Types

### `DomainError` (`src/domain/errors.rs`)

| Variant | Description |
| --- | --- |
| `NotFound(String)` | Entity not found by the given identifier |
| `InvalidInput(String)` | Domain invariant violated |
| `NotImplemented` | Stub implementation placeholder; removed when real logic lands |

### `ApplicationError` (`src/application/errors.rs`)

| Variant | Wraps | Description |
| --- | --- | --- |
| `Domain(DomainError)` | `DomainError` | Propagated domain error |
| `Unexpected(String)` | — | Unexpected runtime error |

### `InfrastructureError` (inline per module)

Defined per infrastructure module using `thiserror`. Maps to `DomainError` at the repository boundary.

---

## State Transitions

Not applicable to the initial skeleton. State machine logic will be introduced when issue lifecycle management is specified.

---

## Schema File

`schema.graphql` — vendored Linear GraphQL schema, placed at the repository root. In the skeleton, this is an empty placeholder file with a comment indicating where the real schema will be placed. The cynic build script validates against this file at compile time.
