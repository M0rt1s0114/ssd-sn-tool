// build.rs
fn main() {
    println!("cargo:warning=构建脚本开始执行");

    // 只在 Windows 平台上设置
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:warning=检测到 Windows 平台");

        // 设置 Windows 子系统为 GUI（隐藏控制台窗口）
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");

        // 检查图标文件是否存在
        let icon_path = "assets/icon.ico";
        if !std::path::Path::new(icon_path).exists() {
            eprintln!("cargo:warning=图标文件 {} 不存在", icon_path);
            return;
        }

        println!("cargo:warning=找到图标文件: {} (106KB)", icon_path);

        // 创建资源文件内容
        let rc_content = r#"
MAINICON ICON "assets/icon.ico"
"#;

        // 写入资源文件
        if let Err(e) = std::fs::write("app.rc", rc_content) {
            eprintln!("cargo:warning=无法创建资源文件: {}", e);
            return;
        }

        // 使用 embed-resource 编译资源
        embed_resource::compile("app.rc", embed_resource::NONE);
        println!("cargo:warning=使用 embed-resource 设置图标成功");

        // 清理临时文件
        let _ = std::fs::remove_file("app.rc");
    } else {
        println!("cargo:warning=非 Windows 平台，跳过图标设置");
    }
}