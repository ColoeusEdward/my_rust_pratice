use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
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
                        // 清理旧备份：保留最新10个 + 体积异常的旧文件
                        cleanup_old_backups(WATCH_FILE, 10);
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

/// 清理旧备份文件，保留最新 max_keep 个 + 体积大于最新备份的旧文件
fn cleanup_old_backups(file_path: &str, max_keep: usize) {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let prefix = format!("{}.backup.", file_name);

    // 收集所有备份文件及其大小
    let mut backups: Vec<(PathBuf, u64)> = Vec::new();
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let entry_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if entry_name.starts_with(&prefix) {
                if let Ok(metadata) = entry.metadata() {
                    backups.push((entry_path, metadata.len()));
                }
            }
        }
    }

    // 数量未超标，无需清理
    if backups.len() <= max_keep {
        return;
    }

    // 按文件名倒序（时间戳格式天然支持字典序 = 时间序），最新在前
    backups.sort_by(|a, b| b.0.file_name().cmp(&a.0.file_name()));

    // 最新备份的体积
    let newest_size = backups[0].1;

    // 保留条件：前 max_keep 个 或 体积大于最新备份
    let keep_set: std::collections::HashSet<PathBuf> = backups
        .iter()
        .enumerate()
        .filter(|(i, (_, size))| *i < max_keep || *size > newest_size)
        .map(|(_, (p, _))| p.clone())
        .collect();

    // 删除不需要保留的文件
    for (p, _) in &backups {
        if !keep_set.contains(p) {
            match fs::remove_file(p) {
                Ok(()) => println!("🗑️ [file_monitor] 已删除旧备份: {}", p.display()),
                Err(e) => eprintln!("❌ [file_monitor] 删除旧备份失败 {}: {}", p.display(), e),
            }
        }
    }
}
