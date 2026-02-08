use super::{Database, UserProfit, UserHistoryProfit, TradeEvent};
use anyhow::Result;
use tracing::trace;

//save_user_profit   save_trade_event get_trade_events
impl Database {
//   pub async fn save_trade_event(&self, trade_event: &TradeEvent) -> Result<()> {
  
//     let sql = r#"
//         INSERT INTO trade_events (
//             addr, coin, action, direction, size, price, value, 
//             closed_pnl, trade_type, order_id, trade_time
//         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
//     "#;

//     sqlx::query(sql)
//       .bind(&trade_event.addr)
//       .bind(&trade_event.coin)
//       .bind(&trade_event.action)
//       .bind(&trade_event.direction)
//         .bind(trade_event.size)
//         .bind(trade_event.price)
//         .bind(trade_event.value)
//         .bind(trade_event.closed_pnl)
//       .bind(&trade_event.trade_type)
//       .bind(&trade_event.order_id)
//         .bind(trade_event.trade_time)
//       .execute(self.pool())
//       .await?;

//     // crate::log!(debug, "repositories", "save_trade_event", 
//     //     "交易事件已保存", 
//     //     "address" => trade_event.addr,
//     //     "coin" => trade_event.coin,
//     //     "action" => trade_event.action,
//     //     "value" => format!("${:.2}", trade_event.value)
//     // );
//     Ok(())  
//   }

  /// 批量保存交易事件
  pub async fn save_trade_events_batch(&self, trade_events: &[TradeEvent]) -> Result<()> {
    use sqlx::QueryBuilder;

    if trade_events.is_empty() {
      return Ok(());
    }

    // 分片写入，避免单次 SQL 过大
    let chunk_size: usize = 100;
    for chunk in trade_events.chunks(chunk_size) {
      let mut builder = QueryBuilder::<sqlx::MySql>::new(
        "INSERT INTO trade_events (`addr`, `coin`, `action`, `direction`, `size`, `price`, `value`, `closed_pnl`, `trade_type`, `order_id`, `trade_time`) "
      );

      builder.push_values(chunk, |mut b, ev| {
        b.push_bind(&ev.addr)
          .push_bind(&ev.coin)
          .push_bind(&ev.action)
          .push_bind(&ev.direction)
          .push_bind(ev.size)
          .push_bind(ev.price)
          .push_bind(ev.value)
          .push_bind(ev.closed_pnl)
          .push_bind(&ev.trade_type)
          .push_bind(&ev.order_id)
          .push_bind(ev.trade_time);
      });

      let query = builder.build();
      query.execute(self.pool()).await?;
    }

    // crate::log!(debug, "repositories", "save_trade_events_batch", 
    //     "批量交易事件已保存", 
    //     "count" => trade_events.len()
    // );

    Ok(())
  }

  pub async fn save_user_profit(&self, user_profit: &UserProfit) -> Result<()> {

    let sql = r#"
      INSERT INTO user_profit (addr, coin, action, position_size,position_value, entry_price, liquidation_price, total_pnl, realized_pnl, unrealized_pnl, roe, start_time, update_time)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON DUPLICATE KEY UPDATE
        action=VALUES(action),
        position_size=VALUES(position_size),
        position_value=VALUES(position_value),
        entry_price=VALUES(entry_price),
        liquidation_price=VALUES(liquidation_price),
        total_pnl=VALUES(total_pnl),
        realized_pnl=VALUES(realized_pnl),
        unrealized_pnl=VALUES(unrealized_pnl),
        roe=VALUES(roe),
        update_time=CURRENT_TIMESTAMP
    "#;

    sqlx::query(sql)
      .bind(&user_profit.addr)
      .bind(&user_profit.coin)  
      .bind(&user_profit.action)
      .bind(&user_profit.position_size)
      .bind(&user_profit.position_value)
      .bind(&user_profit.entry_price)

      .bind(&user_profit.liquidation_price)
      .bind(&user_profit.total_pnl)
      .bind(&user_profit.realized_pnl)
      .bind(&user_profit.unrealized_pnl)
      .bind(&user_profit.roe)
      .bind(&user_profit.start_time)
      .bind(&user_profit.update_time)
      .execute(self.pool()) 
      .await?;

    Ok(())
  } 

  pub async fn save_user_history_profit(&self,user_history_profit: &UserHistoryProfit) -> Result<()> {
    let sql = r#"
      INSERT INTO user_history_profit (addr, coin, pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d, record_time)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      ON DUPLICATE KEY UPDATE
        pnl_12h=VALUES(pnl_12h),
        pnl_24h=VALUES(pnl_24h),
        pnl_3d=VALUES(pnl_3d),
        pnl_7d=VALUES(pnl_7d),
        pnl_30d=VALUES(pnl_30d),
        record_time=VALUES(record_time)
    "#;

    sqlx::query(sql)
      .bind(&user_history_profit.addr)
      .bind(&user_history_profit.coin)
      .bind(&user_history_profit.pnl_12h)
      .bind(&user_history_profit.pnl_24h)
      .bind(&user_history_profit.pnl_3d)
      .bind(&user_history_profit.pnl_7d)
      .bind(&user_history_profit.pnl_30d)
      .bind(&user_history_profit.record_time)
      .execute(self.pool())
      .await?;

      Ok(())
  }

