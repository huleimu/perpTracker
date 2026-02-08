use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow;
use rust_decimal::Decimal;


// TODO:目前用了f64，后续可以考虑使用Decimal
/// 用户当前盈亏数据
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserProfit {
    pub id: Option<i64>,
    pub addr: String,

    //持仓状态
    pub coin: String,
    pub action: String,
    pub position_size: f64,
    pub position_value: f64,
    pub entry_price: f64,

    pub liquidation_price: Option<f64>,


    pub total_pnl: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub roe: f64,
    pub start_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

/// 用户历史盈亏数据
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserHistoryProfit {
    pub addr: String,
    pub coin: String,
    #[sqlx(rename = "pnl_12h")]
    pub pnl_12h: Decimal,
    #[sqlx(rename = "pnl_24h")]
    pub pnl_24h: Decimal,
    #[sqlx(rename = "pnl_3d")]
    pub pnl_3d: Decimal,
    #[sqlx(rename = "pnl_7d")]
    pub pnl_7d: Decimal,
    #[sqlx(rename = "pnl_30d")]
    pub pnl_30d: Decimal,
    pub record_time: DateTime<Utc>,
}

impl UserProfit {
    pub fn new(addr: String, coin: String) -> Self {
        Self {
            id: None,
            addr,
            coin,
            action: "".to_string(),
            position_size: 0.0,
            position_value: 0.0,
            entry_price: 0.0,

            liquidation_price: None,
            total_pnl: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            roe: 0.0,
            start_time: Utc::now(),
            update_time: None,
        }
    }
}

impl UserHistoryProfit {
    pub fn new(addr: String, coin: String) -> Self {
        Self {

            addr,
            coin,
            pnl_12h: Decimal::new(0, 0),
            pnl_24h: Decimal::new(0, 0),
            pnl_3d: Decimal::new(0, 0),
            pnl_7d: Decimal::new(0, 0),
            pnl_30d: Decimal::new(0, 0),
            record_time: Utc::now(),
        }
    }
}

/// 交易事件记录 - 存储到数据库的交易事件
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TradeEvent {
    pub id: Option<i64>,
    pub addr: String,
    pub coin: String,
    pub action: String,          // 开多仓、平多仓
    pub direction: String,       // 买入、卖出
    pub size: f64,              // 交易数量
    pub price: f64,             // 交易价格
    pub value: f64,             // 交易价值
    pub closed_pnl: f64,        // 平仓盈亏
    pub trade_type: String,     // 吃单、挂单
    pub order_id: String,       // 订单ID
    pub trade_time: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
}

impl TradeEvent {
    /// 从 MonitorEvent 创建 TradeEvent  
    pub fn from_monitor_event(event: &crate::types::MonitorEvent) -> anyhow::Result<Self> {
        use crate::utils::parse_timestamp_string;
        
        let trade_time = parse_timestamp_string(&event.timestamp);
        
        Ok(Self {
            id: None,
            addr: format!("{:#x}", event.source_address),
            coin: event.coin.clone(),
            action: event.action.clone(),
            direction: event.direction.clone(),
            size: event.size,
            price: event.price,
            value: event.value,
            closed_pnl: event.closed_pnl,
            trade_type: event.trade_type.clone(),
            order_id: event.order_id.clone(),
            trade_time,
            created_at: None,
        })
    }
} 
