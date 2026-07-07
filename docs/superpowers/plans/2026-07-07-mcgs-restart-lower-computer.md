# MCGS 下位机重启功能 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增一个 CLI 菜单选项，点击后依次点击 MCGS「下载配置」弹窗上的「停止运行」「启动运行」按钮（中间等待 2 秒），用于重启 MCGS 下位机模拟运行。

**Architecture:** 新建独立模块 `src/mcgs_control.rs`，用 `EnumWindows` 按标题关键字「下载配置」查找目标顶层窗口，再用 `EnumChildWindows` 按标题关键字查找其直接子按钮，通过 `SendMessageA(hwnd, BM_CLICK, 0, 0)` 发送标准 Win32 按钮点击消息触发点击（不使用坐标点击，不要求窗口在前台）。`main.rs` 声明该模块；`menu/start.rs` 增加菜单选项触发。

**Tech Stack:** Rust 2021、winapi 0.3.9（`winuser` feature，已在 Cargo.toml 中启用）、tokio（`time::sleep`、`spawn`）。

---

## 参考设计文档

`docs/superpowers/specs/2026-07-07-mcgs-restart-lower-computer-design.md`

## 关键 API 签名（已在本机 winapi 0.3.9 源码核实）

```rust
// winapi::um::winuser
pub const BM_CLICK: UINT = 0x00F5;

pub fn EnumWindows(
    lpEnumFunc: WNDENUMPROC,
    lParam: LPARAM,
) -> BOOL;

pub fn EnumChildWindows(
    hWndParent: HWND,
    lpEnumFunc: WNDENUMPROC,
    lParam: LPARAM,
) -> BOOL;

// WNDENUMPROC 的函数签名：
// unsafe extern "system" fn(HWND, LPARAM) -> BOOL
```

现有代码里 `controllers/wechat_capture.rs` 已经有一个几乎一样的 `find_wechat_window` /
`enum_wechat_windows_callback` 模式可以参考（`EnumWindows` + 用裸指针通过 `LPARAM`
回传结果 + `GetWindowTextW` 读标题 + `IsWindowVisible` 过滤）。本计划的新代码结构
与其保持一致。

---

### Task 1: 新建 mcgs_control 模块骨架 + 顶层窗口查找函数

**Files:**
- Create: `src/mcgs_control.rs`

- [ ] **Step 1: 创建文件，写入模块头部、导入和顶层窗口查找函数**

```rust
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::ptr::null_mut;
use tokio::time::{sleep, Duration};
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{
    EnumChildWindows, EnumWindows, GetWindowTextW, IsWindowVisible, SendMessageA, BM_CLICK,
};

/// 在「下载配置」弹窗上依次点击「停止运行」「启动运行」，中间等待 2 秒。
/// 找不到窗口或按钮时打印错误信息并直接返回，不重试、不 panic。
pub async fn restart_lower_computer() {
    let Some(parent) = find_download_config_window() else {
        eprintln!("未找到「下载配置」窗口，请确认 McgsSetPro 已打开");
        return;
    };
    let Some(stop_btn) = find_child_button(parent, "停止运行") else {
        eprintln!("未找到「停止运行」按钮");
        return;
    };
    click_button(stop_btn);
    println!("已点击「停止运行」");

    sleep(Duration::from_secs(2)).await;

    // 重新查找一次，防止等待期间窗口被关闭重开导致句柄失效
    let Some(parent) = find_download_config_window() else {
        eprintln!("未找到「下载配置」窗口，请确认 McgsSetPro 已打开");
        return;
    };
    let Some(start_btn) = find_child_button(parent, "启动运行") else {
        eprintln!("未找到「启动运行」按钮");
        return;
    };
    click_button(start_btn);
    println!("已点击「启动运行」");
}

fn find_download_config_window() -> Option<HWND> {
    let mut found: HWND = null_mut();
    unsafe {
        EnumWindows(
            Some(enum_top_windows_callback),
            &mut found as *mut HWND as LPARAM,
        );
    }

    if found.is_null() {
        None
    } else {
        Some(found)
    }
}

unsafe extern "system" fn enum_top_windows_callback(hwnd: HWND, data: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }

    let title = window_title(hwnd);
    if title.contains("下载配置") {
        *(data as *mut HWND) = hwnd;
        return 0;
    }

    1
}

unsafe fn window_title(hwnd: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32);

    if len <= 0 {
        return String::new();
    }

    OsString::from_wide(&text[..len as usize])
        .to_string_lossy()
        .to_string()
}
```

注意：此时文件还缺 `find_child_button` 和 `click_button`，`cargo check` 会报
"未找到函数" 之类的错误。这是预期的中间状态，Task 2 会补全。

- [ ] **Step 2: 提交（作为一个中间提交，暂不 checkout 编译）**

先不要执行 `cargo build`，因为 `find_child_button`/`click_button` 还未定义，
编译会失败。直接进行 Task 2，Task 2 结束后一起验证编译。

```bash
git add src/mcgs_control.rs
git commit -m "wip: add mcgs_control module skeleton with top-window finder"
```

