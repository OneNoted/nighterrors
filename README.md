<h1 align="center">nighterrors</h1>

<p align="center">
  Ultra-light Wayland blue-light filter daemon for Hyprland, Niri, and wlroots compositors.<br />
  Foreground daemon or user <code>systemd</code> service, with runtime temperature and gamma control from a single binary.
</p>

<p align="center">
  <img alt="crates.io" src="https://img.shields.io/crates/v/nighterrors?style=flat-square" />
  <img alt="AUR nighterrors-bin" src="https://img.shields.io/aur/version/nighterrors-bin?style=flat-square" />
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache%202.0-000000?style=flat-square" />
  <img alt="Rust 1.93+" src="https://img.shields.io/badge/rust-1.93%2B-000000?style=flat-square&logo=rust" />
  <img alt="Platform Linux" src="https://img.shields.io/badge/platform-linux-1f2937?style=flat-square&logo=linux" />
  <img alt="Display Wayland" src="https://img.shields.io/badge/display-wayland-6f42c1?style=flat-square" />
</p>

<p align="center">
  <a href="#install"><strong>Install</strong></a>
  ·
  <a href="#quick-start"><strong>Quick start</strong></a>
  ·
  <a href="#help"><strong>Help</strong></a>
  ·
  <a href="https://crates.io/crates/nighterrors"><strong>crates.io</strong></a>
  ·
  <a href="https://aur.archlinux.org/packages/nighterrors-bin"><strong>AUR</strong></a>
</p>

Ultra-light Rust Wayland blue-light filter daemon for Hyprland, Niri, and wlroots compositors.

`nighterrors` runs as a small foreground daemon or a user `systemd` service and exposes a minimal CLI for runtime color temperature and gamma control.

## Features

- Small single-binary daemon and control client
- Backend auto-selection with Hyprland CTM support preferred over wlroots gamma control
- Runtime commands for `set`, `get`, `toggle`, `reset`, `outputs`, `exclude`, `status`, and `stop`
- Optional user `systemd` service management via `nighterrors service ...`
- TTY-aware output with `--raw`, `--pretty`, and automatic mode selection
- Output exclude/include controls by output name
- Minimal dependency footprint with no async runtime or CLI framework

## Requirements

- Linux
- Wayland
- A compositor exposing either `hyprland-ctm-control-v1` or `wlr-gamma-control-unstable-v1`

## Install

After publishing:

```bash
cargo install nighterrors
```

From a checkout:

```bash
cargo build --release
./target/release/nighterrors run
```

Arch packaging sources for `nighterrors-git` and `nighterrors-bin` live under
`packaging/aur/`.

## Quick Start

Build from a checkout:

```bash
cargo build --release
```

Start the daemon in the foreground:

```bash
./target/release/nighterrors run -t 5500 -g 95 -i off
```

Control it from another terminal:

```bash
./target/release/nighterrors status
./target/release/nighterrors set temp +200
./target/release/nighterrors toggle
./target/release/nighterrors outputs list
```

Install it as a user service:

```bash
./target/release/nighterrors service install --temp 5500 --gamma 95
./target/release/nighterrors service status
```

## Notes

- `nighterrors` currently supports Linux only.
- If no supported Wayland protocol is exposed, startup fails with a backend-specific error.
- Default socket path: `${XDG_RUNTIME_DIR:-/run/user/<uid>}/nighterrors/${WAYLAND_DISPLAY:-wayland-0}.sock`

## Help

```bash
nighterrors --help
nighterrors help run
nighterrors help toggle
nighterrors help service
```
