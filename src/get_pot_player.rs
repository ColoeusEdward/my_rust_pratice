use crate::enums;
use crate::enums::MyData;
use crate::enums::PlayInfo;
use crate::uitl;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::process::Command;
use std::ptr::null_mut;
use std::time::Duration;
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::um::winuser::WM_CLOSE;
// use winapi::shared::minwindef::{DWORD, HINSTANCE, LPARAM, LPVOID, UINT, WPARAM};
use futures::StreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;
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
    let hwnd = get_potplayer_hwnd().await;
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
                ts: 0,
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

pub async fn get_player_list_file() -> Result<(), std::io::Error> {
    let hwnd = get_potplayer_hwnd().await;
    let is_runing = !hwnd.is_null();
    if is_runing {
        unsafe {
            let res = SendMessageA(hwnd, WM_CLOSE, 0 as WPARAM, 0 as LPARAM);
            tokio::time::sleep(Duration::from_secs(2)).await;
        };
    } else {
        // return Ok(());
    }

    let path = enums::get_list_local_list();
    println!("🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mpath\x1b[0m = {}", path);
    let line = uitl::read_last_lines(path, 6);
    if !line.is_err() {
        let line = line.unwrap().join("");
        let line = line.split("@").collect::<Vec<_>>();
        let line = line[2].split("*").collect::<Vec<_>>();
        let title = line[2];
        println!("🪵 [get_pot_player.rs:149]~ token ~ \x1b[0;32mtitle\x1b[0m = {}", title);
        if title.contains("雾氧") {
            let split_str = title.split(" ").collect::<Vec<_>>();
            let time_str_list = &split_str[1..3];
            let ts = uitl::transform_wuyang_time_ts(time_str_list);
            println!("🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mtime_str_list\x1b[0m = {} {}",ts, title);
            let is_new = check_play_list_new(title.to_string(), ts).await;
            if is_new {
                upload_play_list().await.unwrap();
            } else {
                down_server_play_list().await.unwrap();
            }
            println!("🪵 [get_pot_player.rs:146]~ token ~ \x1b[0;32mis_new\x1b[0m = {}", is_new);
            // down_server_play_list().await.unwrap();
        }
    }else{
        match line {
            Err(e) => {
                println!("🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32me\x1b[0m = {}", e);
            }
            Ok(line) => {
                println!("🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mline\x1b[0m = {}", "playlist 读取成功");
            }
        }
    }
    // let metadata = fs::metadata(path)?; // 获取元数据，错误时直接返回
    // println!("文件 '{}' 的大小: {} 字节", path, metadata.len());
    if is_runing {
        Command::new("powershell")
            .args(&[
                "-Command",
                format!(r#"start "{}""#,enums::get_pot_location()).as_ref()
            ])
            .spawn()
            .expect("执行失败");
        println!(
            "🪵 [get_pot_player.rs:175]~ token ~ \x1b[0;32mCommand\x1b[0m = {}",
            "player启动命令执行"
        );
    }

    

    Ok(())
}

async fn check_play_list_new(title: String, ts: i64) -> bool {
    let client = reqwest::Client::new();
    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("name", title);
    map.insert("ts", ts.to_string());

    let res = client
        .post("https://meamoe.top/koa/newCen/free/checkPlayListNew")
        .json(&map)
        .send()
        .await;

    if res.is_ok() {
        let te: MyData<bool> = res.unwrap().json().await.unwrap();
        // let te:MyData = serde_json::from_str(te).unwrap();
        return te.data;
    } else {
        return false;
        // return println!(format!("请求失败：{}", res.status())); // 格式化错误信息为 String
    }
}

async fn upload_play_list() -> Result<(), Result<(), reqwest::Error>> {
    // 本地文件路径
    let file_path = Path::new(enums::get_list_local_list());

    // 创建 multipart 表单
    let multipart = reqwest::multipart::Form::new()
        // 添加文件，指定字段名"file"
        .file("files", file_path)
        .await
        .unwrap();
    // 可选：添加其他文本字段

    // 发送 POST 请求
    let response = reqwest::Client::new()
        .post("https://meamoe.top/koa/mv_upload/free/uploadTemp")
        .multipart(multipart)
        .send()
        .await;

    println!("上传state: {}", response.as_ref().unwrap().status());
    // 检查响应状态码
    if response.is_ok() && response.as_ref().unwrap().status().is_success() {
        let text = response.unwrap().text().await.unwrap();
        println!("服务器响应: {}", text);
    } else {
        println!("上传失败，状态码: {}", response.unwrap().status());
    }

    Ok(())
}

async fn down_server_play_list() -> Result<(), reqwest::Error> {
    // 文件保存路径
    let save_path = Path::new(enums::get_list_local_list());

    // 发送 GET 请求
    let response = reqwest::get(enums::PLAY_LIST_SERVER_PATH).await?;

    // 检查响应状态码
    if !response.status().is_success() {
        println!("下载失败，状态码: {}", response.status());
        return Err(response.error_for_status().unwrap_err());
    }

    // 将响应内容保存到文件
    let mut file = tokio::fs::File::create(save_path).await.unwrap();
    let mut stream = response.bytes_stream();

    // 逐块写入文件
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await.unwrap();
    }

    println!("文件已成功下载并保存到: {}", save_path.display());
    Ok(())
}

async fn get_potplayer_hwnd() -> HWND {
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
    return hwnd;
}
