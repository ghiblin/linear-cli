# Quickstart: JSON Output Shorthand Flag

**Feature**: 006-add-json-flag

## What Changed

Both `--output json` and `--json` now work on all commands that produce structured output. They are fully interchangeable.

## Usage Examples

### Using `--json` (new shorthand)

```bash
# List issues as JSON
linear issue list --json

# Get a specific project as JSON
linear project get PROJ-123 --json

# Create an issue and get JSON output (useful for scripting)
linear issue create --title "Fix login bug" --team ENG --json

# List teams as JSON
linear team list --json
```

### Using `--output json` (existing syntax, unchanged)

```bash
linear issue list --output json
linear project get PROJ-123 --output json
```

Both produce **identical output**.

## In Scripts and Pipelines

```bash
# Get issue ID from JSON output
linear issue list --json | jq '.[0].id'

# Create and capture the new issue ID
NEW_ID=$(linear issue create --title "Deploy fix" --team ENG --json | jq -r '.id')

# Pipe project list into another tool
linear project list --json | jq '.projects[] | select(.status == "active")'
```

## Precedence

If you supply both flags, `--json` wins:

```bash
# This produces JSON (--json takes precedence)
linear issue list --json --output human
```

## Default Behavior (unchanged)

- **Piped output** (non-TTY): JSON by default, no flag needed
- **Terminal output**: Human-readable by default unless `--json` or `--output json` is set
