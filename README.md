# DiskSage

DiskSage is a local-first macOS and Linux desktop application for understanding disk usage and reviewing safe cleanup recommendations. Safety takes precedence over maximum cleanup.

This repository currently implements Phase 0 through Phase 5's duplicate analysis and trash-first cleanup workflow:

- Tauri 2 + Rust desktop foundation
- React 18, TypeScript, Vite, TailwindCSS, shadcn-compatible component setup, routing, and Zustand
- Read-only mounted-disk information using `sysinfo`
- Validated, atomic JSON settings persistence in the OS application-data directory
- Structured IPC errors and structured Rust logging
- Versioned domain contracts for rules, findings, scans, and immutable cleanup plans
- Protected-path policy and its first mandatory safety tests
- Least-privilege Tauri capability and strict Content Security Policy
- Single-heavy-scan coordinator with cancellation tokens and throttled progress events
- Bounded, symlink-safe, same-filesystem traversal with exclusions and partial-error reporting
- Ten versioned safe cache rules for package managers and browsers
- Incremental flat findings persisted as local NDJSON with paginated IPC reads
- Scan and Findings pages with live progress, cancellation, filtering, and backend-authorized reveal
- Expiring, single-use cleanup plans resolved exclusively from persisted backend finding IDs
- Canonical-path, rule, type, size, modification-time, and symlink revalidation immediately before cleanup
- Independent Move to Trash operations with cancellation, partial-failure reporting, and disk refresh
- Local NDJSON cleanup history with per-item outcomes and a dedicated History page
- Thirty-nine versioned rules spanning package/browser caches, IDEs, project artifacts, Docker, and emulators
- User-configurable project roots with manifest-gated detection and bounded discovery depth
- Careful and expert developer findings that remain review-only and unselected by default
- Docker virtual disk and emulator inspection with guided actions; no direct Docker or emulator deletion
- Native multi-folder picker for explicitly scoped duplicate scans
- Staged duplicate detection by size, sparse BLAKE3 sampling, full BLAKE3, and optional byte verification
- Cancellable hashing progress with local duplicate-group persistence and deterministic keep recommendations
- Duplicate cleanup plans that re-hash every path and enforce at least one surviving copy per group

Permanent deletion remains intentionally unavailable. Cleanup moves only backend-authorized safe caches or content-verified duplicate copies to the operating system Trash; protected user-content roots remain blocked.

## Development

Prerequisites:

- Node.js 20.19+ or 22.12+
- Rust 1.88+
- macOS 10.15+, or the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/)

```sh
npm install
npm run tauri dev
```

Useful checks:

```sh
npm run lint
npm run typecheck
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

On Ubuntu/Debian, install Tauri's WebKitGTK dependencies first:

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Architecture and safety

- [Architecture](docs/architecture.md)
- [Safety policy](docs/safety-policy.md)
- [Threat model](docs/threat-model.md)
- [IPC contracts](docs/ipc-contracts.md)
- [Test strategy](docs/test-strategy.md)

All filesystem data stays local. Logs must not include file contents and should avoid complete sensitive paths.
