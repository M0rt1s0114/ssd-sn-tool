// src/main.rs

// Windows 特定的代码：隐藏控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod firmware_codec;
mod error;
mod ui;

use eframe::egui;
use ui::SsdToolApp;

fn main() -> Result<(), eframe::Error> {
    // 验证配置
    if let Err(e) = config::CONFIG.firmware.validate() {
        eprintln!("固件配置验证失败: {}", e);
        std::process::exit(1);
    }

    // 启动 GUI 应用
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("SSD 工具集 - 固件版本号生成解析"),
        ..Default::default()
    };

    eframe::run_native(
        "SSD Tool",
        options,
        Box::new(|cc| {
            // 设置中文字体
            setup_fonts(&cc.egui_ctx);
            Box::<SsdToolApp>::default()
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 嵌入 Misans 字体
    fonts.font_data.insert(
        "misans".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/Misans.ttf")),
    );

    // 设置为主要字体
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "misans".to_owned());

    ctx.set_fonts(fonts);
}