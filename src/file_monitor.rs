use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

const WATCH_FILES: [&str; 3] = [
    r"C:\Users\11038\.claude.json",
    r"C:\Users\11038\.claude\settings.json",
    r"C:\Users\11038\.config\opencode\opencode.json",
];
const POLL_INTERVAL: u64 = 10; // 每10秒检查一次
const IMAGE_CLEANUP_DIR: &str = r"D:\MCode\Rust";
const IMAGE_KEEP_COUNT: usize = 10;
const IMAGE_CLEANUP_INTERVAL: u64 = 24 * 60 * 60;

/// 监控文件是否被修改，一旦变化就备份
pub async fn start_monitoring() {
    // 初始读取每个文件的 modified 时间戳
    let mut watched_files: Vec<(&str, Option<SystemTime>)> = WATCH_FILES
        .iter()
        .map(|file_path| (*file_path, get_modified_time(file_path)))
        .collect();

    loop {
        sleep(Duration::from_secs(POLL_INTERVAL)).await;

        for (file_path, last_modified) in watched_files.iter_mut() {
            let current_modified = get_modified_time(file_path);

            match (*last_modified, current_modified) {
                (Some(prev), Some(curr)) if curr > prev => {
                    // 文件被修改，执行备份
                    match backup_file(file_path) {
                        Ok(backup_path) => {
                            println!(
                                "📄 [file_monitor] 检测到文件修改: {}，已备份到: {}",
                                file_path, backup_path
                            );
                            // 清理旧备份：保留最新10个 + 体积异常的旧文件
                            cleanup_old_backups(file_path, 10);
                        }
                        Err(e) => {
                            eprintln!("❌ [file_monitor] 备份失败: {}: {}", file_path, e);
                        }
                    }
                    *last_modified = Some(curr);
                }
                (None, Some(_)) => {
                    // 文件之前不存在，现在出现了，记录时间戳（不备份新文件）
                    println!("📄 [file_monitor] 检测到新文件，开始监控: {}", file_path);
                    *last_modified = current_modified;
                }
                (Some(_), None) => {
                    // 文件被删除了
                    println!(
                        "📄 [file_monitor] 文件已被删除，等待重新出现: {}",
                        file_path
                    );
                    *last_modified = None;
                }
                _ => {
                    // 无变化 或 文件始终不存在
                }
            }
        }
    }
}

/// 启动每日图片清理任务：只检查 D:\MCode\Rust 第一层图片，保留最新10个。
pub async fn start_daily_image_cleanup() {
    loop {
        if let Err(e) = cleanup_old_images_in_dir(Path::new(IMAGE_CLEANUP_DIR), IMAGE_KEEP_COUNT) {
            eprintln!(
                "❌ [file_monitor] 图片清理失败 {}: {}",
                IMAGE_CLEANUP_DIR, e
            );
        }

        sleep(Duration::from_secs(IMAGE_CLEANUP_INTERVAL)).await;
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
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

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
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
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

fn cleanup_old_images_in_dir(dir: &Path, max_keep: usize) -> Result<(), std::io::Error> {
    let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            files.push((entry.path(), metadata.modified()?));
        }
    }

    for path in stale_images_to_delete(files, max_keep) {
        match fs::remove_file(&path) {
            Ok(()) => println!("🗑️ [file_monitor] 已删除旧图片: {}", path.display()),
            Err(e) => eprintln!("❌ [file_monitor] 删除旧图片失败 {}: {}", path.display(), e),
        }
    }

    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };

    matches!(
        extension.to_ascii_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
    )
}

fn stale_images_to_delete<I>(files: I, max_keep: usize) -> Vec<PathBuf>
where
    I: IntoIterator<Item = (PathBuf, SystemTime)>,
{
    let mut images: Vec<(PathBuf, SystemTime)> = files
        .into_iter()
        .filter(|(path, _)| is_image_file(path))
        .collect();

    images.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    images
        .into_iter()
        .skip(max_keep)
        .map(|(path, _)| path)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn selects_only_old_images_after_newest_ten() {
        let files: Vec<(PathBuf, SystemTime)> = (0..12)
            .map(|i| {
                (
                    PathBuf::from(format!(r"D:\MCode\Rust\image_{:02}.png", i)),
                    UNIX_EPOCH + Duration::from_secs(i),
                )
            })
            .collect();

        let stale = stale_images_to_delete(files, 10);

        assert_eq!(
            stale,
            vec![
                PathBuf::from(r"D:\MCode\Rust\image_01.png"),
                PathBuf::from(r"D:\MCode\Rust\image_00.png"),
            ]
        );
    }

    #[test]
    fn watches_opencode_config_for_backups() {
        assert!(WATCH_FILES.contains(&r"C:\Users\11038\.config\opencode\opencode.json"));
    }

    #[test]
    fn ignores_non_image_files_when_selecting_stale_images() {
        let files = vec![
            (
                PathBuf::from(r"D:\MCode\Rust\a.png"),
                UNIX_EPOCH + Duration::from_secs(1),
            ),
            (
                PathBuf::from(r"D:\MCode\Rust\b.txt"),
                UNIX_EPOCH + Duration::from_secs(2),
            ),
            (
                PathBuf::from(r"D:\MCode\Rust\c.JPG"),
                UNIX_EPOCH + Duration::from_secs(3),
            ),
        ];

        let stale = stale_images_to_delete(files, 1);

        assert_eq!(stale, vec![PathBuf::from(r"D:\MCode\Rust\a.png")]);
    }

    #[test]
    fn sorts_equal_modified_times_by_path_for_stable_results() {
        let same_time = UNIX_EPOCH + Duration::from_secs(1);
        let files = vec![
            (PathBuf::from(r"D:\MCode\Rust\b.png"), same_time),
            (PathBuf::from(r"D:\MCode\Rust\a.png"), same_time),
            (PathBuf::from(r"D:\MCode\Rust\c.png"), same_time),
        ];

        let stale = stale_images_to_delete(files, 1);

        assert_eq!(
            stale,
            vec![
                PathBuf::from(r"D:\MCode\Rust\b.png"),
                PathBuf::from(r"D:\MCode\Rust\c.png"),
            ]
        );
    }
}
