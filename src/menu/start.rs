
use std::io;
use std::io::Write;
use crate::controllers::file;
use crate::get_pot_player;

pub fn init_menu() -> () {
  loop {
    println!("\n请选择一个选项:");
    println!("1. get_play_list");
    println!("2. start_soft_server");
    println!("3. 退出");

    print!("请输入您的选择: ");
    io::stdout().flush().unwrap(); // 确保提示信息立即显示

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("读取输入失败");

    let choice = choice.trim(); // 移除输入中的换行符和空格

    match choice {
        "1" =>  {
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
            println!("退出程序。");
            break; // 退出循环
        }
        _ => {
            println!("无效的选择，请重新输入。");
        }
    }
}
}