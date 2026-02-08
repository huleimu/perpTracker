use anyhow::Result;
use log::{info, warn, error};
use crate::types::{OrderType, TradeSignal, TradeAction, CopyTradingConfig};
pub use crate::utils::copy_trading_utils::{load_config_from_file, get_private_key_from_config, create_default_config, set_leverage_with_fallback}; 

use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, ClientOrderRequest, ClientOrder, ClientLimit};
use ethers::signers::{LocalWallet};
use std::str::FromStr;
use crate::strategies::take_profit_stop_loss::TakeProfitStopLoss;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::log;
use alloy::signers::local::PrivateKeySigner;
use crate::tg_bot::tg_bot::get_instance;
// 策略特征
pub trait CopyTradingStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn should_copy_trade(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> bool;
    fn calculate_copy_amount(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> f64;
    fn calculate_copy_price(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> f64;
    fn get_order_type(&self, signal: &TradeSignal, config: &CopyTradingConfig) -> OrderType;
}

// 跟单服务
#[derive(Clone)]
pub struct CopyTradingService {
    pub config: CopyTradingConfig,
    pub strategy: Arc<Box<dyn CopyTradingStrategy + Send + Sync>>,
    pub tp_sl_service: Arc<Mutex<Option<TakeProfitStopLoss>>>,
}

impl CopyTradingService {
    pub fn new(config: CopyTradingConfig) -> Self {
        use crate::strategies::factory::StrategyFactory;
        let strategy = StrategyFactory::create_strategy(&config.strategy_type, &config);
        Self { 
            config, 
            strategy: Arc::new(strategy), 
            tp_sl_service: Arc::new(Mutex::new(None)) 
        }
    }

    /// 初始化止盈止损服务
    pub async fn init_take_profit_stop_loss(&self, private_key: &str) -> Result<()> {
        let tp_sl_service = TakeProfitStopLoss::new(self.config.clone(), private_key).await?;
        let mut guard = self.tp_sl_service.lock().await;
        *guard = Some(tp_sl_service);
        info!("止盈止损服务已初始化");
        Ok(())
    }

    pub async fn process_trade_signal(&self, signal: &TradeSignal) -> Result<()> {
        info!("收到交易信号: {:?}", signal);

        if self.strategy.should_copy_trade(signal, &self.config) {
            let amount = self.strategy.calculate_copy_amount(signal, &self.config);
            let price = self.strategy.calculate_copy_price(signal, &self.config);
            let order_type = self.strategy.get_order_type(signal, &self.config);

            self.execute_copy_trade(signal, amount, price, order_type).await?;
        } else {
            info!("策略 {} 决定跳过跟单", self.strategy.name());
            
            // 发送 Telegram 通知 - 跳过跟单
            let bot = get_instance();
            let actual_direction = signal.original_direction.as_ref()
                .and_then(|d| crate::utils::parse_trade_direction(d).ok())
                .map(|info| info.text)
                .unwrap_or_else(|| match signal.action {
                    TradeAction::Buy => "多仓",
                    TradeAction::Sell => "空仓",
                });
            
            let notification = format!(
                "⏸跟单信号已跳过\n\n\
                • 信号详情\n\
                • 代币: `{}`\n\
                • 策略: `{}`\n\
                • 动作: `{}`\n\
                • 时间: `{}`",
                signal.coin,
                self.strategy.name(),
                actual_direction,
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap()).format("%Y-%m-%d %H:%M:%S")
            );
            
            // 异步发送通知，不阻塞主流程
            tokio::spawn(async move {
                if let Err(e) = bot.send_message_async(&notification, None).await {
                    warn!("发送 Telegram 通知失败: {:?}", e);
                }
            });
        }

        Ok(())
    }

    async fn execute_copy_trade(&self, signal: &TradeSignal, amount: f64, price: f64, order_type: OrderType) -> Result<()> {
        // 获取私钥并创建钱包
        let private_key = get_private_key_from_config("config/config.toml")?;
        let wallet = LocalWallet::from_str(&private_key)?;
        
        // 创建交易所客户端
        let wallet: PrivateKeySigner = private_key.parse()?;
        let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Mainnet), None, None).await?;
        // 确定交易方向字符串 - 使用原始方向或默认方向
        let dir = signal.original_direction.as_ref()
            .map(|d| d.as_str())
            .unwrap_or_else(|| match signal.action {
                TradeAction::Buy => "Open Long",
                TradeAction::Sell => "Open Short",
            });

        // 执行跟单交易
        let result = execute_copy_trade_inline(
            &signal.coin,
            &amount.to_string(),
            &price.to_string(),
            dir,
            &self.config,
            &exchange_client,
            order_type
        ).await;

        // 如果开仓成功，记录持仓信息用于止盈止损
        if result.is_ok() {
            // 检查是否为开仓操作
            let is_open_operation = matches!(dir, "Open Long" | "Open Short");
            if is_open_operation {
                self.record_position_for_tp_sl(&signal.coin, &dir, amount, price).await;
            } else {
                // 平仓操作成功，检查是否完全平仓
                self.check_and_update_close_position(&signal.coin, &self.config.wallet).await;
            }
        }

        result
    }

    /// 记录持仓信息用于止盈止损
    async fn record_position_for_tp_sl(&self, coin: &str, dir: &str, size: f64, price: f64) {
        let mut guard = self.tp_sl_service.lock().await;
        if let Some(tp_sl_service) = guard.as_mut() {
            let action = match dir {
                "Open Long" => "多头 (Long)",
                "Open Short" => "空头 (Short)",
                _ => dir,
            };
            
            tp_sl_service.record_open_position(coin, action, size, price);
        }
    }

    /// 检查并更新平仓状态
    async fn check_and_update_close_position(&self, coin: &str, our_wallet_address: &str) {
        let mut guard = self.tp_sl_service.lock().await;
        if let Some(tp_sl_service) = guard.as_mut() {
            // 检查实际持仓数量
            if let Some(position_size) = tp_sl_service.get_actual_position_size(coin, our_wallet_address).await {
                if position_size.abs() == 0.0 {
                    // 完全平仓，更新本地状态
                    tp_sl_service.state.record_close_position(coin);
                    log!(
                        info,
                        "copy_trading_strategy",
                        "check_and_update_close_position",
                        format!("{} 完全平仓，已更新本地状态", coin).as_str()
                    );
                } else {
                    // 部分平仓，不更新本地状态
                    log!(
                        info,
                        "copy_trading_strategy",
                        "check_and_update_close_position",
                        format!("{} 部分平仓，剩余持仓: {:.4}，本地状态保持不变", coin, position_size.abs()).as_str()
                    );
                }
            } else {
                // 没有找到持仓信息，说明已完全平仓
                tp_sl_service.state.record_close_position(coin);
                log!(
                    info,
                    "copy_trading_strategy",
                    "check_and_update_close_position",
                    format!("{} 未找到持仓信息，已更新为完全平仓状态", coin).as_str()
                );
            }
        }
    }

    /// 检查止盈止损
    pub async fn check_take_profit_stop_loss(&self, wallet_address: &str) -> Result<()> {
        let mut guard = self.tp_sl_service.lock().await;
        if let Some(tp_sl_service) = guard.as_mut() {
            tp_sl_service.check_take_profit_stop_loss(wallet_address).await?;
        }
        Ok(())
    }

    /// 检查是否应该跟随目标地址平仓
    pub async fn should_follow_target_close(&self, coin: &str, our_wallet_address: &str) -> bool {
        let guard = self.tp_sl_service.lock().await;
        if let Some(tp_sl_service) = guard.as_ref() {
            return tp_sl_service.should_follow_target_close(coin, our_wallet_address).await;
        }
        true  // 如果没有止盈止损服务，默认跟随
    }
}

