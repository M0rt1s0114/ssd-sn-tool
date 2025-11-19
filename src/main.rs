mod config;
mod codec;
mod error;

use std::io::{self, Write};
use config::CONFIG;
use codec::SnCodec;
use error::SnError;

fn show_interactive_menu() {
    println!("\n========================================");
    println!("    固态硬盘SN与固件号生成解析工具");
    println!("========================================");
    println!("1. 生成SN码");
    println!("2. 解析SN码");
    println!("3. 查看配置信息");
    println!("4. 退出程序");
    println!("========================================");
    print!("请选择操作 (1-4): ");
    io::stdout().flush().unwrap();
}

fn interactive_generate() -> Result<(), SnError> {
    println!("\n--- 生成SN码 ---");

    // 获取生产日期
    println!("请输入生产日期:");
    print!("  年 (2025-2099): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let year: i32 = input.trim().parse().map_err(|_| SnError::InvalidParameter("无效的年份".to_string()))?;

    print!("  月 (1-12): ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let month: u32 = input.trim().parse().map_err(|_| SnError::InvalidParameter("无效的月份".to_string()))?;

    print!("  日 (1-31): ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let day: u32 = input.trim().parse().map_err(|_| SnError::InvalidParameter("无效的日期".to_string()))?;

    // 获取PCB尺寸
    println!("\n请选择PCB尺寸:");
    for (code, desc) in &CONFIG.pcb_sizes {
        println!("  {}: {}", code, desc);
    }
    print!("PCB尺寸代码: ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let pcb_size: u8 = input.trim().parse().map_err(|_| SnError::InvalidParameter("无效的PCB尺寸".to_string()))?;

    // 获取DRAM大小
    println!("\n请选择DRAM大小:");
    println!("  -1: DRAMLess");
    for (code, &size) in &CONFIG.dram_sizes {
        if *code != 'X' {
            println!("  {}: {}", size, CONFIG.get_dram_size_desc(*code));
        }
    }
    print!("DRAM大小(MB, 输入-1表示DRAMLess): ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let dram_size_mb: i32 = input.trim().parse().map_err(|_| SnError::InvalidParameter("无效的DRAM大小".to_string()))?;

    // 获取封装类型
    println!("\n请选择封装类型:");
    for (code, desc) in &CONFIG.packages {
        println!("  {}: {}", code, desc);
    }
    print!("封装代码: ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let package_code = input.trim().chars().next().ok_or_else(|| SnError::InvalidParameter("无效的封装代码".to_string()))?;

    // 获取颗粒数量
    print!("\n请输入颗粒数量 (1-16): ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let chip_count: u8 = input.trim().parse().map_err(|_| SnError::InvalidParameter("无效的颗粒数量".to_string()))?;

    // 生成SN码
    let sn_code = SnCodec::generate_sn_code(year, month, day, pcb_size, dram_size_mb, package_code, chip_count)?;

    println!("\n========================================");
    println!("生成的SN码: {}", sn_code);
    println!("========================================");

    // 显示详细信息
    println!("\n详细信息:");
    println!("  生产日期: {}-{:02}-{:02}", year, month, day);
    println!("  PCB尺寸: {} ({})", pcb_size, CONFIG.get_pcb_size_name(pcb_size));

    let dram_code = SnCodec::dram_size_to_code(dram_size_mb)?;
    println!("  DRAM大小: {}", CONFIG.get_dram_size_desc(dram_code));

    println!("  颗粒封装: {} ({})", package_code, CONFIG.get_package_name(package_code));
    println!("  颗粒个数: {}", chip_count);

    Ok(())
}

fn interactive_parse() -> Result<(), SnError> {
    println!("\n--- 解析SN码 ---");

    print!("请输入SN码: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let sn_code = input.trim();

    let (year, month, day, pcb_size, dram_size_mb, package_code, chip_count) =
        SnCodec::parse_sn_code(sn_code)?;

    println!("\n========================================");
    println!("SN码: {}", sn_code);
    println!("========================================");

    println!("\n解析结果:");
    println!("  生产日期: {}-{:02}-{:02}", year, month, day);
    println!("  PCB尺寸: {} ({})", pcb_size, CONFIG.get_pcb_size_name(pcb_size));

    if dram_size_mb == -1 {
        println!("  DRAM大小: DRAMLess");
    } else {
        println!("  DRAM大小: {}MB", dram_size_mb);
    }

    println!("  颗粒封装: {} ({})", package_code, CONFIG.get_package_name(package_code));
    println!("  颗粒个数: {}", chip_count);

    Ok(())
}

fn handle_command_line() -> Result<(), SnError> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("generate") if args.len() == 9 => {
            let year: i32 = args[2].parse().map_err(|_| SnError::InvalidParameter("无效的年份".to_string()))?;
            let month: u32 = args[3].parse().map_err(|_| SnError::InvalidParameter("无效的月份".to_string()))?;
            let day: u32 = args[4].parse().map_err(|_| SnError::InvalidParameter("无效的日期".to_string()))?;
            let pcb_size: u8 = args[5].parse().map_err(|_| SnError::InvalidParameter("无效的PCB尺寸".to_string()))?;
            let dram_size_mb: i32 = args[6].parse().map_err(|_| SnError::InvalidParameter("无效的DRAM大小".to_string()))?;
            let package_code = args[7].chars().next().ok_or_else(|| SnError::InvalidParameter("无效的封装代码".to_string()))?;
            let chip_count: u8 = args[8].parse().map_err(|_| SnError::InvalidParameter("无效的颗粒数量".to_string()))?;

            let sn_code = SnCodec::generate_sn_code(year, month, day, pcb_size, dram_size_mb, package_code, chip_count)?;
            println!("生成的SN码: {}", sn_code);
        }
        Some("parse") if args.len() == 3 => {
            let sn_code = &args[2];
            let (year, month, day, pcb_size, dram_size_mb, package_code, chip_count) =
                SnCodec::parse_sn_code(sn_code)?;

            println!("SN码: {}", sn_code);
            println!("生产日期: {}-{}-{}", year, month, day);
            println!("PCB尺寸: {} ({})", pcb_size, CONFIG.get_pcb_size_name(pcb_size));

            if dram_size_mb == -1 {
                println!("DRAM大小: DRAMLess");
            } else {
                println!("DRAM大小: {}MB", dram_size_mb);
            }

            println!("颗粒封装: {} ({})", package_code, CONFIG.get_package_name(package_code));
            println!("颗粒个数: {}", chip_count);
        }
        Some("config") if args.len() == 2 => {
            println!("{}", CONFIG.get_config_info());
        }
        _ => {
            SnCodec::print_usage();
            return Err(SnError::InvalidParameter("无效的命令行参数".to_string()));
        }
    }

    Ok(())
}

fn main() {
    // 验证配置
    if let Err(e) = CONFIG.validate() {
        eprintln!("配置验证失败: {}", e);
        std::process::exit(1);
    }

    // 如果有命令行参数，使用命令行模式
    if std::env::args().len() > 1 {
        if let Err(e) = handle_command_line() {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // 交互模式
    println!("固态硬盘SN与固件号生成解析工具 (Rust版本)");

    loop {
        show_interactive_menu();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                if let Err(e) = interactive_generate() {
                    eprintln!("错误: {}", e);
                }
            }
            "2" => {
                if let Err(e) = interactive_parse() {
                    eprintln!("错误: {}", e);
                }
            }
            "3" => {
                println!("\n{}", CONFIG.get_config_info());
            }
            "4" => {
                println!("不要继续做硬盘啦，休息一下吧（");
                break;
            }
            _ => {
                println!("无效选择，请重新输入！");
                continue;
            }
        }

        println!("\n按回车键继续...");
        io::stdin().read_line(&mut input).unwrap();
    }
}