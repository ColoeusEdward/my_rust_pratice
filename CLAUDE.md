# CLAUDE.md

The role of this file is to describe common mistakes andconfusion points that agents might encounter as they work inthis project. If you ever encounter something in the projectthat surprises you, please alert the developer working with youand indicate that this is the case in the AgentMD file to helpprevent future agents from having the same issue.

### Windows-specific dependencies

- `winapi` / `user32-sys`: Win32 SendMessage, FindWindowEx, GetWindowText for PotPlayer IPC
- `enigo` / `rsautogui`: Mouse control (Brave automation)
- `runas` / `is_elevated`: Admin elevation
- `rodio`: Audio playback (MP3 alarm + TTS response audio)
- `edge-tts-rust`: Microsoft Edge TTS for text-to-speech
- `screenshots`: Screen capture
- PowerShell commands: WiFi adapter reset, popup dialogs, process management

## Debug vs Release

- **Debug**: skips UAC elevation, skips PotPlayer monitoring loop, uses port 7655
- **Release**: auto-elevates to admin, enables PotPlayer 5-min monitoring loop, uses port 7654

### MP3 resource

`src/ui/bingbongbangbong.MP3` is the alarm sound played at 21:30. It gets copied to the output directory by `build.rs`. The binary finds it at runtime via `std::env::current_exe()` sibling path.

## MCGS IPC Details (`mcgs_control.rs`)

- Finds MCGS「下载配置」dialog (belongs to `McgsSetPro.exe`, a different process from the
  simulator window `mcgs_app.exe`) by top-level window title match, then finds its
  「停止运行」/「启动运行」child buttons by title match, and clicks via
  `SendMessageA(hwnd, BM_CLICK, 0, 0)`.
- **UIPI gotcha**: `McgsSetPro.exe` normally runs elevated (admin). If `hello_cargo.exe` is
  running in **debug mode** (non-elevated, per this file's Debug/Release section), `BM_CLICK`
  is silently swallowed by Windows' User Interface Privilege Isolation — `SendMessageA` returns
  without error, the program prints "已点击", but the button never actually fires and MCGS's
  「返回信息」 log shows no new entry. This is NOT a bug in window/button lookup — verify by
  checking whether the target process can be opened with `OpenProcess(PROCESS_QUERY_INFORMATION)`
  from a non-elevated process (access denied ⇒ target is elevated ⇒ UIPI blocks the click).
  Only a **release build** (which auto-elevates via UAC) can actually click MCGS's buttons.

## PotPlayer IPC Details

- Finds PotPlayer window by class name `PotPlayer64` (via `FindWindowExA`)
- Sends `WM_USER + 1024` (`REQ_TYPE`) with `WPARAM = 20484` (`POT_GET_PROGRESS_TIME`) via `SendMessageA` to get playback progress in milliseconds
- Reads window title to get the playing file name
- Parses PotPlayer playlist files (`.dpl` format, `@`-delimited) with custom byte-by-byte reading utilities
- Uploads info to `https://meamoe.top/koa/newCen/free/savePotInfo`
