// use std::sync::{Arc, Mutex};
use crate::get_pot_player;
use crate::uitl;
use chrono::Local;
use enigo::*;
use rsautogui::mouse;
use std::process::Command;
use tokio::time::{sleep, Duration};
use warp::Rejection;

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
    async  fn  check_one() -> () {
        let res = uitl::ping("www.baidu.com").unwrap();
        let mut str = "".to_string();
        let str2 = String::from_utf8_lossy(&res.stdout).clone();
        str = str2.trim().to_string();
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
    async fn  relink_wifi() -> (){
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        enigo.key(Key::LWin, Direction::Click);
        sleep(Duration::from_secs(1)).await;
        mouse::move_to(1800, 1055);
        mouse::click(mouse::Button::Left);

        sleep(Duration::from_secs(1)).await;
        mouse::move_to(1637, 730);
        mouse::click(mouse::Button::Left);

        sleep(Duration::from_secs(1)).await;
        mouse::move_to(1880, 640);
        mouse::click(mouse::Button::Left);
        sleep(Duration::from_secs(5)).await;
        mouse::click(mouse::Button::Left);

        // return Ok("".to_string());
    }
    let handle = tokio::spawn(async {
        loop {
            check_one().await;
            sleep(Duration::from_secs(10 * 60)).await;
        }
    });
    // Ok(format!("********"))
}
