# MCGS 下位机重启功能设计

日期：2026-07-07

## 背景

MCGS 组态软件的模拟运行环境由 `McgsSetPro.exe` 管理，其「下载配置」弹窗上有
「停止运行」「启动运行」等原生 Win32 按钮，用于控制下位机（模拟器）的运行状态。
此前通过 ScreenClaw 视觉工具手动验证过「先点停止运行、等待、再点启动运行」这一
操作序列可行。本设计将其固化为项目内的一个可重复调用的功能，通过原生 Win32
消息（而非坐标点击）驱动，不依赖窗口是否在前台、不依赖 ScreenClaw。

## 目标

- 在 `hello_cargo` 项目里新增一个功能：查找「下载配置」弹窗，依次点击其
  「停止运行」「启动运行」按钮，中间间隔 2 秒。
- 通过 CLI 菜单（`menu/start.rs`）触发。
- 每次执行时实时查找窗口和按钮（不依赖缓存的窗口句柄），避免 `McgsSetPro.exe`
  重启后句柄失效的问题。

## 非目标

- 不做 HTTP 路由触发、不做定时/自动触发（本次只要 CLI 菜单）。
- 不处理「下载配置」弹窗不存在时的自动重试或自动打开该窗口。
- 不涉及模拟器画面（Qt 渲染窗口，PID 26912 的 `mcgs_app.exe`）的操作——目标
  按钮所在的「下载配置」弹窗属于 `McgsSetPro.exe`，两者是不同进程。

## 架构

新建独立模块 `src/mcgs_control.rs`，与 `get_pot_player.rs`、
`controllers/wechat_capture.rs` 平级，职责单一：MCGS 下载配置窗口的查找与
按钮点击。

`main.rs` 增加：

```rust
mod mcgs_control;
```

`menu/start.rs` 的 `init_menu()` 增加一个新选项（原有的退出选项顺延编号）：

```
8. 重启MCGS下位机运行(停止+启动)
9. 退出
```

选中后与其他菜单项一致，用 `tokio::spawn` 触发异步任务：

```rust
"8" => {
    tokio::spawn(async {
        mcgs_control::restart_lower_computer().await;
    });
}
```

## 窗口与按钮查找

复用 `wechat_capture.rs` 中 `EnumWindows` + 回调查找顶层窗口的模式，再加一层
`EnumChildWindows` 查找直接子按钮：

1. `find_download_config_window() -> Option<HWND>`
   用 `EnumWindows` 遍历顶层可见窗口，标题包含「下载配置」即命中。

2. `find_child_button(parent: HWND, title_keyword: &str) -> Option<HWND>`
   用 `EnumChildWindows` 遍历 `parent` 的直接子窗口，标题包含
   `title_keyword` 即命中（不需要递归到孙窗口，从实测窗口列表看按钮都是
   「下载配置」的直接子窗口）。

3. `click_button(hwnd: HWND)`
   通过 `SendMessageA(hwnd, BM_CLICK, 0, 0)` 发送标准 Win32 按钮点击消息。
   这是原生 Button 控件的标准点击方式，不需要坐标、不需要窗口在前台、
   比坐标点击更可靠。`BM_CLICK` 是 `winapi::um::winuser` 已提供的常量。

## 主流程

```rust
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

    tokio::time::sleep(Duration::from_secs(2)).await;

    // 重新查找一次，防止等待期间窗口被关闭重开、句柄失效
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
```

两次点击之间不复用第一次拿到的 `HWND`，而是重新执行完整查找，因为
`EnumWindows`/`EnumChildWindows` 开销很小，重新查找的代价可以忽略，但能避免
2 秒等待期间窗口被关闭重开导致句柄失效的问题。

## 错误处理

- 找不到「下载配置」窗口，或找不到目标按钮：`eprintln!` 打印错误信息后直接
  返回，不做自动重试、不 panic。与 `get_pot_player.rs` 现有的错误处理风格
  一致（打印后放弃，不中断主程序）。

## 测试

- Win32 窗口交互无法在单元测试中直接验证（依赖真实的 MCGS 进程/窗口）。
- 对纯逻辑部分（如标题关键字匹配函数）编写单元测试，参照
  `controllers/me.rs` 中 `longest_verification_code` 等函数的测试写法。
- 手动验证：运行程序，选择新菜单项，观察 MCGS「下载配置」弹窗的「返回信息」
  日志出现「停止下位机」「启动下位机通知已发送」等记录。

## 依赖

无需新增 crate 依赖，`winapi` 已在 `Cargo.toml` 中启用 `winuser` feature，
`EnumWindows`、`EnumChildWindows`、`BM_CLICK`、`GetWindowTextW` 均已可用。
