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
