pub mod copy_trading_strategy;
pub mod conservative_strategy;
pub mod aggressive_strategy;
pub mod factory;
pub mod take_profit_stop_loss;
 
pub use conservative_strategy::ConservativeCopyStrategy;
pub use aggressive_strategy::AggressiveCopyStrategy;
pub use copy_trading_strategy::{CopyTradingStrategy, CopyTradingService};
pub use factory::StrategyFactory; 