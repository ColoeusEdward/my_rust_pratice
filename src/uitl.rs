use std::str::FromStr;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime,Local,Datelike};

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