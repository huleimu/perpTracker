use crate::database::Database;
use crate::database::models::UserHistoryProfit;

use anyhow::Result;



pub struct HistoryProfitAnalyzer {
    database: Database,
}

impl HistoryProfitAnalyzer {

    pub async fn new(database: Database) -> Result<Self> {

        Ok(Self { database })
    }


    pub async fn get_user_history_profit(&self, addr: &str) -> Result<Vec<UserHistoryProfit>> {
        
        let user_history_profits = self.database.get_user_history_profits_from_trade_events(addr).await?;
        for profit in &user_history_profits {
            self.save_user_history_profit_to_db(profit).await?;
        }
        Ok(user_history_profits)
    }

    pub async fn save_user_history_profit_to_db(&self, user_history_profit: &UserHistoryProfit) -> Result<()> {
        self.database.save_user_history_profit(user_history_profit).await?;
        Ok(())
    }
}
