use chrono::{DateTime, Local, TimeZone};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
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
const SCREENSHOT_PREFIX: &str = "1685731124";
const OCR_FAILURE_KEYWORD: &str = "再度";
const FREEMODEL_USAGE_SCRIPT: &str = r"D:\NTCode\electron\chat\freemodel_usage.py";
const FREEMODEL_USAGE_LOG: &str = r"D:\NTCode\electron\chat\freemodel_usage_log.jsonl";

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

/// 每天早上8点检查最新的 1685731124 开头截图，OCR识别失败字样并弹窗提醒
pub async fn start_daily_screenshot_ocr_check() {
    loop {
        sleep(duration_until_next_8am(Local::now())).await;

        if let Err(e) = run_screenshot_ocr_check_once() {
            eprintln!("❌ [file_monitor] 截图OCR检查失败: {}", e);
        }
    }
}

/// 每天午夜0点检查微信聊天采集(菜单7)是否仍在运行，若在运行则自动停止
pub async fn start_midnight_wechat_capture_stop() {
    loop {
        sleep(duration_until_next_midnight(Local::now())).await;

        if crate::controllers::wechat_capture::request_stop_wechat_capture() {
            println!("📄 [file_monitor] 已到午夜0点，自动停止微信聊天采集。");
        }
    }
}

/// 每天早上8点调用 freemodel_usage.py 获取7天已用/总额度，追加记录到 jsonl 日志文件
pub async fn start_daily_freemodel_usage_log() {
    loop {
        sleep(duration_until_next_8am(Local::now())).await;

        if let Err(e) = run_freemodel_usage_log_once() {
            eprintln!("❌ [file_monitor] freemodel 额度记录失败: {}", e);
        }
    }
}

fn run_freemodel_usage_log_once() -> Result<(), String> {
    let output = Command::new("py")
        .args([FREEMODEL_USAGE_SCRIPT, "--json"])
        .output()
        .map_err(|e| format!("调用 freemodel_usage.py 失败: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "freemodel_usage.py 退出码非0: {:?}, stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("解析 freemodel_usage.py 输出失败: {}, 原始输出: {}", e, stdout))?;

    let mut record = serde_json::json!({
        "recordedAt": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    });
    if let serde_json::Value::Object(ref mut map) = record {
        if let serde_json::Value::Object(data_map) = data {
            map.extend(data_map);
        }
    }

    append_jsonl_line(FREEMODEL_USAGE_LOG, &record.to_string())
        .map_err(|e| format!("写入额度记录文件失败: {}", e))?;

    println!("📄 [file_monitor] 已记录 freemodel 7天额度到: {}", FREEMODEL_USAGE_LOG);
    Ok(())
}

fn append_jsonl_line(path: &str, line: &str) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", line)
}

/// 计算距离下一个当天/次日 8:00 的时长
fn duration_until_next_8am(now: DateTime<Local>) -> Duration {
    let today_8am = now
        .date_naive()
        .and_hms_opt(8, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single())
        .unwrap_or(now);

    let next_8am = if now < today_8am {
        today_8am
    } else {
        today_8am + chrono::Duration::days(1)
    };

    (next_8am - now).to_std().unwrap_or(Duration::from_secs(0))
}

/// 计算距离下一个午夜0点的时长
fn duration_until_next_midnight(now: DateTime<Local>) -> Duration {
    let next_midnight = (now.date_naive() + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single())
        .unwrap_or(now);

    (next_midnight - now)
        .to_std()
        .unwrap_or(Duration::from_secs(0))
}

fn run_screenshot_ocr_check_once() -> Result<(), String> {
    let Some(image_path) =
        latest_screenshot_by_prefix(Path::new(IMAGE_CLEANUP_DIR), SCREENSHOT_PREFIX)
            .map_err(|e| format!("查找最新截图失败: {}", e))?
    else {
        println!(
            "📄 [file_monitor] 未找到 {} 开头的截图，跳过OCR检查",
            SCREENSHOT_PREFIX
        );
        return Ok(());
    };

    let text = crate::ocr::run_umi_ocr(&image_path)?;

    if contains_failure_keyword(&text) {
        show_ocr_failure_popup(&image_path, &text)?;
    } else {
        println!(
            "📄 [file_monitor] 截图OCR检查通过，未发现\"{}\"字样: {}",
            OCR_FAILURE_KEYWORD,
            image_path.display()
        );
    }

    Ok(())
}

/// 找出目录中以 prefix 开头、最后修改时间最新的文件
fn latest_screenshot_by_prefix(dir: &Path, prefix: &str) -> std::io::Result<Option<PathBuf>> {
    let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            files.push((entry.path(), metadata.modified()?));
        }
    }

    Ok(pick_latest_matching_file(files, prefix))
}

