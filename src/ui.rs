use crate::firmware_codec::FirmwareCodec;
use crate::config::CONFIG;

// 应用状态
pub struct SsdToolApp {
    // 当前激活的标签页
    active_tab: Tab,

    // 固件版本号生成状态
    firmware_generate: FirmwareGenerateState,

    // 固件版本号解析状态
    firmware_parse: FirmwareParseState,

    // 错误信息
    error_message: Option<String>,

    // 成功信息
    success_message: Option<String>,
}

// 标签页枚举
#[derive(PartialEq)]
pub enum Tab {
    Firmware,
}

// 固件版本号生成状态
pub struct FirmwareGenerateState {
    pub year: String,
    pub month: String,
    pub day: String,
    pub pcb_size: String,
    pub dram_size: String,
    pub package_code: String,
    pub chip_count: String,
    pub generated_code: String,
}

// 固件版本号解析状态
pub struct FirmwareParseState {
    pub firmware_code: String,
    pub parsed_result: Option<ParsedFirmware>,
}

// 解析结果
pub struct ParsedFirmware {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub pcb_size: u8,
    pub dram_size_mb: i32,
    pub package_code: char,
    pub chip_count: u8,
}

impl Default for SsdToolApp {
    fn default() -> Self {
        Self {
            active_tab: Tab::Firmware,
            firmware_generate: FirmwareGenerateState {
                year: "2025".to_string(),
                month: "12".to_string(),
                day: "1".to_string(),
                pcb_size: "1".to_string(),
                dram_size: "1024".to_string(),
                package_code: "A".to_string(),
                chip_count: "4".to_string(),
                generated_code: String::new(),
            },
            firmware_parse: FirmwareParseState {
                firmware_code: String::new(),
                parsed_result: None,
            },
            error_message: None,
            success_message: None,
        }
    }
}

impl SsdToolApp {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    // 生成固件版本号
    pub fn generate_firmware(&mut self) {
        // 清空之前的消息
        self.error_message = None;
        self.success_message = None;

        // 解析输入参数
        let year = match self.firmware_generate.year.parse::<i32>() {
            Ok(year) if year >= 2025 && year <= 2099 => year,
            Ok(_) => {
                self.error_message = Some("年份必须在2025-2099之间".to_string());
                return;
            }
            Err(_) => {
                self.error_message = Some("无效的年份".to_string());
                return;
            }
        };

        let month = match self.firmware_generate.month.parse::<u32>() {
            Ok(month) if month >= 1 && month <= 12 => month,
            Ok(_) => {
                self.error_message = Some("月份必须在1-12之间".to_string());
                return;
            }
            Err(_) => {
                self.error_message = Some("无效的月份".to_string());
                return;
            }
        };

        let day = match self.firmware_generate.day.parse::<u32>() {
            Ok(day) if day >= 1 && day <= 31 => day,
            Ok(_) => {
                self.error_message = Some("日期必须在1-31之间".to_string());
                return;
            }
            Err(_) => {
                self.error_message = Some("无效的日期".to_string());
                return;
            }
        };

        let pcb_size = match self.firmware_generate.pcb_size.parse::<u8>() {
            Ok(size) if CONFIG.firmware.is_valid_pcb_size(size) => size,
            Ok(_) => {
                self.error_message = Some("无效的PCB尺寸代码".to_string());
                return;
            }
            Err(_) => {
                self.error_message = Some("无效的PCB尺寸".to_string());
                return;
            }
        };

        let dram_size_mb = match self.firmware_generate.dram_size.parse::<i32>() {
            Ok(size) if size == -1 || size > 0 => size,
            Ok(_) => {
                self.error_message = Some("DRAM大小必须为正数或-1(DRAMLess)".to_string());
                return;
            }
            Err(_) => {
                self.error_message = Some("无效的DRAM大小".to_string());
                return;
            }
        };

        let package_code = match self.firmware_generate.package_code.chars().next() {
            Some(code) if CONFIG.firmware.is_valid_package(code) => code,
            Some(_) => {
                self.error_message = Some("无效的封装代码".to_string());
                return;
            }
            None => {
                self.error_message = Some("请输入封装代码".to_string());
                return;
            }
        };

        let chip_count = match self.firmware_generate.chip_count.parse::<u8>() {
            Ok(count) if CONFIG.firmware.is_valid_chip_count(count) => count,
            Ok(_) => {
                self.error_message = Some("颗粒数量必须在1-16之间".to_string());
                return;
            }
            Err(_) => {
                self.error_message = Some("无效的颗粒数量".to_string());
                return;
            }
        };

        // 生成固件版本号
        match FirmwareCodec::generate_firmware_code(
            year, month, day, pcb_size, dram_size_mb, package_code, chip_count
        ) {
            Ok(code) => {
                self.firmware_generate.generated_code = code;
                self.success_message = Some("固件版本号生成成功！".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("生成失败: {}", e));
            }
        }
    }

