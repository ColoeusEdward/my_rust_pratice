// use std::str::FromStr;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
use std::fs::File;
use std::io::{Seek, SeekFrom, Read};
use std::path::Path;
use winapi::um::winbase::GetUserNameA;
use std::os::raw::c_ulong;

pub fn format_duration_extended(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1000;
    // let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    // let remaining_ms = milliseconds % 1000;

    // format!(
    //     "{}天 {:02}:{:02}:{:02}.{:03}",
    //     days, hours, minutes, seconds, remaining_ms
    // )
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn transform_wuyang_time_ts(arr: &[&str]) -> i64 {
    // 解析日期和时间字符串
    let current_year = "2025";  //统一假设为2025, 跨年判断在服务器做
    let str = format!("{} {}", current_year, arr[0]);
    let date = NaiveDate::parse_from_str(&str, "%Y %m-%d").expect("日期解析失败");
    let time = NaiveTime::parse_from_str(arr[1], "%H_%M_%S").expect("时间解析失败");
    // 合并日期和时间
    let datetime = NaiveDateTime::new(date, time);
    // 转换为时间戳
    datetime.and_utc().timestamp()
}

pub fn read_last_lines<P: AsRef<Path>>(path: P,num:usize) -> std::io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    println!("🪵 [uitl.rs:38]~ token ~ \x1b[0;32mfile_size\x1b[0m = {}", file_size);

    let mut buffer = Vec::new();
    let mut pos = file_size; // 从末尾开始
    let mut line_count = 0;

    // 反向逐字节读取
    while pos > 0 && line_count < num {
        pos -= 1;
        file.seek(SeekFrom::Start(pos))?;
        let mut byte = [0u8; 1];
        file.read_exact(&mut byte)?;

        if byte[0] == b'\n' {
            line_count += 1;
            buffer.push(b'@');
        }

        buffer.push(byte[0]);
    }

    // 处理最后一行无换行符的情况
    if line_count < num && pos == 0 {
        // line_count += 1;
    }

    buffer.reverse(); // 恢复正向顺序
    let content = String::from_utf8_lossy(&buffer).into_owned();
    let lines: Vec<&str> = content.lines().collect();

    // 提取最后两行
    let start = if lines.len() >= num { lines.len() - num } else { 0 };
    Ok(lines[start..].iter().map(|s| s.to_string()).collect())
}

pub fn get_sys_username() -> String {
    let mut username = [0u8; 256];
    let mut size: c_ulong = username.len() as c_ulong;
    unsafe {
        GetUserNameA(username.as_mut_ptr() as *mut i8, &mut size);
    }
    String::from_utf8_lossy(&username).to_string()
}