use anyhow::Result;
use std::fs;

use crate::types::{H160, CopyTradingConfig, ConfigFile};

/// 从配置文件加载跟单配置
pub fn load_config_from_file(file_path: &str) -> Result<CopyTradingConfig> {
    let content = fs::read_to_string(file_path)?;
    let config_file: ConfigFile = toml::from_str(&content)?;
    
    // 简单的配置验证（警告但不阻止）
    if !matches!(config_file.copy_trading.strategy_type.as_str(), "conservative" | "aggressive") {
        crate::log!(warn, "copy_trading_utils", "load_config_from_file", 
            "策略类型无效: {}，使用默认值", 
            "strategy_type" => config_file.copy_trading.strategy_type
        );
    }
    
    if !matches!(config_file.copy_trading.margin_type.as_str(), "isolated" | "cross") {
        crate::log!(warn, "copy_trading_utils", "load_config_from_file", 
            "仓位类型无效: {}，使用默认值", 
            "margin_type" => config_file.copy_trading.margin_type
        );
    }
    
    if config_file.copy_trading.leverage < 1 || config_file.copy_trading.leverage > 100 {
        crate::log!(warn, "copy_trading_utils", "load_config_from_file", 
            "杠杆倍数无效: {}，范围应为1-100", 
            "leverage" => config_file.copy_trading.leverage
        );
    }
    
    Ok(config_file.copy_trading)
}

/// 获取私钥
pub fn get_private_key_from_config(file_path: &str) -> Result<String> {
    let content = fs::read_to_string(file_path)?;
    let config_file: ConfigFile = toml::from_str(&content)?;
    Ok(config_file.copy_trading.private_key)
}

/// 创建默认配置
pub fn create_default_config() -> CopyTradingConfig {
    CopyTradingConfig {
        strategy_type: "conservative".to_string(),
        private_key: "".to_string(),
        target_user: H160::from([0x5e, 0x32, 0xd5, 0x15, 0x77, 0x96, 0xd9, 0x60, 0xed, 0xb6, 0x11, 0xd5, 0x23, 0xb9, 0x05, 0x1a, 0xc9, 0x88, 0x83, 0x52]),
        wallet: "0x6a3204b43ea7D3e79E54168bb0C70CD668Ecd1d3".to_string(),
        copy_ratio: 0.1,
        max_position_value: 1000.0,
        enabled_assets: vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()],
        take_profit_percentage: 0.05,
        stop_loss_percentage: 0.03,
        order_type: "Gtc".to_string(),
        margin_type: "isolated".to_string(),
        leverage: 5,
    }
}

/// 杠杆设置
/// 设置用户配置的杠杆，失败则使用平台默认配置
pub async fn set_leverage_with_fallback(
    exchange_client: &hyperliquid_rust_sdk::ExchangeClient,
    coin: &str,
    user_leverage: u32,
    is_cross: bool
) -> Result<()> {
    // 先尝试用户设置的杠杆
    match exchange_client.update_leverage(user_leverage, coin, is_cross, None).await {
        Ok(response) => {
            // 检查响应是否真的成功
            match response {
                hyperliquid_rust_sdk::ExchangeResponseStatus::Ok(_) => {
                    crate::log!(debug, "copy_trading_utils", "set_leverage_with_fallback", 
                        "杠杆设置成功", 
                        "leverage" => user_leverage,
                        "coin" => coin,
                        "margin_type" => if is_cross { "cross" } else { "isolated" }
                    );
                    Ok(())
                }
                _ => {
                    crate::log!(warn, "copy_trading_utils", "set_leverage_with_fallback", 
                        "杠杆设置失败，将使用平台默认配置", 
                        "user_leverage" => user_leverage,
                        "coin" => coin,
                        "response" => format!("{:?}", response)
                    );
                    Ok(())
                }
            }
        }
        Err(e) => {
            crate::log!(warn, "copy_trading_utils", "set_leverage_with_fallback", 
                "用户杠杆设置失败，将使用平台默认配置", 
                "user_leverage" => user_leverage,
                "coin" => coin,
                "error" => format!("{}", e)
            );
            Ok(())
        }
    }
} 
