# Data Model: Issue Get — Optional Detail Flags

No new domain entities or value objects are introduced by this feature. The existing `Issue` aggregate already carries all required data.

## Relevant Existing Entities

### Issue (`src/domain/entities/issue.rs`)

| Field | Type | Notes |
|-------|------|-------|
| `id` | `IssueId` | Unique identifier |
| `identifier` | `String` | Display ID, e.g. `ENG-42` |
| `title` | `String` | Issue title |
| `description` | `Option<String>` | Markdown body; absent when not set — shown with `--description` |
| `state` | `WorkflowStateRef` | Workflow state (id, name, type) |
| `priority` | `Priority` | Enum: NoPriority / Urgent / High / Medium / Low |
| `assignee_name` | `Option<String>` | Display name |
| `sub_issues` | `Vec<SubIssueRef>` | Child issues — shown with `--subtasks` |
| `parent_id` | `Option<IssueId>` | Parent issue ID if any |
| `parent_title` | `Option<String>` | Parent issue title if any |
| `due_date` | `Option<String>` | ISO date string |
| `estimate` | `Option<f64>` | Story points |

### SubIssueRef (`src/domain/entities/issue.rs`)

| Field | Type | Notes |
|-------|------|-------|
| `id` | `IssueId` | Child issue ID |
| `identifier` | `String` | Display ID, e.g. `ENG-43` |
| `title` | `String` | Child issue title |

## Data Flow

```
Linear API
  └─ IssueDetailNode (children + description already fetched)
       └─ LinearIssueRepository::map
            └─ Issue domain entity
                 └─ GetIssue use case
                      └─ IssueSubcommand::Get handler
                           ├─ format_issue_human(issue, show_description, show_subtasks)
                           └─ IssueDto (JSON — unchanged)
```

No new mappings, queries, or persistence changes are required.
