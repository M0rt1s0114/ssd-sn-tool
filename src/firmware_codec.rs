use chrono::{DateTime, NaiveDate, TimeZone, Utc, Duration, Datelike};
use crate::config::CONFIG;
use crate::error::SnError;

pub struct FirmwareCodec;

impl FirmwareCodec {
    pub fn get_base_date() -> Result<DateTime<Utc>, SnError> {
        let base_date = &CONFIG.firmware.base_date;

        let date = NaiveDate::from_ymd_opt(base_date.year, base_date.month, base_date.day)
            .ok_or_else(|| SnError::DateCodeError("无效的基准日期".to_string()))?;

        let datetime = date.and_hms_opt(12, 0, 0)
            .ok_or_else(|| SnError::DateCodeError("无效的基准日期时间".to_string()))?;

        Ok(Utc.from_utc_datetime(&datetime))
    }

    pub fn date_to_code(year: i32, month: u32, day: u32) -> Result<String, SnError> {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| SnError::DateCodeError("无效的目标日期".to_string()))?;

        let datetime = date.and_hms_opt(12, 0, 0)
            .ok_or_else(|| SnError::DateCodeError("无效的目标日期时间".to_string()))?;

        let target_date = Utc.from_utc_datetime(&datetime);
        let base_date = Self::get_base_date()?;

        let duration = target_date - base_date;
        let days = duration.num_days();

        if days < 0 || days >= 32768 {
            return Err(SnError::DateCodeError("日期超出范围".to_string()));
        }

        let base32_chars = CONFIG.firmware.base32_chars.as_bytes();
        let base = base32_chars.len() as i64;
        let mut days_val = days;
        let mut code = String::with_capacity(3);

        for _ in 0..3 {
            let index = (days_val % base) as usize;
            code.insert(0, base32_chars[index] as char);
            days_val /= base;
        }

