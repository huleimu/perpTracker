//! # 波动率计算详细步骤演示
//!
//! 该文件详细展示了波动率计算的每一个步骤，包括数学公式和具体实现。
//!
//! ## 计算步骤：
//! 1. 获取价格数据
//! 2. 计算每期收益率
//! 3. 计算平均收益率
//! 4. 计算方差
//! 5. 计算标准差（波动率）
//! 6. 年化处理
//!
//! ### 如何运行：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin volatility_detailed_calculation
//! ```

use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("=======================================================");
    println!("波动率计算详细步骤演示");
    println!("=======================================================");
    
    // ---- 获取实际市场数据 ----
    
    println!("\n[步骤 1] 获取价格数据");
    println!("-------------------------------------------------");
    
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    let coin = "ETH";
    let interval = "1h";
    let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let start_timestamp = end_timestamp - (7 * 24 * 60 * 60 * 1000); // 7天数据
    
    match info_client
        .candles_snapshot(coin.to_string(), interval.to_string(), start_timestamp, end_timestamp)
        .await 
    {
        Ok(candles) => {
            println!("成功获取 {} 根K线数据", candles.len());
            
            if candles.len() < 10 {
                println!("数据不足，使用模拟数据进行演示");
                demonstrate_with_sample_data();
                return;
            }
            
            // 取前10根K线进行详细演示
            let sample_candles: Vec<_> = candles.into_iter().take(10).collect();
            demonstrate_detailed_calculation(&sample_candles);
            
        }
        Err(_) => {
            println!("获取实际数据失败，使用模拟数据进行演示");
            demonstrate_with_sample_data();
        }
    }
}

/// 使用实际市场数据进行详细计算演示
fn demonstrate_detailed_calculation(candles: &[hyperliquid_rust_sdk::CandlesSnapshotResponse]) {
    println!("\n使用实际 {} 数据进行计算", candles.first().unwrap().coin);
    
    // 提取收盘价
    let prices: Vec<f64> = candles.iter()
        .map(|c| c.close.parse().unwrap_or(0.0))
        .collect();
    
    println!("\n[步骤 2] 提取收盘价格");
    println!("-------------------------------------------------");
    for (i, price) in prices.iter().enumerate() {
        let time = chrono::DateTime::from_timestamp_millis(candles[i].time_close as i64)
            .unwrap_or_default();
        println!("时间: {} | 收盘价: ${:.2}", time.format("%m-%d %H:%M"), price);
    }
    
    detailed_volatility_calculation(&prices, "1小时");
}

/// 使用模拟数据进行演示
fn demonstrate_with_sample_data() {
    println!("\n使用模拟数据进行计算演示");
    
    // 模拟10天的股价数据
    let prices = vec![
        100.0, 102.0, 98.5, 101.2, 99.8, 
        103.5, 97.2, 105.1, 102.8, 100.5
    ];
    
    println!("\n[步骤 2] 模拟价格数据");
    println!("-------------------------------------------------");
    for (i, price) in prices.iter().enumerate() {
        println!("第{}天: ${:.2}", i + 1, price);
    }
    
    detailed_volatility_calculation(&prices, "日");
}