/// 执行跟单交易
async fn execute_copy_trade_inline(
    coin: &str, 
    sz: &str, 
    px: &str, 
    dir: &str, 
    config: &CopyTradingConfig,
    exchange_client: &ExchangeClient,
    order_type: OrderType
) -> Result<()> {
    // 计算跟单数量
    let original_size: f64 = sz.parse()
        .map_err(|_| anyhow::anyhow!("无法解析交易数量: {}", sz))?;
    
    let copy_size = original_size * config.copy_ratio;
    
    // 添加数量精度处理，四舍五入到0.001 ETH
    let copy_size = (copy_size * 1000.0).round() / 1000.0;
    
    // 检查单笔最大仓位限制 (平仓操作不受此限制)
    let position_value = copy_size * px.parse::<f64>()
        .map_err(|_| anyhow::anyhow!("无法解析价格: {}", px))?;
    
    let is_close_operation = matches!(dir, "Close Long" | "Close Short");
    
    if !is_close_operation && position_value > config.max_position_value {
        warn!("跟单金额 ${:.2} 超过最大限制 ${:.2}，跳过此次跟单", 
            position_value, config.max_position_value);
        return Ok(());
    }
    
    if is_close_operation {
        info!("平仓操作不受最大仓位限制约束");
    }

    // 使用统一的交易方向解析
    let direction_info = match crate::utils::parse_trade_direction(dir) {
        Ok(info) => info,
        Err(e) => {
            warn!("无法确定交易方向: {}", e);
            return Ok(());
        }
    };
    
    let is_buy = direction_info.is_buy;
    let reduce_only = direction_info.reduce_only;

    // 如果是开仓操作，先设置杠杆
    let actual_leverage = if !is_close_operation {
        info!("[仓位设置] 尝试设置杠杆: {}倍 | 仓位类型: {}", config.leverage, config.margin_type);
        
        // 使用智能杠杆设置（带降级处理）
        let is_cross = config.margin_type == "cross";
        match set_leverage_with_fallback(exchange_client, coin, config.leverage, is_cross).await {
            Ok(_) => {
                info!("[杠杆设置] 成功使用用户配置: {}倍", config.leverage);
                config.leverage
            }
            Err(e) => {
                warn!("[杠杆设置完全失败] {}，将使用平台默认杠杆", e);
                // 返回一个特殊值表示使用平台默认
                0
            }
        }
    } else {
        config.leverage // 平仓操作不需要设置杠杆
    };
    
    // 让交易所自动处理保证金计算和分配
    if !is_close_operation {
        info!("[保证金处理] 交易所将自动计算和分配保证金");
    }

    // 构建订单
    let order = ClientOrderRequest {
        asset: coin.to_string(),
        is_buy,
        reduce_only,
        limit_px: px.parse()?,  // 使用相同价格
        sz: copy_size,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: match order_type {
                OrderType::Market => "Ioc".to_string(), // 市价单：立即成交或取消
                OrderType::Limit { .. } => "Gtc".to_string(), // 限价单：直到取消前有效
            },
        }),
    };

    let operation_desc = direction_info.text;

    let order_type_desc = match order_type {
        OrderType::Market => "市价单 (IOC)",
        OrderType::Limit { .. } => "限价单 (GTC)",
    };

    let margin_type_desc = if is_close_operation { 
        "平仓操作" 
    } else { 
        &config.margin_type 
    };

    let leverage_desc = if actual_leverage == 0 {
        "平台默认".to_string()
    } else {
        format!("{}倍", actual_leverage)
    };

    // 在移动 order 之前保存需要的值
    let asset = order.asset.clone();
    let limit_px = order.limit_px;
    let sz = order.sz;

    // 发送订单
    let response = exchange_client.order(order, None).await?;
    
    // 使用Debug输出检查完整响应，然后分析是否成功
    let response_str = format!("{:?}", response);
    
    // 根据响应结果发送对应的 Telegram 通知
    let bot = get_instance();

    if response_str.contains("Error(") {
        // 发送 Telegram 通知 - 跟单失败
        let error_notification = format!(
            "跟单订单执行失败！\n\n\
            失败详情\n\
            • 资产: `{}`\n\
            • 操作: `{}`\n\
            • 价格: `${}`\n\
            • 数量: `{}`\n\
            • 订单类型: `{}`\n\
            • 拒绝原因: `{:?}`\n\n\
            • 失败时间: `{}`",
            asset,
            operation_desc,
            limit_px,
            sz,
            order_type_desc,
            response,
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap()).format("%Y-%m-%d %H:%M:%S ")
        );
        
        tokio::spawn(async move {
            if let Err(e) = bot.send_message_async(&error_notification, None).await {
                warn!("发送 Telegram 通知失败: {:?}", e);
            }
        });
        
        error!("[跟单失败] 订单被拒绝: {:?}", response);
        return Err(anyhow::anyhow!("跟单订单执行失败"));
    } else if response_str.contains("Filled(") {
        let success_notification = format!(
            "跟单订单立即成交！\n\n\
            成交详情\n\
            • 资产: `{}`\n\
            • 操作: `{}`\n\
            • 价格: `${}`\n\
            • 数量: `{}`\n\
            • 订单类型: `{}`\n\n\
            • 成交时间: `{}`",
            asset, operation_desc, limit_px, sz, order_type_desc,
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap()).format("%Y-%m-%d %H:%M:%S ")
        );
        
        tokio::spawn(async move {
            if let Err(e) = bot.send_message_async(&success_notification, None).await {
                warn!("发送 Telegram 通知失败: {:?}", e);
            }
        });
        
        info!("[跟单成功] 订单立即成交");
    } else if response_str.contains("Resting(") {
        // 发送 Telegram 通知 - 订单已挂单
        let pending_notification = format!(
            "跟单订单已挂单\n\n\
            挂单详情\n\
            • 资产: `{}`\n\
            • 操作: `{}`\n\
            • 价格: `${}`\n\
            • 数量: `{}`\n\
            • 订单类型: `{}`\n\n\
            • 挂单时间: `{}`",
            asset, operation_desc, limit_px, sz, order_type_desc,
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap()).format("%Y-%m-%d %H:%M:%S ")
        );
        
        tokio::spawn(async move {
            if let Err(e) = bot.send_message_async(&pending_notification, None).await {
                warn!("发送 Telegram 通知失败: {:?}", e);
            }
        });
        
        info!("[跟单成功] 订单已挂单等待成交");
    } else {
        info!("[跟单提交] 订单状态: {:?}", response);
    }

    Ok(())
}