---

### Task 2: 补全子按钮查找函数与点击函数

**Files:**
- Modify: `src/mcgs_control.rs`

- [ ] **Step 1: 在 `src/mcgs_control.rs` 末尾追加子窗口查找和点击函数**

在文件末尾（`window_title` 函数之后）追加：

```rust

fn find_child_button(parent: HWND, title_keyword: &str) -> Option<HWND> {
    let keyword_utf16: Vec<u16> = title_keyword.encode_utf16().collect();
    let mut ctx = ChildSearchContext {
        keyword: keyword_utf16,
        found: null_mut(),
    };

    unsafe {
        EnumChildWindows(
            parent,
            Some(enum_child_button_callback),
            &mut ctx as *mut ChildSearchContext as LPARAM,
        );
    }

    if ctx.found.is_null() {
        None
    } else {
        Some(ctx.found)
    }
}

struct ChildSearchContext {
    keyword: Vec<u16>,
    found: HWND,
}

unsafe extern "system" fn enum_child_button_callback(hwnd: HWND, data: LPARAM) -> BOOL {
    let ctx = &mut *(data as *mut ChildSearchContext);
    let title = window_title(hwnd);
    let keyword = String::from_utf16_lossy(&ctx.keyword);

    if title.contains(&keyword) {
        ctx.found = hwnd;
        return 0;
    }

    1
}

fn click_button(hwnd: HWND) {
    unsafe {
        SendMessageA(hwnd, BM_CLICK, 0, 0);
    }
}
```

说明：`ChildSearchContext` 用 `keyword: Vec<u16>` 只是为了避免在回调里处理生命周期
和裸指针字符串的额外复杂度；这里直接用 `String::from_utf16_lossy` 转回 `String`
做包含匹配，逻辑简单直接，代价（一次小字符串分配）可以忽略。

- [ ] **Step 2: 运行 `cargo check` 验证编译通过**

Run: `cargo check`
Expected: 编译成功，只有项目原有的警告（如 `已将 music.mp3 复制到构建目录`），
没有新的错误或关于 `mcgs_control` 的警告/错误。

如果报 `unused` 警告（比如 `restart_lower_computer` 未被调用），这是预期的——
Task 3 会在 `main.rs` 里声明模块、在 `menu/start.rs` 里调用它，届时警告会消失。
如果此时 `cargo check` 报的是 **`E0433: failed to resolve` 或类型不匹配**这类
实质性错误，说明签名有误，需要对照本计划开头「关键 API 签名」章节修正。

- [ ] **Step 3: 提交**

```bash
git add src/mcgs_control.rs
git commit -m "feat: add mcgs_control window/button finder and click helper"
```

---

### Task 3: 接入 main.rs 模块声明

**Files:**
- Modify: `src/main.rs:12-20`

- [ ] **Step 1: 在模块声明列表中加入 `mcgs_control`**

当前 `src/main.rs` 第 12-20 行：

```rust
mod controllers;
mod enums;
mod file_monitor;
mod get_pot_player;
mod menu;
mod ocr;
mod router;
mod ui;
mod uitl;
```

改为（按字母顺序插入 `mcgs_control`，与其余声明保持一致的排序风格）：

```rust
mod controllers;
mod enums;
mod file_monitor;
mod get_pot_player;
mod mcgs_control;
mod menu;
mod ocr;
mod router;
mod ui;
mod uitl;
```

- [ ] **Step 2: 运行 `cargo check` 验证编译通过**

Run: `cargo check`
Expected: 编译成功。此时 `mcgs_control::restart_lower_computer` 仍未被任何地方
调用，`cargo check` 可能报 `warning: function is never used`——这是预期的，
Task 4 会消除这个警告。

- [ ] **Step 3: 提交**

```bash
git add src/main.rs
git commit -m "feat: declare mcgs_control module in main.rs"
```

---

### Task 4: 新增 CLI 菜单选项

**Files:**
- Modify: `src/menu/start.rs`

- [ ] **Step 1: 在 `use` 声明中加入 `mcgs_control`**

当前 `src/menu/start.rs` 第 1-4 行：

```rust
use crate::controllers::{self, file};
use crate::get_pot_player;
use std::io;
use std::io::Write;
```

改为：

```rust
use crate::controllers::{self, file};
use crate::get_pot_player;
use crate::mcgs_control;
use std::io;
use std::io::Write;
```

- [ ] **Step 2: 更新菜单打印文本，插入新选项，退出选项编号顺延**

当前 `src/menu/start.rs` 第 8-16 行：

```rust
        println!("1. get_play_list");
        println!("2. start_soft_server");
        println!("3. show mouse xy");
        println!("4. test");
        println!("5. 下载pot NT");
        println!("6. 下载pot ME");
        println!("7. 开始/停止采集微信聊天");
        println!("8. 退出");
```

改为：