fn pick_latest_matching_file(files: Vec<(PathBuf, SystemTime)>, prefix: &str) -> Option<PathBuf> {
    files
        .into_iter()
        .filter(|(path, _)| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with(prefix))
                .unwrap_or(false)
        })
        .max_by_key(|(_, modified)| *modified)
        .map(|(path, _)| path)
}

fn contains_failure_keyword(text: &str) -> bool {
    text.contains(OCR_FAILURE_KEYWORD)
}

fn show_ocr_failure_popup(image_path: &Path, ocr_text: &str) -> Result<(), String> {
    let message = crate::controllers::me::escape_powershell_single_quoted(&format!(
        "检测到截图 {} OCR识别结果包含\"{}\"字样：\n{}",
        image_path.display(),
        OCR_FAILURE_KEYWORD,
        ocr_text
    ));
    let script = format!(
        r#"$ws = New-Object -ComObject WScript.Shell; $ws.popup('{}', 0, '截图OCR检测', 0 + 48)"#,
        message
    );

    spawn_powershell_script(&script)
}

fn spawn_powershell_script(script: &str) -> Result<(), String> {
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("弹窗启动失败: {}", e))
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
    use std::time::{Duration, Instant, UNIX_EPOCH};

    #[test]
    #[ignore]
    fn manual_verify_screenshot_ocr_check_against_fail_test_image() {
        run_screenshot_ocr_check_once().unwrap();
    }

    #[test]
    #[ignore]
    fn manual_verify_freemodel_usage_log_against_live_api() {
        run_freemodel_usage_log_once().unwrap();
    }

    #[test]
    fn powershell_popup_launcher_returns_without_waiting_for_script_completion() {
        let started = Instant::now();

        spawn_powershell_script(r#"Start-Sleep -Seconds 2"#).unwrap();

        assert!(
            started.elapsed() < Duration::from_millis(500),
            "PowerShell launcher should return immediately without waiting for script completion"
        );
    }

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

    #[test]
    fn picks_newest_file_matching_prefix() {
        let files = vec![
            (
                PathBuf::from(r"D:\MCode\Rust\1685731124-100.png"),
                UNIX_EPOCH + Duration::from_secs(100),
            ),
            (
                PathBuf::from(r"D:\MCode\Rust\1685731124-200.png"),
                UNIX_EPOCH + Duration::from_secs(200),
            ),
            (
                PathBuf::from(r"D:\MCode\Rust\2776250164-300.png"),
                UNIX_EPOCH + Duration::from_secs(300),
            ),
        ];

        assert_eq!(
            pick_latest_matching_file(files, "1685731124"),
            Some(PathBuf::from(r"D:\MCode\Rust\1685731124-200.png"))
        );
    }

    #[test]
    fn no_matching_prefix_returns_none() {
        let files = vec![(
            PathBuf::from(r"D:\MCode\Rust\2776250164-300.png"),
            UNIX_EPOCH + Duration::from_secs(300),
        )];

        assert_eq!(pick_latest_matching_file(files, "1685731124"), None);
    }

    #[test]
    fn detects_failure_keyword_in_ocr_text() {
        assert!(contains_failure_keyword("版本确认再度,请再试一次"));
        assert!(!contains_failure_keyword("版本确认成功"));
    }

    #[test]
    fn duration_until_next_8am_same_day_when_before_8am() {
        let now = Local.with_ymd_and_hms(2026, 7, 6, 6, 30, 0).unwrap();
        let duration = duration_until_next_8am(now);
        assert_eq!(duration, Duration::from_secs(90 * 60));
    }

    #[test]
    fn duration_until_next_8am_rolls_to_next_day_when_after_8am() {
        let now = Local.with_ymd_and_hms(2026, 7, 6, 9, 0, 0).unwrap();
        let duration = duration_until_next_8am(now);
        assert_eq!(duration, Duration::from_secs(23 * 60 * 60));
    }

    #[test]
    fn duration_until_next_midnight_counts_down_through_the_day() {
        let now = Local.with_ymd_and_hms(2026, 7, 6, 23, 0, 0).unwrap();
        let duration = duration_until_next_midnight(now);
        assert_eq!(duration, Duration::from_secs(60 * 60));
    }

    #[test]
    fn duration_until_next_midnight_rolls_to_next_day_right_after_midnight() {
        let now = Local.with_ymd_and_hms(2026, 7, 6, 0, 0, 1).unwrap();
        let duration = duration_until_next_midnight(now);
        assert_eq!(duration, Duration::from_secs(24 * 60 * 60 - 1));
    }
}