  // 获取所有 user_history_profit 数据
  pub async fn get_all_user_history_profits(&self) -> Result<Vec<UserHistoryProfit>> {
    let sql = r#"
      SELECT addr, coin, pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d, record_time
      FROM user_history_profit
      ORDER BY record_time DESC
    "#;

    let rows = sqlx::query_as::<_, UserHistoryProfit>(sql)
      .fetch_all(self.pool())
      .await?;

    Ok(rows)
  }

  // 获取所有不重复的地址
  pub async fn get_all_unique_addresses(&self) -> Result<Vec<String>> {
    let sql = r#"
      SELECT DISTINCT addr
      FROM user_history_profit
      ORDER BY addr
    "#;

    let rows = sqlx::query_scalar::<_, String>(sql)
      .fetch_all(self.pool())
      .await?;

    Ok(rows)
  }

  // 根据地址获取该地址的所有历史盈利数据
  pub async fn get_user_history_profits_by_addr(&self, addr: &str) -> Result<Vec<UserHistoryProfit>> {
    let sql = r#"
      SELECT addr, coin, pnl_12h, pnl_24h, pnl_3d, pnl_7d, pnl_30d, record_time
      FROM user_history_profit
      WHERE addr = ?
      ORDER BY record_time DESC
    "#;

    let rows = sqlx::query_as::<_, UserHistoryProfit>(sql)
      .bind(addr)
      .fetch_all(self.pool())
      .await?;

    Ok(rows)
  }

  pub async fn get_user_history_profits_from_trade_events(&self, addr: &str) -> Result<Vec<UserHistoryProfit>> {

    let sql = r#"
        SELECT
            addr,
            coin,
            CAST(SUM(CASE WHEN trade_time >= NOW() - INTERVAL 12 HOUR THEN closed_pnl ELSE 0 END) AS DECIMAL(20,8)) AS pnl_12h,
            CAST(SUM(CASE WHEN trade_time >= NOW() - INTERVAL 24 HOUR THEN closed_pnl ELSE 0 END) AS DECIMAL(20,8)) AS pnl_24h,
            CAST(SUM(CASE WHEN trade_time >= NOW() - INTERVAL 3 DAY THEN closed_pnl ELSE 0 END) AS DECIMAL(20,8)) AS pnl_3d,
            CAST(SUM(CASE WHEN trade_time >= NOW() - INTERVAL 7 DAY THEN closed_pnl ELSE 0 END) AS DECIMAL(20,8)) AS pnl_7d,
            CAST(SUM(CASE WHEN trade_time >= NOW() - INTERVAL 30 DAY THEN closed_pnl ELSE 0 END) AS DECIMAL(20,8)) AS pnl_30d,
            NOW() as record_time
        FROM trade_events
        WHERE addr = ?
        AND closed_pnl != 0
        GROUP BY addr, coin
        
    "#;

    let result = sqlx::query_as::<_, UserHistoryProfit>(sql)
        .bind(addr)
        .fetch_all(self.pool())
        .await?;

    Ok(result)
}

  /// 添加新钱包到数据库
  pub async fn add_wallet(&self, address: &str) -> Result<()> {
      let sql = r#"
          INSERT INTO wallet_addresses (addr) 
          VALUES (?)
          ON DUPLICATE KEY UPDATE 
              updated_at = CURRENT_TIMESTAMP
      "#;

      sqlx::query(sql)
          .bind(address)
          .execute(self.pool())
          .await?;

      trace!("成功添加/更新钱包: {}", address);
      Ok(())
  }

  /// 从数据库移除钱包
  pub async fn remove_wallet(&self, address: &str) -> Result<()> {
      // 开始事务
      let mut transaction = self.pool().begin().await?;
      
      // 1. 从钱包地址表删除
      let sql = r#"
          DELETE FROM wallet_addresses 
          WHERE addr = ?
      "#;

      sqlx::query(sql)
          .bind(address)
          .execute(&mut *transaction)
          .await?;

      // 2. 清理用户盈亏数据
      let sql_cleanup_profit = r#"
          DELETE FROM user_profit 
          WHERE addr = ?
      "#;

      sqlx::query(sql_cleanup_profit)
          .bind(address)
          .execute(&mut *transaction)
          .await?;

      // 3. 清理用户历史盈亏数据
      let sql_cleanup_history = r#"
          DELETE FROM user_history_profit 
          WHERE addr = ?
      "#;

      sqlx::query(sql_cleanup_history)
          .bind(address)
          .execute(&mut *transaction)
          .await?;

      // 4. 清理交易事件数据（可选，取决于是否要保留历史交易记录）
      let sql_cleanup_trades = r#"
          DELETE FROM trade_events 
          WHERE addr = ?
      "#;

      sqlx::query(sql_cleanup_trades)
          .bind(address)
          .execute(&mut *transaction)
          .await?;

      // 提交事务
      transaction.commit().await?;

      trace!("成功移除钱包及相关数据 {}", address);
      Ok(())
  }

