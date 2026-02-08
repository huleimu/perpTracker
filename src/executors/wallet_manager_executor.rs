use anyhow::Result;

use crate::database::Database;
use crate::utils::database_init::init_database;
use crate::utils::prompt_user;
use tracing::{trace, info, warn};

use crate::log;
use crate::error;

/// 钱包管理执行器（只做管理，不再负责监控服务启动/停止）
pub async fn run_wallet_manager_executor() -> Result<()> {
    trace!("启动动态钱包管理系统");
    trace!("====================");

    // 初始化数据库
    let db = match init_database().await {
        Ok(db) => db,
        Err(_) => return Ok(()),
    };
    
    // 主循环：处理用户输入
    loop {
        show_menu(&db).await?;
        
        let choice = prompt_user("请输入选择 (1-7): ")?;

        match choice.as_str() {
            "1" => {
                trace!("添加新钱包");
                add_wallet(&db).await?;
            }
            "2" => {
                trace!("移除钱包");
                remove_wallet(&db).await?;
            }
            "4" => {
                trace!("从文件导入钱包");
                import_wallets_from_file(&db).await?;
            }
            "5" => {
                trace!("查看所有钱包地址");
                show_all_wallet_addresses(&db).await?;
            }
            "6" => {
                trace!("退出程序");
                break;
            }
            "7" => {
                trace!("移除所有钱包");
                remove_all_wallets(&db).await?;
            }
            _ => {
                trace!("无效选择，请重新输入");
            }
        }
    }

    trace!("钱包管理程序已退出");
    Ok(())
}

/// 显示主菜单
async fn show_menu(db: &Database) -> Result<()> {
    let monitored_count = db.get_active_wallets().await?.len();
    info!("钱包管理系统");
    info!("=====================");
    info!("钱包统计: {}个钱包", monitored_count);
    info!("=====================");
    info!("1. 添加新钱包");
    info!("2. 移除钱包");
    info!("4. 从文件导入钱包");
    info!("5. 查看所有钱包地址");
    info!("6. 退出程序");
    info!("7. 移除所有钱包");
    info!("=====================");
    info!("");
    Ok(())
}

/// 添加新钱包
async fn add_wallet(db: &Database) -> Result<()> {
    let address = prompt_user("请输入钱包地址: ")?.trim().to_string();

    if address.is_empty() {
        warn!(
            service = "perpTracker",
            module = "wallet_manager_executor",
            function = "add_wallet",
            file = file!(),
            line = line!(),
            message = "地址不能为空",
            input = "empty"
        );
        return Ok(());
    }

    match db.add_wallet(&address).await {
        Ok(()) => {
            log!(
                info,
                "wallet_manager_executor",
                "add_wallet",
                "添加钱包成功",
                "address" => address
            );
        }
        Err(e) => {
            error!(
                "wallet_manager_executor",
                "add_wallet",
                "添加钱包失败",
                e,
                "address" => address
            );
        }
    }

    Ok(())
}

/// 移除钱包
async fn remove_wallet(db: &Database) -> Result<()> {
            let address = prompt_user("请输入要移除的钱包地址: ")?.trim().to_string();

    if address.is_empty() {
        warn!("地址不能为空");
        return Ok(());
    }

    match db.remove_wallet(&address).await {
        Ok(()) => {
            log!(
                info,
                "wallet_manager_executor",
                "remove_wallet",
                "移除钱包成功",
                "address" => address
            );
        }
        Err(e) => {
            error!(
                "wallet_manager_executor",
                "remove_wallet",
                "移除钱包失败",
                e,
                "address" => address
            );
        }
    }

    Ok(())
}

/// 从文件导入钱包
async fn import_wallets_from_file(db: &Database) -> Result<()> {
    let file_path = prompt_user("请输入钱包文件路径(默认: addresses.txt): ")?.trim().to_string();
    
    let file_path = if file_path.is_empty() { "addresses.txt" } else { &file_path };

    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            let mut success_count = 0;
            let error_count = 0;

            for (line_num, line) in content.lines().enumerate() {
                let line = line.trim();
                
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // 解析地址
                let address = if line.contains('|') {
                    line.split('|').next().unwrap_or("").trim()
                } else {
                    line
                };

                if address.is_empty() {
                    continue;
                }

                match db.add_wallet(address).await {
                    Ok(()) => {
                        success_count += 1;
                        info!("操作成功 - 导入钱包: 第{}行 {}", line_num + 1, address);
                    }
                    Err(e) => {
                        error!(
                        "wallet_manager_executor",
                        "import_wallets_from_file",
                        "导入钱包失败",
                        e,
                        "line_number" => line_num + 1,
                        "address" => address,
                        "file_path" => file_path
                    );
                    }
                }
            }

            info!("导入完成: 成功 {} 个, 失败 {} 个", success_count, error_count);
        }
        Err(e) => {
            error!(
                "wallet_manager_executor",
                "import_wallets_from_file",
                "读取文件失败",
                e,
                "file_path" => file_path
            );
        }
    }

    Ok(())
}

/// 显示所有钱包地址
async fn show_all_wallet_addresses(db: &Database) -> Result<()> {
    let addresses = db.get_active_wallets().await?;
    info!("=====================");
            info!("当前所有钱包地址:");
    for (i, addr) in addresses.iter().enumerate() {
        info!("{}. {}", i + 1, addr);
    }
    info!("=====================");
    Ok(())
} 

/// 移除所有钱包
async fn remove_all_wallets(db: &Database) -> Result<()> {
    info!("用户确认提示: 确定要移除所有钱包吗？此操作不可恢复(y/N): ");
    print!("确定要移除所有钱包吗？此操作不可恢复(y/N): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        let confirmed = input == "y" || input == "yes";
        info!("用户确认结果: {}", if confirmed { "是" } else { "否" });
        
        if !confirmed {
        warn!(
            service = "perpTracker",
            module = "wallet_manager_executor",
            function = "remove_all_wallets",
            file = file!(),
            line = line!(),
            message = "取消批量移除操作",
            user_choice = "no"
        );
        return Ok(());
    }
    match db.clear_all_wallets().await {
        Ok(()) => {
            log!(
                info,
                "wallet_manager_executor",
                "remove_all_wallets",
                "批量移除钱包成功",
                "operation" => "clear_all"
            );
        },
        Err(e) => {
            error!(
                "wallet_manager_executor",
                "remove_all_wallets",
                "批量移除钱包失败",
                e,
                "operation" => "clear_all"
            );
        },
    }
    Ok(())
} 
