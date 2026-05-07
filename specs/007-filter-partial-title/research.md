# Research: Filter by Partial Title

## API Filter Support

**Decision**: Use server-side filtering via `IssueFilter.title` and `ProjectFilter.name` with `containsIgnoreCase`.

**Rationale**: The vendored `schema.graphql` confirms both fields exist:
- `IssueFilter.title: StringComparator` (line 15627 context)
- `ProjectFilter.name: StringComparator` (confirmed via schema inspection)
- `StringComparator.containsIgnoreCase: String` (lines 39064–39065)

**Alternatives considered**:
- Client-side post-filter: rejected because it defeats pagination (must fetch all pages to filter, negating the sub-3-second goal for large workspaces)
- `contains` (case-sensitive): rejected because FR-003/FR-006 require case-insensitive matching

## GraphQL Input Type Changes

**Decision**: Extend the existing `StringComparatorInput` (in `issue_queries.rs`) and `StringComparator` (in `project_queries.rs`) with a `contains_ignore_case` field mapping to `containsIgnoreCase` in the schema.

**Rationale**: Both Rust structs already carry `#[cynic(graphql_type = "StringComparator")]`; adding optional fields with `#[cynic(skip_serializing_if = "Option::is_none")]` is the idiomatic cynic pattern and requires no new types.

**Alternatives considered**: A new dedicated struct per use-case — rejected because it creates duplicate schema mappings that cynic disallows.

## Project Query Filter Threading

**Decision**: Modify `ProjectsVariables` and `TeamProjectsVariables` to include `pub filter: Option<ProjectFilter>`, and update the corresponding `#[arguments]` annotations to pass `filter: $filter`.

**Rationale**: cynic's `QueryVariables` derive supports `Option` fields; a `None` filter is serialized as a missing key (not sent to the API), which is identical in behavior to the unfiltered queries today.

**Alternatives considered**:
- Separate filtered/unfiltered query structs: adds dead code and combinatorial query variants — rejected.
- Client-side filtering without API filter: insufficient for performance goals — rejected.

## Empty-String Handling

**Decision**: In `build_issue_filter` (issues) and the project fetch helper, treat `Some("")` as `None` before constructing the filter. This satisfies FR-009 and the edge-case specification without special-casing at the CLI layer.

**Rationale**: Normalizing at the lowest possible layer ensures every code path (including programmatic API usage) behaves consistently.

## Project Repository Signature

**Decision**: Add `name_contains: Option<String>` as a new parameter to `ProjectRepository::list()`.

**Rationale**: The project side has no `ListProjectsInput` struct (unlike issues). Rather than introducing a new struct (which would be a larger change than the feature warrants), threading a single nullable string through all callers is minimal and explicit.

**Alternatives considered**: Introduce `ListProjectsInput` struct — deferred; worthwhile if a second project filter is added later, but premature now.
