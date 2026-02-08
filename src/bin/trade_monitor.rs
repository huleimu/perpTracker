use anyhow::Result;
use perpTracker::executors::run_trade_monitor_executor;
use perpTracker::utils::logging_config::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    // 启动交易监控服务
    run_trade_monitor_executor().await?;
    Ok(())
}

 
