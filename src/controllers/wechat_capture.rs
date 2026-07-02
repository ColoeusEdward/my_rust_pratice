use screenshots::Screen;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::windows::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{
    EnumWindows, GetAsyncKeyState, GetWindowRect, GetWindowTextW, IsIconic, IsWindowVisible,
    SetForegroundWindow, SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, SW_MAXIMIZE, SW_RESTORE, VK_MENU,
};

static CAPTURE_STOP_FLAG: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();
const OCR_EXCLUDED_LEFT_PX: i32 = 307;
const OCR_EXCLUDED_BOTTOM_PX: i32 = 172;
const COMMA_SPLIT_MIN_CJK_CHARS: usize = 12;
const DUPLICATE_SENTENCE_SIMILARITY: f64 = 0.8;
const APPROXIMATE_DUPLICATE_MIN_CHARS: usize = 8;
const VK_Q: i32 = 0x51;

pub fn toggle_wechat_capture() {
    let state = CAPTURE_STOP_FLAG.get_or_init(|| Mutex::new(None));
    let mut guard = state.lock().unwrap();

    if let Some(stop_flag) = guard.take() {
        stop_flag.store(true, Ordering::SeqCst);
        println!("已停止微信聊天采集。下一轮检测后任务会退出。");
        return;
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    *guard = Some(Arc::clone(&stop_flag));
    println!("已启动微信聊天采集，每6秒采集一次，再次选择菜单项可停止。");

    let hotkey_listener_stop = Arc::new(AtomicBool::new(false));
    let hotkey_listener = start_wechat_stop_hotkey_listener(
        Arc::clone(&stop_flag),
        Arc::clone(&hotkey_listener_stop),
    );

    tokio::spawn(async move {
        if let Err(e) = clear_markdown_file(markdown_path()) {
            eprintln!("清空微信聊天md失败: {}", e);
        }

        run_capture_loop(stop_flag).await;
        hotkey_listener_stop.store(true, Ordering::SeqCst);
        let _ = hotkey_listener.join();

        let state = CAPTURE_STOP_FLAG.get_or_init(|| Mutex::new(None));
        let mut guard = state.lock().unwrap();
        if guard
            .as_ref()
            .map(|flag| flag.load(Ordering::SeqCst))
            .unwrap_or(false)
        {
            *guard = None;
        }
    });
}

async fn run_capture_loop(stop_flag: Arc<AtomicBool>) {
    let mut known_segments = Vec::new();

    while !stop_flag.load(Ordering::SeqCst) {
        if let Err(e) = capture_once(&mut known_segments) {
            eprintln!("微信聊天采集失败: {}", e);
        }

        sleep(Duration::from_secs(6)).await;
    }
}

fn start_wechat_stop_hotkey_listener(
    capture_stop: Arc<AtomicBool>,
    listener_stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut was_alt_q_down = false;

        while !listener_stop.load(Ordering::SeqCst) && !capture_stop.load(Ordering::SeqCst) {
            if alt_q_pressed_edge(is_key_down(VK_MENU), is_key_down(VK_Q), &mut was_alt_q_down) {
                capture_stop.store(true, Ordering::SeqCst);
                println!("Alt+Q pressed: stop wechat OCR capture");
                break;
            }

            thread::sleep(Duration::from_millis(50));
        }
    })
}

fn alt_q_pressed_edge(alt_down: bool, q_down: bool, was_down: &mut bool) -> bool {
    let is_down = alt_down && q_down;
    let pressed = is_down && !*was_down;
    *was_down = is_down;
    pressed
}

fn is_key_down(vkey: i32) -> bool {
    unsafe { (GetAsyncKeyState(vkey) as u16 & 0x8000) != 0 }
}

