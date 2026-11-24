# ram-widget: plan

Personal project. A small, light RAM monitor widget. Rust. Windows first, then macOS and Linux (Debian, Arch, RHEL).

Also a Rust learning project. Keep every phase small and shippable.

## Goal

- Show total RAM capacity and current RAM usage in percent, live (1s refresh).
- Lighter than Task Manager: target under 20 MB RSS, near-zero CPU.
- Three delivery forms, built in order: CLI, tray icon, small always-on-top window.
- Option to start at login, off by default.

## Why Rust

- The GUI toolkit decides RAM cost, not the language. Rust has the smallest cross-platform GUI options.
- `sysinfo` crate reads memory on all three OS with one API.
- No runtime, no GC. Single static exe, no installer.

Rejected: Go (GUI needs cgo or webview, 40 MB+ idle), C++ (Qt is heavy, or three native codebases), Bend (GPU research language, no GUI, no Windows story).

## Dev environment

- WSL2 Ubuntu. Windows is the first run target.
- Repo lives on the WSL filesystem: `/home/bandelaria/dev/ram-widget`. Not under `/mnt/c` (cargo is 5 to 10x slower there).
- Cross-compile to `x86_64-pc-windows-gnu` from WSL. Run the `.exe` straight from the WSL shell; interop launches it on Windows. No Windows-side Rust install.
- Fallback if a crate refuses the gnu target: `cargo-xwin` for the MSVC target from WSL.

## Crate list

| Crate | Purpose | Phase |
|-------|---------|-------|
| `sysinfo` | total and used memory, all OS | 1 |
| `tray-icon` | system tray icon, tooltip, menu | 2 |
| `tao` | event loop; Windows tray needs a message pump | 2 |
| `auto-launch` | start at login; registry Run key, LaunchAgent, XDG autostart | 2b |
| `slint` | small window; software renderer, winit backend | 3 |

egui/eframe skipped: fastest to write, but the GPU context costs 30 to 50 MB. Add only if a richer UI is wanted later.

## Phase 0: toolchain

1. `curl https://sh.rustup.rs -sSf | sh`, then restart the shell.
2. `sudo apt install mingw-w64 build-essential`
3. `rustup target add x86_64-pc-windows-gnu`
4. `cargo init` in this folder (not `cargo new`, folder already has this file).
5. Add `.cargo/config.toml`:

   ```toml
   [target.x86_64-pc-windows-gnu]
   linker = "x86_64-w64-mingw32-gcc"
   ```

6. `cargo build --target x86_64-pc-windows-gnu`
7. Run `./target/x86_64-pc-windows-gnu/debug/ram-widget.exe`. Hello world must appear.
8. `git init`, first commit.

## Phase 1: CLI

Proves the pipeline. First real Rust.

- Dep: `sysinfo`.
- `System::new()`, `refresh_memory()`, `total_memory()`, `used_memory()`. Values are bytes.
- Percent = used / total * 100.
- Print one line every second: `Total: 31.9 GB  Used: 12.2 GB  38.2%`. Ctrl+C exits.
- Done when the `.exe` matches Task Manager "In use" within 1%.

## Phase 2: tray icon

- Deps: `tray-icon`, `tao`.
- Tooltip: `12.2 / 31.9 GB  38%`.
- Right-click menu: Quit.
- A timer thread refreshes memory every 1s and sends the new text to the event loop.
- Icon: one static 16 px PNG embedded with `include_bytes!`. A redrawn percent icon is a later extra.
- Done when the tray entry updates live and Quit exits clean.

## Phase 2b: start at login

- Dep: `auto-launch`.
  - Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. No admin needed.
  - macOS: LaunchAgent plist in `~/Library/LaunchAgents`.
  - Linux: `.desktop` file in `~/.config/autostart`.
- Tray menu gets a check item: **Start at login**. Checked state comes from `is_enabled()` at startup, so no config file. Toggle calls `enable()` or `disable()`.
- Exe path from `std::env::current_exe()`.
- Login start passes an arg such as `--minimized` so only the tray opens, not the Phase 3 window.
- Default: off. User opts in.
- Known limit: the entry stores the absolute exe path. Moving or renaming the `.exe` breaks autostart silently. Re-toggling fixes it.

## Phase 3: small window

- Dep: `slint`, features `renderer-software` and `backend-winit`.
- One `.slint` file. Two text lines: total capacity, used percent.
- Frameless, always on top, fixed size, draggable.
- Same 1s refresh via `slint::Timer`.
- A checkbox for start at login, backed by the same `auto-launch` calls as the tray.
- Done when window and tray show the same numbers.

## Phase 4: cross-platform check (later)

- Native Linux build in WSL: `cargo build`. Tray needs `libgtk-3-dev` and `libxdo-dev` on Linux.
- macOS needs a Mac. No code change expected for the memory part.
- Per-distro packaging (deb, AUR, rpm) is out of scope until the app is stable.

## Order and rules

- 0 → 1 → 2 → 2b → 3 → 4. Each phase shippable. Commit per phase.
- Smallest change that works. No abstraction before the second use.
- Measure RSS after each GUI phase. If a phase pushes idle RSS over 20 MB, stop and find out why before adding more.

## Open risks

- `windows-gnu` target can hit linker issues in crates that assume MSVC. Fallback: `cargo-xwin`.
- Slint software renderer under `windows-gnu` not verified yet.
- `auto-launch` under a gnu cross-build not verified, but it is plain registry calls and should link clean.
- Any GUI window on Windows allocates a DWM surface. Under 10 MB holds only for the tray-only form.
