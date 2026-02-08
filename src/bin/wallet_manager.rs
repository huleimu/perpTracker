use anyhow::Result;
use perpTracker::executors::run_wallet_manager_executor;
use perpTracker::utils::logging_config::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    // 启动钱包管理系统
    run_wallet_manager_executor().await?;
    Ok(())
} 
