// build.rs
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 1. 获取输出目录 (target/debug/build/...)
    // 注意：Rust 构建脚本无法直接获取最终的 target/debug 目录，
    // 但通常我们可以把文件复制到 Cargo 告诉我们的 OUT_DIR，或者直接操作相对路径。

    // 简单粗暴且有效的方法：
    // 直接读取项目根目录的 mp3，复制到生成的 exe 旁边。

    // 获取 manifest (Cargo.toml) 所在的目录
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let source_path = Path::new(&manifest_dir).join("src/ui/bingbongbangbong.MP3");

    // 获取构建目标目录 (这就有点 tricky，但通常对于 cargo run 来说，
    // 我们想要的是 target/debug)。
    // 这里为了简单，我们使用 profile 获取构建类型 (debug 或 release)
    let profile = env::var("PROFILE").unwrap();
    let target_dir = Path::new(&manifest_dir).join("target").join(profile);

    // 目标文件路径
    let dest_path = target_dir.join("bingbongbangbong.MP3");

    // 执行复制
    // 只有当文件存在时才复制
    if source_path.exists() {
        // 创建目录以防万一（虽然 target/debug 通常已存在）
        fs::create_dir_all(&target_dir).ok();

        match fs::copy(&source_path, &dest_path) {
            Ok(_) => println!("cargo:warning=已将 music.mp3 复制到构建目录"),
            Err(e) => println!("cargo:warning=复制资源失败: {}", e),
        }
    } else {
        println!("cargo:warning=未找到 music.mp3，请确保它在项目根目录");
    }

    // 【关键】：告诉 Cargo，如果 music.mp3 变了，要重新运行这个脚本
    println!("cargo:rerun-if-changed=music.mp3");
}
