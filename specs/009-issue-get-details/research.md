# Research: Issue Get — Optional Detail Flags

## Finding 1: No GraphQL Changes Required

**Decision**: Keep the existing `IssueDetailNode` fragment unchanged.

**Rationale**: `IssueDetailNode` in `src/infrastructure/graphql/queries/issue_queries.rs` already includes both `description: Option<String>` and `children: SubIssueConnection`. The `fetch_issue` function already fetches and returns both fields. There is no need to modify the query or add query variables to conditionally omit fields.

**Alternatives considered**: Removing these fields from the query when flags are absent to reduce payload size. Rejected — the payload difference is negligible for a single-issue fetch, and adding query variable complexity for display preferences would couple the presentation layer to the API layer.

---

## Finding 2: Scope Is Confined to `src/cli/commands/issue.rs`

**Decision**: Only `src/cli/commands/issue.rs` needs to change.

**Rationale**: The `Issue` domain entity already has `pub description: Option<String>` and `pub sub_issues: Vec<SubIssueRef>`. The application use case (`GetIssue`) already returns the full `Issue`. The infrastructure repository already maps `children` → `sub_issues`. The only gap is the CLI presentation function `format_issue_human` which omits description entirely and shows sub-issues unconditionally.

**Alternatives considered**: Adding a separate "detail" use case or a display-options struct passed through application/domain layers. Rejected — these are purely presentation concerns. The constitution (Principle I) requires the domain and application layers to be free of presentation concerns.

---

## Finding 3: Sub-Issues Currently Shown Unconditionally

**Decision**: The `--subtasks` flag makes sub-issues opt-in; the current unconditional display is removed.

**Rationale**: `format_issue_human` currently prints sub-issues whenever `issue.sub_issues` is non-empty, without any flag. Per FR-004 in the spec, the default output must not include the sub-issue list. Removing the unconditional display and gating it behind `--subtasks` is the correct change. This is not a breaking change to JSON output (unchanged) and aligns with the goal of keeping default output concise.

**Alternatives considered**: Keeping the unconditional display and adding `--subtasks` only as an alias. Rejected — the spec is explicit that the flag must be the only trigger.

---

## Finding 4: Flag Implementation Pattern

**Decision**: Extend `IssueSubcommand::Get` with `description: bool` and `subtasks: bool` clap fields; pass them as parameters to `format_issue_human`.

**Rationale**: All existing boolean flags in the codebase (e.g., `all: bool`, `dry_run: bool`, `json: bool`) use the same `#[arg(long)] foo: bool` pattern. Consistency with this pattern is correct. `format_issue_human` currently takes only `&Issue`; adding `show_description: bool` and `show_subtasks: bool` parameters is the minimal change.

**Alternatives considered**: A `DisplayOptions` struct. Acceptable but over-engineered for two booleans. Rejected in favour of direct parameters matching the codebase style.

---

## Finding 5: No New Tests Infrastructure Needed

**Decision**: Unit tests live inside `src/cli/commands/issue.rs` in a `#[cfg(test)]` module, following the constitution's co-location rule.

**Rationale**: The feature touches only one function (`format_issue_human`). The existing test module in `issue.rs` is the right place. Tests can use captured stdout (via `assert!` on printed output using `format_issue_human` indirectly) or extract the display logic into a helper that returns a `String` to make it directly testable without I/O capture.

**Alternatives considered**: `insta` snapshot tests for the full output. Acceptable but heavier than needed for two simple conditions. Direct `String`-returning helper + `assert_eq!` is sufficient.
