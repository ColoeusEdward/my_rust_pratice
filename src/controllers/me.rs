// use std::sync::{Arc, Mutex};
use chrono::Local;
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

pub async fn potplay(s: String) -> Result<String, Rejection> {
    Ok(s)
}

pub async fn test() -> Result<String, Rejection> {
    Ok(format!("********"))
}
