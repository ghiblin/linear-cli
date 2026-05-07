# CLI Contract: Filter by Partial Title

## New Flags

### `linear issue list --title <SUBSTRING>`

| Property | Value |
|----------|-------|
| Flag name | `--title` |
| Value type | `String` |
| Required | No (optional) |
| Default | None (no title filter applied) |
| Composition | Composes with all existing flags (`--team`, `--state`, `--assignee`, `--priority`, `--label`, `--all`, `--limit`, `--cursor`) |

**Behavior**:
- Omitted or empty string → no filter, all results returned
- Non-empty string → only issues whose `title` contains the substring (case-insensitive) are returned
- Exit code 0 with zero items when no match found (not an error)
- `--json` output schema identical to unfiltered list (no new fields in output)

**Examples**:
```
linear issue list --title "login"
linear issue list --title "LOGIN"   # same results as above
linear issue list --title "auth" --state "in_progress" --team <team-id>
linear issue list --title "xyznonexistent"   # → empty result, exit 0
```

---

### `linear project list --name <SUBSTRING>`

| Property | Value |
|----------|-------|
| Flag name | `--name` |
| Value type | `String` |
| Required | No (optional) |
| Default | None (no name filter applied) |
| Composition | Composes with `--team`, `--all`, `--limit`, `--cursor` |

**Behavior**:
- Omitted or empty string → no filter
- Non-empty string → only projects whose `name` contains the substring (case-insensitive)
- Exit code 0 with zero items when no match found
- `--json` output schema unchanged

**Examples**:
```
linear project list --name "Platform"
linear project list --name "platform"   # same results
linear project list --name "Platform" --team <team-id>
```

## JSON Output Schema (unchanged)

The output of `issue list --json` and `project list --json` follows the same schema regardless of whether `--title` / `--name` is supplied. The filter flag affects which items are returned, not the shape of the output.

Issue list item schema (unchanged):
```json
{
  "id": "string",
  "identifier": "string",
  "title": "string",
  "state": { "id": "string", "name": "string", "state_type": "string" },
  "priority": "number",
  "team_id": "string",
  "assignee_id": "string|null",
  "assignee_name": "string|null",
  "label_ids": ["string"],
  "created_at": "string",
  "updated_at": "string"
}
```

Project list item schema (unchanged): same as current `project list --json` output.
