# Security Policy

## Supported versions

Security fixes are provided for the latest released version of DiskSage.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability involving cleanup authorization, protected paths, symlink handling, path traversal, signature verification, or secret exposure. Use the repository host's private security-advisory feature and include:

- the affected version and platform;
- reproduction steps using disposable files only;
- the expected and observed safety boundary;
- logs or diagnostics with sensitive paths removed.

Do not test against files you do not own or systems you are not authorized to assess. Maintainers should acknowledge a complete report within seven days, provide status updates during investigation, and coordinate disclosure after a fix is available.

## Security model

The React frontend cannot authorize an arbitrary cleanup path. Rust resolves persisted finding or duplicate identifiers into immutable, expiring, single-use plans and revalidates paths immediately before mutation. Protected roots, parent symlink redirects, changed items, and attempts to remove every duplicate copy are rejected. Move to Trash is the default and permanent deletion is disabled by default.

See [docs/threat-model.md](docs/threat-model.md) and [docs/safety-policy.md](docs/safety-policy.md) for the detailed trust boundaries.

## Release integrity

Public macOS artifacts must be Developer ID signed, notarized, and stapled. Public Windows installers must be Authenticode signed and timestamped. Release CI obtains signing and notarization credentials only from encrypted repository secrets; secrets and signing material must never be committed. Linux releases are built on native Ubuntu runners and published as AppImage, Debian, and RPM packages.
