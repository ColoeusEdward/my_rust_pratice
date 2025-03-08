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
mod router;
mod uitl;

#[tokio::main]
async fn main() {
    if !cfg!(debug_assertions){
        let handle = tokio::spawn(async {
            loop {
                get_pot_player::save_pot_play_info().await;
                sleep(Duration::from_secs(5 * 60)).await;
            }
        });
    }
    unsafe {
        enums::set_user();
    }
    
    get_pot_player::get_pot_first_info();
    // get_pot_player::get_player_list_file().await;
    //获取路由k
    // // 定义一个简单的 GET 路由 release go
    // let hello = warp::path!("hello" / String)
    //     .map(|name| format!("Hello, {}!", name));

    // // 组合路由
    // let route = hello;
    let route = router::get_router();
    let port: u16 = if cfg!(debug_assertions) { 7655 } else { 7654 };
    warp::serve(route)
        .run(([0, 0, 0, 0 ], port))
        .await; 
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
