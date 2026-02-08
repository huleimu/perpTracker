use anyhow::Result;
use perpTracker::executors::run_pnl_calculator_executor;
use perpTracker::utils::logging_config::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    // 启动盈亏计算服务
    run_pnl_calculator_executor().await?;
    Ok(())
}

 
