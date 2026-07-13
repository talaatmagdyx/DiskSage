# Installing DiskSage

DiskSage supports macOS 10.15 or later and Ubuntu 22.04 or a compatible modern Linux distribution.

## macOS

Download the DMG matching your Mac:

- `aarch64` for Apple silicon;
- `x86_64` for Intel.

Open the DMG and drag DiskSage to Applications. Public release artifacts are expected to be Developer ID signed, notarized, and stapled. Development builds are unsigned and may require explicit approval in **System Settings → Privacy & Security**.

## Linux AppImage

Download the x86_64 AppImage, then run:

```sh
chmod +x DiskSage_*.AppImage
./DiskSage_*.AppImage
```

The AppImage is built on Ubuntu 22.04 to maintain a conservative glibc baseline. Some distributions require FUSE 2 compatibility to launch AppImages.

## Debian package

Download the amd64 `.deb`, then run:

```sh
sudo apt install ./DiskSage_*_amd64.deb
```

The package installs a desktop entry and application icon. Runtime WebKitGTK dependencies are resolved by the package manager.

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

## Building from source

See the Development section in [README.md](README.md). Source builds are development artifacts and are not notarized.
