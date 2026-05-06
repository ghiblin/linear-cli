# Feature Specification: JSON Output Shorthand Flag

**Feature Branch**: `006-add-json-flag`  
**Created**: 2026-05-06  
**Status**: Draft  
**Input**: User description: "I tested the CLI application in Claude Code, and a lot of time in requests a `--output json` flag. So the CLI application should support the double flags: `--output json` and `--json`."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use `--json` Shorthand for JSON Output (Priority: P1)

A user (or AI agent) wants to receive command output in JSON format and uses `--json` as a concise alternative to `--output json`. The shorter flag reduces friction when integrating the CLI with scripts, pipelines, or AI tools that frequently request structured output.

**Why this priority**: This is the core of the feature. The existing `--output json` flag already works, but the `--json` shorthand reduces typing and matches conventions from other popular CLI tools. AI agents like Claude Code consistently request `--output json`, and a dedicated flag lowers the chance of flag misuse.

**Independent Test**: Can be fully tested by running any command with `--json` and verifying the output is valid JSON — identical to the output of the same command with `--output json`.

**Acceptance Scenarios**:

1. **Given** a user runs any supported command with the `--json` flag, **When** the command executes successfully, **Then** the output is formatted as valid JSON, identical to what `--output json` would produce.
2. **Given** a user runs a command with `--json`, **When** the command encounters an error, **Then** the error is also reported in JSON format (consistent with `--output json` error behavior).
3. **Given** a user runs `--help` on any command that supports output formatting, **When** they view the help text, **Then** both `--output json` and `--json` are listed as options for JSON output.

---

### User Story 2 - Interoperability: Both Flags Produce Identical Output (Priority: P2)

A user who has scripts or workflows using `--output json` and another using `--json` can trust that both flags are perfectly interchangeable — switching between them requires no other changes.

**Why this priority**: Consistency is critical for automation trust. If the two flags produce subtly different output, downstream tools break silently.

**Independent Test**: Run the same command with `--output json` and with `--json`, then compare output byte-for-byte to confirm they are identical.

**Acceptance Scenarios**:

1. **Given** a command is run with `--output json`, **When** the same command is run with `--json` under identical conditions, **Then** both outputs are structurally and content-wise identical.
2. **Given** a user provides both `--output json` and `--json` simultaneously, **When** the command runs, **Then** the system produces JSON output without error (no conflict or double-application).

---

### Edge Cases

- What happens when `--json` is combined with a conflicting `--output` value (e.g., `--json --output table`)? The system should apply a clear precedence rule or report an error.
- What happens when a command does not support the `--output` flag at all? The `--json` flag should behave consistently with how `--output json` would be handled in that context.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST accept `--json` as a valid flag on any command that already supports `--output json`.
- **FR-002**: The output produced by `--json` MUST be identical to the output produced by `--output json` for the same command and inputs.
- **FR-003**: When both `--json` and `--output <value>` are provided, the CLI MUST either apply a defined precedence rule (last flag wins) or report a clear, user-friendly conflict error — the behavior MUST be documented.
- **FR-004**: The `--json` flag MUST appear in the help text of every command that supports it, alongside the existing `--output` flag description.
- **FR-005**: Error output when using `--json` MUST also be in JSON format, consistent with `--output json` error behavior.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of commands that support `--output json` also accept `--json` and produce identical output.
- **SC-002**: Users can switch from `--output json` to `--json` in any existing script without any change in output format or content.
- **SC-003**: Help text for all supported commands lists `--json` as an available flag within one screen of output.
- **SC-004**: Zero regressions — all commands that previously worked with `--output json` continue to work correctly after the change.

## Assumptions

- All commands that currently support `--output json` will also support `--json`; no command-specific exclusions are in scope.
- The `--json` flag is a strict shorthand alias — it adds no additional behavior beyond enabling JSON output.
- Commands that do not currently support `--output` at all are out of scope for this feature.
- When both `--json` and `--output <value>` are supplied, last-flag-wins is the default resolution strategy unless project conventions dictate otherwise.
