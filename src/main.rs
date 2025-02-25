#![allow(unused_variables, dead_code)]
// use std::io;
// use utf8_slice::slice;
use tokio::time::{sleep, Duration};
use tokio::{self};
use warp::Filter;

// use std::thread::sleep;
// use std::time::Duration;

mod controllers;
mod enums;
mod get_pot_player;
mod router;
mod uitl;

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        loop {
            get_pot_player::save_pot_play_info().await;
            sleep(Duration::from_secs(5 * 60)).await;
        }
    });

    //获取路由te
    // // 定义一个简单的 GET 路由 release go
    // let hello = warp::path!("hello" / String)
    //     .map(|name| format!("Hello, {}!", name));

    // // 组合路由
    // let route = hello;
    let route = router::get_router();
    
    warp::serve(route)
        .run(([0, 0, 0, 0 ], 7654))
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
