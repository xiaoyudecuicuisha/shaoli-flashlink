//! 构建脚本：注入构建时间环境变量，执行 Tauri 构建流程

fn main() {
    // 注入构建时间
    let build_time = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    tauri_build::build()
}
