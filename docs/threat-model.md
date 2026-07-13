# Threat model

## Assets

- User-created files and directories
- Credentials, configuration, keychains, and local databases
- Integrity of scan findings and cleanup history
- Privacy of paths, metadata, hashes, and diagnostic logs
- UI availability during large disk operations

## Adversaries and failures

The model covers a compromised or buggy frontend, malicious path input, traversal, symlink races, mount changes, stale plans, files changed after scanning, permission failures, corrupt local settings, unbounded resource use, and accidental activation of destructive features. It does not assume elevated/root execution.

## Controls

| Threat | Required control |
| --- | --- |
| Frontend submits arbitrary path | IPC accepts finding IDs or backend-issued plan IDs, never cleanup paths |
| Path traversal | Absolute-path validation, lexical normalization, canonicalization where possible |
| Symlink swap or target deletion | `symlink_metadata`, no target following, immediate revalidation |
| Protected content matched by broad rule | Protected-root deny policy plus exact backend rule allowlist |
| Stale or replayed cleanup | Expiring immutable plan and single-use confirmation token |
| File changes after scan | Type, size, modification time, root, and rule revalidation; skip on mismatch |
| Partial permission or trash failure | Per-item result, continue safely, structured recoverable error |
| Data exfiltration | No network service, strict CSP, no remote scripts, local-only persistence |
| Shell injection | No shell capability; future controlled commands use executable + fixed argument arrays |
| UI/main-thread denial of service | Bounded workers, throttled events, cancellation, paged findings |
| Settings corruption | Schema version, strict deserialization, validation, atomic replacement |

## Current attack surface

The capability file grants only Tauri core defaults. No dialog, shell, broad filesystem, opener, or destructive custom command is exposed. `get_disk_info` only accepts a mount path and returns it only when it matches the operating system's current mounted-disk list.

## Security gates for later phases

Scanner work requires cancellation, boundary/exclusion tests, and bounded-memory evidence. Cleanup work requires all mandatory protected-path, symlink, stale-plan, changed-item, duplicate-retention, and permanent-confirmation tests before any executor is registered.

