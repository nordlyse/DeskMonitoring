# DeskMonitoring

A Conky-style desktop overlay built with **Rust** and **GTK4**. It draws a transparent, neon HUD with live charts for CPU, memory, disk, network, mail, today's calendar, and weather for the nearest city.

## First setup

On the first launch the app asks:

- **Panel position:** left, right, top, bottom, or center
- **Color palette:** Matrix green, turquoise, blue, pink, or yellow
- Optional IMAP host for incoming / outgoing / total / unread mail counts
- Optional `.ics` file or folder for today's calendar events

Settings are stored in `~/.config/desk-monitoring/config.toml` (or the platform config directory). Remove that file to run setup again.

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
