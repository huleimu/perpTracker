use crate::{
    info::{AssetPosition, Level, MarginSummary},
    DailyUserVlm, Delta, FeeSchedule, OrderInfo, Referrer, ReferrerState, UserTokenBalance,
};
use serde::Deserialize;

/// 用户状态响应，包含资产持仓、保证金信息等
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserStateResponse {
    /// 用户所有资产持仓列表
    pub asset_positions: Vec<AssetPosition>,
    /// 全仓保证金汇总信息
    pub cross_margin_summary: MarginSummary,
    /// 当前保证金汇总信息
    pub margin_summary: MarginSummary,
    /// 可提取余额
    pub withdrawable: String,
}

/// 用户代币余额响应
#[derive(Deserialize, Debug)]
pub struct UserTokenBalanceResponse {
    /// 代币余额列表
    pub balances: Vec<UserTokenBalance>,
}

/// 用户手续费相关信息响应
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFeesResponse {
    /// 当前生效的推荐人折扣
    pub active_referral_discount: String,
    /// 用户每日交易量明细
    pub daily_user_vlm: Vec<DailyUserVlm>,
    /// 手续费分层结构
    pub fee_schedule: FeeSchedule,
    /// 用户主动成交手续费率
    pub user_add_rate: String,
    /// 用户被动成交手续费率
    pub user_cross_rate: String,
}

/// 用户当前挂单信息
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrdersResponse {
    /// 币种
    pub coin: String,
    /// 限价
    pub limit_px: String,
    /// 订单ID
    pub oid: u64,
    /// 买卖方向
    pub side: String,
    /// 数量
    pub sz: String,
    /// 下单时间戳
    pub timestamp: u64,
    /// 客户端订单ID（可选）
    pub cloid: Option<String>,
}

/// 用户成交明细响应
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFillsResponse {
    /// 本次成交带来的已实现盈亏
    pub closed_pnl: String,
    /// 币种
    pub coin: String,
    /// 是否为全仓模式
    pub crossed: bool,
    /// 方向（long/short）
    pub dir: String,
    /// 成交哈希
    pub hash: String,
    /// 订单ID
    pub oid: u64,
    /// 成交价格
    pub px: String,
    /// 买卖方向
    pub side: String,
    /// 成交前的持仓数量
    pub start_position: String,
    /// 本次成交的数量
    pub sz: String,
    /// 成交时间戳
    pub time: u64,
    /// 手续费
    pub fee: String,
}

/// 资金费率历史响应
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FundingHistoryResponse {
    /// 币种
    pub coin: String,
    /// 资金费率
    pub funding_rate: String,
    /// 溢价
    pub premium: String,
    /// 时间戳
    pub time: u64,
}

/// 用户资金变化响应
#[derive(Deserialize, Debug)]
pub struct UserFundingResponse {
    /// 时间戳
    pub time: u64,
    /// 资金变化哈希
    pub hash: String,
    /// 资金变化明细
    pub delta: Delta,
}

/// L2盘口快照响应
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2SnapshotResponse {
    /// 币种
    pub coin: String,
    /// 盘口深度档位（二维数组，买卖盘）
    pub levels: Vec<Vec<Level>>,
    /// 快照时间戳
    pub time: u64,
}

/// 最新成交明细响应
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecentTradesResponse {
    /// 币种
    pub coin: String,
    /// 买卖方向
    pub side: String,
    /// 成交价格
    pub px: String,
    /// 成交数量
    pub sz: String,
    /// 成交时间戳
    pub time: u64,
    /// 成交哈希
    pub hash: String,
}

/// K线快照响应
#[derive(serde::Deserialize, Debug)]
pub struct CandlesSnapshotResponse {
    /// K线开始时间
    #[serde(rename = "t")]
    pub time_open: u64,
    /// K线结束时间
    #[serde(rename = "T")]
    pub time_close: u64,
    /// 币种
    #[serde(rename = "s")]
    pub coin: String,
    /// K线周期
    #[serde(rename = "i")]
    pub candle_interval: String,
    /// 开盘价
    #[serde(rename = "o")]
    pub open: String,
    /// 收盘价
    #[serde(rename = "c")]
    pub close: String,
    /// 最高价
    #[serde(rename = "h")]
    pub high: String,
    /// 最低价
    #[serde(rename = "l")]
    pub low: String,
    /// 成交量
    #[serde(rename = "v")]
    pub vlm: String,
    /// 成交笔数
    #[serde(rename = "n")]
    pub num_trades: u64,
}

/// 订单状态响应
#[derive(Deserialize, Debug)]
pub struct OrderStatusResponse {
    /// 订单状态
    pub status: String,
    /// 订单信息（可选，未找到时为 None）
    #[serde(default)]
    pub order: Option<OrderInfo>,
}

/// 推荐人相关信息响应
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferralResponse {
    /// 推荐人信息（可选）
    pub referred_by: Option<Referrer>,
    /// 累计交易量
    pub cum_vlm: String,
    /// 未领取奖励
    pub unclaimed_rewards: String,
    /// 已领取奖励
    pub claimed_rewards: String,
    /// 推荐人状态
    pub referrer_state: ReferrerState,
} 