//! # Hyperliquid 用户累积资金费率查询示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 获取用户的**累积资金费率**信息。
//!
//! 累积资金费率包含三个重要维度：
//! 1. **历史总计 (all_time)**: 该仓位从创建以来的累积资金费率总和
//! 2. **开仓至今 (since_open)**: 当前仓位开仓以来的累积资金费率
//! 3. **最近变化 (since_change)**: 上次仓位大小变更以来的累积资金费率
//!
//! 这些数据对于计算持仓成本、风险管理和盈亏分析至关重要。
//!
//! ## 关于输出的说明
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin cumulative_funding_example
//! ```

use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use ethers::types::H160;
use std::str::FromStr;
use log::info;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始查询用户累积资金费率信息...");
    println!("=======================================================");
    // =========================================================

    // ---- Part 1: 初始化客户端 ----
    
    println!("[步骤 1/3] 正在初始化 InfoClient (连接到测试网)...");
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    
    // 指定要查询的用户地址
    let user_address = H160::from_str("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8").unwrap();
    println!("    - 查询用户地址: {:?}", user_address);

    // ---- Part 2: 获取用户状态 ----
    
    println!("\n[步骤 2/3] 正在查询用户状态和累积资金费率...");
    
    match info_client.user_state(user_address).await {
        Ok(user_state) => {
            println!("    - 查询成功！");
            
            // ---- Part 3: 分析和展示累积资金费率 ----
            
            println!("\n[步骤 3/3] 累积资金费率详细信息:");
            println!("=======================================================");
            
            if user_state.asset_positions.is_empty() {
                println!("该用户当前没有任何持仓。");
            } else {
                println!("发现 {} 个资产仓位:\n", user_state.asset_positions.len());
                
                for (index, asset_position) in user_state.asset_positions.iter().enumerate() {
                    let position = &asset_position.position;
                    let cum_funding = &position.cum_funding;
                    
                    println!("【仓位 {} - {}】", index + 1, position.coin);
                    println!("  ┌─ 基本信息:");
                    println!("  │  ├─ 资产代码: {}", position.coin);
                    println!("  │  ├─ 仓位大小: {}", position.szi);
                    println!("  │  ├─ 杠杆倍数: {}x", position.leverage.value);
                    if let Some(entry_px) = &position.entry_px {
                        println!("  │  └─ 开仓价格: {}", entry_px);
                    } else {
                        println!("  │  └─ 开仓价格: 无");
                    }
                    
                    println!("  └─ 累积资金费率:");
                    println!("     ├─ 历史总计: {} USDC", cum_funding.all_time);
                    println!("     │   (该仓位从创建以来的累积资金费率总和)");
                    println!("     ├─ 开仓至今: {} USDC", cum_funding.since_open);
                    println!("     │   (当前仓位开仓以来的累积资金费率)");
                    println!("     └─ 最近变化: {} USDC", cum_funding.since_change);
                    println!("         (上次仓位变更以来的累积资金费率)");
                    
                    // 解析并分析资金费率数据
                    analyze_funding_impact(cum_funding);
                    println!();
                }
                
                // 计算总体资金费率影响
                calculate_total_funding_impact(&user_state.asset_positions);
            }
            
            println!("=======================================================");
        }
        Err(e) => {
            println!("查询失败: {:?}", e);
            info!("Failed to get user state: {:?}", e);
        }
    }
    
    println!("查询完成！");
}

/// 分析单个仓位的资金费率影响
fn analyze_funding_impact(cum_funding: &hyperliquid_rust_sdk::CumulativeFunding) {
    // 尝试解析数值进行分析
    if let (Ok(all_time), Ok(since_open), Ok(since_change)) = (
        cum_funding.all_time.parse::<f64>(),
        cum_funding.since_open.parse::<f64>(),
        cum_funding.since_change.parse::<f64>()
    ) {
                        println!("    数据分析:");
        
        // 判断资金费率是正是负
        let status_all = if all_time >= 0.0 { "收取" } else { "支付" };
        let status_open = if since_open >= 0.0 { "收取" } else { "支付" };
        let status_change = if since_change >= 0.0 { "收取" } else { "支付" };
        
        println!("        ├─ 历史总计: {} {:.4} USDC", status_all, all_time.abs());
        println!("        ├─ 开仓至今: {} {:.4} USDC", status_open, since_open.abs());
        println!("        └─ 最近变化: {} {:.4} USDC", status_change, since_change.abs());
        
        // 判断趋势
        if since_change.abs() > since_open.abs() * 0.1 {
            println!("        注意: 最近的资金费率变化较大");
        }
    }
}

/// 计算所有仓位的总体资金费率影响
fn calculate_total_funding_impact(positions: &[hyperliquid_rust_sdk::AssetPosition]) {
    println!("总体资金费率汇总:");
    
    let mut total_all_time = 0.0;
    let mut total_since_open = 0.0;
    let mut total_since_change = 0.0;
    let mut parsed_count = 0;
    
    for position in positions {
        let cum_funding = &position.position.cum_funding;
        
        if let (Ok(all_time), Ok(since_open), Ok(since_change)) = (
            cum_funding.all_time.parse::<f64>(),
            cum_funding.since_open.parse::<f64>(),
            cum_funding.since_change.parse::<f64>()
        ) {
            total_all_time += all_time;
            total_since_open += since_open;
            total_since_change += since_change;
            parsed_count += 1;
        }
    }
    
    if parsed_count > 0 {
        println!("  ├─ 总历史累积: {:.4} USDC", total_all_time);
        println!("  ├─ 总开仓至今: {:.4} USDC", total_since_open);
        println!("  └─ 总最近变化: {:.4} USDC", total_since_change);
        
        let net_status = if total_all_time >= 0.0 {
            "您是资金费率的净收益者"
        } else {
            "您是资金费率的净支付者"
        };
        println!("  总体评估: {}", net_status);
    } else {
        println!("  无法解析资金费率数据");
    }
} 