# Quickstart: Project CRUD Operations

**Branch**: `004-project-crud` | **Date**: 2026-04-23

## Prerequisites

1. Authenticated Linear CLI (`linear auth login` or `LINEAR_API_KEY` env var set)
2. At least one Linear team ID (obtainable from `linear team list` or Linear's UI)

---

## List Projects

```bash
# Default: first 50 projects, sorted by updatedAt desc
linear project list

# Filter by team
linear project list --team <team-uuid>

# JSON output (pipe-friendly)
linear project list --output json

# Fetch all pages automatically
linear project list --all

# Manual pagination
linear project list --limit 10
linear project list --limit 10 --cursor <cursor-from-previous-response>
```

---

## View a Single Project

```bash
# By display ID
linear project get PRJ-1

# By UUID
linear project get 9cfb482a-81e3-4154-b5b9-2c805e70a02d

# JSON output
linear project get PRJ-1 --output json
```

---

## Create a Project

```bash
# Minimal: name + team
linear project create --name "Q3 Platform" --team <team-uuid>

# Full options
linear project create \
  --name "Q3 Platform" \
  --team <team-uuid> \
  --description "Platform modernisation for Q3" \
  --lead <user-uuid> \
  --start-date 2026-07-01 \
  --target-date 2026-09-30

# Preview without creating
linear project create --name "Test" --team <team-uuid> --dry-run

# JSON output (useful for piping the new ID)
linear project create --name "Q3 Platform" --team <team-uuid> --output json
```

---

## Update a Project

```bash
# Change state
linear project update PRJ-1 --state started

# Rename
linear project update PRJ-1 --name "Q3 Platform (revised)"

# Set a new target date
linear project update PRJ-1 --target-date 2026-10-31

# Multiple fields at once
linear project update PRJ-1 --state paused --description "On hold until Q4"

# Preview without applying
linear project update PRJ-1 --state completed --dry-run
```

Valid `--state` values: `planned`, `started`, `paused`, `completed`, `cancelled`

---

## Archive a Project

```bash
# Archive by display ID
linear project archive PRJ-1

# Archive by UUID
linear project archive 9cfb482a-81e3-4154-b5b9-2c805e70a02d

# Preview without archiving
linear project archive PRJ-1 --dry-run
```

---

## Troubleshooting

```bash
# Show step-by-step progress (stderr)
linear project list --verbose

# Show raw GraphQL request + response (stderr) — implies --verbose
linear project list --debug

# Check authentication
linear auth status
```

---

## Piping Examples (AI Agent Patterns)

```bash
# Get project ID for scripting
PROJECT_ID=$(linear project create --name "Sprint X" --team <team> --output json | jq -r .id)

# List project IDs as newline-separated list
linear project list --output json | jq -r '.projects[].id'

# Check project state before updating
linear project get PRJ-1 --output json | jq .state
```
