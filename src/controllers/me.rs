// use std::sync::{Arc, Mutex};
use crate::controllers::abogen_tts::{self, AbogenSpeakOptions};
use crate::get_pot_player;
use crate::uitl;
use chrono::Local;
use chrono::Timelike;
use edge_tts_rust::Boundary;
use edge_tts_rust::EdgeTtsClient;
use edge_tts_rust::SpeakOptions;
use enigo::*;
use rsautogui::mouse;
use serde::Deserialize;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::task;
use tokio::time::{sleep, Duration};
use warp::Rejection;

use rodio::Decoder;
use std::fs::File;
use std::io::Cursor;
use winapi::um::winuser::{GetAsyncKeyState, VK_MENU};

const VK_B: i32 = 0x42;

pub async fn charge() -> Result<String, Rejection> {
    let now = Local::now();
    println!("当前系统时间: {:?}", now);
    tokio::spawn(async {
        let script = r#"$ConfirmPreference = 'None';$ws = New-Object -ComObject WScript.Shell;$wsr = $ws.popup("The software has installed successfully, please restart your computer to take effect. Press OK to restart later.",0,"Reboot Attention!",0 + 64)"#;
        sleep(Duration::from_secs(270)).await;
        let output = Command::new("powershell.exe")
            .args(&["-Command", &script])
            .output()
            .expect("执行失败");
    });

    Ok(format!("charge up"))
}

pub async fn start_barve() -> Result<String, Rejection> {
    let now = Local::now();
    println!("当前系统时间: {:?}", now);
    tokio::spawn(async {
        let script = r#"start brave  https://game.mahjongsoul.com"#;
        let output = Command::new("powershell.exe")
            .args(&["-Command", &script])
            .output()
            .expect("执行失败");

        mouse::move_to(200, 550);

        sleep(Duration::from_secs(270)).await;
        sleep(Duration::from_secs(1)).await;
        // Perform a left-click
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(1)).await;
        mouse::click(mouse::Button::Left);

        mouse::move_to(300, 550);
        sleep(Duration::from_secs(1)).await;
        // Perform a left-click
        mouse::click(mouse::Button::Left);

        uitl::screen_shot();
        sleep(Duration::from_secs(5)).await;
        let script = r#"Stop-Process -Name "Brave""#;
        let output = Command::new("powershell.exe")
            .args(&["-Command", &script])
            .output()
            .expect("执行失败");
    });

    Ok(format!("brave up"))
}

pub async fn play_list() -> Result<String, Rejection> {
    let res = get_pot_player::get_player_list_file().await;
    match res {
        Err(e) => {
            println!("文件读取失败: {}", e);
            return Ok("文件读取失败".to_string());
        }
        Ok(_) => Ok("播放列表更新成功".to_string()),
    }
}

pub async fn potplay(s: String) -> Result<String, Rejection> {
    Ok(s)
}

#[derive(Deserialize)]
pub struct ToastQuery {
    text: String,
}

pub async fn toast_notify(query: ToastQuery) -> Result<String, Rejection> {
    let text = query.text;
    let code = longest_verification_code(&text);

    let res = task::spawn_blocking({
        let text = text.clone();
        let code = code.clone();

        move || {
            show_windows_toast(&text)?;

            if let Some(code) = code {
                let mut clipboard =
                    arboard::Clipboard::new().map_err(|e| format!("剪贴板打开失败: {}", e))?;
                clipboard
                    .set_text(code)
                    .map_err(|e| format!("剪贴板写入失败: {}", e))?;
            }

            Ok::<(), String>(())
        }
    })
    .await;

    match res {
        Ok(Ok(())) => match code {
            Some(code) => Ok(format!("toast 成功，验证码已复制: {}", code)),
            None if text.contains("验证码") => Ok("toast 成功，未找到验证码数字".to_string()),
            None => Ok("toast 成功".to_string()),
        },
        Ok(Err(e)) => Ok(format!("toast 失败: {}", e)),
        Err(e) => Ok(format!("toast 失败: {}", e)),
    }
}

