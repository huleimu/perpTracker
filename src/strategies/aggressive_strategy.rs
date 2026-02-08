use crate::strategies::copy_trading_strategy::CopyTradingStrategy;
use crate::types::{TradeSignal, CopyTradingConfig, OrderType};

/// 激进策略 - 所有交易都跟，放大跟单金额
#[derive(Debug)]
pub struct AggressiveCopyStrategy;

// 为AggressiveCopyStrategy实现Send + Sync
unsafe impl Send for AggressiveCopyStrategy {}
unsafe impl Sync for AggressiveCopyStrategy {}

impl CopyTradingStrategy for AggressiveCopyStrategy {
    fn name(&self) -> &str {
        "aggressive"
    }

    fn should_copy_trade(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> bool {
        // 检查资产是否在允许列表中
        if !config.enabled_assets.is_empty() && !config.enabled_assets.contains(&signal.coin) {
            return false;
        }
        
        // 激进策略：所有交易都跟，但检查最大限制
        let trade_value = signal.amount * signal.price;
        if trade_value > config.max_position_value * 2.0 {
            return false;
        }
        
        true
    }

    fn calculate_copy_amount(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> f64 {
        // 激进策略：放大跟单金额
        signal.amount * config.copy_ratio * 2.0
    }

    fn calculate_copy_price(&self, signal: &TradeSignal, _config: &CopyTradingConfig) -> f64 {
        // 激进策略：使用市价，快速成交
        signal.price
    }

    fn get_order_type(&self, _signal: &TradeSignal, config: &CopyTradingConfig) -> OrderType {
        // 根据配置文件中的order_type设置返回对应的订单类型
        match config.order_type.as_str() {
            "Market" => OrderType::Market,
            "Limit" => OrderType::Limit { price_slippage: 0.002 }, // 0.2%滑点
            _ => {
                // 激进策略默认使用市价单，快速执行
                OrderType::Market
            }
        }
    }
} 