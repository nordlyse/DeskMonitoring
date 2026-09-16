# DeskMonitoring

A Conky-style desktop overlay built with **Rust** and **GTK4**. It draws a transparent, neon HUD with live charts for CPU, memory, disk, network, mail, today's calendar, and weather for the nearest city.

## Setup

Every launch asks for settings before the overlay starts. Saved values are filled in when present, so you can confirm what will run or change them:

- **Panel position:** left, right, top, bottom, or center
- **Color palette:** Matrix green, turquoise, blue, pink, or yellow
- Optional IMAP host for incoming / outgoing / total / unread mail counts
- Optional `.ics` file or folder for today's calendar events

Settings are stored in `~/.config/desk-monitoring/config.toml` (or the platform config directory).

Drag the panel to move it. On Linux Wayland, build with the `layer-shell` feature so the chosen edge/center is applied as a desktop overlay.

## Metrics

| Panel | Source |
| --- | --- |
| CPU / RAM / disk / network | [sysinfo](https://crates.io/crates/sysinfo) |
| Mail | IMAP `STATUS` when configured, otherwise local Maildir |
| Calendar | Today's events from `.ics` files, or `0` when none |
| Weather | IP geolocation via ipwho.is, forecast via Open-Meteo |

Mail fields:

- **IN** — inbox message count
- **OUT** — sent mailbox count
- **TOTAL** — IN + OUT
- **UNREAD** — unseen inbox messages

## Packages

GitHub Actions builds installers on tag `v*` or from **Actions → Package → Run workflow**:

| Platform | Artifact | Notes |
| --- | --- | --- |
| macOS | `DeskMonitor-macos-arm64.dmg` / `DeskMonitor-macos-x64.dmg` | Open the disk image and drag **Desk Monitor** onto **Applications** |
| Linux | `DeskMonitor-linux-x64` `.deb` | Needs GTK 4 from the distro (`libgtk-4-1`) |
| Windows | `DeskMonitor-windows-x64.zip` | Run `DeskMonitor.bat`; GTK DLLs are inside the zip |

Local packaging:

```bash
# macOS
bash packaging/package-macos.sh

# Linux (native)
bash packaging/package-linux.sh

# Linux (Ubuntu 24.04 Docker)
bash packaging/package-linux-docker.sh

# Windows (PowerShell, after gvsbuild GTK4)
powershell -File packaging/package-windows.ps1
```

macOS output is `dist/macos/Desk Monitor.app` and `dist/DeskMonitor-macos.dmg` (drag the app onto Applications).
Linux output is `dist/*.deb`.
Windows output is `dist/DeskMonitor-windows.zip`.

NSIS and WiX are not used (license). Windows ships as a zip.

## Build

Install GTK4 development files, then:

```bash
cargo run --release
```

Linux overlay (needs `gtk4-layer-shell` on the system):

```bash
cargo run --release --features layer-shell
```

## Licenses

This project is MIT. Direct crates are MIT and/or Apache-2.0 only:

| Crate | License |
| --- | --- |
| gtk4 (crate name `gtk`) | MIT |
| glib | MIT |
| sysinfo | MIT |
| serde / serde_json | MIT OR Apache-2.0 |
| toml | MIT OR Apache-2.0 |
| dirs | MIT OR Apache-2.0 |
| chrono | MIT OR Apache-2.0 |
| reqwest | MIT OR Apache-2.0 |
| tokio | MIT |
| native-tls / tokio-native-tls | MIT OR Apache-2.0 |
| icalendar | MIT OR Apache-2.0 |
| gtk4-layer-shell (optional) | MIT |

GTK4 itself is a system library (LGPL) required by the toolkit you asked for. No GPL crates are included.

Weather HTTP APIs are used at runtime; they are not bundled libraries.
