# DiskSage Privacy Policy

Effective: July 14, 2026

DiskSage is a local-first desktop application. By default, it does not transmit filenames, filesystem paths, file contents, hashes, scan findings, cleanup history, settings, or diagnostics to DiskSage contributors or any third party.

## Data processed locally

DiskSage reads filesystem metadata needed for user-initiated scans, including paths, sizes, file types, modification times, and locally computed hashes for duplicate verification. These values remain on the device. Findings, settings, duplicate groups, and cleanup history are stored in the operating system's application-data directory.

## Network access and telemetry

DiskSage v0.1.0 contains no analytics, advertising, crash-reporting upload, account system, cloud synchronization, or automatic telemetry. The application Content Security Policy does not permit remote scripts or arbitrary remote content.

## Diagnostics exports

A diagnostics export occurs only when the user selects **Export diagnostics**. The report is written locally and contains application/platform information, aggregate counts, error codes, and redacted configuration. It excludes filenames, configured path values, mount paths, hashes, file contents, and cleanup item records. DiskSage does not upload the report.

## File operations

Cleanup actions are explicitly initiated by the user. Move to Trash is the default. Permanent deletion is disabled by default and requires additional confirmation. Cleanup history remains local until the user clears it or removes the application data.

## Deletion and retention

Uninstalling the executable does not automatically remove local application data. See [INSTALL.md](INSTALL.md#uninstall) for the platform-specific data locations and removal steps.

## Changes

Material privacy changes will be documented in release notes and this policy's effective date will be updated.
