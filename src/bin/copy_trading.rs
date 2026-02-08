use anyhow::Result;
use perpTracker::executors::run_copy_trading;
use perpTracker::utils::logging_config::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    // 启动跟单交易服务
    run_copy_trading().await?;
    Ok(())
} 
