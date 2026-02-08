use anyhow::Result;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription, UserData};
use alloy::primitives::Address;
use std::io::{self, Write};
use std::str::FromStr;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{sleep, Duration};
use crate::strategies::copy_trading_strategy::CopyTradingService;
use crate::utils::copy_trading_utils::{
    create_default_config, load_config_from_file, get_private_key_from_config
};
use crate::utils::address_utils::is_valid_eth_address;
use crate::utils::database_init::init_database;
use crate::utils::safe_parse_f64;

use crate::utils::prompt_user;
use crate::types::TradeSignal;
use rust_decimal::prelude::ToPrimitive;
use crate::log;
use crate::error;


// 重连策略的配置常量
const INITIAL_RECONNECT_DELAY_S: u64 = 1;
const MAX_RECONNECT_DELAY_S: u64 = 60;
const TP_SL_CHECK_INTERVAL_S: u64 = 30; // 止盈止损检查间隔

/// 运行跟单交易服务
pub async fn run_copy_trading() -> Result<()> {
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "启动跟单交易服务"
    );
    
    // 选择跟单目标地址
    let target_user = select_target_user().await?;
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "启动跟单交易服务",
        "target_user" => format!("{:?}", target_user)
    );
    
    // 加载配置
    let mut config = match load_config_from_file("config/config.toml") {
        Ok(config) => {
            log!(
                debug,
                "copy_trading_executor",
                "run_copy_trading",
                "成功从配置文件加载配置",
                "config_source" => "file"
            );
            config
        }
        Err(e) => {
            crate::log!(warn,
                "copy_trading_executor",
                "run_copy_trading",
                "配置文件加载失败，使用默认配置",
                "error" => format!("{}", e)
            );
            create_default_config()
        }
    };
    
    // 验证策略类型
    if !matches!(config.strategy_type.as_str(), "conservative" | "aggressive") {
        crate::log!(warn,
            "copy_trading_executor",
            "validate_config",
            "策略类型配置无效，使用默认策略",
            "invalid_strategy" => config.strategy_type.clone(),
            "default_strategy" => "conservative"
        );
        config.strategy_type = "conservative".to_string();
        log!(
            info,
            "copy_trading_executor",
            "validate_config",
            "已切换到默认保守策略"
        );
    }
    
    // 验证仓位类型
    if !matches!(config.margin_type.as_str(), "isolated" | "cross") {
        crate::log!(warn,
            "copy_trading_executor",
            "validate_config",
            "仓位类型配置无效，使用默认仓位类型",
            "invalid_margin_type" => config.margin_type.clone(),
            "default_margin_type" => "isolated"
        );
        config.margin_type = "isolated".to_string();
        log!(
            info,
            "copy_trading_executor",
            "validate_config",
            "已切换到默认逐仓模式"
        );
    }
    
    // 验证杠杆倍数
    if config.leverage < 1 || config.leverage > 100 {
        crate::log!(warn,
            "copy_trading_executor",
            "validate_config",
            "杠杆倍数配置无效，使用默认杠杆倍数",
            "invalid_leverage" => config.leverage,
            "default_leverage" => 5
        );
        config.leverage = 5;
        log!(
            info,
            "copy_trading_executor",
            "validate_config",
            "已切换到默认5倍杠杆"
        );
    }
    
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "策略类型: {}",
        "strategy" => config.strategy_type
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "仓位类型: {} ({})",
        "margin_type" => config.margin_type,
        "margin_type_description" => if config.margin_type == "isolated" { "逐仓模式" } else { "全仓模式" }
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "杠杆倍数: {}",
        "leverage" => config.leverage
    );
    
    // 更新目标用户
    config.target_user = target_user;

    // 获取私钥
    let private_key = match get_private_key_from_config("config/config.toml") {
        Ok(key) if key != "your_private_key_here" && !key.is_empty() => {
            log!(
                info,
                "copy_trading_executor",
                "run_copy_trading",
                "从配置文件加载私钥"
            );
            key
        }
        _ => {
            log!(
                info,
                "copy_trading_executor",
                "run_copy_trading",
                "请在配置文件中填入真实私钥"
            );
            String::new()
        }
    };
    
    if private_key.is_empty() {
        log!(
            info,
            "copy_trading_executor",
            "run_copy_trading",
            "私钥不能为空"
        );
        return Ok(());
    }

    // 显示完整配置信息并确认
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "========== 跟单配置确认 =========="
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "目标用户: {:?}",
        "target_user" => format!("{:?}", config.target_user)
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "策略类型: {} ({})",
        "strategy" => config.strategy_type,
        "strategy_description" => if config.strategy_type == "conservative" { "保守策略" } else { "激进策略" }
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "仓位类型: {} ({})",
        "margin_type" => config.margin_type,
        "margin_type_description" => if config.margin_type == "isolated" { "逐仓模式 - 风险隔离" } else { "全仓模式 - 风险共享" }
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "杠杆倍数: {}",
        "leverage" => config.leverage
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "跟单比例: {:.1}%",
        "copy_ratio" => config.copy_ratio * 100.0
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "最大仓位价值: ${:.2}",
        "max_position_value" => config.max_position_value
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "启用资产: {:?}",
        "enabled_assets" => format!("{:?}", config.enabled_assets)
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "止盈比例: {:.1}%",
        "take_profit_percentage" => config.take_profit_percentage * 100.0
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "止损比例: {:.1}%",
        "stop_loss_percentage" => config.stop_loss_percentage * 100.0
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "订单类型: {}",
        "order_type" => config.order_type
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "提示: 如需修改配置，请编辑 config/config.toml 文件"
    );
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "按回车键开始跟单交易.."
    );
            prompt_user("确认启动: ")?;
    log!(
        info,
        "copy_trading_executor",
        "run_copy_trading",
        "开始跟单交易！"
    );

    // 创建钱包和交易客户端
    let wallet: LocalWallet = private_key.parse()
        .map_err(|_| anyhow::anyhow!("无效的私钥格式"))?;
    
    // 创建跟单服务
    let copy_service = CopyTradingService::new(config);
    
    // 初始化止盈止损服务
    if let Err(e) = copy_service.init_take_profit_stop_loss(&private_key).await {
                    crate::log!(warn,
                        "copy_trading_executor",
                        "run_copy_trading",
                        "止盈止损服务初始化失败",
                        "error" => format!("{}", e)
                    );
        log!(
            info,
            "copy_trading_executor",
            "run_copy_trading",
            "将继续运行，但不启用止盈止损功能"
        );
    } else {
        log!(
            info,
            "copy_trading_executor",
            "run_copy_trading",
            "止盈止损服务已启用"
        );
        log!(
            info,
            "copy_trading_executor",
            "run_copy_trading",
            "止盈比例: {:.1}%",
            "take_profit_percentage" => copy_service.config.take_profit_percentage * 100.0
        );
        log!(
            info,
            "copy_trading_executor",
            "run_copy_trading",
            "止损比例: {:.1}%",
            "stop_loss_percentage" => copy_service.config.stop_loss_percentage * 100.0
        );
    }

    // 启动止盈止损监控
    let copy_service_clone = copy_service.clone();
    let wallet_address = format!("{:?}", wallet.address());
    tokio::spawn(async move {
        loop {
            if let Err(e) = copy_service_clone.check_take_profit_stop_loss(&wallet_address).await {
                error!(
                    "copy_trading_executor",
                    "run_copy_trading",
                    "止盈止损检查失败 {}",
                    format!("{}", e)
                );
            }
            sleep(Duration::from_secs(TP_SL_CHECK_INTERVAL_S)).await;
        }
    });

    // 运行跟单监控
    run_copy_trading_monitor(&copy_service).await?;

    Ok(())
}

