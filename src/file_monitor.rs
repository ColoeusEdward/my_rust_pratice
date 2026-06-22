use chrono::Local;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

const WATCH_FILE: &str = r"C:\Users\11038\.claude.json";
const POLL_INTERVAL: u64 = 10; // 每10秒检查一次

/// 监控文件是否被修改，一旦变化就备份
pub async fn start_monitoring() {
    // 初始读取文件的 modified 时间戳
    let mut last_modified = get_modified_time(WATCH_FILE);

    loop {
        sleep(Duration::from_secs(POLL_INTERVAL)).await;

        let current_modified = get_modified_time(WATCH_FILE);

        match (last_modified, current_modified) {
            (Some(prev), Some(curr)) if curr > prev => {
                // 文件被修改，执行备份
                match backup_file(WATCH_FILE) {
                    Ok(backup_path) => {
                        println!(
                            "📄 [file_monitor] 检测到文件修改，已备份到: {}",
                            backup_path
                        );
                    }
                    Err(e) => {
                        eprintln!("❌ [file_monitor] 备份失败: {}", e);
                    }
                }
                last_modified = Some(curr);
            }
            (None, Some(_)) => {
                // 文件之前不存在，现在出现了，记录时间戳（不备份新文件）
                println!("📄 [file_monitor] 检测到新文件，开始监控");
                last_modified = current_modified;
            }
            (Some(_), None) => {
                // 文件被删除了
                println!("📄 [file_monitor] 文件已被删除，等待重新出现");
                last_modified = None;
            }
            _ => {
                // 无变化 或 文件始终不存在
            }
        }
    }
}

/// 获取文件的最后修改时间
fn get_modified_time(path: &str) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// 备份文件到同目录，文件名加时间戳后缀
fn backup_file(file_path: &str) -> Result<String, std::io::Error> {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}.backup.{}", file_name, timestamp);
    let backup_path = parent.join(backup_name);

    fs::copy(file_path, &backup_path)?;

    Ok(backup_path.to_string_lossy().to_string())
}
