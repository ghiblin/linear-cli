# Quickstart: Issue Get Detail Flags

## Show an issue's description

```bash
linear issue get ENG-42 --description
```

Output:
```
Identifier:   ENG-42
Title:        Implement login
State:        In Progress
Priority:     high
Description:
  Implement the login flow using the existing auth service.
  Supports email/password only in v1.
```

If the issue has no description, the `Description:` block is omitted and the command exits cleanly.

## Show an issue's subtasks

```bash
linear issue get ENG-42 --subtasks
```

Output:
```
Identifier: ENG-42
Title:      Implement login
State:      In Progress
Priority:   high
Sub-issues:
  ENG-43 — Add login form
  ENG-44 — Wire up auth service
```

If the issue has no sub-issues, the `Sub-issues:` block is omitted and the command exits cleanly.

## Show both at once

```bash
linear issue get ENG-42 --description --subtasks
```

## Machine-readable output

JSON output is unchanged by these flags — description and sub_issues are always included:

```bash
linear issue get ENG-42 --json
```