/// 显示 user_history_profit 表数据并让用户选择
async fn select_target_user() -> Result<H160> {
    // 首先尝试从配置文件获取默认地址
    if let Ok(config) = load_config_from_file("config/config.toml") {
        log!(
            info,
            "copy_trading_executor",
            "select_target_user",
            "从配置文件加载默认跟单地址: {:?}",
            "target_user" => format!("{:?}", config.target_user)
        );
        log!(
            info,
            "copy_trading_executor",
            "select_target_user",
            "提示: 如果想从数据库选择其他地址，请输入 'db'"
        );
        log!(
            info,
            "copy_trading_executor",
            "select_target_user",
            "提示: 如果想手动输入地址，请输入 'custom'"
        );
        log!(
            info,
            "copy_trading_executor",
            "select_target_user",
            ""
        );
        
        let choice = prompt_user("请选择 (直接回车使用默认地址, 'db' 从数据库选择, 'custom' 手动输入): ")?.trim().to_string();
        
        match choice.as_str() {
            "" => {
                log!(
                    info,
                    "copy_trading_executor",
                    "select_target_user",
                    "使用配置文件中的默认地址"
                );
                return Ok(config.target_user);
            }
            "db" => {
                log!(
                    info,
                    "copy_trading_executor",
                    "select_target_user",
                    "尝试从数据库加载可跟单地址列表..."
                );
                return select_from_database().await;
            }
            "custom" => {
                  log!(
                    info,
                    "copy_trading_executor",
                    "select_target_user",
                    "请输入要跟单的目标地址:"
                );
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let mut target_address = input.trim().to_string();
                
                // 验证地址格式
                if !is_valid_eth_address(&target_address) {
                    log!(
                        info,
                        "copy_trading_executor",
                        "select_target_user",
                        "地址格式无效: {}",
                        "target_address" => target_address
                    );
                    log!(
                        info,
                        "copy_trading_executor",
                        "select_target_user",
                        "请输入有效的以太坊地址 (42字符，以0x开头):"
                    );
                    let mut input2 = String::new();
                    io::stdin().read_line(&mut input2)?;
                    target_address = input2.trim().to_string();
                }
                
                return Ok(H160::from_str(&target_address)?);
            }
            _ => {
                log!(
                    info,
                    "copy_trading_executor",
                    "select_target_user",
                    "无效选择，使用默认地址"
                );
                return Ok(config.target_user);
            }
        }
    }
    
    // 如果配置文件不存在或读取失败，使用默认地址
    log!(
        info,
        "copy_trading_executor",
        "select_target_user",
        "配置文件读取失败，使用默认地址"
    );
    let default_address = H160::from([0x5e, 0x32, 0xd5, 0x15, 0x77, 0x96, 0xd9, 0x60, 0xed, 0xb6, 0x11, 0xd5, 0x23, 0xb9, 0x05, 0x1a, 0xc9, 0x88, 0x83, 0x52]);
    log!(
        info,
        "copy_trading_executor",
        "select_target_user",
        "使用默认跟单地址: {:?}",
        "default_address" => format!("{:?}", default_address)
    );
    Ok(default_address)
}

