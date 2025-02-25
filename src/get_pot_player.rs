use crate::enums;
use crate::enums::PlayInfo;
use crate::uitl;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, WPARAM};
// use winapi::shared::minwindef::{DWORD, HINSTANCE, LPARAM, LPVOID, UINT, WPARAM};
use winapi::shared::windef::HWND;
// use tokio::runtime::Runtime;
use winapi::um::winuser::{
    FindWindowExA, GetWindowTextW, IsWindowVisible, RealGetWindowClassA, SendMessageA,
};

// 回调函数，用于枚举窗口
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, _: LPARAM) -> i32 {
    // 检查窗口是否可见
    if IsWindowVisible(hwnd) != 0 {
        // println!("is visiiable ",);
        // 获取窗口标题
        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32);

        if len > 0 {
            // 转换为 Rust 字符串
            let os_string = OsString::from_wide(&text[..len as usize]);

            if let Some(title) = os_string.to_str() {
                // && title.unwrap().contains("PotPlayer")
                if title.to_lowercase().contains("- potplayer") {
                    let mut buffer = [0i8; 256];
                    let len = RealGetWindowClassA(hwnd, buffer.as_mut_ptr(), buffer.len() as u32);
                    let class_name = CStr::from_bytes_with_nul_unchecked(
                        std::slice::from_raw_parts(buffer.as_ptr() as *const u8, len as usize + 1),
                    )
                    .to_string_lossy();
                    println!("Window Title: {} class: {}", title, class_name);
                }
            }
        }
    }
    1 // 继续枚举
}

fn get_win_title(hwnd: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    unsafe {
        let len = GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32);
        if len > 0 {
            // 转换为 Rust 字符串
            let os_string = OsString::from_wide(&text[..len as usize]);
            if let Some(title) = os_string.to_str() {
                return title.to_string();
            }
        }
        String::new()
    }
}

async fn upload_info(info: PlayInfo) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut map = HashMap::new();
    map.insert("name", info.name);
    map.insert("time", info.time);
    map.insert("ts", info.ts.to_string());
    let res = client
        .post("https://meamoe.top/koa/newCen/free/savePotInfo")
        .json(&map)
        .send()
        .await?;

    if res.status().is_success() {
    } else {
        // return println!(format!("请求失败：{}", res.status())); // 格式化错误信息为 String
    }
    println!(
        "🪵 [get_pot_player.rs:72]~ token ~ \x1b[0;32mres\x1b[0m = {}",
        res.text().await?
    );

    Ok(())
}

pub async fn save_pot_play_info() {
    let potplayer_class_name = CString::new("PotPlayer64").unwrap(); // Or the actual class name
    let potplayer_window_title = CString::new("PotPlayer").unwrap(); // Or part of the title

    let hwnd = unsafe {
        FindWindowExA(
            null_mut(), // Parent window (null for top-level windows)
            null_mut(), // Child window (null to start from the top)
            potplayer_class_name.as_ptr(),
            null_mut(),
        )
    };
    // println!("hwnd null: {}", hwnd.is_null());
    if !hwnd.is_null() {
        let mut info = unsafe {
            let res = SendMessageA(
                hwnd,
                enums::REQ_TYPE,
                enums::POT_GET_PROGRESS_TIME as WPARAM,
                0 as LPARAM,
            );
            let time_str = uitl::format_duration_extended(res as u64);
            let title = get_win_title(hwnd);
            enums::PlayInfo {
                name: title.replace(" - PotPlayer", ""),
                time: time_str,
                ts:0
            }
            // println!("Window progress time: {} ", time_str);
        };
        if info.name.contains("雾氧") {
            let split_str = info.name.split(" ").collect::<Vec<_>>();
            let time_str_list = &split_str[1..3];
            info.ts = uitl::transform_wuyang_time_ts(time_str_list);
            let fk = upload_info(info).await;
        }
    }

    // println!("hwnd null: {}", hwnd.);

    // unsafe {
    //     enum_windows_callback(hwnd, 0);
    // }
}

// pub fn start() {
//     unsafe {
//         EnumWindows(Some(enum_windows_callback), 0);
//     }
// }

// pub fn start2() {
// }