pub async fn test() -> Result<String, Rejection> {
    Ok(format!("********"))
}
pub async fn test2(s: String) -> Result<String, Rejection> {
    Ok(format!("********"))
}

pub fn show_mouse_xy() -> () {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let (x, y) = enigo.location().unwrap();
    println!("鼠标坐标: x={}, y={}", x, y);
}

pub fn check_network() -> () {
    async fn check_one() -> () {
        let res = uitl::ping("www.baidu.com").unwrap();
        let str2 = String::from_utf8_lossy(&res.stdout).clone();
        let str = str2.trim().to_string();
        println!("output 字符串{}", str);
        println!(
            "🪵 [me.rs:118]~ token ~ \x1b[0;32mstr.len()\x1b[0m = {}",
            str.len()
        );
        if str.len() > 200 {
            // println!("网络连接成功");
        } else {
            println!("网络连接失败");
            relink_wifi().await;
        }
    }
    async fn relink_wifi() -> () {
        // let mut enigo = Enigo::new(&Settings::default()).unwrap();
        // let _ =enigo.key(Key::LWin, Direction::Click);
        // sleep(Duration::from_secs(1)).await;
        // mouse::move_to(1800, 1055);
        // mouse::click(mouse::Button::Left);

        // sleep(Duration::from_secs(1)).await;
        // mouse::move_to(1637, 730);
        // mouse::click(mouse::Button::Left);

        // sleep(Duration::from_secs(1)).await;
        // mouse::move_to(1880, 640);
        // mouse::click(mouse::Button::Left);
        // sleep(Duration::from_secs(5)).await;
        // mouse::click(mouse::Button::Left);

        // return Ok("".to_string());

        tokio::spawn(async {
            //查看instance id方法
            // get-PnpDevice | ? {$_.class -eq "NET"} | sort friendlyname | select friendlyname,instanceid
            let script = r#"Disable-PnpDevice -InstanceId  "PCI\VEN_10EC&DEV_8812&SUBSYS_881210EC&REV_01\4&33186293&0&00E8""#;
            // sleep(Duration::from_secs(270)).await;
            let output = Command::new("powershell.exe")
                .args(&["-Command", &script])
                .output()
                .expect("执行失败");
            println!(
                "🪵 [me.rs:142]~ token ~ \x1b[0;32moutput\x1b[0m = {}",
                String::from_utf8_lossy(output.stdout.as_slice())
            );

            sleep(Duration::from_secs(6)).await;

            let script = r#"Enable-PnpDevice -InstanceId  "PCI\VEN_10EC&DEV_8812&SUBSYS_881210EC&REV_01\4&33186293&0&00E8""#;
            // sleep(Duration::from_secs(270)).await;
            let output = Command::new("powershell.exe")
                .args(&["-Command", &script])
                .output()
                .expect("执行失败");
            println!(
                "🪵 [me.rs:142]~ token ~ \x1b[0;32moutput\x1b[0m = {}",
                String::from_utf8_lossy(output.stdout.as_slice())
            );
        });
    }
    let handle = tokio::spawn(async {
        loop {
            check_one().await;
            sleep(Duration::from_secs(10 * 60)).await;
        }
    });
    // Ok(format!("********"))
}

pub fn play_bingbong() -> () {
    // 获取当前的本地时间
    let now = Local::now();
    // 获取小时和分钟
    let hour = now.hour();
    let minute = now.minute();
    // println!("当前时间是: {:02}:{:02}", hour, minute);

    // 检查是否是 21:30
    if hour == 21 && minute == 30 {
        println!("到时间了！现在是 21:30。");
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        // 【关键步骤 1】：获取当前可执行文件 (.exe) 的完整路径
        let mut music_path = std::env::current_exe().unwrap();

        // 【关键步骤 2】：去掉文件名，只保留目录路径
        // 例如：从 "C:\Game\release\game.exe" 变成 "C:\Game\release\"
        music_path.pop();

        // 【关键步骤 3】：拼接音频文件名
        // 建议把资源放在一个 assets 文件夹里，更整洁，这里假设就在同级目录
        music_path.push("bingbongbangbong.MP3");
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = File::open(music_path).unwrap();
        // Decode that sound file into a source
        let source = Decoder::try_from(file).unwrap();
        // Play the sound directly on the device
        stream_handle.mixer().add(source);

        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        std::thread::sleep(std::time::Duration::from_secs(5));
    } else {
        // println!("还没到时间，或者已经过了。");
    }
}