/// 从数据库选择跟单地址
async fn select_from_database() -> Result<H160> {
    // 使用工具函数初始化数据库
    let database = match init_database().await {
        Ok(db) => db,
        Err(e) => {
            log!(
                error,
                "copy_trading_executor",
                "select_from_database",
                "数据库连接失败 {}",
                "error" => format!("{}", e)
            );
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "请检查配置文件中的数据库配置或数据库服务"
            );
                    log!(
                        info,
                        "copy_trading_executor",
                        "select_from_database",
                        "请输入要跟单的目标地址:"
                    );
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let mut target_address = input.trim().to_string();
        
        // 验证地址格式
        if !is_valid_eth_address(&target_address) {
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "地址格式无效: {}",
                "target_address" => target_address
            );
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "请输入有效的以太坊地址 (42字符，以0x开头):"
            );
            let mut input2 = String::new();
            io::stdin().read_line(&mut input2)?;
            target_address = input2.trim().to_string();
        }
            
            return Ok(H160::from_str(&target_address)?);
        }
    };
    
    // 获取所有历史盈利数据
    let history_profits = match database.get_all_user_history_profits().await {
        Ok(profits) => {
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "成功获取历史盈利数据，共 {} 条记录",
                "count" => profits.len()
            );
            profits
        },
        Err(e) => {
            error!(
                "copy_trading_executor",
                "select_from_database",
                "获取历史盈利数据失败: {}",
                format!("{}", e)
            );
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "请输入要跟单的目标地址:"
            );
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let mut target_address = input.trim().to_string();
            
            // 验证地址格式
            if !is_valid_eth_address(&target_address) {
                log!(
                    info,
                    "copy_trading_executor",
                    "select_from_database",
                    "地址格式无效: {}",
                    "target_address" => target_address
                );
                log!(
                    info,
                    "copy_trading_executor",
                    "select_from_database",
                    "请输入有效的以太坊地址 (42字符，以0x开头):"
                );
                let mut input2 = String::new();
                io::stdin().read_line(&mut input2)?;
                target_address = input2.trim().to_string();
            }
            
            return Ok(H160::from_str(&target_address)?);
        }
    };
    
    if history_profits.is_empty() {
        log!(
            info,
            "copy_trading_executor",
            "select_from_database",
            "数据库中没有找到历史盈利数据"
        );
        log!(
            info,
            "copy_trading_executor",
            "select_from_database",
            "请输入要跟单的目标地址:"
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let mut target_address = input.trim().to_string();
        
        // 验证地址格式
        if !is_valid_eth_address(&target_address) {
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "地址格式无效: {}",
                "target_address" => target_address
            );
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "请输入有效的以太坊地址 (42字符，以0x开头):"
            );
            let mut input2 = String::new();
            io::stdin().read_line(&mut input2)?;
            target_address = input2.trim().to_string();
        }
        
        return Ok(H160::from_str(&target_address)?);
    }
    
    // 按地址分组，计算每个地址的总盈亏
    let mut address_profits: std::collections::HashMap<String, (f64, f64, f64, f64, f64)> = std::collections::HashMap::new();
    
    log!(
        info,
        "copy_trading_executor",
        "select_from_database",
        "开始处理 {} 条历史盈利记录",
        "count" => history_profits.len()
    );
    
    for profit in &history_profits {
        let entry = address_profits.entry(profit.addr.clone()).or_insert((0.0, 0.0, 0.0, 0.0, 0.0));
        entry.0 += profit.pnl_12h.to_f64().unwrap_or(0.0);
        entry.1 += profit.pnl_24h.to_f64().unwrap_or(0.0);
        entry.2 += profit.pnl_3d.to_f64().unwrap_or(0.0);
        entry.3 += profit.pnl_7d.to_f64().unwrap_or(0.0);
        entry.4 += profit.pnl_30d.to_f64().unwrap_or(0.0);
    }
    
    log!(
        info,
        "copy_trading_executor",
        "select_from_database",
        "分组后共 {} 个不同地址",
        "count" => address_profits.len()
    );
    
    // 转换为向量并排序
    let mut address_list: Vec<(String, f64, f64, f64, f64, f64)> = address_profits
        .into_iter()
        .map(|(addr, (pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d))| {
            (addr, pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d)
        })
        .collect();
    
    // 30天盈利排序
    address_list.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));
    
    log!(
        info,
        "copy_trading_executor",
        "select_from_database",
        "可跟单地址列表 (来自 user_history_profit 表)"
    );
    log!(
        info,
        "copy_trading_executor",
        "select_from_database",
        "==============================================="
    );
    
    if address_list.is_empty() {
        crate::log!(warn,
            "copy_trading_executor",
            "select_from_database",
            "警告：地址列表为空"
        );
    }
    
    for (i, (addr, pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d)) in address_list.iter().enumerate() {
        let short_addr = if addr.len() > 10 {
            format!("{}...{}", &addr[..6], &addr[addr.len()-4..])
        } else {
            addr.clone()
        };
        
        println!("{}. {} - 12h: ${:.2} | 24h: ${:.2} | 3d: ${:.2} | 7d: ${:.2} | 30d: ${:.2}", 
                 i + 1, short_addr, pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d);
    }
    
    println!("\n请选择要跟单的地址序号 (或输入 'custom' 手动输入地址): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();
    
    if choice.to_lowercase() == "custom" {
        log!(
            info,
            "copy_trading_executor",
            "select_from_database",
            "请输入要跟单的目标地址:"
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let mut target_address = input.trim().to_string();
        
        // 验证地址格式
        if !is_valid_eth_address(&target_address) {
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "地址格式无效: {}",
                "target_address" => target_address
            );
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "请输入有效的以太坊地址 (42字符，以0x开头):"
            );
            let mut input2 = String::new();
            io::stdin().read_line(&mut input2)?;
            target_address = input2.trim().to_string();
        }
        
        return Ok(H160::from_str(&target_address)?);
    }
    
    // 验证输入是否为有效数字
    if choice.is_empty() {
        return Err(anyhow::anyhow!("请输入有效的选择"));
    }
    
    let index: usize = choice.parse()
        .map_err(|_| anyhow::anyhow!("请输入有效的数字序号"))?;
    
    if index > 0 && index <= address_list.len() {
        let selected_addr = &address_list[index - 1].0;
        log!(
            info,
            "copy_trading_executor",
            "select_from_database",
            "已选择跟单地址: {}",
            "selected_addr" => selected_addr
        );
        
        // 验证地址格式
        if !is_valid_eth_address(selected_addr) {
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "地址格式无效: {}",
                "selected_addr" => selected_addr
            );
            log!(
                info,
                "copy_trading_executor",
                "select_from_database",
                "请输入有效的以太坊地址 (42字符，以0x开头):"
            );
            let mut input2 = String::new();
            io::stdin().read_line(&mut input2)?;
            let mut target_address = input2.trim().to_string();
            
            // 验证地址格式
            if !is_valid_eth_address(&target_address) {
                log!(
                    info,
                    "copy_trading_executor",
                    "select_from_database",
                    "地址格式无效: {}",
                    "target_address" => target_address
                );
                log!(
                    info,
                    "copy_trading_executor",
                    "select_from_database",
                    "请输入有效的以太坊地址 (42字符，以0x开头):"
                );
                let mut input2 = String::new();
                io::stdin().read_line(&mut input2)?;
                target_address = input2.trim().to_string();
            }
            
            return Ok(H160::from_str(&target_address)?);
        }
        
        return Ok(H160::from_str(selected_addr)?);
    } else {
        return Err(anyhow::anyhow!("无效的序号选择"));
    }
}

