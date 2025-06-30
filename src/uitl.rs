// use std::str::FromStr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Read, Seek, SeekFrom};
use std::os::raw::c_ulong;
use std::path::Path;
use winapi::um::winbase::GetUserNameA;
use screenshots::Screen;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::process::{Command, Output};
use std::error::Error;

use crate::enums::{BvInfo, MyData};






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
    let current_year = "2025"; //统一假设为2025, 跨年判断在服务器做
    let str = format!("{} {}", current_year, arr[0]);
    let time_str = arr[1].trim_end();
    let date = NaiveDate::parse_from_str(&str, "%Y %m-%d").expect("日期解析失败");
    let time = NaiveTime::parse_from_str(time_str, "%H_%M_%S").expect("时间解析失败");
    // 合并日期和时间
    let datetime = NaiveDateTime::new(date, time);
    // 转换为时间戳
    datetime.and_utc().timestamp()
}

pub fn read_last_lines<P: AsRef<Path>>(path: P, num: usize) -> std::io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    println!(
        "🪵 [uitl.rs:38]~ token ~ \x1b[0;32mfile_size\x1b[0m = {}",
        file_size
    );

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
    let start = if lines.len() >= num {
        lines.len() - num
    } else {
        0
    };
    Ok(lines[start..].iter().map(|s| s.to_string()).collect())
}

pub fn read_first_lines<P: AsRef<Path>>(path: P, num: usize) -> std::io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    // println!("🪵 [uitl.rs:38]~ token ~ \x1b[0;32mfile_size\x1b[0m = {}", file_size);

    let mut buffer = Vec::new();
    let mut pos = 0;
    let mut line_count = 0;

    // 正向逐字节读取
    while pos < file_size-1 && line_count < num {
        pos += 1;
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
    if line_count < num && pos == file_size {
        // line_count += 1;
    }

    // buffer.reverse(); // 恢复正向顺序
    let content = String::from_utf8_lossy(&buffer).into_owned();
    let lines: Vec<&str> = content.lines().collect();
    // println!("🪵 [uitl.rs:108]~ token ~ \x1b[0;32mlines.len()\x1b[0m = {} {}", lines.len(),content);

    // 提取最后n行
    let start = if lines.len() >= num {
        lines.len() - num
    } else {
        0
    };
    Ok(lines[start..].iter().map(|s| s.to_string()).collect())
}

pub fn read_lines<P: AsRef<Path>>(
    path: P,
    start_num: usize,
    num: usize,
) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let end_num = start_num + num;
    // println!("🪵 [uitl.rs:38]~ token ~ \x1b[0;32mfile_size\x1b[0m = {}", file_size);

    let mut buffer = Vec::new();
    let mut pos = 0;
    let mut line_count = 0;

    // 正向逐字节读取
    while pos < file_size-1 && line_count < end_num {
        pos += 1;
        file.seek(SeekFrom::Start(pos))?;
        let mut byte = [0u8; 1];
        file.read_exact(&mut byte)?;

        if byte[0] == b'\n' {
            line_count += 1;
            if line_count >= start_num {
                buffer.push(b'@');
            }
        }
        if line_count >= start_num {
            buffer.push(byte[0]);
        }
    }

    // 处理最后一行无换行符的情况
    if line_count < num && pos == file_size {
        // line_count += 1;
    }

    // buffer.reverse(); // 恢复正向顺序
    let content = String::from_utf8_lossy(&buffer).into_owned();
    let lines: Vec<&str> = content.lines().collect();
  

    // 提取最后n行
    // let start = if lines.len() >= num {
    //     lines.len() - num
    // } else {
    //     0
    // };
    let start = 0;
    // let res:Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    // println!("🪵 [uitl.rs:174]~ token ~ \x1b[0;32mres\x1b[0m = {}",&res.join("!!"));
    Ok(content)
}

pub fn get_sys_username() -> String {
    let mut username = [0u8; 256];
    let mut size: c_ulong = username.len() as c_ulong;
    unsafe {
        GetUserNameA(username.as_mut_ptr() as *mut i8, &mut size);
    }
    String::from_utf8_lossy(&username).to_string()
}

/// 查找字符串坐标（行号从1开始，列号从0开始）
pub fn find_string_coordinates<P: AsRef<Path>>(
    file_path: P,
    target: &str,
) -> Result<Vec<(usize, usize)>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut positions = Vec::new();
    let target_bytes = target.as_bytes();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        // line.find(target);
        let line_bytes = line.as_bytes();

        // 遍历每一字节查找匹配
        let mut pos = 0;
        while pos <= line_bytes.len().saturating_sub(target_bytes.len()) {
            if line_bytes[pos..].starts_with(target_bytes) {
                positions.push((line_num + 1, pos + 1)); // 列号从1开始
                pos += target_bytes.len();
            } else {
                pos += 1;
            }
        }
    }

    Ok(positions)
}

pub fn screen_shot() -> () {
    let start = Instant::now();
    let screens = Screen::all().unwrap();
    let ts= SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    for screen in screens {
        println!("Capturer {:?}", screen);
        let image = screen.capture().unwrap();
        image
            .save(format!("D:\\MCode\\Rust/{}-{}.png", screen.display_info.id,ts))
            .unwrap();

        // image = screen.capture_area(300, 300, 300, 300).unwrap();
        // image
        //     .save(format!("target/{}-2.png", screen.display_info.id))
        //     .unwrap();
    }


    println!("运行耗时: {:?}", start.elapsed());
    // Ok("截图成功".to_string())
}


pub fn ping(target: &str) -> Result<Output, Box<dyn Error>> {
    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .arg(target)
            .output()?
    } else {
        Command::new("ping")
            .arg("-c") // 指定发送的包数量 (Linux/macOS)
            .arg("4")
            .arg(target)
            .output()?
    };

    Ok(output)
}

pub async fn get_bv_info(bvid: &str) -> BvInfo {
    let url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid);
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await;
    if res.is_ok() {
        let data: MyData<BvInfo> = res.unwrap().json().await.unwrap();
        // let resp = res.unwrap().json::<HashMap<String, String>>().await;
        // let resp = match resp {
        //     Ok(resp) => resp,
        //     Err(e) => {
        //         println!("请求失败,返回json错误：{}", e);
        //         return map;
        //     }
        // };
        return data.data;
    }else{
        return BvInfo::new();
    }
}