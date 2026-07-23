# Changelog

All notable changes to DiskSage are documented here.

## Unreleased

### Added

- Explicit Windows 10/11 platform support with protected system roots, case-insensitive path enforcement, NTFS reparse-point refusal, and allocated-size measurement.
- Windows cache and developer-tool intelligence for major package managers, browsers, editors, Android tooling, Docker Desktop/WSL, and local AI model stores.
- Read-only Windows application inventory backed by standard uninstall registry entries, with handoff to Windows Installed Apps for publisher-owned removal.
- Windows Explorer reveal, privacy settings guidance, Recycle Bin errors, native x86_64 CI, and NSIS/MSI preview packaging.

## [0.1.0] - 2026-07-14

Initial Version 1 release candidate.

### Added

- Local disk overview and targeted Quick, Developer, Full, and Custom scans.
- Thirty-nine versioned cache, developer-tool, project, browser, container, and emulator rules.
- Cancellable, bounded scanning with locally persisted findings and structured partial errors.
- Immutable, expiring cleanup plans with protected-path and symlink-safe revalidation.
- Move-to-Trash cleanup, local audit history, partial-failure reporting, and disk refresh.
- Staged BLAKE3 duplicate detection, optional byte verification, deterministic keep selection, and keep-one cleanup enforcement.
- Review-only large-file and old-installer analysis.
- Installed-application inventory with last-used and size sorting, opt-in system-app visibility, and App-only, identified-data, and Expert deep-clean uninstall reviews.
- Running-app preflight, actionable macOS permission guidance, safe fresh-plan retry, and per-item partial uninstall failure reporting.
- Feature-gated permanent deletion with native and expert typed confirmation.
- First-run onboarding, system/light/dark themes, reduced motion, keyboard shortcuts, accessible dialogs and notifications, virtualized large lists, and redacted local diagnostics export.
- macOS Apple silicon and Intel release targets plus Linux AppImage and Debian packaging workflows.

### Safety notes

- Move to Trash is the default action.
- Permanent deletion is disabled by default.
- Docker and emulator state remain inspection or guided-action findings; DiskSage does not directly remove Docker virtual disks or emulator state.
- Full and Custom large/old analysis produces review suggestions, not automatic junk classification.
