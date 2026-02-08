use anyhow::Result;
use std::io;
use perpTracker::executors::{run_wallet_manager_executor, run_pnl_calculator_executor, run_trade_monitor_executor, run_copy_trading, run_price_collector_executor};
use perpTracker::database::repositories::reset_database_tables;
use perpTracker::utils::logging_config;
use perpTracker::log;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    logging_config::init_logging();

    log!(info, "main", "main", 
        "欢迎使用 perpTracker 区块链交易监控系统"
    );
    log!(info, "main", "main", 
        "=========================================="
    );
    log!(info, "main", "main", 
        "请选择要启动的服务"
    );
    log!(info, "main", "main", 
        "1. 交易监控服务 - 实时监控指定地址的交易活跃"
    );
    log!(info, "main", "main", 
        "2. 盈亏计算服务 - 计算并分析交易盈亏情况"
    );
    log!(info, "main", "main", 
        "3. 钱包管理 - 添加/删除监控的钱包地址"
    );
    log!(info, "main", "main", 
        "4. 跟单交易 - 跟随指定地址进行交易"
    );
    log!(info, "main", "main", 
        "5. 重置数据库表 - 删除并重新创建所有表"
    );
    log!(info, "main", "main", 
        "6. 价格收集服务 - 收集HL价格数据"
    );
    log!(info, "main", "main", 
        "7. 退出程序"
    );
    log!(info, "main", "main", 
        "请输入选择 (1-7)"
    );

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => {
            log!(info, "main", "main", 
                "启动交易监控服务"
            );
            run_trade_monitor_executor().await?;
        }
        "2" => {
            log!(info, "main", "main", 
                "启动盈亏计算服务"
            );
            run_pnl_calculator_executor().await?;
        }
        "3" => {
            log!(info, "main", "main", 
                "启动钱包管理"
            );
            run_wallet_manager_executor().await?;
        }
        "4" => {
            log!(info, "main", "main", 
                "启动跟单交易"
            );
            run_copy_trading().await?;
        }
        "5" => {
            log!(info, "main", "main", 
                "重置数据库表 - 删除并重新创建所有表"
            );
            reset_database_tables().await?;
        }
        "6" => {
            log!(info, "main", "main", 
                "启动价格收集服务"
            );
            run_price_collector_executor().await?;
        }
        "7" => {
            log!(info, "main", "main", 
                "程序退出"
            );
            return Ok(());
        }
        _ => {
            tracing::info!(
                service = "perpTracker",
                module = "main",
                function = "main",
                message = "无效选择,程序退出",
                choice = choice
            );
            return Ok(());
        }
    }

    Ok(())
}
