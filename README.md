# DiskSage

DiskSage is a local-first macOS and Linux desktop application for understanding disk usage and, in later phases, reviewing safe cleanup recommendations. Safety takes precedence over maximum cleanup.

This repository currently implements Phase 0 and Phase 1 only:

- Tauri 2 + Rust desktop foundation
- React 18, TypeScript, Vite, TailwindCSS, shadcn-compatible component setup, routing, and Zustand
- Read-only mounted-disk information using `sysinfo`
- Validated, atomic JSON settings persistence in the OS application-data directory
- Structured IPC errors and structured Rust logging
- Versioned domain contracts for rules, findings, scans, and immutable cleanup plans
- Protected-path policy and its first mandatory safety tests
- Least-privilege Tauri capability and strict Content Security Policy

There is intentionally no scanner, trash operation, permanent deletion, shell command, or cleanup IPC command in this phase.

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
