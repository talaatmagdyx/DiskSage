# Test strategy

## Phase 0–1 gates

- Protected path matching covers root, home, sensitive descendants, platform roots, and traversal normalization.
- Settings tests cover safe defaults, strict validation, atomic round-trip, and corruption preservation.
- Disk metadata tests cover invalid input and command error shape.
- Frontend tests cover disk loading/error state, error normalization, and settings/store transitions.
- Scanner tests cover exclusion-before-descent, relative exclusion rejection, cancellation, symlink skipping, flat persistence paging, identifier traversal rejection, rule catalog quality, and profile availability.
- CI runs formatting, lint, type checks, unit tests, Clippy with warnings denied, a Tauri no-bundle build, dependency audit, and license/source policy.

Tests use temporary directories and never inspect or modify the developer's real files.

## Mandatory gate before Phase 2

Add fixtures for nested trees, permission denial, symlink loops, broken links, sparse files, mutation during scan, Unicode and long paths, hidden files, and large file counts. Prove bounded concurrency, throttled progress, partial results, and cancellation response.

## Phase 3 safety gate

Tests demonstrate that cleanup cannot target a filesystem root, home, protected path, item outside a backend plan, expired or reused plan, symlink target, changed item, parent-symlink redirect, or unsupported duplicate finding. Permanent deletion is rejected before any plan is created. Store tests prove execution does not begin until a backend plan is reviewed and that partial outcomes remain visible.

## Phase 4 rule gate

Catalog tests enforce unique, versioned rule IDs, at least 25 definitions, relative home targets, and disabled defaults for every careful/expert rule. Project fixtures prove artifacts are absent without manifest context and detected only after a supported project indicator exists. Discovery fixtures prove known artifact trees are not recursively treated as nested projects.

## Phase 5 duplicate gate

Hashing fixtures prove same-size non-identical files never form a group, identical files do, optional byte verification detects differences, and a cancelled token interrupts hashing. Persistence tests cover paged groups and traversal-resistant IDs. Cleanup tests prove the backend rejects any selection that includes the keep copy or would Trash every copy, while valid plans freeze one survivor. Frontend store tests prove automatic keep selection, keep reassignment, and rejection of attempts to toggle the keep copy into Trash.

## Phase 6 destructive and analysis gate

Permanent-executor tests cover regular deletion and symlink refusal. Cleanup tests prove the feature flag blocks plan creation by default, enabled plans preserve the permanent action, and expert plans reject missing or mismatched typed phrases. Custom-analysis fixtures prove large files are Careful review-only findings and hidden/general old files are excluded. Frontend tests prove native confirmation precedes permanent execution and Custom Scan forwards explicit bounded options.
