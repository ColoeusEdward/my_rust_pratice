# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build           # Build debug
cargo build --release # Build release (enables admin elevation, PotPlayer monitoring)
cargo run             # Run debug (port 7655, skips admin/UAC, skips PotPlayer monitor)
cargo run --release   # Run release (port 7654, auto-elevates via UAC, enables all features)
```

No test harness or linter is configured.

## Project Overview

This is a Windows desktop automation program ("hello_cargo") written in Rust. It was born as a personal utility for a PotPlayer user but has grown into a multi-featured background agent.

**Key capabilities:**
- Monitors PotPlayer (video player) via WinAPI `SendMessage` and `FindWindowExA` — reads current play progress/title and uploads it to a remote server every 5 minutes
- Runs a **HTTP server** (warp on ports 7654/7655) with remote-control endpoints: play text-to-speech, start Brave browser, manage PotPlayer playlists, trigger system restart, file serving
- Plays an MP3 alarm at 21:30 daily via rodio
- Monitors network connectivity and auto-reconnects WiFi by disabling/re-enabling a specific PCI network adapter via PowerShell
- Background PotPlayer playlist sync: uploads local `.dpl` playlists to a remote server, downloads remote versions back
- A **terminal menu** (stdin) for manual operations — accessible even when the HTTP server is running
- Admin elevation: release builds relaunch themselves with UAC if not already elevated
- Two-user aware: different file paths and server endpoints for user "huangwen" vs "kaf"

## Architecture

```
src/
├── main.rs               # Entry point: sets up user, starts PotPlayer monitor (release only),
│                          #   network checker, audio alarm loop, HTTP server, terminal menu
├── enums.rs               # Shared constants, config, types (PlayInfo, BvInfo, MyData)
├── router.rs              # Warp routes (GET, POST) with CORS
├── uitl.rs                # Utilities: file I/O (reverse line reading), screenshots,
│                          #   ping, Bilibili API client, time formatting, GetUserNameA
├── get_pot_player.rs      # PotPlayer WinAPI interaction (SendMessage for playback info),
│                          #   playlist parsing/upload/download, window enumeration
├── controllers/
│   ├── mod.rs
│   ├── me.rs              # HTTP handlers: charge, start_barve, play_list, potplay,
│   │                      #   test, play_text (TTS via edge-tts-rust), check_network,
│   │                      #   play_bingbong (rodio alarm at 21:30)
│   └── file.rs            # Axum-based static file server for D:\Software (port 8654)
├── menu/
│   ├── mod.rs
│   └── start.rs           # Terminal interactive menu loop (stdin-based)
└── ui/
    ├── mod.rs
    └── index.rs            # Empty — UI module skeleton (slint commented out)
```

### Control flow

1. **main()** detects the system username → configures paths/endpoints per user
2. Spawns concurrent tokio tasks: HTTP server, PotPlayer monitor (5min interval, release only), network check (10min interval, "huangwen" only), audio alarm (30s check loop for 21:30)
3. Enters the terminal menu loop (blocks main thread for stdin)

### HTTP Endpoints (warp, ports 7654/7655)

| Path | Method | Handler | Description |
|------|--------|---------|-------------|
| `/hello/{name}` | GET | `potplay` | Echo route |
| `/charge` | GET | `charge` | Triggers a PowerShell popup after 270s delay |
| `/brave` | GET | `start_barve` | Opens mahjongsoul.com in Brave, clicks, screenshots |
| `/playList` | GET | `play_list` | Syncs PotPlayer playlist |
| `/playText` | POST (JSON) | `play_text` | TTS via edge-tts-rust |
| `/test` | POST | `test` | Stub |

### Static file server

- Axum-based, serves `D:\Software` at `/soft` on port 8654

### Windows-specific dependencies

- `winapi` / `user32-sys`: Win32 SendMessage, FindWindowEx, GetWindowText for PotPlayer IPC
- `enigo` / `rsautogui`: Mouse control (Brave automation)
- `runas` / `is_elevated`: Admin elevation
- `rodio`: Audio playback (MP3 alarm + TTS response audio)
- `edge-tts-rust`: Microsoft Edge TTS for text-to-speech
- `screenshots`: Screen capture
- PowerShell commands: WiFi adapter reset, popup dialogs, process management

### CI/CD

- `.github/workflows/rust.yml`: GitHub Actions — on `v*` tag push, builds `--release` and creates a GitHub Release with the `.exe`
- `build.bat`: manual release build script
- `build.rs`: Copies `src/ui/bingbongbangbong.MP3` next to the compiled binary so rodio can find it at runtime

## Debug vs Release

- **Debug**: skips UAC elevation, skips PotPlayer monitoring loop, uses port 7655
- **Release**: auto-elevates to admin, enables PotPlayer 5-min monitoring loop, uses port 7654

### MP3 resource
`src/ui/bingbongbangbong.MP3` is the alarm sound played at 21:30. It gets copied to the output directory by `build.rs`. The binary finds it at runtime via `std::env::current_exe()` sibling path.

## PotPlayer IPC Details

- Finds PotPlayer window by class name `PotPlayer64` (via `FindWindowExA`)
- Sends `WM_USER + 1024` (`REQ_TYPE`) with `WPARAM = 20484` (`POT_GET_PROGRESS_TIME`) via `SendMessageA` to get playback progress in milliseconds
- Reads window title to get the playing file name
- Parses PotPlayer playlist files (`.dpl` format, `@`-delimited) with custom byte-by-byte reading utilities
- Uploads info to `https://meamoe.top/koa/newCen/free/savePotInfo`