#[derive(Deserialize)]
pub struct PlayTextData {
    str: String,
}
pub async fn play_text(dat: PlayTextData) -> Result<String, Rejection> {
    // dat: PlayTextData
    let res = tokio::spawn(async move {
        let client = EdgeTtsClient::new().unwrap();
        let result = client
            .synthesize(
                dat.str.as_str(),
                // "我早已麻痹",
                SpeakOptions {
                    voice: "zh-CN-XiaoxiaoNeural".into(),
                    boundary: Boundary::Sentence,
                    rate: "+20%".into(),
                    volume: "-10%".into(),
                    ..SpeakOptions::default()
                },
            )
            .await
            .unwrap();

        println!("audio bytes: {}", result.audio.len());
        println!("boundaries: {}", result.boundaries.len());
        play_audio_from_vec(result.audio).await;
        // Ok(())
    });

    match res.await {
        Ok(res) => Ok(format!("play_text 成功")),
        Err(e) => {
            println!("play_text 失败: {}", e);
            return Ok(format!("play_text 失败"));
        }
    }
}

pub async fn play_text_abogen(dat: PlayTextData) -> Result<String, Rejection> {
    let res = tokio::spawn(async move {
        abogen_tts::synthesize_and_play(dat.str.as_str(), AbogenSpeakOptions::default()).await
    });

    match res.await {
        Ok(Ok(())) => Ok(format!("play_text_abogen 成功")),
        Ok(Err(e)) => {
            println!("play_text_abogen 失败: {}", e);
            Ok(format!("play_text_abogen 失败: {}", e))
        }
        Err(e) => {
            println!("play_text_abogen 失败: {}", e);
            Ok(format!("play_text_abogen 失败"))
        }
    }
}

async fn play_audio_from_vec(audio_data: Vec<u8>) {
    let res = task::spawn_blocking(move || {
        // 1. 获取默认输出设备的句柄
        // _stream 必须保持存活，否则声音会立即停止
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = Arc::new(rodio::Sink::connect_new(&stream_handle.mixer()));

        // 3. 将 Vec<u8> 包装在 Cursor 中，因为它需要实现 Read + Seek
        let cursor = Cursor::new(audio_data);

        // 4. 解码音频数据（自动识别 MP3, WAV, Vorbis, Flac 等）
        let source = Decoder::new(cursor).unwrap();

        // 5. 将音频源放入 Sink 播放
        sink.append(source);
        let stop_hotkey_listener = Arc::new(AtomicBool::new(false));
        let hotkey_listener =
            start_audio_hotkey_listener(Arc::clone(&sink), Arc::clone(&stop_hotkey_listener));

        // 6. 阻塞当前线程直到音频播放完毕（否则函数结束释放资源声音就没了）
        sink.sleep_until_end();
        stop_hotkey_listener.store(true, Ordering::SeqCst);
        let _ = hotkey_listener.join();
    });
    res.await.unwrap();
}