```rust
        println!("1. get_play_list");
        println!("2. start_soft_server");
        println!("3. show mouse xy");
        println!("4. test");
        println!("5. 下载pot NT");
        println!("6. 下载pot ME");
        println!("7. 开始/停止采集微信聊天");
        println!("8. 重启MCGS下位机运行(停止+启动)");
        println!("9. 退出");
```

- [ ] **Step 3: 更新 `match` 分支，新增选项 8 的处理逻辑，退出分支改为匹配 "9"**

当前 `src/menu/start.rs` 第 68-77 行：

```rust
            "7" => {
                controllers::wechat_capture::toggle_wechat_capture();
            }
            "8" => {
                println!("退出程序。");
                break; // 退出循环
            }
            _ => {
                println!("无效的选择，请重新输入。");
            }
```

改为：

```rust
            "7" => {
                controllers::wechat_capture::toggle_wechat_capture();
            }
            "8" => {
                tokio::spawn(async {
                    mcgs_control::restart_lower_computer().await;
                });
            }
            "9" => {
                println!("退出程序。");
                break; // 退出循环
            }
            _ => {
                println!("无效的选择，请重新输入。");
            }
```

- [ ] **Step 4: 运行 `cargo check` 验证编译通过，且不再有 unused 警告**

Run: `cargo check`
Expected: 编译成功。此时 `mcgs_control::restart_lower_computer` 已被调用，
之前的 `never used` 警告应消失。确认输出中不再出现与 `mcgs_control` 相关的
warning。

- [ ] **Step 5: 提交**

```bash
git add src/menu/start.rs
git commit -m "feat: add CLI menu option to restart MCGS lower computer"
```

---

### Task 5: 手动验证（真实 MCGS 环境）

**Files:** 无代码修改，仅验证。

- [ ] **Step 1: 构建 debug 版本**

Run: `cargo build`
Expected: 构建成功，生成 `target\debug\hello_cargo.exe`。

- [ ] **Step 2: 确认 MCGS 环境已打开**

确认 `McgsSetPro.exe` 正在运行，且「下载配置」弹窗当前可见（标题栏显示
「下载配置」，与本次会话前面用 ScreenClaw 截图确认过的窗口一致）。如果弹窗
被关闭，需要先在 MCGS 组态环境里重新打开它（工具菜单或对应快捷操作，具体
入口以用户环境为准）。

- [ ] **Step 3: 运行程序并选择新菜单项**

Run: `target\debug\hello_cargo.exe`（debug 模式下不会触发 UAC 提权、不会启动
PotPlayer 监控循环，参照 `CLAUDE.md` 中 Debug vs Release 的说明）

在菜单提示出现后输入 `8` 并回车。

Expected 终端输出（顺序）：
```
已点击「停止运行」
已点击「启动运行」
```
（中间有约 2 秒停顿）

- [ ] **Step 4: 核对 MCGS「下载配置」弹窗的返回信息日志**

观察「下载配置」弹窗中的「返回信息」列表区域，应新增两条时间相近的记录，
形如：
```
YYYY-MM-DD HH:MM:SS 测试模拟环境
YYYY-MM-DD HH:MM:SS 停止下位机
YYYY-MM-DD HH:MM:SS 下位机退出运行状态
YYYY-MM-DD HH:MM:SS 测试模拟环境
YYYY-MM-DD HH:MM:SS 启动下位机通知已发送
```
如果这些记录没有出现，说明点击未生效，需要回到 Task 1/2 检查
`find_download_config_window`/`find_child_button` 的标题匹配逻辑（可能是窗口
标题、按钮标题的实际文本与假设不完全一致，需要用 ScreenClaw 或其他窗口查看
工具重新确认准确文本）。

- [ ] **Step 5: 测试「下载配置」窗口不存在时的错误处理**

关闭「下载配置」弹窗（或确保 `McgsSetPro.exe` 未运行），再次在菜单中输入 `8`。

Expected 终端输出：
```
未找到「下载配置」窗口，请确认 McgsSetPro 已打开
```
程序应正常返回菜单提示，不应 panic 或卡死。

- [ ] **Step 6: 退出程序**

在菜单中输入 `9`，确认程序正常退出（原先的退出编号 `8` 现在应触发
"无效的选择，请重新输入。"）。

---

## Self-Review 记录

- **Spec 覆盖检查**：设计文档中的「架构」「窗口与按钮查找」「主流程」
  「错误处理」「依赖」章节均已对应到 Task 1-4 的具体代码；「测试」章节的
  手动验证步骤对应 Task 5；设计文档明确的「非目标」（HTTP 路由、定时触发、
  Qt 模拟器画面操作）未在计划中出现，范围一致。
- **占位符检查**：未发现 TBD/TODO/"适当处理"等模糊描述，所有步骤含完整代码
  或精确命令。
- **类型一致性检查**：`find_download_config_window() -> Option<HWND>`、
  `find_child_button(parent: HWND, title_keyword: &str) -> Option<HWND>`、
  `click_button(hwnd: HWND)`、`restart_lower_computer() -> ()`（`async fn`）
  在 Task 1、2、4 中签名与调用处保持一致。
