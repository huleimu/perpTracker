use ethers::types::H160;
use serde::{Deserialize, Serialize};

/// 杠杆信息结构体
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    /// 杠杆类型（如 cross/isolated）
    #[serde(rename = "type")]
    pub type_string: String,
    /// 杠杆倍数
    pub value: u32,
    /// 原始美元金额（可选）
    pub raw_usd: Option<String>,
}

/// 累计资金费率信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CumulativeFunding {
    /// 总累计资金费率
    pub all_time: String,
    /// 自开仓以来的累计资金费率
    pub since_open: String,
    /// 自上次变更以来的累计资金费率
    pub since_change: String,
}

/// 持仓详细信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    /// 币种
    pub coin: String,
    /// 开仓价格（可选）
    pub entry_px: Option<String>,
    /// 杠杆信息
    pub leverage: Leverage,
    /// 强平价格（可选）
    pub liquidation_px: Option<String>,
    /// 已用保证金
    pub margin_used: String,
    /// 持仓价值
    pub position_value: String,
    /// 收益率
    pub return_on_equity: String,
    /// 持仓数量
    pub szi: String,
    /// 未实现盈亏
    pub unrealized_pnl: String,
    /// 最大杠杆倍数
    pub max_leverage: u32,
    /// 累计资金费率
    pub cum_funding: CumulativeFunding,
}

/// 资产持仓结构体，包含持仓数据和类型
#[derive(Deserialize, Debug)]
pub struct AssetPosition {
    /// 持仓数据
    pub position: PositionData,
    /// 类型字符串
    #[serde(rename = "type")]
    pub type_string: String,
}

/// 账户保证金汇总信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {

   
    //ROI 的官方定义：ROI = 盈亏 / max(100, 起始账户价值 + 最大净存款)

    /// 账户总价值
    pub account_value: String,


    /// 总共使用的保证金
    pub total_margin_used: String,
    /// 总名义持仓
    pub total_ntl_pos: String,
    /// 总原始美元价值
    pub total_raw_usd: String,
}

/// 盘口深度档位信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    /// 档位编号
    pub n: u64,
    /// 价格
    pub px: String,
    /// 数量
    pub sz: String,
}

/// 资金变化信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Delta {
    /// 类型
    #[serde(rename = "type")]
    pub type_string: String,
    /// 币种
    pub coin: String,
    /// USDC 变化量
    pub usdc: String,
    /// 币种数量变化
    pub szi: String,
    /// 资金费率
    pub funding_rate: String,
}

/// 用户每日交易量信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DailyUserVlm {
    /// 日期
    pub date: String,
    /// 交易所
    pub exchange: String,
    /// 用户主动成交量
    pub user_add: String,
    /// 用户被动成交量
    pub user_cross: String,
}

/// 手续费结构
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeSchedule {
    /// 主动挂单手续费
    pub add: String,
    /// 被动吃单手续费
    pub cross: String,
    /// 推荐人折扣
    pub referral_discount: String,
    /// 分层结构
    pub tiers: Tiers,
}

/// 手续费分层信息
#[derive(Deserialize, Debug)]
pub struct Tiers {
    /// 做市商分层
    pub mm: Vec<Mm>,
    /// VIP 分层
    pub vip: Vec<Vip>,
}

/// 做市商分层信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mm {
    /// 主动挂单手续费
    pub add: String,
    /// 做市商分界线
    pub maker_fraction_cutoff: String,
}

/// VIP 分层信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vip {
    /// 主动挂单手续费
    pub add: String,
    /// 被动吃单手续费
    pub cross: String,
    /// 名义持仓分界线
    pub ntl_cutoff: String,
}

/// 用户代币余额信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserTokenBalance {
    /// 币种
    pub coin: String,
    /// 持有数量
    pub hold: String,
    /// 总数量
    pub total: String,
    /// 名义持仓
    pub entry_ntl: String,
}

/// 订单信息（含基础信息和状态）
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
    /// 基础订单信息
    pub order: BasicOrderInfo,
    /// 订单状态
    pub status: String,
    /// 状态时间戳
    pub status_timestamp: u64,
}

/// 基础订单信息
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BasicOrderInfo {
    /// 币种
    pub coin: String,
    /// 买卖方向
    pub side: String,
    /// 限价
    pub limit_px: String,
    /// 数量
    pub sz: String,
    /// 订单ID
    pub oid: u64,
    /// 下单时间戳
    pub timestamp: u64,
    /// 触发条件
    pub trigger_condition: String,
    /// 是否为触发单
    pub is_trigger: bool,
    /// 触发价格
    pub trigger_px: String,
    /// 是否为持仓止盈止损单
    pub is_position_tpsl: bool,
    /// 是否仅减仓
    pub reduce_only: bool,
    /// 订单类型
    pub order_type: String,
    /// 原始下单数量
    pub orig_sz: String,
    /// 订单有效期类型
    pub tif: String,
    /// 客户端订单ID（可选）
    pub cloid: Option<String>,
}

/// 推荐人信息
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Referrer {
    /// 推荐人地址
    pub referrer: H160,
    /// 推荐码
    pub code: String,
}

/// 推荐人状态
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerState {
    /// 阶段
    pub stage: String,
    /// 推荐人数据
    pub data: ReferrerData,
}

/// 推荐人数据
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerData {
    /// 所需条件
    pub required: String,
} 