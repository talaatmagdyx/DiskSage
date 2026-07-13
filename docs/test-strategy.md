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

## Mandatory gate before Phase 3

Demonstrate that the executor cannot delete a filesystem root, home, protected path, item outside a backend plan, expired or reused plan, symlink target, changed item, or every member of a duplicate group. Permanent deletion must remain absent until its separate Phase 6 gate.
