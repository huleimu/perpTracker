//! # Hyperliquid 波动率计算示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 获取K线数据并计算资产的波动率。
//!
//! ## 什么是波动率？
//! 
//! 波动率 (Volatility) 是衡量价格变动剧烈程度的统计指标：
//! - **高波动率**: 价格变化大，风险高，但潜在收益也高
//! - **低波动率**: 价格变化小，相对稳定，风险较低
//! - **计算公式**: 通常使用价格收益率的标准差
//! - **年化波动率**: 将日波动率转换为年化数值，便于比较
//!
//! ## 计算方法
//!
//! 1. 获取历史K线数据（包含开高低收价格）
//! 2. 计算每日收益率：(今日收盘价 - 昨日收盘价) / 昨日收盘价
//! 3. 计算收益率的标准差
//! 4. 年化处理：日波动率 × √365
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin volatility_calculator
//! ```

use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use log::info;
use std::f64;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始获取K线数据并计算波动率...");
    println!("=======================================================");
    // =========================================================

    // ---- Part 1: 初始化客户端 ----
    
    println!("[步骤 1/4] 正在初始化 InfoClient (连接到测试网)...");
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    
    // 指定要分析的资产和时间范围
    let coin = "ETH";
    let interval = "1h"; // 1小时K线
    let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let start_timestamp = end_timestamp - (30 * 24 * 60 * 60 * 1000); // 30天前
    
    println!("    - 分析资产: {}", coin);
    println!("    - K线间隔: {}", interval);
    println!("    - 时间范围: {} 天", (end_timestamp - start_timestamp) / (24 * 60 * 60 * 1000));

    // ---- Part 2: 获取历史K线数据 ----
    
    println!("\n[步骤 2/4] 正在获取历史K线数据...");
    
    match info_client
        .candles_snapshot(coin.to_string(), interval.to_string(), start_timestamp, end_timestamp)
        .await 
    {
        Ok(candles) => {
            println!("    - 成功获取 {} 根K线", candles.len());
            
            if candles.len() < 2 {
                println!("K线数据不足，无法计算波动率");
                return;
            }

            // ---- Part 3: 计算波动率 ----
            
            println!("\n[步骤 3/4] 正在计算波动率...");
            
            let volatilities = calculate_volatility(&candles);
            
            // ---- Part 4: 展示结果 ----
            
            println!("\n[步骤 4/4] 波动率分析结果:");
            println!("=======================================================");
            
            display_volatility_results(coin, &volatilities, &candles);
            
        }
        Err(e) => {
            println!("获取K线数据失败: {:?}", e);
            info!("Failed to get candles data: {:?}", e);
        }
    }
    
    println!("=======================================================");
    println!("波动率计算完成！");
}

/// 波动率计算结果结构体
#[derive(Debug)]
struct VolatilityResult {
    daily_volatility: f64,      // 日波动率
    weekly_volatility: f64,     // 周波动率  
    monthly_volatility: f64,    // 月波动率
    annual_volatility: f64,     // 年化波动率
    average_return: f64,        // 平均收益率
    max_daily_change: f64,      // 最大日变化
    min_daily_change: f64,      // 最小日变化
}

/// 计算各种时间维度的波动率
fn calculate_volatility(candles: &[hyperliquid_rust_sdk::CandlesSnapshotResponse]) -> VolatilityResult {
    // 计算每根K线的收益率
    let mut returns = Vec::new();
    
    for i in 1..candles.len() {
        let prev_close: f64 = candles[i-1].close.parse().unwrap_or(0.0);
        let current_close: f64 = candles[i].close.parse().unwrap_or(0.0);
        
        if prev_close > 0.0 {
            let return_rate = (current_close - prev_close) / prev_close;
            returns.push(return_rate);
        }
    }
    
    if returns.is_empty() {
        return VolatilityResult {
            daily_volatility: 0.0,
            weekly_volatility: 0.0,
            monthly_volatility: 0.0,
            annual_volatility: 0.0,
            average_return: 0.0,
            max_daily_change: 0.0,
            min_daily_change: 0.0,
        };
    }
    
    // 计算平均收益率
    let average_return = returns.iter().sum::<f64>() / returns.len() as f64;
    
    // 计算方差（收益率与平均值的差的平方和）
    let variance = returns.iter()
        .map(|r| (r - average_return).powi(2))
        .sum::<f64>() / returns.len() as f64;
    
    // 计算标准差（波动率）
    let std_dev = variance.sqrt();
    
    // 根据K线间隔调整时间因子
    let time_factor: f64 = match candles.first().unwrap().candle_interval.as_str() {
        "1m" => 1440.0,    // 1分钟K线，一天1440分钟
        "5m" => 288.0,     // 5分钟K线，一天288个
        "15m" => 96.0,     // 15分钟K线
        "1h" => 24.0,      // 1小时K线，一天24小时
        "4h" => 6.0,       // 4小时K线
        "1d" => 1.0,       // 日K线
        _ => 24.0,         // 默认按小时计算
    };
    
    // 计算不同时间维度的波动率
    let daily_volatility = std_dev * time_factor.sqrt();
    let weekly_volatility = daily_volatility * (7.0_f64).sqrt();
    let monthly_volatility = daily_volatility * (30.0_f64).sqrt();
    let annual_volatility = daily_volatility * (365.0_f64).sqrt();
    
    // 找出最大最小变化
    let max_daily_change = returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_daily_change = returns.iter().cloned().fold(f64::INFINITY, f64::min);
    
    VolatilityResult {
        daily_volatility,
        weekly_volatility,
        monthly_volatility,
        annual_volatility,
        average_return,
        max_daily_change,
        min_daily_change,
    }
}