/// 详细展示波动率计算的每一步
fn detailed_volatility_calculation(prices: &[f64], period: &str) {
    if prices.len() < 2 {
        println!("价格数据不足");
        return;
    }
    
    println!("\n[步骤 3] 计算收益率");
    println!("-------------------------------------------------");
    println!("公式: 收益率 = (当前价格 - 前一价格) / 前一价格");
    println!();
    
    let mut returns = Vec::new();
    
    for i in 1..prices.len() {
        let prev_price = prices[i-1];
        let current_price = prices[i];
        let return_rate = (current_price - prev_price) / prev_price;
        
        returns.push(return_rate);
        
        println!("第{}期: ({:.2} - {:.2}) / {:.2} = {:.6} ({:.2}%)", 
                 i, current_price, prev_price, prev_price, return_rate, return_rate * 100.0);
    }
    
    println!("\n[步骤 4] 计算平均收益率");
    println!("-------------------------------------------------");
    println!("公式: 平均收益率 = Σ(收益率) / n");
    println!();
    
    let sum_returns: f64 = returns.iter().sum();
    let mean_return = sum_returns / returns.len() as f64;
    
    println!("收益率总和: {:.6}", sum_returns);
    println!("数据点数量: {}", returns.len());
    println!("平均收益率: {:.6} ({:.4}%)", mean_return, mean_return * 100.0);
    
    println!("\n[步骤 5] 计算方差");
    println!("-------------------------------------------------");
    println!("公式: 方差 = Σ[(收益率 - 平均收益率)²] / n");
    println!();
    
    let mut variance_sum = 0.0;
    
    for (i, &return_rate) in returns.iter().enumerate() {
        let deviation = return_rate - mean_return;
        let squared_deviation = deviation.powi(2);
        variance_sum += squared_deviation;
        
        println!("第{}期: ({:.6} - {:.6})² = {:.8}", 
                 i + 1, return_rate, mean_return, squared_deviation);
    }
    
    let variance = variance_sum / returns.len() as f64;
    
    println!("偏差平方和: {:.8}", variance_sum);
    println!("方差: {:.8}", variance);
    
    println!("\n[步骤 6] 计算标准差（波动率）");
    println!("-------------------------------------------------");
    println!("公式: 标准差 = √方差");
    println!();
    
    let volatility = variance.sqrt();
    
    println!("{}波动率: {:.6} ({:.2}%)", period, volatility, volatility * 100.0);
    
    println!("\n[步骤 7] 年化处理");
    println!("-------------------------------------------------");
    
    let (time_factor, factor_description): (f64, &str) = match period {
        "分钟" => (365.25 * 24.0 * 60.0, "365.25 × 24 × 60 (一年的分钟数)"),
        "小时" | "1小时" => (365.25 * 24.0, "365.25 × 24 (一年的小时数)"),
        "日" => (365.25, "365.25 (一年的天数)"),
        "周" => (52.0, "52 (一年的周数)"),
        "月" => (12.0, "12 (一年的月数)"),
        _ => (252.0, "252 (一年的交易日数)"),
    };
    
    let annualized_volatility = volatility * time_factor.sqrt();
    
    println!("年化公式: {}波动率 × √时间调整因子", period);
    println!("时间调整因子: {:.2} ({})", time_factor, factor_description);
    println!("年化波动率: {:.6} × √{:.2} = {:.6} ({:.2}%)", 
             volatility, time_factor, annualized_volatility, annualized_volatility * 100.0);
    
    println!("\n[步骤 8] 结果解读");
    println!("-------------------------------------------------");
    
    interpret_volatility_results(volatility, annualized_volatility, period);
    
    println!("\n[步骤 9] 风险指标计算");
    println!("-------------------------------------------------");
    
    calculate_risk_metrics(&returns, mean_return, volatility);
}

/// 解读波动率结果
fn interpret_volatility_results(period_volatility: f64, annual_volatility: f64, period: &str) {
    println!("波动率解读:");
    println!();
    
    let period_percent = period_volatility * 100.0;
    let annual_percent = annual_volatility * 100.0;
    
    println!("• {}波动率 {:.2}% 意味着:", period, period_percent);
    println!("  - 68%的情况下，{}收益率在 ±{:.2}% 范围内", period, period_percent);
    println!("  - 95%的情况下，{}收益率在 ±{:.2}% 范围内", period, period_percent * 2.0);
    println!();
    
    println!("• 年化波动率 {:.2}% 意味着:", annual_percent);
    println!("  - 68%的情况下，年收益率在 ±{:.2}% 范围内", annual_percent);
    println!("  - 95%的情况下，年收益率在 ±{:.2}% 范围内", annual_percent * 2.0);
    println!();
    
    // 风险等级评估
    let risk_level = if annual_percent < 15.0 {
        "低风险"
    } else if annual_percent < 25.0 {
        "中低风险"
    } else if annual_percent < 40.0 {
        "中高风险"
    } else if annual_percent < 60.0 {
        "高风险"
    } else {
        "极高风险"
    };
    
    println!("• 风险等级: {}", risk_level);
}

/// 计算其他风险指标
fn calculate_risk_metrics(returns: &[f64], mean_return: f64, volatility: f64) {
    println!("其他风险指标:");
    println!();
    
    // 夏普比率 (假设无风险利率为0)
    let sharpe_ratio = if volatility > 0.0 {
        mean_return / volatility
    } else {
        0.0
    };
    
    println!("• 夏普比率: {:.4}", sharpe_ratio);
    println!("  (衡量每单位风险的超额收益，越高越好)");
    println!();
    
    // 最大回撤计算
    let mut cumulative_returns = vec![1.0];
    for &ret in returns {
        let new_value = cumulative_returns.last().unwrap() * (1.0 + ret);
        cumulative_returns.push(new_value);
    }
    
    let mut max_drawdown = 0.0;
    let mut peak = cumulative_returns[0];
    
    for &value in &cumulative_returns {
        if value > peak {
            peak = value;
        }
        let drawdown = (peak - value) / peak;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }
    
    println!("• 最大回撤: {:.4} ({:.2}%)", max_drawdown, max_drawdown * 100.0);
    println!("  (从峰值到谷底的最大跌幅)");
    println!();
    
    // VaR (风险价值) - 95%置信度
    let mut sorted_returns = returns.to_vec();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let var_95_index = (returns.len() as f64 * 0.05) as usize;
    let var_95 = if var_95_index < sorted_returns.len() {
        sorted_returns[var_95_index]
    } else {
        sorted_returns[0]
    };
    
    println!("• VaR (95%置信度): {:.4} ({:.2}%)", var_95, var_95 * 100.0);
    println!("  (95%的情况下，损失不会超过这个值)");
} 