use anyhow::Result;
use crate::database::Database;
use crate::types::ConfigFile;

/// 初始化数据库连接
/// - 创建表结构
pub async fn init_database() -> Result<Database> {
    // 直接从配置文件读取数据库URL
    let content = std::fs::read_to_string("config/config.toml")
        .map_err(|_| anyhow::anyhow!("无法读取配置文件"))?;
    
    let config: ConfigFile = toml::from_str(&content)
        .map_err(|_| anyhow::anyhow!("配置文件格式错误"))?;
    
    let database_url = config.database_url;
    
    crate::log!(info, "database_init", "init_database", 
        "使用数据库URL", 
        "url" => database_url
    );

    let db = Database::new(&database_url).await
        .map_err(|e| {
            crate::log!(error, "database_init", "init_database", 
                "数据库初始化失败", 
                "error" => format!("{}", e)
            );
            anyhow::anyhow!("数据库初始化失败: {}", e)
        })?;

    // 测试连接
    db.test_connection().await.map_err(|e| {
        crate::log!(error, "database_init", "init_database", 
            "数据库连接失败", 
            "error" => format!("{}", e)
        );
        anyhow::anyhow!("数据库连接失败: {}", e)
    })?;
    
    // 创建表结构
    db.create_tables().await.map_err(|e| {
        crate::log!(error, "database_init", "init_database", 
            "创建数据库表失败", 
            "error" => format!("{}", e)
        );
        anyhow::anyhow!("创建数据库表失败: {}", e)
    })?;
    
    crate::log!(info, "database_init", "init_database", 
        "数据库初始化完成"
    );

    Ok(db)
} 
