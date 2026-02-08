use crate::strategies::copy_trading_strategy::CopyTradingStrategy;
use crate::types::{TradeSignal, CopyTradingConfig, OrderType};

/// 保守策略 - 只跟小额交易，使用挂单模式
#[derive(Debug)]
pub struct ConservativeCopyStrategy;

// 为ConservativeCopyStrategy实现Send + Sync
unsafe impl Send for ConservativeCopyStrategy {}
unsafe impl Sync for ConservativeCopyStrategy {}

impl CopyTradingStrategy for ConservativeCopyStrategy {
    fn name(&self) -> &str {
        "conservative"
    }

    fn should_copy_trade(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> bool {
        // 检查资产是否在允许列表中
        if !config.enabled_assets.is_empty() && !config.enabled_assets.contains(&signal.coin) {
            return false;
        }
        
        // 保守策略：只跟小额交易
        let trade_value = signal.amount * signal.price;
        if trade_value > config.max_position_value {
            return false;
        }
        
        true
    }

    fn calculate_copy_amount(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> f64 {
        // 保守策略：使用较小的跟单比例
        signal.amount * config.copy_ratio * 0.5
    }

    fn calculate_copy_price(&self, signal: &TradeSignal, _config: &CopyTradingConfig) -> f64 {
        // 保守策略：使用相同价格，避免滑点
        signal.price
    }

    fn get_order_type(&self, _signal: &TradeSignal, config: &CopyTradingConfig) -> OrderType {
        // 根据配置文件中的order_type设置返回对应的订单类型
        match config.order_type.as_str() {
            "Market" => OrderType::Market,
            "Limit" => OrderType::Limit { price_slippage: 0.001 }, // 0.1%滑点
            _ => {
                // 保守策略默认使用限价单，避免滑点
                OrderType::Limit { price_slippage: 0.001 }
            }
        }
    }
} 