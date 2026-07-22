<p align="center">
  <img src="src-tauri/icons/128x128.png" width="96" height="96" alt="DiskSage app icon">
</p>

<h1 align="center">DiskSage</h1>

<p align="center">
  <strong>Understand what is using your disk. Remove only what you reviewed.</strong>
</p>

<p align="center">
  A local-first disk intelligence and cleanup app for macOS and Linux,<br>
  built around explicit review, revalidation, and Trash-first recovery.
</p>

<p align="center">
  <img alt="macOS 10.15+" src="https://img.shields.io/badge/macOS-10.15%2B-111827?logo=apple&logoColor=white">
  <img alt="Linux" src="https://img.shields.io/badge/Linux-Ubuntu%2022.04%2B-111827?logo=linux&logoColor=white">
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8D8?logo=tauri&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.88-000000?logo=rust&logoColor=white">
  <img alt="React and TypeScript" src="https://img.shields.io/badge/React%20%2B%20TypeScript-18-3178C6?logo=react&logoColor=white">
  <a href="LICENSE"><img alt="MIT license" src="https://img.shields.io/badge/License-MIT-2EA44F"></a>
</p>

![DiskSage storage overview](docs/screenshots/overview.png)

## Why DiskSage

Most disk cleaners optimize for the largest possible number. DiskSage optimizes for a trustworthy decision.

- **Local by design** — filenames, paths, hashes, scan results, and file contents stay on the device.
- **Evidence before action** — every finding explains its source, boundary, safety tier, and estimated size.
- **Trash first** — eligible cleanup moves reviewed items to the operating-system Trash; permanent deletion is feature-gated and off by default.
- **Immutable plans** — DiskSage creates a cleanup plan, shows exactly what it contains, and revalidates each path immediately before execution.
- **No silent automation** — no background cleanup daemon, cloud account, telemetry, or scheduled deletion.

## Product capabilities

| Capability | What it provides |
| --- | --- |
| **Targeted scans** | Quick, Developer, Full Analysis, and Custom scan profiles with progress, cancellation, exclusions, and persisted results. |
| **Developer intelligence** | Rules for IDEs, package managers, browser tooling, Xcode, containers, VM disks, local models, build artifacts, and regenerable caches. |
| **Safety-ranked findings** | Safe, Careful, and Expert tiers keep low-risk cache cleanup separate from app data, sparse virtual disks, models, and owner-managed tools. |
| **Duplicate verification** | Folder-scoped discovery with staged metadata checks, partial BLAKE3 hashing, full hashing, and optional byte comparison. |
| **Application inventory** | Sort and filter installed apps by name, size, location, or last-used date; preview app-only or complete-uninstall plans before moving anything. |
| **Storage Map** | Read-only comparison of logical and allocated folder sizes without turning every large folder into a cleanup recommendation. |
| **Permission Center** | Read-only visibility checks, Full Disk Access guidance, retries, and clear reporting when protected containers make results partial. |
| **Local history** | Schema-versioned scan, finding, duplicate, and cleanup records stored locally with bounded retention. |

## Safety model

DiskSage does not treat “large” as a synonym for “safe to delete.” Rules declare their cleanup boundary and are restricted by tier.

| Tier | Typical content | Allowed action |
| --- | --- | --- |
| **Safe** | Regenerable caches and downloaded artifacts with narrow boundaries | Can be selected for a reviewed Trash plan; every path is revalidated before execution. |
| **Careful** | IDE state, app support data, archives, runtimes, and other potentially disruptive content | Requires explicit selection and a typed confirmation phrase; Trash only. |
| **Expert** | Docker/VM disks, local models, active runtime bundles, and owner-managed stores | Review-only or guided owner command; DiskSage does not remove the underlying asset directly. |

![DiskSage findings with Safe, Careful, and Expert tiers](docs/screenshots/findings.png)

For example, DiskSage may explain both the allocated and virtual capacity of `Docker.raw`, then guide you to Docker's own maintenance tools. It never removes that virtual disk directly.

## A cleanup is a transaction, not a button

```text
Scan  →  Review evidence  →  Build an immutable plan  →  Revalidate  →  Move to Trash  →  Record the result
```

Before an item moves, the Rust backend checks that it still matches the reviewed record, remains inside the allowed root, has not become a symlink, and has not crossed a protected boundary. Changed or inaccessible items fail closed and remain on disk.

## Application intelligence

DiskSage inventories application bundles separately from their support data. On macOS, it can display last-used metadata, exclude system apps, detect running targets, and build two distinct plans:

- **App only** moves the selected `.app` bundle.
- **Complete uninstall** adds positively attributed containers, preferences, caches, saved state, logs, and other reviewed leftovers.