    // 解析固件版本号
    pub fn parse_firmware(&mut self) {
        // 清空之前的消息
        self.error_message = None;
        self.success_message = None;
        self.firmware_parse.parsed_result = None;

        let code = self.firmware_parse.firmware_code.trim();
        if code.is_empty() {
            self.error_message = Some("请输入固件版本号".to_string());
            return;
        }

        match FirmwareCodec::parse_firmware_code(code) {
            Ok((year, month, day, pcb_size, dram_size_mb, package_code, chip_count)) => {
                self.firmware_parse.parsed_result = Some(ParsedFirmware {
                    year,
                    month,
                    day,
                    pcb_size,
                    dram_size_mb,
                    package_code,
                    chip_count,
                });
                self.success_message = Some("固件版本号解析成功！".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("解析失败: {}", e));
            }
        }
    }

    // 显示错误消息
    fn show_error(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::RED, message);
    }

    // 显示成功消息
    fn show_success(&self, ui: &mut egui::Ui, message: &str) {
        ui.colored_label(egui::Color32::GREEN, message);
    }

    #[allow(dead_code)]
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    // 渲染UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // 标签页选择
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, Tab::Firmware, "🔧 固件版本号");
        });

        ui.separator();

        // 显示消息
        if let Some(error) = &self.error_message {
            self.show_error(ui, error);
        }
        if let Some(success) = &self.success_message {
            self.show_success(ui, success);
        }

        ui.add_space(10.0);

        // 根据当前标签页显示内容
        match self.active_tab {
            Tab::Firmware => self.firmware_ui(ui),
        }
    }

    // 固件版本号UI
    fn firmware_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("固件版本号工具");
            ui.label("格式: S + 日期编码(3) + PCB尺寸(1) + DRAM大小(1) + 封装(1) + 颗粒数(1)");

            ui.add_space(15.0);

            // 生成固件版本号部分
            ui.heading("生成固件版本号");
            self.firmware_generate_ui(ui);

            ui.add_space(20.0);

            // 解析固件版本号部分
            ui.heading("解析固件版本号");
            self.firmware_parse_ui(ui);
        });
    }

    // 固件版本号生成UI
    fn firmware_generate_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("firmware_generate_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // 生产日期
                ui.label("📅 生产日期:");
                ui.horizontal(|ui| {
                    ui.label("年");
                    ui.add(egui::TextEdit::singleline(&mut self.firmware_generate.year)
                        .desired_width(60.0));
                    ui.label("月");
                    ui.add(egui::TextEdit::singleline(&mut self.firmware_generate.month)
                        .desired_width(40.0));
                    ui.label("日");
                    ui.add(egui::TextEdit::singleline(&mut self.firmware_generate.day)
                        .desired_width(40.0));
                });
                ui.end_row();

                // PCB尺寸
                ui.label("📐 PCB尺寸:");
                ui.horizontal(|ui| {
                    let pcb_size_text = format!("{} - {}",
                                                self.firmware_generate.pcb_size,
                                                CONFIG.firmware.get_pcb_size_name(
                                                    self.firmware_generate.pcb_size.parse().unwrap_or(0)
                                                )
                    );

                    egui::ComboBox::from_id_source("pcb_size")
                        .selected_text(pcb_size_text)
                        .show_ui(ui, |ui| {
                            for (code, desc) in &CONFIG.firmware.pcb_sizes {
                                let code_str = code.to_string();
                                if ui.selectable_label(
                                    &self.firmware_generate.pcb_size == &code_str,
                                    format!("{} - {}", code, desc)
                                ).clicked() {
                                    self.firmware_generate.pcb_size = code_str;
                                }
                            }
                        });
                });
                ui.end_row();

                // DRAM大小
                ui.label("💾 DRAM大小:");
                ui.horizontal(|ui| {
                    if ui.button("DRAMLess").clicked() {
                        self.firmware_generate.dram_size = "-1".to_string();
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.firmware_generate.dram_size)
                        .desired_width(80.0));
                    ui.label("MB");
                });
                ui.end_row();

                // 封装类型
                ui.label("📦 封装类型:");
                let package_text = format!("{} - {}",
                                           self.firmware_generate.package_code,
                                           CONFIG.firmware.get_package_name(
                                               self.firmware_generate.package_code.chars().next().unwrap_or('0')
                                           )
                );

                egui::ComboBox::from_id_source("package")
                    .selected_text(package_text)
                    .show_ui(ui, |ui| {
                        for (code, desc) in &CONFIG.firmware.packages {
                            let code_str = code.to_string();
                            if ui.selectable_label(
                                &self.firmware_generate.package_code == &code_str,
                                format!("{} - {}", code, desc)
                            ).clicked() {
                                self.firmware_generate.package_code = code_str;
                            }
                        }
                    });
                ui.end_row();

                // 颗粒数量
                ui.label("🔢 颗粒数量:");
                ui.add(egui::TextEdit::singleline(&mut self.firmware_generate.chip_count)
                    .desired_width(60.0));
                ui.end_row();
            });

        ui.add_space(10.0);

        // 生成按钮
        if ui.button("🚀 生成固件版本号").clicked() {
            self.generate_firmware();
        }

        // 显示生成结果
        if !self.firmware_generate.generated_code.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            ui.heading("生成结果");

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("固件版本号:");
                        ui.monospace(&self.firmware_generate.generated_code);
                        if ui.button("📋").clicked() {
                            ui.ctx().copy_text(self.firmware_generate.generated_code.clone());
                        }
                    });
                });
        }
    }

    // 固件版本号解析UI
    fn firmware_parse_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("固件版本号:");
            ui.add(egui::TextEdit::singleline(&mut self.firmware_parse.firmware_code)
                .desired_width(150.0));

            if ui.button("🔍 解析").clicked() {
                self.parse_firmware();
            }
        });

        // 显示解析结果
        if let Some(parsed) = &self.firmware_parse.parsed_result {
            ui.add_space(10.0);
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    egui::Grid::new("parse_result_grid")
                        .num_columns(2)
                        .spacing([10.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("生产日期:");
                            ui.label(format!("{}-{:02}-{:02}",
                                             parsed.year, parsed.month, parsed.day));
                            ui.end_row();

                            ui.label("PCB尺寸:");
                            ui.label(format!("{} ({})",
                                             parsed.pcb_size,
                                             CONFIG.firmware.get_pcb_size_name(parsed.pcb_size)));
                            ui.end_row();

                            ui.label("DRAM大小:");
                            if parsed.dram_size_mb == -1 {
                                ui.label("DRAMLess");
                            } else {
                                ui.label(format!("{}MB", parsed.dram_size_mb));
                            }
                            ui.end_row();

                            ui.label("封装类型:");
                            ui.label(format!("{} ({})",
                                             parsed.package_code,
                                             CONFIG.firmware.get_package_name(parsed.package_code)));
                            ui.end_row();

                            ui.label("颗粒数量:");
                            ui.label(parsed.chip_count.to_string());
                            ui.end_row();
                        });
                });
        }
    }
}

// 为 eframe::App trait 实现必要的方法
impl eframe::App for SsdToolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}