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
| Symlink swap or target deletion | Frozen canonical plan target, `symlink_metadata`, no target following, immediate revalidation |
| Protected content matched by broad rule | Protected-root deny policy plus exact backend rule allowlist |
| Generic `build` directory misclassified | User-configured roots plus manifest-gated project context; review-only result |
| Stale or replayed cleanup | Expiring immutable plan and single-use confirmation token |
| File changes after scan | Type, size, modification time, root, and rule revalidation; skip on mismatch |
| Same-size but different files | Sparse hash filter followed by full BLAKE3; optional byte-for-byte verification |
| Duplicate plan removes every copy | Backend validates keep membership and requires Trash count below copy count |
| Keep or duplicate changes after review | Re-hash keep and target, recheck metadata/canonical paths, then skip on mismatch |
| Permanent deletion accidentally enabled | Persisted feature flag defaults off; enabling does not create or execute a plan |
| Destructive action confused with Trash | Separate red action, immutable action field, native confirmation, detailed history |
| Expert confirmation bypass | Backend-issued exact phrase validated before the plan is consumed |
| Destructive transient failure | One attempt only; record per-item failure and do not retry |
| Partial permission or trash failure | Per-item result, continue safely, structured recoverable error |
| Data exfiltration | No network service, strict CSP, no remote scripts, local-only persistence |
| Shell injection | No shell capability; future controlled commands use executable + fixed argument arrays |
| UI/main-thread denial of service | Bounded workers, throttled events, cancellation, paged findings |
| Settings corruption | Schema version, strict deserialization, validation, atomic replacement |

## Current attack surface

The capability file grants Tauri core defaults plus native directory-open and confirmation dialogs. It does not grant shell or frontend filesystem mutation. Custom cleanup IPC accepts backend-owned IDs for planning and a backend-issued plan ID plus token for execution. Trash remains the default; the permanent executor is backend-only and feature-gated.

## Security gates for later phases

Careful/expert findings are visible but not executable. Duplicate cleanup has a dedicated content and keep-one gate. Execution of guided commands and permanent deletion still require later dedicated gates.
