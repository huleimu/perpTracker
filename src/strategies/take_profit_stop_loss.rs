use anyhow::Result;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, ExchangeClient, MarketCloseParams};
use ethers::signers::LocalWallet;
use crate::types::{TakeProfitStopLossState, PositionInfo, CopyTradingConfig};
use crate::log;
use crate::error;
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
/// 止盈止损
pub struct TakeProfitStopLoss {
    pub config: CopyTradingConfig,
    pub state: TakeProfitStopLossState,
    pub info_client: InfoClient,
    pub exchange_client: Option<ExchangeClient>,
}

impl TakeProfitStopLoss {
    pub async fn new(config: CopyTradingConfig, private_key: &str) -> Result<Self> {
        let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
        
        // 创建交易所客户端（用于平仓）
        let wallet: PrivateKeySigner = private_key.parse()?;
        let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Mainnet), None, None).await?;        
        Ok(Self {
            config,
            state: TakeProfitStopLossState::new(),
            info_client,
            exchange_client: Some(exchange_client),
        })
    }

    /// 记录开仓
    pub fn record_open_position(&mut self, coin: &str, action: &str, size: f64, entry_price: f64) {
        let is_long = action.contains("多头") || action.contains("Long");
        
        // 计算止盈止损价格
        let (take_profit_price, stop_loss_price) = self.calculate_tp_sl_prices(entry_price, is_long);
        
        let position_info = PositionInfo {
            coin: coin.to_string(),
            action: action.to_string(),
            position_size: size,
            entry_price,
            current_price: entry_price,  // 初始价格等于开仓价
            unrealized_pnl: 0.0,
            take_profit_price,
            stop_loss_price,
            is_long,
        };
        
        self.state.record_open_position(coin, position_info);
    }

    /// 计算止盈止损价格
    pub fn calculate_tp_sl_prices(&self, entry_price: f64, is_long: bool) -> (f64, f64) {
        if is_long {
            // 多头：止盈价 > 开仓价，止损价 < 开仓价
            let take_profit_price = entry_price * (1.0 + self.config.take_profit_percentage);
            let stop_loss_price = entry_price * (1.0 - self.config.stop_loss_percentage);
            (take_profit_price, stop_loss_price)
        } else {
            // 空头：止盈价 < 开仓价，止损价 > 开仓价
            let take_profit_price = entry_price * (1.0 - self.config.take_profit_percentage);
            let stop_loss_price = entry_price * (1.0 + self.config.stop_loss_percentage);
            (take_profit_price, stop_loss_price)
        }
    }

    /// 检查止盈止损
    pub async fn check_take_profit_stop_loss(&mut self, wallet_address: &str) -> Result<()> {
        // 获取当前持仓
        let positions = self.get_current_positions(wallet_address).await?;
        
        // 获取当前价格
        let current_prices = self.get_current_prices().await?;
        
        // 检查每个持仓
        for position in positions {
            let coin = &position.coin;
            
            // 跳过已平仓的币种
            if self.state.is_position_closed(coin) {
                continue;
            }
            
            // 获取当前价格
            let current_price = current_prices.get(coin).unwrap_or(&position.entry_price);
            
            // 更新持仓信息中的当前价格
            self.state.update_current_price(coin, *current_price);
            
            // 获取持仓信息
            if let Some(position_info) = self.state.get_position_info(coin) {
                // 检查是否触发止盈止损
                let should_close = self.check_tp_sl_trigger(position_info, *current_price);
                
                if should_close {
                    log!(
                        info,
                        "take_profit_stop_loss",
                        "check_take_profit_stop_loss",
                        format!("触发止盈止损: {} 当前价格 ${:.4}", coin, current_price).as_str()
                    );
                    
                    // 执行平仓
                    if let Err(e) = self.execute_close_position(coin).await {
                        error!(
                            "take_profit_stop_loss",
                            "check_take_profit_stop_loss",
                            format!("平仓失败 {}", coin).as_str(),
                            e
                        );
                    } else {
                        self.state.record_close_position(coin);
                    }
                }
            }
        }
        
        self.state.last_check_time = chrono::Utc::now();
        Ok(())
    }

    /// 检查是否触发止盈止损
    pub fn check_tp_sl_trigger(&self, position_info: &PositionInfo, current_price: f64) -> bool {
        if position_info.is_long {
            // 多头：价格 >= 止盈价 或 价格 <= 止损价
            current_price >= position_info.take_profit_price || current_price <= position_info.stop_loss_price
        } else {
            // 空头：价格 <= 止盈价 或 价格 >= 止损价
            current_price <= position_info.take_profit_price || current_price >= position_info.stop_loss_price
        }
    }

    /// 获取当前持仓
    async fn get_current_positions(&self, wallet_address: &str) -> Result<Vec<crate::types::PositionPnl>> {
        use crate::utils::safe_parse_f64;
        use std::str::FromStr;
        use ethers::types::H160;
        
        // 直接从交易所API获取实时持仓
        
        let target_addr: Address = Address::from_slice(&H160::from_str(wallet_address)?.as_bytes());
        let user_state = self.info_client.user_state(target_addr).await?;
        
        let mut positions = Vec::new();
        
        for asset_position in &user_state.asset_positions {
            let position = &asset_position.position;
            
            // 仓位数据
            let szi = safe_parse_f64(&position.szi, 0.0);
            if szi == 0.0 {
                continue; // 跳过空仓位
            }
            
            let action: &'static str = if szi > 0.0 { "多头 (Long)" } else { "空头 (Short)" };
            let position_size = szi.abs();
            
            let entry_price = position.entry_px
                .as_deref()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            
            let unrealized_pnl = safe_parse_f64(&position.unrealized_pnl, 0.0);
            let liquidation_price: Option<f64> = position.liquidation_px
                .as_deref()
                .and_then(|px| px.parse().ok());
            
            let position_value = safe_parse_f64(&position.position_value, 0.0);
            let roe = safe_parse_f64(&position.return_on_equity, 0.0);
            
            positions.push(crate::types::PositionPnl {
                coin: position.coin.clone(),
                action: action.to_string(),
                position_size,
                position_value,
                entry_price,
                liquidation_price,
                unrealized_pnl,
                roe,
            });
        }
        
        Ok(positions)
    }

    /// 获取当前价格
    async fn get_current_prices(&self) -> Result<std::collections::HashMap<String, f64>> {
        let all_mids = self.info_client.all_mids().await?;
        
        // 将String转换为f64
        let mut prices = std::collections::HashMap::new();
        for (coin, price_str) in all_mids {
            if let Ok(price) = price_str.parse::<f64>() {
                prices.insert(coin, price);
            }
        }
        
        Ok(prices)
    }

    /// 执行平仓
    async fn execute_close_position(&self, coin: &str) -> Result<()> {
        if let Some(exchange_client) = &self.exchange_client {
            let market_close_params = MarketCloseParams {
                asset: coin,
                sz: None,  // None表示平掉全部仓位
                px: None,  // 市价
                slippage: Some(0.01),  // 1%滑点保护
                cloid: None,
                wallet: None,
            };
            
            let response = exchange_client.market_close(market_close_params).await?;
            
            match response {
                hyperliquid_rust_sdk::ExchangeResponseStatus::Ok(_) => {
                    log!(
                        info,
                        "take_profit_stop_loss",
                        "execute_close_position",
                        format!("平仓成功: {}", coin).as_str()
                    );
                    Ok(())
                }
                hyperliquid_rust_sdk::ExchangeResponseStatus::Err(e) => {
                    error!(
                        "take_profit_stop_loss",
                        "execute_close_position",
                        format!("平仓失败: {}", coin).as_str(),
                        e
                    );
                    Err(anyhow::anyhow!("平仓失败: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("交易所客户端未初始化"))
        }
    }

    /// 获取实际持仓数量
    pub async fn get_actual_position_size(&self, coin: &str, our_wallet_address: &str) -> Option<f64> {
        if let Ok(our_address) = our_wallet_address.parse() {
            if let Ok(user_state) = self.info_client.user_state(our_address).await {
                for asset_position in &user_state.asset_positions {
                    if asset_position.position.coin == coin {
                        let position_size = asset_position.position.szi.parse::<f64>().unwrap_or(0.0);
                        return Some(position_size);
                    }
                }
            }
        }
        None  // 没有找到持仓信息
    }

    /// 检查是否应该跟随目标地址平仓
    pub async fn should_follow_target_close(&self, coin: &str, our_wallet_address: &str) -> bool {
        // 1. 检查本地状态：如果本地标记为已平仓，先输出日志
        if self.state.is_position_closed(coin) {
            log!(
                info,
                "take_profit_stop_loss",
                "should_follow_target_close",
                format!("本地状态显示 {} 已平仓，但仍需检查实际持仓", coin).as_str()
            );
        }
        
        // 2. 检查实际持仓数量
        if let Some(position_size) = self.get_actual_position_size(coin, our_wallet_address).await {
            if position_size.abs() > 0.0 {
                log!(
                    info,
                    "take_profit_stop_loss",
                    "should_follow_target_close",
                    format!("发现 {} 实际持仓: {:.4}，允许跟随平仓", coin, position_size.abs()).as_str()
                );
                return true;  // 有实际持仓，允许跟随平仓
            } else {
                log!(
                    info,
                    "take_profit_stop_loss",
                    "should_follow_target_close",
                    format!("{} 实际持仓为0，跳过跟随平仓", coin).as_str()
                );
                return false;  // 没有实际持仓，不允许跟随平仓
            }
        }
        
        log!(
            warn,
            "take_profit_stop_loss",
            "should_follow_target_close",
            format!("未找到 {} 的持仓信息，跳过跟随平仓", coin).as_str()
        );
        false  // 没有找到持仓信息，不允许跟随平仓
    }
} 