fn capture_once(known_segments: &mut Vec<String>) -> Result<(), String> {
    let hwnd = find_wechat_window().ok_or_else(|| "未找到微信窗口".to_string())?;
    let image_path = capture_window(hwnd)?;
    let text = run_tesseract(&image_path)?;
    let _ = fs::remove_file(&image_path);
    let text = clean_ocr_text(&text);

    let captured_segments = ocr_text_to_segments(&text);
    let new_segments = collect_new_segments(known_segments, captured_segments);

    if new_segments.is_empty() {
        return Ok(());
    }

    append_segments(markdown_path(), &new_segments)?;
    known_segments.extend(new_segments);
    Ok(())
}

fn markdown_path() -> PathBuf {
    PathBuf::from("wechat_chat.md")
}

fn clear_markdown_file(path: PathBuf) -> Result<(), String> {
    fs::write(path, "").map_err(|e| e.to_string())
}

fn append_segments(path: PathBuf, segments: &[String]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    for segment in segments {
        writeln!(file, "{}", segment).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn clean_ocr_text(text: &str) -> String {
    let mut cleaned_lines: Vec<String> = Vec::new();

    for line in text.lines() {
        let line = strip_ocr_line_prefix(line.trim());
        let line = normalize_ocr_line(&line);
        if line.is_empty() || !is_meaningful_ocr_line(&line) {
            continue;
        }

        if should_join_with_previous(cleaned_lines.last().map(String::as_str), &line) {
            if let Some(previous) = cleaned_lines.last_mut() {
                previous.push_str(&line);
            }
        } else {
            cleaned_lines.push(line);
        }
    }

    cleaned_lines
        .iter()
        .flat_map(|line| split_meaningful_sentences(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ocr_text_to_segments(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn strip_ocr_line_prefix(line: &str) -> String {
    line.strip_prefix('图').unwrap_or(line).trim().to_string()
}

fn normalize_ocr_line(line: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_space = false;

    for ch in line.chars() {
        if ch.is_whitespace() {
            previous_was_space = true;
            continue;
        }

        if is_allowed_ocr_char(ch) {
            if previous_was_space && should_keep_space_before(&normalized, ch) {
                normalized.push(' ');
            }
            normalized.push(ch);
        }

        previous_was_space = false;
    }

    normalized.trim().to_string()
}

fn is_allowed_ocr_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || matches!(
            ch,
            '。' | '，'
                | '、'
                | '！'
                | '？'
                | '：'
                | '；'
                | '@'
                | '.'
                | '-'
                | '_'
                | '('
                | ')'
                | '（'
                | '）'
        )
}

fn should_keep_space_before(value: &str, next: char) -> bool {
    value
        .chars()
        .last()
        .map(|previous| previous.is_ascii_alphanumeric() && next.is_ascii_alphanumeric())
        .unwrap_or(false)
}

fn split_meaningful_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut pending_sentence_end = false;
    let mut attaching_mention = false;

    for ch in text.chars() {
        if pending_sentence_end && ch == '@' {
            attaching_mention = true;
        }

        if pending_sentence_end && !attaching_mention && ch != '@' {
            push_meaningful_sentence(&mut sentences, &mut current);
            pending_sentence_end = false;
        }

        if matches!(ch, '，' | ',') && has_enough_cjk_for_comma_split(&current) {
            push_meaningful_sentence(&mut sentences, &mut current);
            pending_sentence_end = false;
            attaching_mention = false;
            continue;
        }

        current.push(ch);

        if matches!(ch, '。' | '！' | '？') {
            pending_sentence_end = true;
        }

        if current.ends_with("@所有人") {
            push_meaningful_sentence(&mut sentences, &mut current);
            pending_sentence_end = false;
            attaching_mention = false;
        }
    }

    push_meaningful_sentence(&mut sentences, &mut current);
    sentences
}

fn push_meaningful_sentence(sentences: &mut Vec<String>, current: &mut String) {
    let sentence = current.trim().trim_end_matches(['，', ',']).trim();
    if !sentence.is_empty() && is_meaningful_ocr_line(sentence) {
        sentences.push(sentence.to_string());
    }
    current.clear();
}

fn has_enough_cjk_for_comma_split(text: &str) -> bool {
    text.chars().filter(|ch| is_cjk_char(*ch)).count() >= COMMA_SPLIT_MIN_CJK_CHARS
}

fn is_meaningful_ocr_line(line: &str) -> bool {
    if is_wechat_ui_noise_line(line) {
        return false;
    }

    let chinese_count = line.chars().filter(|ch| is_cjk_char(*ch)).count();
    let semantic_punctuation = line
        .chars()
        .any(|ch| matches!(ch, '。' | '，' | '！' | '？' | '：' | '@'));

    (chinese_count >= 2 && semantic_punctuation) || chinese_count >= 8
}

fn is_wechat_ui_noise_line(line: &str) -> bool {
    (line.contains("有限公司") && line.chars().any(|ch| ch.is_ascii_digit()))
        || line.chars().all(|ch| !is_cjk_char(ch))
}

fn should_join_with_previous(previous: Option<&str>, current: &str) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    !ends_message(previous) && !starts_new_message(current) && is_meaningful_ocr_line(current)
}

fn ends_message(line: &str) -> bool {
    line.contains("@所有人")
        || line
            .chars()
            .last()
            .map(|ch| matches!(ch, '。' | '！' | '？'))
            .unwrap_or(false)
}

fn starts_new_message(line: &str) -> bool {
    line.starts_with("各位")
}

fn is_cjk_char(ch: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&ch)
}

fn normalize_chat_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !current.is_empty() {
                segments.push(current.join("\n"));
                current.clear();
            }
            continue;
        }

        current.push(line.to_string());
    }

    if !current.is_empty() {
        segments.push(current.join("\n"));
    }

    segments
}

