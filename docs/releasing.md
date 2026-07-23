# Release workflow

DiskSage production releases are created only from version tags such as `v0.1.0`. The release workflow validates version consistency and uploads platform packages to a draft GitHub release. Preview workflows build Linux x86_64/ARM64 and Windows x86_64 packages against an existing prerelease tag.

## Required repository secrets

macOS signing and notarization require:

- `APPLE_CERTIFICATE`: base64-encoded Developer ID Application `.p12`;
- `APPLE_CERTIFICATE_PASSWORD`: export password for that certificate;
- `APPLE_SIGNING_IDENTITY`: full Developer ID Application identity;
- `APPLE_ID`: Apple developer account email;
- `APPLE_PASSWORD`: app-specific password;
- `APPLE_TEAM_ID`: Apple developer team identifier.

Windows production installers require an Authenticode certificate or a managed signing service such as Azure Trusted Signing. Configure the provider credentials as encrypted repository secrets and make the signing command fail closed when credentials or timestamping are unavailable.

The GitHub token is supplied by Actions. Grant the workflow `contents: write`; never add signing files or secret values to the repository.

## Preparing a release

1. Update the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Add the matching section to `CHANGELOG.md`.
3. Refresh screenshots from deterministic release-preview fixtures.
4. Run `npm run release:check -- <version>` and the full CI suite.
5. Complete all applicable pre-tag checks in `docs/release-checklist.md`.
6. Create and push `v<version>`.
7. Inspect the draft artifacts, verify signatures/notarization, and perform clean-machine, upgrade, and uninstall tests.
8. Publish the draft only after every blocking checklist item passes.

Unsigned macOS and Windows builds are preview/development artifacts, not production releases.
