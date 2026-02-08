use crate::database::{Database, UserProfit};
use anyhow::Result;
use chrono::Utc;
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use alloy::primitives::Address;
use std::str::FromStr;
use crate::types::{UserPnlSummary, PositionPnl, HistoryTradePnL};


pub struct PnLAnalyzer {
    pub database: Database,
    pub info_client: InfoClient,
}



impl PnLAnalyzer {
    
    pub async fn new(database:Database)->Result<Self> {
        let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
        
        Ok(Self {
            database,
            info_client,
        })
    }

    pub async fn get_user_pnl_summary(&self, addr: &str)->Result<UserPnlSummary> {
        let target_addr = H160::from_str(addr)?;

        let positions = self.get_user_positions(&target_addr).await?;

        let history_trades = self.get_user_history_trades(&target_addr).await?;

        let total_realized_pnl = history_trades.iter().map(|t| t.realized_pnl).sum::<f64>();
        let total_unrealized_pnl = positions.iter().map(|p| p.unrealized_pnl).sum::<f64>();
        let total_pnl = total_unrealized_pnl + total_realized_pnl;
        
        let summary = UserPnlSummary {
           addr: addr.to_string(),
           positions,
           history_trades,
           total_pnl,
           total_realized_pnl,
           total_unrealized_pnl,
           last_updated: Utc::now(),
        };
        self.save_user_profit_to_db(&summary).await?;
        Ok(summary)
    }
    
    pub async fn get_user_positions(&self, addr: &H160)->Result<Vec<PositionPnl>> {
        use crate::utils::safe_parse_f64;
        let address: Address = Address::from_slice(&addr.as_bytes());
        let user_state = self.info_client.user_state(address).await?;

        let mut positions = Vec::new();

        for asset_position in &user_state.asset_positions {
            let  position = &asset_position.position;

            //仓位数据
            let szi = safe_parse_f64(&position.szi, 0.0);
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

            positions.push(PositionPnl {
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

    pub async fn get_user_history_trades(&self,addr: &H160)->Result<Vec<HistoryTradePnL>> {

        let address: Address = Address::from_slice(&addr.as_bytes());
        let user_fills = self.info_client.user_fills(address).await?;

        let mut history_trades = Vec::new();

        // 只获取最近100笔交易
        let recent_fills = user_fills.iter().take(100);      

        for fill in recent_fills {
            use crate::utils::safe_parse_f64;
            let coin = fill.coin.clone();
            let realized_pnl = safe_parse_f64(&fill.closed_pnl, 0.0);
            history_trades.push(HistoryTradePnL { coin, realized_pnl });
        }

    Ok(history_trades)
    }

    pub async fn save_user_profit_to_db(&self, summary: &UserPnlSummary)->Result<()> {
        // 按币种聚合历史交易的已实现PnL
        let mut coin_realized_pnl: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        
        for trade in &summary.history_trades {
            *coin_realized_pnl.entry(trade.coin.clone()).or_insert(0.0) += trade.realized_pnl;
        }
    
        for position in &summary.positions {
            // 获取该币种的已实现PnL
            let coin_realized = coin_realized_pnl.get(&position.coin).unwrap_or(&0.0);
            
            // 计算该币种的总PnL = 未实现PnL + 已实现PnL
            let coin_total_pnl = position.unrealized_pnl + coin_realized;
            
            let user_profit = UserProfit {
                id: None,
                addr: summary.addr.clone(),
    
                //持仓状态
                coin: position.coin.clone(),
                action: position.action.clone(),
                position_size: position.position_size,
                position_value: position.position_value,
                entry_price: position.entry_price,
                liquidation_price: position.liquidation_price,
    
                // 修正：使用币种级别的PnL数据
                total_pnl: coin_total_pnl,                    // 该币种的总PnL
                realized_pnl: *coin_realized,                 // 该币种的已实现PnL
                unrealized_pnl: position.unrealized_pnl,     // 该币种的未实现PnL (来自当前持仓)
                roe: position.roe,
                start_time: summary.last_updated,
                update_time: Some(summary.last_updated),
            };
    
            self.database.save_user_profit(&user_profit).await?;
        }
    
        Ok(())
    }


}
