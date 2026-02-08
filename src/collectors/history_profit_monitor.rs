use crate::database::Database;
use anyhow::Result;
use crate::analyzers::HistoryProfitAnalyzer;
use tokio::time::{interval, Duration};

pub struct HistoryProfitMonitor {
    history_profit_analyzer: HistoryProfitAnalyzer,
    database: Database,
}

impl HistoryProfitMonitor {
    pub async fn new(database: Database, _check_interval_hours: u64) -> Result<Self> {
        let history_profit_analyzer = HistoryProfitAnalyzer::new(database.clone()).await?;

        Ok(Self {
            history_profit_analyzer,
            database,
        })
    }

    pub async fn start(&self) -> Result<()> {
        crate::log!(info, "history_profit_monitor", "start", 
            "启动历史利润分析"
        );

        if let Err(e) = self.analyze_all_addresses().await {
            crate::error!("history_profit_monitor", "start", 
                "历史利润分析失败", e
            );
        }

        // 每分钟分析一次
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            crate::log!(debug, "history_profit_monitor", "start", 
                "历史利润分析开始"
            );

            if let Err(e) = self.analyze_all_addresses().await {
                crate::error!("history_profit_monitor", "start", 
                    "历史利润分析失败", e
                );
            }
        }   
    }

    async fn analyze_all_addresses(&self) -> Result<()> {
        // 获取活跃地址
        let monitored_addresses = match self.database.get_active_wallets().await {
            Ok(addresses) => addresses,
            Err(e) => {
                crate::error!("history_profit_monitor", "analyze_all_addresses", 
                    "获取活跃地址失败", e
                );
                return Ok(());
            }
        };

        let total_addresses = monitored_addresses.len();

        if total_addresses == 0 {
            crate::log!(info, "history_profit_monitor", "analyze_all_addresses", 
                "没有活跃地址"
            );
            return Ok(());
        }

        crate::log!(debug, "history_profit_monitor", "analyze_all_addresses", 
            "开始分析所有地址", 
            "total_addresses" => total_addresses
        );

        for (index, addr) in monitored_addresses.iter().enumerate() {
            crate::log!(debug, "history_profit_monitor", "analyze_all_addresses", 
                "分析地址", 
                "address" => addr,
                "progress" => format!("{}/{}", index + 1, total_addresses)
            );
            match self.history_profit_analyzer.get_user_history_profit(addr).await {
                Ok(profits) => {
                    crate::log!(debug, "history_profit_monitor", "analyze_all_addresses", 
                        "分析地址完成", 
                        "address" => addr,
                        "progress" => format!("{}/{}", index + 1, total_addresses),
                        "records_count" => profits.len()
                    );
                }
                Err(e) => {
                    crate::error!("history_profit_monitor", "analyze_all_addresses", 
                        "分析地址失败", e, 
                        "address" => addr,
                        "progress" => format!("{}/{}", index + 1, total_addresses)
                    );
                }
            }
        }   
        
        crate::log!(debug, "history_profit_monitor", "analyze_all_addresses", 
            "分析所有地址完成"
        );
        Ok(())
    }
}