/// 跟单监控主循环
async fn run_copy_trading_monitor(copy_service: &CopyTradingService) -> Result<()> {
    let mut current_delay_s = INITIAL_RECONNECT_DELAY_S;

    loop {
        log!(
            info,
            "copy_trading_executor",
            "run_copy_trading_monitor",
            "正在尝试连接 Hyperliquid WebSocket 并订阅用户 {} 的交易事件..",
            "target_user" => format!("{:?}", copy_service.config.target_user)
        );

        let connection_result = async {
            let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
            let (sender, mut receiver) = unbounded_channel();

            log!(
                info,
                "copy_trading_executor",
                "run_copy_trading_monitor",
                "订阅目标用户的交易事件.."
            );
            let target_user: Address = Address::from_slice(&copy_service.config.target_user.as_bytes());
            info_client
                .subscribe(Subscription::UserEvents { user: target_user }, sender.clone())
                .await?;

            log!(
                info,
                "copy_trading_executor",
                "run_copy_trading_monitor",
                "订阅目标用户的订单更新.."
            );
           
            info_client
                .subscribe(Subscription::OrderUpdates { user: target_user }, sender)
                .await?;

            log!(
                info,
                "copy_trading_executor",
                "run_copy_trading_monitor",
                "连接成功! 跟单系统已启动，正在监控目标地址的交易.."
            );
            current_delay_s = INITIAL_RECONNECT_DELAY_S;

            let mut ticker = tokio::time::interval(Duration::from_secs(30 * 60));
            loop {
                tokio::select! {
                    maybe_msg = receiver.recv() => {
                        match maybe_msg {
                            Some(message) => {
                                if let Err(e) = process_copy_trading_message(message, copy_service).await {
                                    error!(
                                        "copy_trading_executor",
                                        "run_copy_trading_monitor",
                                        "处理跟单消息时发生错误 {}",
                                        format!("{}", e)
                                    );
                                }
                            }
                            None => break,
                        }
                    }
                    _ = ticker.tick() => {
                        log!(
                            info,
                            "copy_trading_executor",
                            "run_copy_trading_monitor",
                            " 正在监控目标地址..."
                        );
                    }
                }
            }

            anyhow::Ok(())
        }
        .await;

        if let Err(e) = connection_result {
            error!(
                "copy_trading_executor",
                "run_copy_trading_monitor",
                "连接或订阅过程中发生错误: {}",
                format!("{}", e)
            );
        }

        error!(
            "copy_trading_executor",
            "run_copy_trading_monitor",
            "连接已断开！将在 {} 秒后尝试重连...",
            current_delay_s
        );
        sleep(Duration::from_secs(current_delay_s)).await;
        current_delay_s = (current_delay_s * 2).min(MAX_RECONNECT_DELAY_S);
    }
}

