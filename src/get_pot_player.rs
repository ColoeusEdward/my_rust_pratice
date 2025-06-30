use crate::enums;
use crate::enums::get_upload_server_path;
use crate::enums::MyData;
use crate::enums::PlayInfo;
use crate::enums::PLAY_LIST_SERVER_PATH_ME;
use crate::enums::PLAY_LIST_SERVER_PATH_NT;
use crate::uitl;
use crate::uitl::get_bv_info;
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
    map.insert("pgTime", info.pg_time.to_string());
    map.insert("ts", info.ts.to_string());
    map.insert("playBv", info.play_bv);
    map.insert("playTime", info.play_time);
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
    println!("hwnd null: {}", hwnd.is_null());
    if !hwnd.is_null() {
        let info = unsafe {
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
                time: time_str, //播放进度时间
                pg_time: res,   //ms的播放进度时间
                ts: 0,          //日期
                play_bv: "".to_string(),
                play_time: "".to_string(),
            }
            // println!("Window progress time: {} ", time_str);
        };
        let fk = upload_info(info).await;

        // if info.name.contains("雾氧") {
        //     //这一块其实没有意义, 因为播放器没关闭文件不会更新
        //     // let (bv, play_time,now_title) = get_pot_first_info();
        //     // info.play_bv = bv;
        //     // info.play_time = play_time;

        //     // let split_str = info.name.split(" ").collect::<Vec<_>>();
        //     // let time_str_list = &split_str[1..3];
        //     // info.ts = uitl::transform_wuyang_time_ts(time_str_list);
        //     let fk = upload_info(info).await;
        // }
    }

    // println!("hwnd null: {}", hwnd.);

    // unsafe {
    //     enum_windows_callback(hwnd, 0);
    // }
}

pub fn save_pot_play_info2() {

}

pub fn get_pot_first_info() -> (String, String, String) {
    let path = enums::get_list_local_list();
    let line_first = uitl::read_lines(path, 0, 37);

    let bv = "";
    let play_time = "";
    let play_title = "";
    let mut line_str = String::new();
    match line_first {
        Ok(str) => line_str = str,
        Err(e) => eprintln!("读取文件失败: {}", e),
    }
    let line_str = line_str;
    let line_str = line_str.split("@").collect::<Vec<_>>();

    let line_str_first: &[&str] = &line_str[1..3];
    // println!("🪵 [get_pot_player.rs:151]~ token ~ \x1b[0;32mline_str_first\x1b[0m = {}", line_str_first.join(""));
    let bv = line_str_first[0].split("/").collect::<Vec<_>>();
    let bv = bv.last().unwrap();
    let play_time = line_str_first[1].split("=").last().unwrap();

    let line_str_after = &line_str[4..];
    // println!("🪵 [get_pot_player.rs:156]~ token ~ \x1b[0;32mline_str_after\x1b[0m = {}", line_str_after.join(""));
    let index = line_str_after.iter().position(|&x| x.contains(bv)); // 返回 Some(2)
    let index = match index {
        Some(value) => value,
        None => {
            eprintln!("前排没有搜到bv号: ",);
            0
        }
    };
    let (bv2, title): (&str, &str) = if index > 0 {
        println!(
            "🪵 [get_pot_player.rs:164]~ token ~ \x1b[0;32mindex\x1b[0m = {}",
            index
        );
        let line_bv_title = &line_str_after[index..index + 2];
        let bv2 = line_bv_title[0].split("/").collect::<Vec<_>>();
        let bv2 = bv2.last().unwrap();
        let title = line_bv_title[1].split("*").last().unwrap();
        (bv2, title)
    } else {
        ("", "")
    };
    // println!("🪵 [get_pot_player.rs:166]~ token ~ \x1b[0;32mbv2\x1b[0m = {}", bv2);
    // println!("🪵 [get_pot_player.rs:169]~ token ~ \x1b[0;32mtitle\x1b[0m = {}", title);
    // println!("🪵 [get_pot_player.rs:153]~ token ~ \x1b[0;32mbv\x1b[0m = {}", bv);
    // println!("🪵 [get_pot_player.rs:155]~ token ~ \x1b[0;32mplay_time\x1b[0m = {}", play_time);

    (
        bv.trim_end().to_string(),
        play_time.trim_end().to_string(),
        title.trim_end().to_string(),
    )
}

