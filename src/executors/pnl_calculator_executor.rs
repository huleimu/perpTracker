use anyhow::Result;
use crate::collectors::{PnLMonitor, HistoryProfitMonitor};
use crate::utils::database_init::init_database;
use crate::log;
use crate::error;

pub async fn run_pnl_calculator_executor() -> Result<()> {
    // 初始化数据库
    let db = match init_database().await {
        Ok(db) => db,
        Err(_) => return Ok(()),
    };

    log!(
        info,
        "pnl_calculator_executor",
        "run_pnl_calculator_executor",
        "开始启动盈亏计算服务"
    );

    // 启动PnL监控任务
    let pnl_monitor = PnLMonitor::new(db.clone(), 12).await?;
    let pnl_handle = tokio::spawn(async move {
        if let Err(e) = pnl_monitor.start().await {
            error!(
                "pnl_calculator_executor",
                "pnl_monitor_task",
                "PnL监控任务失败",
                e,
                "task_type" => "pnl_monitor"
            );
        }
    });

    // 启动历史盈亏监控任务
    let history_profit_monitor = HistoryProfitMonitor::new(db.clone(), 12).await?;
    let history_profit_handle = tokio::spawn(async move {
        if let Err(e) = history_profit_monitor.start().await {
            error!(
                "pnl_calculator_executor",
                "history_profit_monitor_task",
                "历史盈亏监控任务失败",
                e,
                "task_type" => "history_profit_monitor"
            );
        }
    });

    // 等待任务完成或被中断
    tokio::select! {
        _ = pnl_handle => {
            log!(
                info,
                "pnl_calculator_executor",
                "run_pnl_calculator_executor",
                "PnL监控任务结束"
            );
        },
        _ = history_profit_handle => {
            log!(
                info,
                "pnl_calculator_executor",
                "run_pnl_calculator_executor",
                "历史盈亏监控任务结束"
            );
        },
        _ = tokio::signal::ctrl_c() => {
            log!(
                info,
                "pnl_calculator_executor",
                "run_pnl_calculator_executor",
                "接收到停止信号，正在关闭盈亏计算服务"
            );
        }
    }
    
    log!(
        info,
        "pnl_calculator_executor",
        "run_pnl_calculator_executor",
        "盈亏计算服务已停止"
    );
    Ok(())
} 
