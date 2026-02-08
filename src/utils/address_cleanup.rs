use crate::database::Database;
use std::collections::HashSet;

/// 清理 trade_events 表中所有已被移除的钱包地址的数
pub async fn cleanup_orphan_trade_events(database: &Database) {
    let db_addresses: HashSet<String> = match database.get_active_wallets().await {
        Ok(addrs) => {
            // 统一转换为小写格式
            addrs.into_iter()
                .map(|addr| addr.to_lowercase())
                .collect()
        },
        Err(_) => return,
    };
    
    let trade_event_addresses: HashSet<String> = match database.get_all_trade_event_wallets().await {
        Ok(addrs) => {
            // 统一转换为小写格式
            addrs.into_iter()
                .map(|addr| addr.to_lowercase())
                .collect()
        },
        Err(_) => return,
    };
    
    for addr in trade_event_addresses.difference(&db_addresses) {
        if let Err(e) = database.delete_trade_events_by_addr(addr).await {
            crate::error!("address_cleanup", "cleanup_orphan_trade_events", 
                "清理 trade_events 过期地址失败", e, 
                "address" => addr
            );
        } else {
            crate::log!(debug, "address_cleanup", "cleanup_orphan_trade_events", 
                "已自动清理 trade_events 中已移除的钱包地址", 
                "address" => addr
            );
        }
    }
} 