  /// 获取所有活跃钱包地址
  pub async fn get_active_wallets(&self) -> Result<Vec<String>> {
      let sql = r#"
          SELECT addr 
          FROM wallet_addresses 
          ORDER BY created_at DESC
      "#;

      let rows = sqlx::query_as::<_, (String,)>(sql)
          .fetch_all(self.pool())
          .await?;

      let addresses: Vec<String> = rows.into_iter().map(|(addr,)| addr).collect();
      Ok(addresses)
  }

  /// 检查钱包是否存在
  pub async fn wallet_exists(&self, address: &str) -> Result<bool> {
      let sql = r#"
          SELECT COUNT(*) as count 
          FROM wallet_addresses 
          WHERE addr = ?
      "#;

      let count: i64 = sqlx::query_scalar(sql)
          .bind(address)
          .fetch_one(self.pool())
          .await?;
      
      Ok(count > 0)
  }

  /// 清空所有钱包及相关数据
  pub async fn clear_all_wallets(&self) -> Result<()> {
      let mut transaction = self.pool().begin().await?;
      // 1. 清空钱包地址
      let sql_wallets = "DELETE FROM wallet_addresses";
      sqlx::query(sql_wallets)
          .execute(&mut *transaction)
          .await?;
      // 2. 清空用户盈亏
      let sql_profit = "DELETE FROM user_profit";
      sqlx::query(sql_profit)
          .execute(&mut *transaction)
          .await?;
      // 3. 清空用户历史盈亏
      let sql_history = "DELETE FROM user_history_profit";
      sqlx::query(sql_history)
          .execute(&mut *transaction)
          .await?;
      // 4. 清空交易事件
      let sql_trades = "DELETE FROM trade_events";
      sqlx::query(sql_trades)
          .execute(&mut *transaction)
          .await?;
      // 5. 清空 HL 价格表
      let sql_prices = "DELETE FROM hl_prices";
      sqlx::query(sql_prices)
          .execute(&mut *transaction)
          .await?;
      transaction.commit().await?;
      trace!("已清空所有钱包及相关数据");
      Ok(())
  }

  /// 获取 trade_events 表中所有唯一钱包地址
  pub async fn get_all_trade_event_wallets(&self) -> Result<Vec<String>> {
      let sql = r#"
          SELECT DISTINCT addr FROM trade_events
      "#;
      let rows = sqlx::query_as::<_, (String,)>(sql)
          .fetch_all(self.pool())
          .await?;
      Ok(rows.into_iter().map(|(addr,)| addr).collect())
  }

  /// 删除 trade_events 表中指定地址的所有数据
  pub async fn delete_trade_events_by_addr(&self, address: &str) -> Result<()> {
      let sql = r#"
          DELETE FROM trade_events WHERE addr = ?
      "#;
      sqlx::query(sql)
          .bind(address)
          .execute(self.pool())
          .await?;
      Ok(())
  }

  /// 保存 HL 价格数据
  pub async fn save_hl_prices(&self, coin: &str, bid: f64, ask: f64, index_price: Option<f64>) -> Result<()> {
      let now = chrono::Utc::now();
      
      let sql = r#"
          INSERT INTO hl_prices (ts, coin, best_bid, best_ask, index_price)
          VALUES (?, ?, ?, ?, ?)
          ON DUPLICATE KEY UPDATE
          best_bid = VALUES(best_bid),
          best_ask = VALUES(best_ask),
          index_price = VALUES(index_price)
      "#;
      
      sqlx::query(sql)
          .bind(now)
          .bind(coin)
          .bind(bid)
          .bind(ask)
          .bind(index_price)
          .execute(self.pool())
          .await?;
      
      Ok(())
  }


}
/// 重置数据库表
pub async fn reset_database_tables() -> Result<()> {
    trace!("警告：这将删除所有数据库表和数据");
    trace!("自动确认重置数据..");
  
    // 使用工具函数初始化数据库
    let db = match crate::utils::database_init::init_database().await {
        Ok(db) => db,
        Err(e) => {
            trace!("数据库初始化失败: {}", e);
            trace!("请检查配置文件中的数据库配置");
            return Ok(());
        }
    };
    
    trace!("正在删除旧表...");
  
    // 删除表（忽略错误，因为表可能不存在）
    let _ = sqlx::query("DROP TABLE IF EXISTS trade_events")
        .execute(db.pool())
        .await;
  
    let _ = sqlx::query("DROP TABLE IF EXISTS user_history_profit")
        .execute(db.pool())
        .await;
        
    let _ = sqlx::query("DROP TABLE IF EXISTS user_profit")
        .execute(db.pool())
        .await;
  
    let _ = sqlx::query("DROP TABLE IF EXISTS wallet_addresses")
        .execute(db.pool())
        .await;
  
    trace!("正在重新创建表..");
  
    // 重新创建表
    if let Err(e) = db.create_tables().await {
        trace!("重新创建表失败: {}", e);
        return Ok(());
    }
  
    trace!("数据库表重置完成");

    Ok(())
}