        Ok(code)
    }

    pub fn code_to_date(code: &str) -> Result<(i32, u32, u32), SnError> {
        if code.len() != 3 {
            return Err(SnError::DateCodeError("日期编码必须是3位字符".to_string()));
        }

        let base32_chars = CONFIG.firmware.base32_chars.as_bytes();
        let base = base32_chars.len();
        let mut days: i64 = 0;

        for c in code.chars() {
            let upper_c = c.to_ascii_uppercase();
            let pos = base32_chars.iter()
                .position(|&ch| ch == upper_c as u8)
                .ok_or_else(|| SnError::DateCodeError(format!("无效日期编码字符: {}", c)))?;

            days = days * base as i64 + pos as i64;
        }

        let base_date = Self::get_base_date()?;
        let target_date = base_date + Duration::days(days);

        let naive_date = target_date.naive_utc().date();
        let year = naive_date.year();
        let month = naive_date.month();
        let day = naive_date.day();

        Ok((year, month, day))
    }

    pub fn dram_size_to_code(size_mb: i32) -> Result<char, SnError> {
        if size_mb == -1 {
            return Ok('X');
        }

        CONFIG.firmware.dram_sizes.iter()
            .find(|(&code, &size)| code != 'X' && size == size_mb)
            .map(|(&code, _)| code)
            .ok_or_else(|| SnError::InvalidParameter("不支持的DRAM大小".to_string()))
    }

    pub fn chip_count_to_char(count: u8) -> Result<char, SnError> {
        match count {
            1..=9 => Ok((b'0' + count) as char),
            10..=15 => Ok((b'A' + (count - 10)) as char),
            16 => Ok('G'),
            _ => Err(SnError::InvalidParameter("颗粒个数超出范围 (1-16)".to_string()))
        }
    }

    pub fn char_to_chip_count(c: char) -> Result<u8, SnError> {
        match c {
            '1'..='9' => Ok(c as u8 - b'0'),
            'A'..='F' => Ok(10 + (c as u8 - b'A')),
            'a'..='f' => Ok(10 + (c as u8 - b'a')),
            'G' | 'g' => Ok(16),
            _ => Err(SnError::InvalidParameter("无效的颗粒个数代码".to_string()))
        }
    }

    pub fn generate_firmware_code(
        year: i32,
        month: u32,
        day: u32,
        pcb_size: u8,
        dram_size_mb: i32,
        package_code: char,
        chip_count: u8,
    ) -> Result<String, SnError> {
        // 验证输入参数
        if !CONFIG.firmware.is_valid_pcb_size(pcb_size) {
            return Err(SnError::InvalidParameter("无效的PCB尺寸代码".to_string()));
        }

        if !CONFIG.firmware.is_valid_chip_count(chip_count) {
            return Err(SnError::InvalidParameter("颗粒个数超出范围 (1-16)".to_string()));
        }

        if !CONFIG.firmware.is_valid_package(package_code) {
            return Err(SnError::InvalidParameter("无效的封装代码".to_string()));
        }

        let date_code = Self::date_to_code(year, month, day)?;
        let dram_code = Self::dram_size_to_code(dram_size_mb)?;
        let chip_char = Self::chip_count_to_char(chip_count)?;

        let firmware_code = format!(
            "S{}{}{}{}{}",
            date_code,
            pcb_size,
            dram_code,
            package_code.to_ascii_uppercase(),
            chip_char
        );

        Ok(firmware_code)
    }

    pub fn parse_firmware_code(firmware_code: &str) -> Result<(i32, u32, u32, u8, i32, char, u8), SnError> {
        if firmware_code.len() != 8 || !firmware_code.starts_with('S') {
            return Err(SnError::SnFormatError("无效的固件版本号格式".to_string()));
        }

        let chars: Vec<char> = firmware_code.chars().collect();

        // 解析日期
        let date_code: String = chars[1..4].iter().collect();
        let (year, month, day) = Self::code_to_date(&date_code)?;

        // 解析PCB尺寸
        let pcb_size = chars[4].to_digit(10)
            .ok_or_else(|| SnError::SnFormatError("无效的PCB尺寸代码".to_string()))? as u8;

        if !CONFIG.firmware.is_valid_pcb_size(pcb_size) {
            return Err(SnError::SnFormatError("无效的PCB尺寸代码".to_string()));
        }

        // 解析DRAM大小
        let dram_code = chars[5];
        if !CONFIG.firmware.is_valid_dram_code(dram_code) {
            return Err(SnError::SnFormatError("无效的DRAM大小代码".to_string()));
        }
        let dram_size_mb = *CONFIG.firmware.dram_sizes.get(&dram_code.to_ascii_uppercase())
            .unwrap();

        // 解析颗粒封装
        let package_code = chars[6];
        if !CONFIG.firmware.is_valid_package(package_code) {
            return Err(SnError::SnFormatError("无效的封装代码".to_string()));
        }

        // 解析颗粒个数
        let chip_count = Self::char_to_chip_count(chars[7])?;
        if !CONFIG.firmware.is_valid_chip_count(chip_count) {
            return Err(SnError::SnFormatError("无效的颗粒个数".to_string()));
        }

        Ok((year, month, day, pcb_size, dram_size_mb, package_code, chip_count))
    }

    #[allow(dead_code)]
    pub fn print_usage() {
        println!("固件版本号生成解析工具");
        println!("固件版本号格式: {}", CONFIG.firmware.format);
        println!("\n用法:");
        println!("  生成固件版本号: ssd_tool firmware generate <年> <月> <日> <PCB尺寸> <DRAM大小MB> <封装代码> <颗粒数>");
        println!("  解析固件版本号: ssd_tool firmware parse <固件版本号>");
        println!("  查看配置: ssd_tool firmware config");
        println!("\n示例:");
        println!("  生成(有DRAM): ssd_tool firmware generate 2025 12 1 1 1024 A 4");
        println!("  生成(DRAMLess): ssd_tool firmware generate 2025 12 1 1 -1 A 4");
        println!("  生成(16颗粒): ssd_tool firmware generate 2025 12 1 1 1024 A 16");
        println!("  解析: ssd_tool firmware parse S01E1A4");
    }
}