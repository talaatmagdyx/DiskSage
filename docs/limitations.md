# Version 1 limitations

- Windows support currently targets Windows 10 and 11 x86_64. Cache, project, custom, duplicate, storage-map, and Recycle Bin flows require clean-machine validation before a production release.
- Windows application inventory reads standard per-user and machine uninstall registry entries. Microsoft Store-only applications and registrations without a usable install location may be absent.
- DiskSage never deletes Windows installation directories directly. Application removal remains owned by the publisher-registered uninstaller in Windows Installed Apps.
- Quick and Developer scans target known paths and configured projects rather than recursively classifying the entire home directory.
- Large and old files are review suggestions. Size or age alone never authorizes cleanup.
- Docker and emulator findings are inspection or guided-action records. DiskSage does not directly remove Docker virtual disks, volumes, or emulator state.
- Duplicate preview does not render file contents. Equality is established through staged local hashing and optional byte comparison.
- Items moved to the operating-system Trash are not restored by DiskSage. Use the platform's Trash interface.
- macOS may block protected app containers or administrator-owned applications. Quit the target app and grant DiskSage Full Disk Access when prompted; some applications may still need to be moved with Finder.
- Windows can deny or defer Recycle Bin operations for files that are locked, administrator-owned, protected by security software, or stored on filesystems without normal Recycle Bin support.
- Actual free-space change can differ from selected logical size because of Trash retention, sparse files, copy-on-write storage, compression, snapshots, or filesystem accounting.
- The app has no scheduler, background cleanup daemon, cloud synchronization, account system, remote telemetry, or automatic updater in v0.1.0.
- Full Analysis requires configured project roots and can take significant time. Dedicated duplicate detection remains explicitly folder-scoped.
