# Wechat OCR Rule Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove noisy OCR fragments from WeChat capture results while preserving meaningful chat sentences.

**Architecture:** Keep the cleanup local to `src/controllers/wechat_capture.rs`. Add pure rule-based text cleanup functions after OCR and before segment normalization, with unit tests driven by the observed noisy capture sample.

**Tech Stack:** Rust 2021, existing `cargo test` unit tests, no new dependencies.

---

### Task 1: Add Regression Tests For OCR Cleanup

**Files:**
- Modify: `src/controllers/wechat_capture.rs`

- [ ] **Step 1: Add a failing test for noisy OCR sample cleanup**

Add this test in the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn clean_ocr_text_removes_noise_and_keeps_meaningful_chinese_sentences() {
    let raw = "东莞市新杰电工机械有限公司(106)                                                                                                                              加- Sw\nBe cevenesnuen\n6A248 10:57\n图  各位同事早上好。为了日后方便管理，请各位没备注名字的同事备注好自己的姓名。@所有人，\n时期六12407\n图  各位同事中午好，今日下午1点半会对后房进行大扫除活动，也会清洁餐点。请大家午餐后务必将自己的饭确收拾\n好。 以免弄及，谢谢配合@所有人\nSE\" WAT RS\n图  各位同事早上好，6月考勤已出，请大家于今天下午5点30分前核对完毕，途期当默认处理@所有人\n202606考勤Jls         加\n412.0K\n& aE\n,";

    assert_eq!(
        clean_ocr_text(raw),
        "各位同事早上好。为了日后方便管理，请各位没备注名字的同事备注好自己的姓名。@所有人，\n各位同事中午好，今日下午1点半会对后房进行大扫除活动，也会清洁餐点。请大家午餐后务必将自己的饭确收拾好。以免弄及，谢谢配合@所有人\n各位同事早上好，6月考勤已出，请大家于今天下午5点30分前核对完毕，途期当默认处理@所有人"
    );
}
```

- [ ] **Step 2: Add a failing test for semantic line scoring**

Add this test in the same test module:

```rust
#[test]
fn meaningful_ocr_line_keeps_chinese_content_and_rejects_noise() {
    assert!(is_meaningful_ocr_line("各位同事早上好。为了日后方便管理"));
    assert!(is_meaningful_ocr_line("好。以免弄及，谢谢配合@所有人"));
    assert!(!is_meaningful_ocr_line("Be cevenesnuen"));
    assert!(!is_meaningful_ocr_line("SE\" WAT RS"));
    assert!(!is_meaningful_ocr_line("& aE"));
    assert!(!is_meaningful_ocr_line(","));
}
```

- [ ] **Step 3: Run the targeted tests to verify they fail**

Run: `cargo test wechat_capture`

Expected: FAIL with missing functions `clean_ocr_text` and `is_meaningful_ocr_line`.

### Task 2: Implement Rule-Based Cleanup

**Files:**
- Modify: `src/controllers/wechat_capture.rs`

- [ ] **Step 1: Add pure cleanup functions**

Add these functions near `normalize_chat_segments`:

```rust
fn clean_ocr_text(text: &str) -> String {
    let mut cleaned_lines = Vec::new();

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

    cleaned_lines.join("\n")
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
        || matches!(ch, '。' | '，' | '、' | '！' | '？' | '：' | '；' | '@' | '.' | '-' | '_' | '(' | ')' | '（' | '）')
}

fn should_keep_space_before(value: &str, next: char) -> bool {
    value
        .chars()
        .last()
        .map(|previous| previous.is_ascii_alphanumeric() && next.is_ascii_alphanumeric())
        .unwrap_or(false)
}

fn is_meaningful_ocr_line(line: &str) -> bool {
    let chinese_count = line.chars().filter(|ch| ('\u{4e00}'..='\u{9fff}').contains(ch)).count();
    let semantic_punctuation = line.chars().any(|ch| matches!(ch, '。' | '，' | '！' | '？' | '：' | '@'));

    chinese_count >= 4 || (chinese_count >= 2 && semantic_punctuation)
}

fn should_join_with_previous(previous: Option<&str>, current: &str) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    let previous_ends_sentence = previous
        .chars()
        .last()
        .map(|ch| matches!(ch, '。' | '！' | '？'))
        .unwrap_or(false);

    !previous_ends_sentence && is_meaningful_ocr_line(current)
}
```

- [ ] **Step 2: Wire cleanup into capture flow**

Change `capture_once` from:

```rust
let text = run_tesseract(&image_path)?;
let _ = fs::remove_file(&image_path);

let captured_segments = normalize_chat_segments(&text);
```

to:

```rust
let text = run_tesseract(&image_path)?;
let _ = fs::remove_file(&image_path);
let text = clean_ocr_text(&text);

let captured_segments = normalize_chat_segments(&text);
```

- [ ] **Step 3: Run targeted tests**

Run: `cargo test wechat_capture`

Expected: all `wechat_capture` tests pass.

### Task 3: Format And Verify

**Files:**
- Modify: `src/controllers/wechat_capture.rs`

- [ ] **Step 1: Format code**

Run: `cargo fmt`

Expected: command exits successfully.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`

Expected: all tests pass with `0 failed`.

- [ ] **Step 3: Review working tree scope**

Run: `git status --short`

Expected: this task only intentionally adds `docs/superpowers/plans/2026-07-02-wechat-ocr-rule-cleanup.md` and modifies `src/controllers/wechat_capture.rs`; unrelated pre-existing changes may remain and must not be reverted.
