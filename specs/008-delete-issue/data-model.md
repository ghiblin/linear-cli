# Data Model: Delete Issue

## Entities Involved

**Issue** (existing domain entity — no changes to fields)
- Used only as the target of deletion; no new attributes required.

## Domain Input

No new input struct needed. The operation takes a single `IssueId` value object (already defined in `src/domain/value_objects/issue_id.rs`).

## Application Outcome

```
DeleteOutcome
  └─ Deleted    // operation succeeded; issue enters 30-day grace period
```

Defined in `src/application/use_cases/delete_issue.rs`.

## Repository Trait Extension

```
IssueRepository::delete(id: IssueId) -> Result<(), DomainError>
```

Added to the existing trait in `src/domain/repositories/issue_repository.rs`.

## GraphQL Response Mapping

| GraphQL field | Mapped to |
|---------------|-----------|
| `success: Boolean!` | `true` → `Ok(())`; `false` → `Err(DomainError::NotFound(...))` |
| `entity: Issue?` | ignored (always null for delete) |
| `lastSyncId: Float!` | ignored |

## State Transitions

Issues in Linear enter a 30-day grace period after `issueDelete` before permanent removal. This is governed entirely by the Linear platform and is transparent to the CLI.