/// 处理跟单消息
async fn process_copy_trading_message(message: Message, copy_service: &CopyTradingService) -> Result<()> {
    match message {
        Message::User(user_event) => {
            match user_event.data {
                UserData::Fills(fills) => {
                    if !fills.is_empty() {
                        log!(
                            info,
                            "copy_trading_executor",
                            "process_copy_trading_message",
                            "[跟单检测] 目标用户发生交易! {} 笔交易",
                            "count" => fills.len()
                        );

                        // 合并相同代币和方向的交易，避免重复跟单
                        let mut merged_trades: std::collections::HashMap<String, (f64, f64, String, i64)> = std::collections::HashMap::new();
                        
                        for fill in &fills {
                            if let Ok(direction_info) = crate::utils::parse_trade_direction(&fill.dir) {
                                let key = format!("{}_{}", fill.coin, fill.dir);
                                
                                if let Some((total_amount, total_value, _, timestamp)) = merged_trades.get_mut(&key) {
                                    // 累加数量和价值
                                    let amount = safe_parse_f64(&fill.sz, 0.0);
                                    let price = safe_parse_f64(&fill.px, 0.0);
                                    *total_amount += amount;
                                    *total_value += amount * price;
                                } else {
                                    // 新的交易类型
                                    let amount = safe_parse_f64(&fill.sz, 0.0);
                                    let price = safe_parse_f64(&fill.px, 0.0);
                                    let timestamp = fill.time as i64;
                                    merged_trades.insert(key, (amount, amount * price, fill.dir.clone(), timestamp));
                                }
                            }
                        }

                        // 处理合并后的交易
                        for (key, (total_amount, total_value, dir, timestamp)) in merged_trades {
                            let parts: Vec<&str> = key.split('_').collect();
                            if parts.len() != 2 {
                                continue;
                            }
                            
                            let coin = parts[0];
                            let direction = parts[1];
                            
                            if let Ok(direction_info) = crate::utils::parse_trade_direction(direction) {
                                let is_close_operation = direction_info.reduce_only;
                                
                                if is_close_operation {
                                    // 平仓操作：检查是否应该跟随平仓
                                    if copy_service.should_follow_target_close(coin, &copy_service.config.wallet).await {
                                        log!(
                                            info,
                                            "copy_trading_executor",
                                            "process_copy_trading_message",
                                            "[跟随平仓] 目标用户平仓 {}, 本地跟随平仓 {}",
                                            "coin" => coin,
                                            "coin" => coin
                                        );
                                        
                                        // 创建平仓信号
                                        let signal = TradeSignal {
                                            wallet_address: copy_service.config.target_user,
                                            coin: coin.to_string(),
                                            action: direction_info.action,
                                            amount: total_amount,
                                            price: total_value / total_amount, // 
                                            timestamp,
                                            original_direction: Some(direction.to_string()),
                                        };

                                        // 执行跟随平仓
                                        if let Err(e) = copy_service.process_trade_signal(&signal).await {
                                            error!(
                                                "copy_trading_executor",
                                                "process_copy_trading_message",
                                                "跟随平仓失败: {}",
                                                format!("{}", e)
                                            );
                                        }
                                    } else {
                                        log!(
                                            info,
                                            "copy_trading_executor",
                                            "process_copy_trading_message",
                                            "[跳过跟随平仓] {} 已被止盈止损平仓",
                                            "coin" => coin
                                        );
                                    }
                                } else {
                                    // 开仓操作：正常跟单
                                    let signal = TradeSignal {
                                        wallet_address: copy_service.config.target_user,
                                        coin: coin.to_string(),
                                        action: direction_info.action,
                                        amount: total_amount,
                                        price: total_value / total_amount, // 加权平均价格
                                        timestamp,
                                        original_direction: Some(direction.to_string()),
                                    };

                                    // 使用策略服务处理交易信号
                                    if let Err(e) = copy_service.process_trade_signal(&signal).await {
                                        error!(
                                            "copy_trading_executor",
                                            "process_copy_trading_message",
                                            "跟单执行失败: {}",
                                            format!("{}", e)
                                        );
                                    }
                                }
                            } else {
                                log!(
                                    info,
                                    "copy_trading_executor",
                                    "process_copy_trading_message",
                                    "无法解析交易动作: {}",
                                    "fill_dir" => direction
                                );
                            }
                        }
                    }
                }
                _ => {
                    // 其他用户事件暂不处理
                }
            }
        }
        _ => {
            // 其他类型消息暂不处理
        }
    }
    Ok(())
} 
