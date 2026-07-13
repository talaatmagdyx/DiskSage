# Release checklist

Every item must be completed for the exact tagged commit. A successful build alone is not a release approval.

## Automated gate

- [ ] Frontend lint, type check, unit tests, and production build pass.
- [ ] Rust formatting, Clippy with warnings denied, unit/integration tests, audit, and license policy pass.
- [ ] The tag matches `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `CHANGELOG.md`.
- [ ] macOS Apple silicon and Intel DMGs build.
- [ ] macOS artifacts are Developer ID signed, notarized, and stapled; `codesign --verify --deep --strict` and `spctl --assess --type execute` pass.
- [ ] Ubuntu 22.04 x86_64 AppImage and amd64 Debian package build.
- [ ] Release artifact checksums are recorded by the release owner.

## Clean-machine gate

- [ ] Install each macOS architecture artifact on a clean supported machine.
- [ ] Install AppImage and Debian artifacts on clean Ubuntu 22.04 Wayland and X11 sessions.
- [ ] Launch, onboarding, disk loading, fixture scan, cancellation, reveal, Trash cleanup, duplicate keep-one flow, history, and diagnostics export pass.
- [ ] Permission-denied paths produce understandable errors without a crash.
- [ ] Memory and responsiveness remain within the documented targets on fixture datasets.

## Upgrade and uninstall gate

- [ ] Upgrade from the previous release without losing valid settings or local history.
- [ ] Corrupt or unsupported settings fail safely and offer recovery.
- [ ] Application uninstall removes the package but preserves user data unless the documented manual data-removal step is chosen.
- [ ] Manual data removal affects only the documented `com.disksage.desktop` application directories.

## Documentation and publication gate

- [ ] README screenshots match the tagged build and contain no sensitive paths.
- [ ] Privacy, security, install, limitations, and release notes are reviewed.
- [ ] Draft release artifacts are smoke-tested before publication.
- [ ] The release is published only after all blocking items above are checked by a release owner.
