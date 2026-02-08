pub use ethers::types::H160;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 监控事件 - 从WebSocket接收到的交易事件
#[derive(Debug, Clone)]
pub struct MonitorEvent {
    pub source_address: H160,      // 源地址
    pub timestamp: String,          // 时间
    pub coin: String,              // 币种
    pub action: String,            // 操作类型
    pub direction: String,          // 买卖方向
    pub closed_pnl: f64,           // 已实现盈亏
    pub size: f64,                 // 数量
    pub price: f64,                // 价格
    pub value: f64,                // 价值
    pub trade_type: String,        // 交易类型
    pub order_id: String,          // 订单ID
}

/// 最大事件缓冲区大小
pub const MAX_EVENT_BUFFER: usize = 10000;

    //user_profit表数据汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPnlSummary {
    pub addr: String,
    pub positions: Vec<PositionPnl>,
    pub history_trades: Vec<HistoryTradePnL>,
    pub total_pnl: f64,
    pub total_realized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub last_updated: DateTime<Utc>,
}

/// 单个仓位PnL数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPnl {
    pub coin: String,
    pub action: String,
    pub position_size: f64,
    pub position_value: f64,
    pub entry_price: f64,

    pub liquidation_price: Option<f64>,
    pub unrealized_pnl: f64,
    pub roe: f64,
}

/// 历史交易PnL数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryTradePnL {
    pub coin: String,
    pub realized_pnl: f64,
}

// ========== 跟单相关类型定义 ==========

/// 订单类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit { price_slippage: f64 },
}

/// 交易信号
#[derive(Debug, Clone)]
pub struct TradeSignal {
    pub wallet_address: H160,
    pub coin: String,
    pub action: TradeAction,
    pub amount: f64,
    pub price: f64,
    pub timestamp: i64,
    pub original_direction: Option<String>,  // 原始交易方向，用于区分开仓和平仓
}

/// 交易动作
#[derive(Debug, Clone)]
pub enum TradeAction {
    Buy,
    Sell,
}

/// 统一的交易方向信息
#[derive(Debug, Clone)]
pub struct TradeDirectionInfo {
    pub is_buy: bool,           // 是否为买入操作
    pub reduce_only: bool,       // 是否为平仓操作
    pub action: TradeAction,     // 交易动作枚举
    pub description: &'static str, // 操作描述（用于日志）
    pub text: &'static str,      // 简短文本描述
}

/// 跟单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradingConfig {
    pub strategy_type: String,
    pub private_key: String,
    pub target_user: H160,
    pub wallet: String,               // 钱包地址
    pub copy_ratio: f64,
    pub max_position_value: f64,
    pub enabled_assets: Vec<String>,
    pub take_profit_percentage: f64,
    pub stop_loss_percentage: f64,
    pub order_type: String,
    pub margin_type: String,      // 仓位类型：isolated(逐仓), cross(全仓)
    pub leverage: u32,            // 杠杆倍数-100
}

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub database_url: String,           // 数据库连接URL
    pub network: String,                // 网络配置：mainnet, testnet
    pub copy_trading: CopyTradingConfig, // 跟单策略配置
}



// ========== 止盈止损==========

/// 持仓信息（用于止盈止损）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub coin: String,
    pub action: String,        // "多头 (Long)" 或 "空头 (Short)"
    pub position_size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub take_profit_price: f64,
    pub stop_loss_price: f64,
    pub is_long: bool,         // true=多头, false=空头
}

/// 止盈止损状态
#[derive(Debug, Clone)]
pub struct TakeProfitStopLossState {
    pub closed_positions: std::collections::HashSet<String>,  // 已平仓的币种
    pub position_info: std::collections::HashMap<String, PositionInfo>,  // 持仓信息
    pub last_check_time: chrono::DateTime<chrono::Utc>,
}

impl TakeProfitStopLossState {
    pub fn new() -> Self {
        Self {
            closed_positions: std::collections::HashSet::new(),
            position_info: std::collections::HashMap::new(),
            last_check_time: chrono::Utc::now(),
        }
    }

    /// 记录开仓
    pub fn record_open_position(&mut self, coin: &str, position: PositionInfo) {
        // 先获取需要的字段，避免移动引用
        let action = position.action.clone();
        let entry_price = position.entry_price;
        
        self.closed_positions.remove(coin);  // 清除已平仓标记
        self.position_info.insert(coin.to_string(), position);
        log::info!("记录开仓: {} {} @ ${:.4}", coin, action, entry_price);
    }

    /// 记录平仓
    pub fn record_close_position(&mut self, coin: &str) {
        self.closed_positions.insert(coin.to_string());
        self.position_info.remove(coin);
        log::info!("记录平仓: {}", coin);
    }

    /// 检查是否已平仓
    pub fn is_position_closed(&self, coin: &str) -> bool {
        self.closed_positions.contains(coin)
    }

    /// 获取持仓信息
    pub fn get_position_info(&self, coin: &str) -> Option<&PositionInfo> {
        self.position_info.get(coin)
    }

    /// 更新当前价格
    pub fn update_current_price(&mut self, coin: &str, current_price: f64) {
        if let Some(position) = self.position_info.get_mut(coin) {
            position.current_price = current_price;
        }
    }
}

// ========== 价格收集 ==========

use std::time::Instant;

/// 价格数据结构
#[derive(Debug, Clone)]
pub struct PriceData {
    pub coin: String,                    // 币种名称（如 ETH、BTC）
    pub best_bid: Option<f64>,          // 最优买价
    pub best_ask: Option<f64>,          // 最优卖价
    pub index_price: Option<f64>,       // 指数价
    pub last_update: Instant,            // 最后更新时间
}
