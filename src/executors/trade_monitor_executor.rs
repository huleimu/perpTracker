use anyhow::Result;
use crate::collectors::run_multi_address_monitor;
use crate::utils::{init_database, parse_address_list};

use crate::log;
use crate::error;

pub async fn run_trade_monitor_executor() -> Result<()> {
    // 初始化数据库
    log!(
        info,
        "trade_monitor_executor",
        "run_trade_monitor_executor",
        "初始化数据库"
    );
    let db = match init_database().await {
        Ok(db) => {
            log!(
                info,
                "trade_monitor_executor",
                "run_trade_monitor_executor",
                "数据库初始化成功"
            );
            db
        },
        Err(e) => {
            error!(
                "trade_monitor_executor",
                "run_trade_monitor_executor",
                "数据库初始化失败",
                e
            );
            return Ok(());
        }
    };

    // 从数据库加载地址
    log!(
        info,
        "trade_monitor_executor",
        "run_trade_monitor_executor",
        "从数据库加载监控地址"
    );
    let address_strings = match db.get_active_wallets().await {
        Ok(addrs) => {
            log!(
                info,
                "trade_monitor_executor",
                "run_trade_monitor_executor",
                "成功加载监控地址",
                "count" => addrs.len()
            );
            addrs
        },
        Err(e) => {
            error!(
                "trade_monitor_executor",
                "run_trade_monitor_executor",
                "从数据库获取地址失败",
                e
            );
            return Ok(());
        }
    };
    
    let addresses = match parse_address_list(&address_strings) {
        Ok(addrs) => {
            log!(
                info,
                "trade_monitor_executor",
                "run_trade_monitor_executor",
                "成功解析监控地址",
                "count" => addrs.len()
            );
            addrs
        },
        Err(e) => {
            error!(
                "trade_monitor_executor",
                "run_trade_monitor_executor",
                "解析地址失败",
                e
            );
            return Ok(());
        }
    };

    if addresses.is_empty() {
        log!(
            warn,
            "trade_monitor_executor",
            "run_trade_monitor_executor",
            "数据库中没有找到有效的地址"
        );
        log!(
            info,
            "trade_monitor_executor",
            "run_trade_monitor_executor",
            "请先使用钱包监控服务添加地址到数据库"
        );
        return Ok(());
    }

    let address_count = addresses.len();
    let addresses_str = format!("{:?}", addresses);
    
    log!(
        info,
        "trade_monitor_executor",
        "run_trade_monitor_executor",
        "开始监控交易活动",
        "address_count" => address_count,
        "addresses" => addresses_str
    );

    // 启动实时交易监控
    log!(
        info,
        "trade_monitor_executor",
        "run_trade_monitor_executor",
        "启动实时交易监控",
        "address_count" => address_count
    );
    if let Err(e) = run_multi_address_monitor(&db).await {
        error!(
            "trade_monitor_executor",
            "run_trade_monitor_executor",
            "交易监控服务失败",
            e,
            "address_count" => address_count
        );
    }

    log!(
        info,
        "trade_monitor_executor",
        "run_trade_monitor_executor",
        "交易监控服务已停止"
    );
    Ok(())
} 
