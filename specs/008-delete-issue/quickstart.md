# Quickstart: Delete Issue

## Delete an issue

```bash
linear issue delete ENG-42
# Deleted issue ENG-42
```

## Preview before deleting

```bash
linear issue delete ENG-42 --dry-run
# [dry-run] Would delete issue: ENG-42
```

## JSON output (for scripting)

```bash
linear issue delete ENG-42 --json
# {"deleted":true,"id":"ENG-42"}
```

## Pipe-friendly usage

```bash
# Delete and check result in a script
linear issue delete ENG-42 --json | jq '.deleted'
```