fn collect_new_segments(existing: &[String], captured: Vec<String>) -> Vec<String> {
    let mut seen = existing.to_vec();
    let mut new_segments = Vec::new();

    for segment in captured {
        if seen
            .iter()
            .any(|known_segment| is_duplicate_sentence(known_segment, &segment))
        {
            continue;
        }

        seen.push(segment.clone());
        new_segments.push(segment);
    }

    new_segments
}

fn is_duplicate_sentence(left: &str, right: &str) -> bool {
    if left.trim() == right.trim() {
        return true;
    }

    sentence_similarity(left, right) >= DUPLICATE_SENTENCE_SIMILARITY
}

fn sentence_similarity(left: &str, right: &str) -> f64 {
    let left = normalize_sentence_for_compare(left);
    let right = normalize_sentence_for_compare(right);
    let shorter_len = left.len().min(right.len());

    if shorter_len < APPROXIMATE_DUPLICATE_MIN_CHARS {
        return 0.0;
    }

    longest_common_subsequence_len(&left, &right) as f64 / shorter_len as f64
}

fn normalize_sentence_for_compare(sentence: &str) -> Vec<char> {
    sentence
        .chars()
        .filter(|ch| is_cjk_char(*ch) || ch.is_ascii_alphanumeric())
        .collect()
}

fn longest_common_subsequence_len(left: &[char], right: &[char]) -> usize {
    let mut previous = vec![0; right.len() + 1];
    let mut current = vec![0; right.len() + 1];

    for left_ch in left {
        for (index, right_ch) in right.iter().enumerate() {
            current[index + 1] = if left_ch == right_ch {
                previous[index] + 1
            } else {
                current[index].max(previous[index + 1])
            };
        }

        std::mem::swap(&mut previous, &mut current);
        current.fill(0);
    }

    previous[right.len()]
}

fn find_wechat_window() -> Option<HWND> {
    let mut found: HWND = null_mut();
    unsafe {
        EnumWindows(
            Some(enum_wechat_windows_callback),
            &mut found as *mut HWND as LPARAM,
        );
    }

    if found.is_null() {
        None
    } else {
        Some(found)
    }
}

