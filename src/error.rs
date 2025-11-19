use std::fmt;

#[derive(Debug)]
pub enum SnError {
    ConfigError(String),
    DateCodeError(String),
    SnFormatError(String),
    InvalidParameter(String),
    // ParseError(String),
}

impl std::error::Error for SnError {}

impl fmt::Display for SnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            SnError::DateCodeError(msg) => write!(f, "日期编码错误: {}", msg),
            SnError::SnFormatError(msg) => write!(f, "SN码格式错误: {}", msg),
            SnError::InvalidParameter(msg) => write!(f, "无效的参数: {}", msg),
            // SnError::ParseError(msg) => write!(f, "解析错误: {}", msg),
        }
    }
}