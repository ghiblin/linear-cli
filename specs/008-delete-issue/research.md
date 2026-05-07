# Research: Delete Issue

## API Capability

**Decision**: Use the `issueDelete(id: String!, permanentlyDelete: Boolean): IssueArchivePayload!` mutation.
**Rationale**: Confirmed present in vendored `schema.graphql`. Returns `IssueArchivePayload` with `success: Boolean!`, `lastSyncId: Float!`, and `entity: Issue?` (null when permanently deleted). The `permanentlyDelete` flag is admin-only and out of scope per spec.
**Alternatives considered**: Archiving instead of deleting — rejected because the spec asks for deletion, which is the distinct `issueDelete` mutation (not `issueArchive`).

## Pattern Selection

**Decision**: Mirror the `archive_project` pattern exactly.
**Rationale**: `archive_project` already implements the same "single-entity destructive mutation → `Result<(), DomainError>`" flow across all four layers. Consistency reduces cognitive overhead and keeps the codebase uniform.
**Alternatives considered**: Reusing the archive infrastructure — rejected because `issueDelete` and `issueArchive` are separate mutations with different semantics.

## Return Type

**Decision**: `IssueRepository::delete` returns `Result<(), DomainError>`. The use case returns `Result<DeleteOutcome, DomainError>` where `DeleteOutcome` is a unit-like enum (`Deleted`).
**Rationale**: The `IssueArchivePayload.entity` is null after a successful delete; there is no meaningful data to return. Matches `archive` on `ProjectRepository`.
**Alternatives considered**: Returning the deleted issue ID — unnecessary since the caller already has it.

## cynic Mutation Shape

**Decision**: Add `IssueDeleteMutation` + `IssueArchivePayload` + `IssueDeleteVariables` to `issue_mutations.rs` using `cynic` derives, matching the `ProjectArchiveMutation` shape.
**Rationale**: All mutations in the project use cynic for build-time schema validation. The delete mutation takes a single `id: String!` argument, making it the simplest possible cynic mutation.
**Alternatives considered**: Hand-written serde structs (used for `IssueUpdateInput`) — not needed here since there is no optional-null semantics complexity.

## CLI UX

**Decision**: `linear issue delete <id> [--dry-run] [--json] [--output <format>]`
**Rationale**: Consistent with `issue create` and `issue update`. `--dry-run` is required by Constitution Principle IV for all mutating commands.
**Alternatives considered**: Confirmation prompt — deferred; dry-run flag is sufficient for v1.