unsafe extern "system" fn enum_wechat_windows_callback(hwnd: HWND, data: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }

    let title = window_title(hwnd);
    let title_lower = title.to_lowercase();
    if title.contains("微信") || title_lower.contains("wechat") {
        *(data as *mut HWND) = hwnd;
        return 0;
    }

    1
}

unsafe fn window_title(hwnd: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32);

    if len <= 0 {
        return String::new();
    }

    OsString::from_wide(&text[..len as usize])
        .to_string_lossy()
        .to_string()
}

fn capture_window(hwnd: HWND) -> Result<PathBuf, String> {
    prepare_window_for_capture(hwnd);
    let rect = window_rect(hwnd)?;
    let (screen_x, screen_y) = screen_lookup_point(rect);
    let screen = Screen::from_point(screen_x, screen_y).map_err(|e| e.to_string())?;
    let (capture_x, capture_y, width, height) =
        ocr_capture_area(rect, screen.display_info.x, screen.display_info.y);
    let image = screen
        .capture_area(capture_x, capture_y, width, height)
        .map_err(|e| e.to_string())?;
    let path = temp_image_path();
    image.save(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

fn prepare_window_for_capture(hwnd: HWND) {
    let is_minimized = unsafe { IsIconic(hwnd) != 0 };
    let policy = capture_preparation_policy(is_minimized);

    if policy.restore {
        unsafe {
            ShowWindow(hwnd, SW_RESTORE);
        }
    }

    if policy.maximize {
        unsafe {
            ShowWindow(hwnd, SW_MAXIMIZE);
        }
    }

    if policy.foreground {
        unsafe {
            SetForegroundWindow(hwnd);
        }
    }

    if policy.topmost {
        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }
    }

    std::thread::sleep(Duration::from_millis(300));
}

fn should_restore_window(is_minimized: bool) -> bool {
    is_minimized
}

struct CapturePreparationPolicy {
    restore: bool,
    maximize: bool,
    foreground: bool,
    topmost: bool,
}

fn capture_preparation_policy(is_minimized: bool) -> CapturePreparationPolicy {
    CapturePreparationPolicy {
        restore: should_restore_window(is_minimized),
        maximize: true,
        foreground: true,
        topmost: true,
    }
}

fn screen_lookup_point(rect: RECT) -> (i32, i32) {
    (
        rect.left + (rect.right - rect.left) / 2,
        rect.top + (rect.bottom - rect.top) / 2,
    )
}

fn ocr_capture_area(rect: RECT, display_x: i32, display_y: i32) -> (i32, i32, u32, u32) {
    let window_width = (rect.right - rect.left).max(1);
    let window_height = (rect.bottom - rect.top).max(1);
    let excluded_left = OCR_EXCLUDED_LEFT_PX.min(window_width.saturating_sub(1));
    let excluded_bottom = OCR_EXCLUDED_BOTTOM_PX.min(window_height.saturating_sub(1));

    (
        rect.left + excluded_left - display_x,
        rect.top - display_y,
        (window_width - excluded_left) as u32,
        (window_height - excluded_bottom) as u32,
    )
}

fn window_rect(hwnd: HWND) -> Result<RECT, String> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
    if ok == 0 {
        return Err("获取微信窗口位置失败".to_string());
    }

    Ok(rect)
}

fn temp_image_path() -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    std::env::temp_dir().join(format!("wechat-chat-{}.png", ts))
}

