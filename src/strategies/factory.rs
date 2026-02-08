use crate::types::CopyTradingConfig;
use crate::strategies::{ConservativeCopyStrategy, AggressiveCopyStrategy, CopyTradingStrategy};
use tracing::{info, warn};

/// 策略工厂 - 根据配置创建不同的策略实例
pub struct StrategyFactory;

impl StrategyFactory {
    /// 根据策略类型创建对应的策略实例
    /// 
    /// # 参数
    /// * `strategy_type` - 策略类型字符串
    /// * `config` - 策略配置
    /// 
    /// # 返回
    /// * `Box<dyn CopyTradingStrategy>` - 策略实例
    /// 
    /// # 示例
    /// ```
    /// let strategy = StrategyFactory::create_strategy("conservative", &config);
    /// ```
    pub fn create_strategy(strategy_type: &str, _config: &CopyTradingConfig) -> Box<dyn CopyTradingStrategy> {
        match strategy_type {
            "conservative" => {
                info!("使用保守策略");
                Box::new(ConservativeCopyStrategy)
            }
            "aggressive" => {
                info!("使用激进策略");
                Box::new(AggressiveCopyStrategy)
            }
            _ => {
                warn!("未知策略类型: {}，使用默认保守策略", strategy_type);
                warn!("可用策略类型: conservative(保守), aggressive(激进)");
                Box::new(ConservativeCopyStrategy)
            }
        }
    }


} 