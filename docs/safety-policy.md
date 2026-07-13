# Safety policy

## Non-negotiable invariants

1. No filesystem item is changed immediately after a scan.
2. Frontend-provided paths never authorize cleanup.
3. Trash is the default action; permanent deletion is disabled by default.
4. Symlinks are not followed. A supported symlink operation may affect only the link object.
5. Filesystem roots, home directories, system locations, credentials, and user-content roots are protected.
6. A protected descendant is cleanable only through an exact child allowlist owned by a versioned, tested backend rule.
7. Every plan item is revalidated immediately before execution; a changed or unverifiable item is skipped.
8. Partial failure is reported per item and never converted to success.
9. Duplicate cleanup can never select the keep copy and can never remove every copy in a group.

## Protected roots

Common policy includes `/`, the current home directory, `.ssh`, `.gnupg`, `.aws`, Kubernetes credentials, Documents, Desktop, Pictures, Movies, and Music.

macOS adds `/System`, `/Applications`, critical Unix roots, `~/Library/Keychains`, Mail, Messages, and `/private/var/db`.

Linux adds critical Unix roots plus `/proc`, `/sys`, `/dev`, `/run`, `/root`, and `/lost+found`.

The Rust policy performs lexical normalization before comparison so `..` cannot bypass a protected prefix. Phase 3 additionally freezes the canonical target in the plan and uses symlink metadata immediately before execution, preventing a parent-symlink redirect from silently changing the approved target.

## Cleanup authorization

```text
selected finding IDs
  -> load backend findings
  -> enforce rule/risk policy
  -> construct immutable expiring plan
  -> show exact plan to user
  -> receive plan ID + confirmation token
  -> load server-side plan
  -> revalidate root, type, symlink metadata, size, mtime, and rule
  -> independently trash eligible items
```

The cleanup executor never accepts `Vec<String>` paths from IPC. Plans expire after 15 minutes and are consumed once. Permanent deletion remains rejected by the backend and will require a separate future security gate.

Duplicate scan roots are the deliberate exception for read-only analysis: the native picker returns absolute folders, which the backend canonicalizes, narrows, and rejects when they are filesystem, home, credential, symlink, or system roots. These paths do not authorize cleanup. Duplicate cleanup resolves paths only from persisted group and copy IDs, freezes both keep and Trash paths, and verifies size, modification time, canonical resolution, and full BLAKE3 content before every move. Protected Documents, Desktop, Pictures, Movies, and Music locations remain analysis-only.

## Risk defaults

- Safe: may become preselectable through an explicit setting after rules have tests.
- Careful: never selected by default; Phase 4 project artifacts and IDE caches are review-only.
- Expert: never selected by default; Docker virtual disks and emulator state expose guided inspection only.

Project artifact names are never trusted outside project context. A `build`, `target`, or `logs` directory is considered only under a user-configured root after a matching manifest such as `package.json`, `Cargo.toml`, `pyproject.toml`, or `pom.xml` is found. Discovery skips symlinks and known artifact trees.