pub fn search_front_now_play() {
    let path = enums::get_list_local_list();
    let line_first = uitl::read_lines(path, 5, 25);
    let bv = "";
    let play_time = "";
    let play_title = "";
    // let mut line_str = String::new();
    //     match line_first {
    //         Ok(idx) => idx,
    //         Err(e) => {eprintln!("读取文件失败: {}", e);String::new()},
    //     }
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
    println!(
        "🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mpath\x1b[0m = {}",
        path
    );
    let line = uitl::read_last_lines(path, 6); //最后一排
    if !line.is_err() {
        let line = line.unwrap().join("");
        let line = line.split("@").collect::<Vec<_>>();
        let line = line[2].split("*").collect::<Vec<_>>();
        let title = line[2];
        println!(
            "🪵 [get_pot_player.rs:149]~ token ~ \x1b[0;32mtitle\x1b[0m = {}",
            title
        );
        let (bv, play_time, now_title) = get_pot_first_info(); //前排
        let bv_info = get_bv_info(&bv).await;
        // println!("🪵 [get_pot_player.rs:228]~ token ~ \x1b[0;32mnow_title\x1b[0m = {}", now_title);

        // let split_str = title.split(" ").collect::<Vec<_>>();
        // let time_str_list = &split_str[1..3]; //标题里的时间
        // let ts = uitl::transform_wuyang_time_ts(time_str_list); //日期

        // let split_str = now_title.split(" ").collect::<Vec<_>>(); //
        // let time_str_list = &split_str[1..3];
        // let now_play_ts = uitl::transform_wuyang_time_ts(time_str_list); //日期
         let ts = "";
         let now_play_ts = bv_info.pubdate;
        println!(
                "🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mtime_str_list\x1b[0m = {} {} {} {} {} {}",
                ts, title, bv, play_time, now_play_ts, now_title
            );
        // let is_new = check_play_list_new(
        //     title.to_string(),
        //     ts,
        //     &bv,
        //     &play_time,
        //     now_play_ts,
        //     &now_title,
        // )
        // .await;

        // if is_new {
            upload_play_list().await.unwrap();
        // } else {
        //     down_server_play_list().await.unwrap();
        // }
        // println!(
        //     "🪵 [get_pot_player.rs:146]~ token ~ \x1b[0;32mis_new\x1b[0m = {}",
        //     is_new
        // );

        // if title.contains("雾氧") {
        //     let (bv, play_time,now_title) = get_pot_first_info();
        //     // println!("🪵 [get_pot_player.rs:228]~ token ~ \x1b[0;32mnow_title\x1b[0m = {}", now_title);

        //     let split_str = title.split(" ").collect::<Vec<_>>();
        //     let time_str_list = &split_str[1..3];  //标题里的时间
        //     let ts = uitl::transform_wuyang_time_ts(time_str_list); //日期

        //     let split_str = now_title.split(" ").collect::<Vec<_>>();//
        //     let time_str_list = &split_str[1..3];
        //     let now_play_ts = uitl::transform_wuyang_time_ts(time_str_list); //日期
        //     println!(
        //         "🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mtime_str_list\x1b[0m = {} {} {} {} {} {}",
        //         ts, title, bv, play_time, now_play_ts, now_title
        //     );
        //     let is_new = check_play_list_new(title.to_string(), ts, &bv, &play_time , now_play_ts,&now_title).await;
        //     if is_new {
        //         upload_play_list().await.unwrap();
        //     } else {
        //         down_server_play_list().await.unwrap();
        //     }
        //     println!(
        //         "🪵 [get_pot_player.rs:146]~ token ~ \x1b[0;32mis_new\x1b[0m = {}",
        //         is_new
        //     );

        //     // down_server_play_list().await.unwrap();
        // }
    } else {
        match line {
            Err(e) => {
                println!(
                    "🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32me\x1b[0m = {}",
                    e
                );
            }
            Ok(line) => {
                println!(
                    "🪵 [get_pot_player.rs:142]~ token ~ \x1b[0;32mline\x1b[0m = {}",
                    "playlist 读取成功"
                );
            }
        }
    }
    // let metadata = fs::metadata(path)?; // 获取元数据，错误时直接返回
    // println!("文件 '{}' 的大小: {} 字节", path, metadata.len());
    if is_runing {
        Command::new("powershell")
            .args(&[
                "-Command",
                format!(r#"start "{}""#, enums::get_pot_location()).as_ref(),
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

async fn check_play_list_new(
    title: String,
    ts: i64,
    bv: &str,
    play_time: &str,
    now_play_ts: i64,
    now_title: &str,
) -> bool {
    //title最末端标题,ts最末端日期ts
    let client = reqwest::Client::new();
    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("name", title);
    map.insert("ts", ts.to_string());
    map.insert("playBv", bv.to_string());
    map.insert("playTime", play_time.to_string());
    map.insert("playTs", now_play_ts.to_string());
    map.insert("nowTitle", now_title.to_string());

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

    let server_path = get_upload_server_path();
    // 创建 multipart 表单
    let multipart = reqwest::multipart::Form::new()
        // 添加文件，指定字段名"file"
        .file("files", file_path)
        .await
        .unwrap();
    // 可选：添加其他文本字段

    // 发送 POST 请求
    let response = reqwest::Client::new()
        .post(server_path)
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

pub async fn down_server_play_list(from_type: String) -> Result<(), reqwest::Error> {
    let server_path = if from_type == "nt" {
        PLAY_LIST_SERVER_PATH_NT
    } else if from_type == "me" {
        PLAY_LIST_SERVER_PATH_ME
    } else {
        PLAY_LIST_SERVER_PATH_NT
    };
    // 文件保存路径
    let save_path = Path::new(enums::get_list_local_list());

    // 发送 GET 请求
    let response = reqwest::get(server_path).await?;

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
