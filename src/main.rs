#![allow(unused_variables, dead_code)]

// ===== x86_ubuntu 分支：仅保留“获取与播放音频”功能 =====
// 该分支只启动 warp Web 服务，暴露 /playText 与 /getTextAudio 两个 Edge TTS 接口。
// 其余 Windows 专有功能（UAC 提权、PotPlayer/MCGS IPC、截图、鼠标键盘、文件监控、
// 交互菜单等）在本分支全部注释掉，以便在 x86 Ubuntu 上编译运行。

// use runas::Command;               // Windows-only：UAC 提权
// use std::io;
// use utf8_slice::slice;
// use tokio::time::{sleep, Duration};
use tokio::{self};

// use std::thread::sleep;
// use std::time::Duration;

mod controllers;
// mod enums;          // Windows 用户名/PotPlayer 路径等，Linux 不需要
// mod file_monitor;   // 依赖 screenshots / Windows 路径
// mod get_pot_player; // PotPlayer IPC（winapi）
// mod mcgs_control;   // MCGS IPC（winapi）
// mod menu;           // 交互菜单，依赖大量 Windows 功能
// mod ocr;            // 截图 OCR
mod router;
// mod ui;
// mod uitl;           // 依赖 screenshots / winapi GetUserNameA

#[tokio::main]
async fn main() {
    // 仅启动 Web 服务：/playText 合成并本机播放，/getTextAudio 合成后返回音频字节。
    // 端口沿用原逻辑：debug=7655，release=7654。
    let handle = tokio::spawn(async {
        let port: u16 = if cfg!(debug_assertions) { 7655 } else { 7654 };
        let route = router::get_router();
        warp::serve(route).run(([0, 0, 0, 0], port)).await;
    });

    // 保持主线程存活，直到 Web 服务任务结束。
    let _ = handle.await;
}

// type File = String;

// fn open(f: &mut File) -> bool {
//     true
// }
// fn close(f: &mut File) -> bool {
//     true
// }

// #[allow(dead_code)]
// fn read(f: &mut File, save_to: &mut Vec<u8>) -> ! {
//     unimplemented!()
// }

// fn main() {
//     let mut f1 = File::from("f1.txt");
//     open(&mut f1);
//     read(&mut f1, &mut vec![]);
//     close(&mut f1);
// }
