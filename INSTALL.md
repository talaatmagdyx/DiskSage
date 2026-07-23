# Installing DiskSage

DiskSage supports macOS 10.15 or later, Windows 10 and 11, and Ubuntu 22.04 or a compatible modern Linux distribution.

## macOS

Download the DMG matching your Mac:

- `aarch64` for Apple silicon;
- `x86_64` for Intel.

Open the DMG and drag DiskSage to Applications. Public release artifacts are expected to be Developer ID signed, notarized, and stapled. Development builds are unsigned and may require explicit approval in **System Settings → Privacy & Security**.

## Windows

Download either the x86_64 NSIS setup executable for a guided current-user installation or the MSI package for managed deployment. Preview installers may be unsigned and can trigger Microsoft Defender SmartScreen; install them only when the release digest matches the published checksum.

Windows application inventory is read-only. Use **Manage** in DiskSage to open **Settings → Apps → Installed apps**, then run the publisher-registered uninstaller.

## Linux AppImage

Download the AppImage matching your architecture (`amd64`/`x86_64` or `aarch64`/ARM64), then run:

```sh
chmod +x DiskSage_*.AppImage
./DiskSage_*.AppImage
```

The AppImage is built on Ubuntu 22.04 to maintain a conservative glibc baseline. Some distributions require FUSE 2 compatibility to launch AppImages.

## Debian package

Download the `.deb` matching your architecture, then run:

```sh
sudo apt install ./DiskSage_*.deb
```

The package installs a desktop entry and application icon. Runtime WebKitGTK dependencies are resolved by the package manager.

## RPM package

On Fedora, RHEL, or a compatible RPM-based distribution, download the `.rpm` matching your architecture, then run:

```sh
sudo dnf install ./DiskSage-*.rpm
```

The AppImage remains the broadest preview option for other modern Linux distributions.

## Upgrades

Quit DiskSage and install the newer package over the existing version. Settings and local history use schema-versioned files in the application-data directory and should be preserved. Back up that directory before testing prerelease builds.

## Uninstall

Removing DiskSage does not remove user-created files or items already moved to Trash.

On macOS, remove `DiskSage.app`. To also remove local DiskSage settings, findings, duplicate groups, history, cache, and diagnostics, remove the directories named `com.disksage.desktop` under `~/Library/Application Support`, `~/Library/Caches`, and `~/Library/Preferences` if present.

On Debian-based Linux:

```sh
sudo apt remove disksage
```

To also remove local DiskSage data, remove the `com.disksage.desktop` directories under `${XDG_CONFIG_HOME:-~/.config}`, `${XDG_DATA_HOME:-~/.local/share}`, and `${XDG_CACHE_HOME:-~/.cache}` if present. Review every path before removal.

On Windows, uninstall DiskSage from **Settings → Apps → Installed apps**. To remove local DiskSage settings and history, review the `com.disksage.desktop` directories under `%APPDATA%` and `%LOCALAPPDATA%`; remove only paths clearly owned by DiskSage.

## Building from source

See the Development section in [README.md](README.md). Source builds are development artifacts and are not notarized.
