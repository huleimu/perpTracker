use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use anyhow::Result;


///数据库连接器
/// 
#[derive(Clone)]
pub struct Database {
    pool: MySqlPool,
}


impl Database {

    ///创建数据库连接
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(200)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    //获取数据库连接引用
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }


    /// 测试数据库连接
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        crate::log!(info, "connection", "test_connection", 
            "数据库连接测试成功"
        );
        Ok(())
    }      
    /// 创建数据库表
    pub async fn create_tables(&self) -> Result<()> {
        // 创建用户盈亏表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_profit (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                addr VARCHAR(42) NOT NULL,
                coin VARCHAR(20) NOT NULL,
                action VARCHAR(10),
                position_size DECIMAL(28, 18) NOT NULL,
                position_value DECIMAL(28, 18) NOT NULL,
                entry_price DECIMAL(18, 8),

                liquidation_price DECIMAL(18, 8),
                total_pnl DECIMAL(28,18),
                realized_pnl DECIMAL(28,18),
                unrealized_pnl DECIMAL(28,18),
                roe DECIMAL(28,18),
                start_time TIMESTAMP NOT NULL,
                update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                UNIQUE KEY unique_addr_coin (addr, coin)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#
        )
        .execute(&self.pool)
        .await?;
        crate::log!(debug, "connection", "create_tables", 
            "user_profit 表已创建/验证成功"
        );

        // 创建历史盈亏表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_history_profit (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                addr VARCHAR(42) NOT NULL,
                coin VARCHAR(20) NOT NULL,
                pnl_12h DECIMAL(18, 8),
                pnl_24h DECIMAL(18, 8),
                pnl_3d DECIMAL(18, 8),
                pnl_7d DECIMAL(18, 8),
                pnl_30d DECIMAL(18, 8),
                record_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE KEY unique_addr_coin (addr, coin),
                INDEX idx_addr_coin_time (addr, coin, record_time)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#
        )
        .execute(&self.pool)
        .await?;
        crate::log!(debug, "connection", "create_tables", 
            "user_history_profit 表已创建/验证成功"
        );
        
        // 创建交易事件表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trade_events (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                addr VARCHAR(42) NOT NULL,
                coin VARCHAR(20) NOT NULL,
                action VARCHAR(20) NOT NULL,
                direction VARCHAR(10) NOT NULL,
                size DOUBLE NOT NULL,
                price DOUBLE NOT NULL,
                value DOUBLE NOT NULL,
                closed_pnl DOUBLE NOT NULL,
                trade_type VARCHAR(20) NOT NULL,
                order_id VARCHAR(100) NOT NULL,
                trade_time TIMESTAMP NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                INDEX idx_addr_time (addr, trade_time),
                INDEX idx_coin_time (coin, trade_time),
                INDEX idx_order_id (order_id)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#
        )
        .execute(&self.pool)
        .await?;
        crate::log!(debug, "connection", "create_tables", 
            "trade_events 表已创建/验证成功"
        );
        crate::log!(debug, "connection", "create_tables", 
            "数据库表创建成功"
        );


        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_addresses (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                addr VARCHAR(42) NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                INDEX idx_addr (addr)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#
        )
        .execute(&self.pool)
        .await?;
        crate::log!(debug, "connection", "create_tables", 
            "wallet_addresses 表已创建/验证成功"
        );

        // 创建 HL 价格表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hl_prices (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                ts DATETIME(3) NOT NULL,            -- 时间戳（毫秒精度）
                coin VARCHAR(64) NOT NULL,          -- 币种名称
                index_price DECIMAL(32,12) NULL,    -- 指数价
                best_bid DECIMAL(32,12) NOT NULL,   -- 买一价
                best_ask DECIMAL(32,12) NOT NULL,   -- 卖一价
                UNIQUE KEY uniq_coin_ts (coin, ts), -- 防止重复数据
                KEY idx_ts_coin (ts, coin)          -- 时间查询索引
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#
        )
        .execute(&self.pool)
        .await?;
        crate::log!(debug, "connection", "create_tables", 
            "hl_prices 表已创建/验证成功"
        );

        Ok(())
    }

}