/// 展示波动率分析结果
fn display_volatility_results(
    coin: &str, 
    volatility: &VolatilityResult, 
    candles: &[hyperliquid_rust_sdk::CandlesSnapshotResponse]
) {
    println!("{} 波动率分析报告", coin);
    println!();
    
    // 基本信息
    println!("基本信息:");
    println!("  ├─ 分析周期: {} 根K线", candles.len());
    println!("  ├─ K线间隔: {}", candles.first().unwrap().candle_interval);
    if let (Some(first), Some(last)) = (candles.first(), candles.last()) {
        let start_time = chrono::DateTime::from_timestamp_millis(first.time_open as i64)
            .unwrap_or_default();
        let end_time = chrono::DateTime::from_timestamp_millis(last.time_close as i64)
            .unwrap_or_default();
        println!("  ├─ 开始时间: {}", start_time.format("%Y-%m-%d %H:%M:%S"));
        println!("  └─ 结束时间: {}", end_time.format("%Y-%m-%d %H:%M:%S"));
    }
    println!();
    
    // 波动率数据
    println!("波动率指标:");
    println!("  ├─ 日波动率: {:.4} ({:.2}%)", volatility.daily_volatility, volatility.daily_volatility * 100.0);
    println!("  ├─ 周波动率: {:.4} ({:.2}%)", volatility.weekly_volatility, volatility.weekly_volatility * 100.0);
    println!("  ├─ 月波动率: {:.4} ({:.2}%)", volatility.monthly_volatility, volatility.monthly_volatility * 100.0);
    println!("  └─ 年化波动率: {:.4} ({:.2}%)", volatility.annual_volatility, volatility.annual_volatility * 100.0);
    println!();
    
    // 收益率数据
    println!("收益率统计:");
    println!("  ├─ 平均收益率: {:.6} ({:.4}%)", volatility.average_return, volatility.average_return * 100.0);
    println!("  ├─ 最大单期涨幅: {:.6} ({:.2}%)", volatility.max_daily_change, volatility.max_daily_change * 100.0);
    println!("  └─ 最大单期跌幅: {:.6} ({:.2}%)", volatility.min_daily_change, volatility.min_daily_change * 100.0);
    println!();
    
    // 价格信息
    if let (Some(first), Some(last)) = (candles.first(), candles.last()) {
        let start_price: f64 = first.open.parse().unwrap_or(0.0);
        let end_price: f64 = last.close.parse().unwrap_or(0.0);
        let total_return = if start_price > 0.0 { 
            (end_price - start_price) / start_price 
        } else { 
            0.0 
        };
        
        println!("价格变化:");
        println!("  ├─ 起始价格: ${:.2}", start_price);
        println!("  ├─ 结束价格: ${:.2}", end_price);
        println!("  └─ 总收益率: {:.4} ({:.2}%)", total_return, total_return * 100.0);
        println!();
    }
    
    // 风险评估
    println!("风险评估:");
    let risk_level = assess_risk_level(volatility.annual_volatility);
    println!("  └─ 风险等级: {}", risk_level);
    
    // 交易建议
    println!();
    println!("交易建议:");
    provide_trading_suggestions(volatility);
}

/// 评估风险等级
fn assess_risk_level(annual_volatility: f64) -> &'static str {
    let annual_vol_percent = annual_volatility * 100.0;
    
    if annual_vol_percent < 20.0 {
        "低风险 (年化波动率 < 20%)"
    } else if annual_vol_percent < 50.0 {
        "中风险 (年化波动率 20-50%)"
    } else if annual_vol_percent < 100.0 {
        "高风险 (年化波动率 50-100%)"
    } else {
        "极高风险 (年化波动率 > 100%)"
    }
}

/// 提供交易建议
fn provide_trading_suggestions(volatility: &VolatilityResult) {
    let annual_vol_percent = volatility.annual_volatility * 100.0;
    
    if annual_vol_percent < 30.0 {
        println!("  ├─ 适合长期持有和大仓位交易");
        println!("  ├─ 可以使用较高杠杆");
        println!("  └─ 波动率较低，适合稳健投资");
    } else if annual_vol_percent < 70.0 {
        println!("  ├─ 适合中短期交易");
        println!("  ├─ 建议适度杠杆");
        println!("  └─ 注意风险控制和止损");
    } else {
        println!("  ├─ 建议小仓位交易");
        println!("  ├─ 避免使用高杠杆");
        println!("  ├─ 严格止损管理");
        println!("  └─ 高风险高收益，谨慎操作");
    }
} 