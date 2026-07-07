use crate::controllers::{self, file};
use crate::get_pot_player;
use crate::mcgs_control;
use std::io;
use std::io::Write;

pub fn init_menu() -> () {
    loop {
        println!("\n请选择一个选项:");
        println!("1. get_play_list");
        println!("2. start_soft_server");
        println!("3. show mouse xy");
        println!("4. test");
        println!("5. 下载pot NT");
        println!("6. 下载pot ME");
        println!("7. 开始/停止采集微信聊天");
        println!("8. 重启MCGS下位机运行(停止+启动)");
        println!("9. 退出");

        print!("请输入您的选择: ");
        io::stdout().flush().unwrap(); // 确保提示信息立即显示

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("读取输入失败");

        let choice = choice.trim(); // 移除输入中的换行符和空格

        match choice {
            "1" => {
                tokio::spawn(async {
                    get_pot_player::get_player_list_file().await.unwrap();
                });
                ()
            }
            "2" => {
                tokio::spawn(async {
                    file::axum_init().await;
                });
                ()
                // 在这里添加执行操作 B 的代码
            }
            "3" => {
                println!("等待5秒");
                tokio::spawn(async {
                    tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
                    controllers::me::show_mouse_xy();
                });
            }
            "4" => {
                tokio::spawn(async {
                    get_pot_player::upload_play_list().await.unwrap();
                });
            }
            "5" => {
                tokio::spawn(async {
                    get_pot_player::down_server_play_list("nt".to_string())
                        .await
                        .unwrap();
                });
                ()
            }
            "6" => {
                tokio::spawn(async {
                    get_pot_player::down_server_play_list("me".to_string())
                        .await
                        .unwrap();
                });
                ()
            }
            "7" => {
                controllers::wechat_capture::toggle_wechat_capture();
            }
            "8" => {
                tokio::spawn(async {
                    mcgs_control::restart_lower_computer().await;
                });
            }
            "9" => {
                println!("退出程序。");
                break; // 退出循环
            }
            _ => {
                println!("无效的选择，请重新输入。");
            }
        }
    }
}