Documents, projects, and shared Group Containers remain excluded unless ownership is unambiguous and the user explicitly includes them. Partial permission failures are reported per item and can be retried after granting access.

![DiskSage application inventory and Permission Center](docs/screenshots/applications.png)

## Explain usage without recommending deletion

Storage Map is deliberately read-only. It compares logical size with blocks allocated on disk, stays inside the selected filesystem boundary, and does not follow symlinks. This distinction matters for sparse files, compressed files, APFS snapshots, copy-on-write data, and virtual disks.

![DiskSage read-only Storage Map](docs/screenshots/storage-map.png)

## Privacy from the first launch

DiskSage performs scanning, rule evaluation, duplicate hashing, and cleanup locally. Diagnostic exports contain aggregate counts and redacted configuration—not filenames, full paths, file contents, or raw scan results.

![DiskSage privacy onboarding](docs/screenshots/onboarding.png)

Read the complete [privacy policy](PRIVACY.md), [threat model](docs/threat-model.md), and [security policy](SECURITY.md).

## Install

DiskSage is currently a pre-release project. Release artifacts target:

- macOS 10.15 or later: Apple silicon and Intel DMGs
- Ubuntu 22.04 or a compatible modern Linux distribution: AppImage and Debian package

Public macOS release artifacts are expected to be Developer ID signed, notarized, and stapled. Development builds are unsigned. See [INSTALL.md](INSTALL.md) for installation, upgrade, uninstall, and source-build details.

## Build from source

### Prerequisites

- Node.js `20.19+` or `22.12+`
- npm
- Rust `1.88` via `rustup`
- Tauri 2 platform prerequisites for [macOS](https://v2.tauri.app/start/prerequisites/#macos) or [Linux](https://v2.tauri.app/start/prerequisites/#linux)

### Run the desktop app

```bash
npm ci
npm run tauri dev
```

### Frontend-only development

```bash
npm run dev
```

The frontend preview uses deterministic development fixtures when a documented `release-preview` route is enabled. Production builds always call the Rust backend.

## Quality gates

```bash
npm run lint
npm run typecheck
npm test
npm run build
npm run release:check

cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Continuous integration runs frontend linting, type checking, tests, production builds, Rust formatting, Clippy, tests, dependency audits, and release packaging checks.

## Architecture

```text
React + TypeScript UI
        │ typed Tauri commands and events
        ▼
Rust command boundary
        │ validation, cancellation, and error normalization
        ▼
Scanner · Rule engine · Duplicate verifier · App intelligence · Cleanup planner
        │
        ├── Read-only filesystem metadata
        ├── Immutable cleanup plans + immediate revalidation
        └── Local schema-versioned NDJSON persistence
```

The UI owns presentation and transient interaction state. Rust owns filesystem access, rule evaluation, hashes, cleanup authorization, and persistence. See [Architecture](docs/architecture.md) and [IPC contracts](docs/ipc-contracts.md) for the detailed boundaries.

## Platform notes

- macOS provides the richest application inventory, last-used metadata, Full Disk Access guidance, and application-uninstall flow.
- Linux supports scans, findings, duplicates, storage analysis, cleanup planning, Trash integration, and local history; platform-specific application attribution varies by desktop environment.
- Items in Trash still occupy disk space until the operating-system Trash is emptied.
- Displayed logical size may differ from immediately reclaimed space because of sparse allocation, compression, snapshots, clones, and filesystem accounting.

Review all known constraints in [Version 1 limitations](docs/limitations.md).

## Documentation

| Topic | Document |
| --- | --- |
| System design | [Architecture](docs/architecture.md) |
| Cleanup guarantees | [Safety policy](docs/safety-policy.md) |
| Security analysis | [Threat model](docs/threat-model.md) |
| Frontend/backend boundary | [IPC contracts](docs/ipc-contracts.md) |
| Test coverage | [Test strategy](docs/test-strategy.md) |
| Release readiness | [Release checklist](docs/release-checklist.md) and [releasing guide](docs/releasing.md) |
| Installation | [Install, upgrade, and uninstall](INSTALL.md) |
| Vulnerability reporting | [Security policy](SECURITY.md) |

## Contributing

Contributions should preserve DiskSage's core invariant: **no cleanup without a visible, bounded, revalidated plan**. Add or change rules with fixture-backed tests, explicit safety classification, symlink and boundary coverage, cancellation behavior, and user-facing evidence.

Before opening a change, run the quality gates above and review the [test strategy](docs/test-strategy.md).

## License

DiskSage is available under the [MIT License](LICENSE).
