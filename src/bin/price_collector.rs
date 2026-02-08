use anyhow::Result;
use perpTracker::executors::run_price_collector_executor;
use perpTracker::log;
use perpTracker::utils::logging_config::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    log!(info, "price_collector_bin", "main", "启动价格收集器服务");

    // 设置 Ctrl+C 信号处理
    ctrlc::set_handler(|| {
        log!(info, "price_collector_bin", "main", "收到退出信号，正在关闭服务");
        std::process::exit(0);
    })?;

    // 运行价格收集器
    match run_price_collector_executor().await {
        Ok(_) => {
            log!(info, "price_collector_bin", "main", "价格收集器服务正常退出");
        }
        Err(e) => {
            log!(error, "price_collector_bin", "main", "价格收集器服务异常退出", "error" => format!("{:?}", e));
            std::process::exit(1);
        }
    }

    Ok(())
} 