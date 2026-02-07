// use std::sync::{Arc, Mutex};
use crate::get_pot_player;
use crate::uitl;
use chrono::Local;
use chrono::Timelike;
use enigo::*;
use rsautogui::mouse;
use std::process::Command;
use tokio::time::{sleep, Duration};
use warp::Rejection;

use rodio::Decoder;
use std::fs::File;

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

pub async fn test() -> Result<String, Rejection> {
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
    println!("当前时间是: {:02}:{:02}", hour, minute);

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
        println!("还没到时间，或者已经过了。");
    }
}
