#![allow(unused_variables, dead_code)]

// use std::io;
// use utf8_slice::slice;
use tokio::time::{sleep, Duration};
use tokio::{self};

// use std::thread::sleep;
// use std::time::Duration;

mod controllers;
mod enums;
mod get_pot_player;
mod menu;
mod router;
mod ui;
mod uitl;

#[tokio::main]
async fn main() {
    if !cfg!(debug_assertions) {
        let handle = tokio::spawn(async {
            loop {
                get_pot_player::save_pot_play_info().await;
                sleep(Duration::from_secs(5 * 60)).await;
            }
        });
    }

    #[cfg(windows)] // 仅在 Windows 平台上编译和运行此代码
    {
        use winresource::WindowsResource;

        if let Err(e) = WindowsResource::new()
                // .set_icon("icon.ico") // 可选：设置应用程序图标的路径
                .set_manifest(r#"
    <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
        <assemblyIdentity version="1.0.0.0" processorArchitecture="*" name="hallo_cargo" type="win32"/>
        <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
            <securityIdentity>
                <requestedPrivileges>
                    <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
                </requestedPrivileges>
            </securityIdentity>
        </trustInfo>
    </assembly>
    "#)
    .compile()
    {
    eprintln!("Error compiling Windows resource: {}", e);
    }
    }
    unsafe {
        enums::set_user();
    }

    // get_pot_player::save_pot_play_info().await;
    // get_pot_player::get_player_list_file().await;
    // let (bv, play_time,now_title)  = get_pot_player::get_pot_first_info();
    // println!("🪵 [main.rs:30]~ token ~ \x1b[0;32mnow_title\x1b[0m = {}", now_title);
    // let split_str = now_title.split(" ").collect::<Vec<_>>();
    // let time_str_list = &split_str[1..3];
    // let now_play_ts = uitl::transform_wuyang_time_ts(time_str_list); //日期

    // get_pot_player::get_player_list_file().await;
    //获取路由k
    // // 定义一个简单的 GET 路由 release go
    // let hello = warp::path!("hello" / String)
    //     .map(|name| format!("Hello, {}!", name));

    // // 组合路由
    // let route = hello;
    let handle = tokio::spawn(async {
        let port: u16 = if cfg!(debug_assertions) { 7655 } else { 7654 };
        let route = router::get_router();
        warp::serve(route).run(([0, 0, 0, 0], port)).await;
    });

    controllers::me::check_network();
    // let path = enums::get_list_local_list();
    // let line_first = uitl::read_lines(path, 0, 37).unwrap();
    // println!("🪵 [main.rs:60]~ token ~ \x1b[0;32mline_first\x1b[0m = {}", line_first);

    menu::start::init_menu();
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