fn start_audio_hotkey_listener(sink: Arc<rodio::Sink>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut was_alt_b_down = false;

        while !stop.load(Ordering::SeqCst) {
            if alt_b_pressed_edge(is_key_down(VK_MENU), is_key_down(VK_B), &mut was_alt_b_down) {
                if sink.is_paused() {
                    sink.play();
                    println!("Alt+B pressed: resume audio");
                } else {
                    sink.pause();
                    println!("Alt+B pressed: pause audio");
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    })
}

fn alt_b_pressed_edge(alt_down: bool, b_down: bool, was_down: &mut bool) -> bool {
    let is_down = alt_down && b_down;
    let pressed = is_down && !*was_down;
    *was_down = is_down;
    pressed
}

fn is_key_down(vkey: i32) -> bool {
    unsafe { (GetAsyncKeyState(vkey) as u16 & 0x8000) != 0 }
}

fn longest_verification_code(text: &str) -> Option<String> {
    if !text.contains("验证码") {
        return None;
    }

    let mut best: Option<&str> = None;
    let mut run_start: Option<usize> = None;

    for (idx, ch) in text.char_indices() {
        if ch.is_ascii_digit() {
            if run_start.is_none() {
                run_start = Some(idx);
            }
            continue;
        }

        if let Some(start) = run_start.take() {
            best = choose_longer_code(best, &text[start..idx]);
        }
    }

    if let Some(start) = run_start {
        best = choose_longer_code(best, &text[start..]);
    }

    best.map(|code| code.to_string())
}

fn choose_longer_code<'a>(best: Option<&'a str>, candidate: &'a str) -> Option<&'a str> {
    if candidate.len() < 4 {
        return best;
    }

    match best {
        Some(current) if current.len() >= candidate.len() => best,
        _ => Some(candidate),
    }
}

fn show_windows_toast(text: &str) -> Result<(), String> {
    let script = windows_toast_script(text);

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|e| format!("PowerShell 启动失败: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("PowerShell 退出码: {}", output.status))
        } else {
            Err(stderr)
        }
    }
}

fn windows_toast_script(text: &str) -> String {
    let title = escape_powershell_single_quoted("hello_cargo");
    let body = escape_powershell_single_quoted(text);

    format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$notify = New-Object System.Windows.Forms.NotifyIcon
$notify.Icon = [System.Drawing.SystemIcons]::Information
$notify.BalloonTipIcon = [System.Windows.Forms.ToolTipIcon]::Info
$notify.BalloonTipTitle = '{title}'
$notify.BalloonTipText = '{body}'
$notify.Visible = $true
$notify.ShowBalloonTip(5000)
Start-Sleep -Milliseconds 5500
$notify.Dispose()
"#
    )
}

fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "&apos;").replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_edge_triggers_once_while_alt_b_is_held() {
        let mut was_down = false;

        assert!(alt_b_pressed_edge(true, true, &mut was_down));
        assert!(!alt_b_pressed_edge(true, true, &mut was_down));
        assert!(!alt_b_pressed_edge(true, true, &mut was_down));
    }

    #[test]
    fn hotkey_edge_rearms_after_alt_b_is_released() {
        let mut was_down = false;

        assert!(alt_b_pressed_edge(true, true, &mut was_down));
        assert!(!alt_b_pressed_edge(false, false, &mut was_down));
        assert!(alt_b_pressed_edge(true, true, &mut was_down));
    }

    #[test]
    fn verification_code_is_none_without_keyword() {
        assert_eq!(longest_verification_code("登录代码 123456"), None);
    }

    #[test]
    fn verification_code_extracts_four_or_more_digits() {
        assert_eq!(
            longest_verification_code("你的验证码是1234，请勿泄露"),
            Some("1234".to_string())
        );
    }

    #[test]
    fn verification_code_chooses_longest_digit_run() {
        assert_eq!(
            longest_verification_code("验证码 1234 订单 987654"),
            Some("987654".to_string())
        );
    }

    #[test]
    fn verification_code_keeps_first_when_lengths_tie() {
        assert_eq!(
            longest_verification_code("验证码 12345 和 67890 都出现"),
            Some("12345".to_string())
        );
    }

    #[test]
    fn verification_code_ignores_short_digit_runs() {
        assert_eq!(longest_verification_code("验证码 12 345"), None);
    }

    #[test]
    fn toast_script_uses_notify_icon_balloon_tip() {
        let script = windows_toast_script("测试通知");

        assert!(script.contains("System.Windows.Forms"));
        assert!(script.contains("NotifyIcon"));
        assert!(script.contains("ShowBalloonTip"));
        assert!(script.contains("测试通知"));
    }
}
