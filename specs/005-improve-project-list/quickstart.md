# Quickstart: Improve Project List Identifiers

**Branch**: `005-improve-project-list`

## What changes

All changes are confined to a single file: `src/cli/commands/project.rs`.

No new files, no new modules, no new dependencies.

## Changes summary

| Location | Change |
|----------|--------|
| `ProjectDto` struct | Add `slug_id: String` field |
| `From<&Project> for ProjectDto` | Map `p.slug_id.clone()` |
| `MutationResultDto` struct | Add `slug_id: String` field |
| `project list` human output | Add slug column (`{:<22}`) between name and state; reduce name column from `{:<40}` to `{:<35}` |
| `project get` human output | Add `Slug: {slug_id}` line after `Name:` |
| `project create` human output | Change `(uuid)` → `(slug)` in success message |
| `project create` JSON output | `MutationResultDto` now serializes `slug_id` |
| `project update` human output | Change `project {uuid}:` → `project {slug}:` in success message |
| `project update` JSON output | `MutationResultDto` now serializes `slug_id` |

## TDD order

Per Constitution Principle II, write tests first:

1. **Unit test**: `ProjectDto::from(&project)` — assert `slug_id` field populated correctly.
2. **Unit test**: `MutationResultDto` with `slug_id` — assert JSON serialization includes field.
3. **Snapshot test** (insta): `project list` human output — assert slug column present and correctly formatted.
4. **Snapshot test** (insta): `project get` human output — assert `Slug:` line present at correct position.
5. **Snapshot test** (insta): `project create` human success message — assert slug in parentheses.
6. **Snapshot test** (insta): `project update` human success message — assert slug used.

Then implement until all tests pass.

## Verify

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```
