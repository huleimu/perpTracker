use chrono::{DateTime, Utc};

/// 将时间戳转换为可读的时间格式
/// 
/// # 参数
/// * `timestamp` - 毫秒时间戳
/// 
/// # 返回
/// * `String` - 格式化的时间字符串(YYYY-MM-DD HH:MM:SS UTC)
/// 
/// # 示例
/// ```
/// let timestamp = 1640995200000; // 2022-01-01 00:00:00 UTC
/// let formatted = format_timestamp(timestamp);
/// assert_eq!(formatted, "2022-01-01 00:00:00 UTC");
/// ```
pub fn format_timestamp(timestamp: i64) -> String {
    DateTime::from_timestamp_millis(timestamp)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "时间解析失败".to_string())
}

/// 解析时间字符串为 DateTime<Utc>
/// 
/// # 参数
/// * `timestamp_str` - 时间字符串
/// 
/// # 返回
/// * `DateTime<Utc>` - 解析后的时间，失败时返回当前时间
/// 
/// # 示例
/// ```
/// let time = parse_timestamp_string("2022-01-01 00:00:00 UTC");
/// ```
pub fn parse_timestamp_string(timestamp_str: &str) -> DateTime<Utc> {
    // 尝试多种时间格式解析
    if let Ok(dt) = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S UTC") {
        dt.with_timezone(&Utc)
    } else if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S UTC") {
        naive_dt.and_utc()
    } else {
        // 如果解析失败，使用当前时间
        crate::log!(warn, "common_utils", "parse_timestamp_string", 
            "时间解析失败，使用当前时间", 
            "timestamp_str" => timestamp_str
        );
        Utc::now()
    }
}

/// 安全解析字符串为 f64，失败时返回默认值
/// 
/// # 参数
/// * `value` - 要解析的字符串
/// * `default` - 解析失败时的默认值
/// 
/// # 返回
/// * `f64` - 解析后的数值或默认值
/// 
/// # 示例
/// ```
/// let result = safe_parse_f64("123.45", 0.0);
/// assert_eq!(result, 123.45);
/// 
/// let result = safe_parse_f64("invalid", 0.0);
/// assert_eq!(result, 0.0);
/// ```
pub fn safe_parse_f64(value: &str, default: f64) -> f64 {
    value.parse().unwrap_or(default)
}

/// 统一的交易方向解析函数
pub fn parse_trade_direction(dir: &str) -> Result<crate::types::TradeDirectionInfo, anyhow::Error> {
    use crate::types::{TradeAction, TradeDirectionInfo};
    
    match dir {
        "Open Long" | "Buy" => Ok(TradeDirectionInfo {
            is_buy: true,
            reduce_only: false,
            action: TradeAction::Buy,
            description: "买入开仓",
            text: "开多仓",
        }),
        "Open Short" | "Sell" => Ok(TradeDirectionInfo {
            is_buy: false,
            reduce_only: false,
            action: TradeAction::Sell,
            description: "卖出开仓",
            text: "开空仓",
        }),
        "Close Long" => Ok(TradeDirectionInfo {
            is_buy: false,
            reduce_only: true,
            action: TradeAction::Sell,
            description: "卖出平仓",
            text: "平多仓",
        }),
        "Close Short" => Ok(TradeDirectionInfo {
            is_buy: true,
            reduce_only: true,
            action: TradeAction::Buy,
            description: "买入平仓",
            text: "平空仓",
        }),
        _ => Err(anyhow::anyhow!("无法确定交易方向: {}", dir)),
    }
}

pub fn parse_trade_side(side: &str) -> String {
    match side {
        "B" => "买入",              // Buy
        "A" => "卖出",              // Ask (卖出)
        _ => side,
    }.to_string()
}

/// 根据交易类型确定是否为吃单
/// 
/// # 参数
/// * `crossed` - 是否立即成交
/// 
/// # 返回
/// * `String` - 交易类型描述
/// 
/// # 示例
/// ```
/// let trade_type = get_trade_type(true);
/// assert_eq!(trade_type, "吃单(Taker)");
/// ```
pub fn get_trade_type(crossed: bool) -> String {
    if crossed {
        "吃单(Taker)".to_string()   // 立即成交，支付手续费
    } else {
        "挂单(Maker)".to_string()   // 提供流动性，获得返佣
    }
}

pub fn calculate_backoff_delay(retry_count: u32, base_delay: u64, max_delay: u64) -> u64 {
    let delay = base_delay * (1 << retry_count); // 2, 4, 8, 16, ...
    delay.min(max_delay)
}

/// 用户交互日志工具
/// 获取用户输入并记录到日志
pub fn prompt_user(prompt: &str) -> Result<String, std::io::Error> {
    use std::io::Write;
    
    print!("{}", prompt);
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();
    
    crate::log!(info, "common_utils", "prompt_user", 
        "用户交互", 
        "prompt" => prompt,
        "has_input" => !input.is_empty()
    );
    Ok(input)
}



 
