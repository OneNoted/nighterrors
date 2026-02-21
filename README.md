# nighterrors

<p align="left">
  <img alt="Rust 2021" src="https://img.shields.io/badge/rust-2021-orange?logo=rust" />
  <img alt="Platform Linux" src="https://img.shields.io/badge/platform-linux-informational?logo=linux" />
  <img alt="Display Wayland" src="https://img.shields.io/badge/display-wayland-6f42c1" />
  <img alt="Backends Hyprland + wlroots" src="https://img.shields.io/badge/backends-hyprland%20%2B%20wlroots-2ea44f" />
  <img alt="Dependencies minimal" src="https://img.shields.io/badge/dependencies-minimal-blue" />
</p>

Ultra-light Rust Wayland blue-light filter daemon.

`nighterrors` provides runtime temperature/gamma control with backend auto-selection:
- `hyprland-ctm-control-v1` when available (preferred)
- `wlr-gamma-control-unstable-v1` as fallback

## Features

- Small single-binary daemon + CLI control interface
- Runtime control: `set`, `get`, `reset`, `outputs`, `exclude`, `stop`, `status`
- TTY-aware output (`--raw`, `--pretty`, auto mode)
- Output exclude/include controls (by output name or `@<global_id>`)
- Minimal dependency footprint (no clap/tokio/serde)

## Quick Start

### 1) Build

```bash
cargo build --release
```

### 2) Run daemon (foreground)

```bash
./target/release/nighterrors run
```

Example startup with options:

```bash
./target/release/nighterrors run -t 5500 -g 95 -i off --exclude eDP-1
```

### 3) Control from another terminal

```bash
./target/release/nighterrors status
./target/release/nighterrors set temp 4800
./target/release/nighterrors set g -5
./target/release/nighterrors exclude add eDP-1
./target/release/nighterrors get state --raw
```

### 4) Stop daemon

```bash
./target/release/nighterrors stop
```

## Command Help

```bash
nighterrors help
nighterrors help run
nighterrors help set
```

## Notes

- Linux + Wayland only.
- If no compatible protocol is exposed by your compositor, startup fails with a clear backend/protocol error.
- Default socket path:
  `${XDG_RUNTIME_DIR:-/run/user/<uid>}/nighterrors/${WAYLAND_DISPLAY:-wayland-0}.sock`