fn run_tesseract(image_path: &Path) -> Result<String, String> {
    let output = Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .args(["-l", "chi_sim+eng", "--psm", "6"])
        .output()
        .map_err(|e| format!("启动 tesseract 失败，请确认已安装并加入 PATH: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("tesseract 退出码: {}", output.status)
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_chat_segments_groups_non_empty_lines_between_blank_lines() {
        let raw = "Alice\n你好\n\nBob\n收到\n\n\n";

        assert_eq!(
            normalize_chat_segments(raw),
            vec!["Alice\n你好".to_string(), "Bob\n收到".to_string()]
        );
    }

    #[test]
    fn collect_new_segments_skips_existing_whole_segments() {
        let existing = vec!["Alice\n你好".to_string()];
        let captured = vec!["Alice\n你好".to_string(), "Bob\n收到".to_string()];

        assert_eq!(
            collect_new_segments(&existing, captured),
            vec!["Bob\n收到".to_string()]
        );
    }

    #[test]
    fn collect_new_segments_deduplicates_repeated_segments_in_same_capture() {
        let captured = vec!["Bob\n收到".to_string(), "Bob\n收到".to_string()];

        assert_eq!(
            collect_new_segments(&[], captured),
            vec!["Bob\n收到".to_string()]
        );
    }

    #[test]
    fn collect_new_segments_skips_80_percent_similar_sentences() {
        let existing = vec!["请大家于今天下午5点30分前核对完毕".to_string()];
        let captured = vec!["请大家今天下午5点30分前核对完毕".to_string()];

        assert!(collect_new_segments(&existing, captured).is_empty());
    }

    #[test]
    fn collect_new_segments_keeps_less_similar_sentences() {
        let existing = vec!["请大家于今天下午5点30分前核对完毕".to_string()];
        let captured = vec!["明天上午十点开会请准时参加".to_string()];

        assert_eq!(
            collect_new_segments(&existing, captured),
            vec!["明天上午十点开会请准时参加".to_string()]
        );
    }

    #[test]
    fn collect_new_segments_deduplicates_80_percent_similar_sentences_in_same_capture() {
        let captured = vec![
            "请大家于今天下午5点30分前核对完毕".to_string(),
            "请大家今天下午5点30分前核对完毕".to_string(),
        ];

        assert_eq!(
            collect_new_segments(&[], captured),
            vec!["请大家于今天下午5点30分前核对完毕".to_string()]
        );
    }

    #[test]
    fn clean_ocr_text_removes_noise_and_keeps_meaningful_chinese_sentences() {
        let raw = "东莞市新杰电工机械有限公司(106)                                                                                                                              加- Sw\nBe cevenesnuen\n6A248 10:57\n图  各位同事早上好。为了日后方便管理，请各位没备注名字的同事备注好自己的姓名。@所有人，\n时期六12407\n图  各位同事中午好，今日下午1点半会对后房进行大扫除活动，也会清洁餐点。请大家午餐后务必将自己的饭确收拾\n好。 以免弄及，谢谢配合@所有人\nSE\" WAT RS\n图  各位同事早上好，6月考勤已出，请大家于今天下午5点30分前核对完毕，途期当默认处理@所有人\n202606考勤Jls         加\n412.0K\n& aE\n,";

        assert_eq!(
            clean_ocr_text(raw),
            "各位同事早上好。\n为了日后方便管理，请各位没备注名字的同事备注好自己的姓名。@所有人\n各位同事中午好，今日下午1点半会对后房进行大扫除活动\n也会清洁餐点。\n请大家午餐后务必将自己的饭确收拾好。\n以免弄及，谢谢配合@所有人\n各位同事早上好，6月考勤已出\n请大家于今天下午5点30分前核对完毕\n途期当默认处理@所有人"
        );
    }

    #[test]
    fn append_segments_writes_one_sentence_per_line_without_blank_lines() {
        let path = std::env::temp_dir().join(format!(
            "wechat-chat-test-{}.md",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let segments = vec!["第一句。".to_string(), "第二句。".to_string()];

        append_segments(path.clone(), &segments).unwrap();
        let written = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(written, "第一句。\n第二句。\n");
    }

    #[test]
    fn cleaned_ocr_text_lines_become_independent_segments() {
        assert_eq!(
            ocr_text_to_segments("第一句。\n第二句。\n"),
            vec!["第一句。".to_string(), "第二句。".to_string()]
        );
    }

    #[test]
    fn clean_ocr_text_splits_long_comma_separated_text_into_lines() {
        let raw = "西瓜，哈密瓜都特别好吃可惜我买的哈密瓜不够大ia发明和培育出这两种瓜大部分好吃品种的吴明珠女士简直是世间大神，给我们带来如此好吃的瓜如果你买整个的，就看哈密瓜表面那个纹路，隆起的程度高不高，隆起的程度高会甜点Kaf可惜我买的哈密瓜不够型移兰了之后就跟他说，我要你切给我的那_半藉双入发明和培育出这两种丰大部分好吃品种的吴明珠女二简直是世间大神，给我们巷来如此好吃的瓜而且买这种瓜类，一定要扩一下重不重手，通常来说重手-一点，汁水就会丰盈一点";

        assert_eq!(
            clean_ocr_text(raw),
            "西瓜，哈密瓜都特别好吃可惜我买的哈密瓜不够大ia发明和培育出这两种瓜大部分好吃品种的吴明珠女士简直是世间大神\n给我们带来如此好吃的瓜如果你买整个的\n就看哈密瓜表面那个纹路，隆起的程度高不高\n隆起的程度高会甜点Kaf可惜我买的哈密瓜不够型移兰了之后就跟他说\n我要你切给我的那_半藉双入发明和培育出这两种丰大部分好吃品种的吴明珠女二简直是世间大神\n给我们巷来如此好吃的瓜而且买这种瓜类\n一定要扩一下重不重手，通常来说重手-一点\n汁水就会丰盈一点"
        );
    }

    #[test]
    fn meaningful_ocr_line_keeps_chinese_content_and_rejects_noise() {
        assert!(is_meaningful_ocr_line("各位同事早上好。为了日后方便管理"));
        assert!(is_meaningful_ocr_line("好。以免弄及，谢谢配合@所有人"));
        assert!(!is_meaningful_ocr_line("Be cevenesnuen"));
        assert!(!is_meaningful_ocr_line("SE\" WAT RS"));
        assert!(!is_meaningful_ocr_line("& aE"));
        assert!(!is_meaningful_ocr_line(","));
    }

    #[test]
    fn ocr_capture_area_excludes_wechat_sidebar_and_input_bar() {
        let rect = RECT {
            left: 100,
            top: 50,
            right: 1100,
            bottom: 850,
        };

        assert_eq!(ocr_capture_area(rect, 0, 0), (407, 50, 693, 628));
    }

    #[test]
    fn alt_q_edge_triggers_once_while_keys_are_held() {
        let mut was_down = false;

        assert!(alt_q_pressed_edge(true, true, &mut was_down));
        assert!(!alt_q_pressed_edge(true, true, &mut was_down));
        assert!(!alt_q_pressed_edge(true, true, &mut was_down));
    }

    #[test]
    fn alt_q_edge_rearms_after_keys_are_released() {
        let mut was_down = false;

        assert!(alt_q_pressed_edge(true, true, &mut was_down));
        assert!(!alt_q_pressed_edge(false, false, &mut was_down));
        assert!(alt_q_pressed_edge(true, true, &mut was_down));
    }

    #[test]
    fn screen_lookup_point_uses_window_center_when_top_left_is_offscreen() {
        let rect = RECT {
            left: -8,
            top: -8,
            right: 792,
            bottom: 592,
        };

        assert_eq!(screen_lookup_point(rect), (392, 292));
    }

    #[test]
    fn should_restore_window_only_when_it_is_minimized() {
        assert!(should_restore_window(true));
        assert!(!should_restore_window(false));
    }

    #[test]
    fn capture_preparation_policy_always_maximizes_foregrounds_and_sets_topmost() {
        let minimized = capture_preparation_policy(true);
        assert!(minimized.restore);
        assert!(minimized.maximize);
        assert!(minimized.foreground);
        assert!(minimized.topmost);

        let normal = capture_preparation_policy(false);
        assert!(!normal.restore);
        assert!(normal.maximize);
        assert!(normal.foreground);
        assert!(normal.topmost);
    }
}
