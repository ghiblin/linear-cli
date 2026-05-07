# CLI Contract: Delete Issue

## Command Signature

```
linear issue delete <id> [OPTIONS]

Arguments:
  <id>    Issue ID (UUID or display identifier, e.g. ENG-42)

Options:
  --dry-run           Preview the deletion without making API calls
  --json              Output result as JSON (alias for --output json)
  --output <format>   Output format: "json" | "text" (default: text when TTY, json when piped)
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User/input error (invalid ID, missing args) |
| 2 | API/network error |
| 3 | Auth error |

## Human Output (TTY)

**Success:**
```
Deleted issue ENG-42
```

**Dry-run:**
```
[dry-run] Would delete issue: <id>
```

## JSON Output

**Success:**
```json
{ "deleted": true, "id": "<id>" }
```

**Dry-run:**
```json
{ "dry_run": true, "id": "<id>" }
```

## Error Output (stderr)

All errors are written to stderr. stdout contains only data output on success.

```
Error: issue not found: <id>
Error: permission denied
```
