use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::error::SnError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnConfig {
    pub base_date: BaseDate,
    pub base32_chars: String,
    pub pcb_sizes: HashMap<u8, String>,
    pub dram_sizes: HashMap<char, i32>,
    pub packages: HashMap<char, String>,
    pub chip_count: ChipCount,
    pub sn_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipCount {
    pub min: u8,
    pub max: u8,
}

impl Default for SnConfig {
    fn default() -> Self {
        let config_str = include_str!("../config/default.yaml");
        serde_yaml::from_str(config_str)
            .expect("Failed to parse default configuration")
    }
}

impl SnConfig {
    pub fn new() -> Result<Self, SnError> {
        Ok(Self::default())
    }

    pub fn validate(&self) -> Result<(), SnError> {
        if self.base32_chars.is_empty() {
            return Err(SnError::ConfigError("Base32字符集不能为空".to_string()));
        }
        if self.pcb_sizes.is_empty() {
            return Err(SnError::ConfigError("PCB尺寸定义不能为空".to_string()));
        }
        if self.dram_sizes.is_empty() {
            return Err(SnError::ConfigError("DRAM大小定义不能为空".to_string()));
        }
        if self.packages.is_empty() {
            return Err(SnError::ConfigError("封装定义不能为空".to_string()));
        }
        if self.chip_count.min > self.chip_count.max {
            return Err(SnError::ConfigError("颗粒数量范围无效".to_string()));
        }

        Ok(())
    }

    pub fn is_valid_pcb_size(&self, size: u8) -> bool {
        self.pcb_sizes.contains_key(&size)
    }

    pub fn is_valid_dram_code(&self, code: char) -> bool {
        self.dram_sizes.contains_key(&code.to_ascii_uppercase())
    }

    pub fn is_valid_package(&self, code: char) -> bool {
        self.packages.contains_key(&code.to_ascii_uppercase())
    }

    pub fn is_valid_chip_count(&self, count: u8) -> bool {
        count >= self.chip_count.min && count <= self.chip_count.max
    }

    pub fn get_pcb_size_name(&self, size: u8) -> String {
        self.pcb_sizes.get(&size)
            .cloned()
            .unwrap_or_else(|| "未知尺寸".to_string())
    }

    pub fn get_dram_size_desc(&self, code: char) -> String {
        self.dram_sizes.get(&code.to_ascii_uppercase())
            .map(|&size| {
                if size == -1 {
                    "DRAMLess".to_string()
                } else if size < 1024 {
                    format!("{}MB", size)
                } else {
                    format!("{}GB", size / 1024)
                }
            })
            .unwrap_or_else(|| "未知大小".to_string())
    }

    pub fn get_package_name(&self, code: char) -> String {
        self.packages.get(&code.to_ascii_uppercase())
            .cloned()
            .unwrap_or_else(|| "未知封装".to_string())
    }

    pub fn get_config_info(&self) -> String {
        format!(
            "配置信息:\n  基准日期: {}-{}-{}\n  PCB尺寸: {} 种\n  DRAM大小: {} 种\n  封装类型: {} 种\n  颗粒数量范围: {} - {} (1-F/G)\n  SN码格式: {}",
            self.base_date.year,
            self.base_date.month,
            self.base_date.day,
            self.pcb_sizes.len(),
            self.dram_sizes.len(),
            self.packages.len(),
            self.chip_count.min,
            self.chip_count.max,
            self.sn_format
        )
    }
}

lazy_static! {
    pub static ref CONFIG: SnConfig = SnConfig::new()
        .expect("Failed to load configuration");
}