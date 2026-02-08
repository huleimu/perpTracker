use crate::analyzers::PnLAnalyzer;
use crate::database::Database;
use anyhow::Result;
use std::time::Duration;
use tokio::time::{interval, sleep};




pub struct PnLMonitor {
    pnl_analyzer: PnLAnalyzer,
    database: Database,
    check_interval_hours: u64,
}

impl PnLMonitor {
    /// 创建新的PnL监控
    pub async fn new(
        database: Database,
        check_interval_hours: u64,
    ) -> Result<Self> {
        let pnl_analyzer = PnLAnalyzer::new(database.clone()).await?;
        
        Ok(Self {
            pnl_analyzer,
            database,
            check_interval_hours,
        })
    }

    /// 启动PnL监控
    pub async fn start(&self) -> Result<()> {
        crate::log!(info, "pnl_monitor", "start", 
            "启动PnL监控", 
            "check_interval_hours" => self.check_interval_hours
        );

        let interval2 = 1;
        // 创建定时器
        let mut interval = interval(Duration::from_secs(interval2 * 60));

        loop {
            interval.tick().await;
                    crate::log!(debug, "pnl_monitor", "start", 
            "定时PnL分析开始"    
        );
            
            if let Err(e) = self.analyze_all_addresses().await {
                crate::error!("pnl_monitor", "start", 
                    "定时PnL分析失败", e
                );
            }
            
            crate::log!(info, "pnl_monitor", "start", 
                "等待下次分析"
            );
        }
    }

    /// 分析所有监控地址的PnL
    async fn analyze_all_addresses(&self) -> Result<()> {
        // 从数据库获取最新的地址列表
        let monitored_addresses = match self.database.get_active_wallets().await {
            Ok(addresses) => addresses,
            Err(e) => {
                crate::error!("pnl_monitor", "analyze_all_addresses", 
                    "从数据库获取地址失败", e
                );
                return Ok(());
            }
        };

        let total_addresses = monitored_addresses.len();
        let mut successful_analyses = 0;
        let mut failed_analyses = 0;

        if total_addresses == 0 {
            crate::log!(info, "pnl_monitor", "analyze_all_addresses", 
                "数据库中没有活跃的地址，跳过PnL分析"
            );
            return Ok(());
        }

        crate::log!(debug, "pnl_monitor", "analyze_all_addresses", 
            "开始分析地址的PnL", 
            "total_addresses" => total_addresses
        );

        for (index, address) in monitored_addresses.iter().enumerate() {
            crate::log!(debug, "pnl_monitor", "analyze_all_addresses", 
                "分析地址", 
                "address" => address,
                "progress" => format!("{}/{}", index + 1, total_addresses)
            );
            
            match self.pnl_analyzer.get_user_pnl_summary(address).await {
                Ok(_summary) => {
                    successful_analyses += 1;
                    crate::log!(debug, "pnl_monitor", "analyze_all_addresses", 
                        "分析完成", 
                        "address" => address,
                        "progress" => format!("{}/{}", index + 1, total_addresses)
                    );
                } 
                Err(e) => {
                    failed_analyses += 1;
                    crate::error!("pnl_monitor", "analyze_all_addresses", 
                        "分析失败", e, 
                        "address" => address,
                        "progress" => format!("{}/{}", index + 1, total_addresses)
                    );
                }
            }

            // 避免API限流，每次分析后稍微等待
            if index < total_addresses - 1 {
                sleep(Duration::from_millis(500)).await;
            }
        }

        // 输出汇总统计
        let success_rate = (successful_analyses as f64 / total_addresses as f64) * 100.0;
        
        crate::log!(info, "pnl_monitor", "analyze_all_addresses", 
            "PnL分析完成", 
            "successful_analyses" => successful_analyses,
            "failed_analyses" => failed_analyses,
            "total_addresses" => total_addresses,
            "success_rate" => format!("{:.1}%", success_rate)
        );

        if failed_analyses > 0 {
            crate::log!(warn, "pnl_monitor", "analyze_all_addresses", 
                "有地址分析失败，请检查日志", 
                "failed_count" => failed_analyses
            );
        }

        Ok(())
    }
}
