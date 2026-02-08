use anyhow::Result;
use crate::collectors::price_collector::PriceCollector;
use crate::log;
use std::fs;
use toml;

pub async fn run_price_collector_executor() -> Result<()> {
    log!(info, "price_collector_executor", "run_price_collector_executor", 
        "启动价格收集服务"
    );
    
    // 直接读取配置文件，获取币种列表
    let config_content = fs::read_to_string("config/config.toml")?;
    let config: toml::Value = toml::from_str(&config_content)?;
    
    let enabled_assets = config["copy_trading"]["enabled_assets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("配置文件中缺少 copy_trading.enabled_assets"))?
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<String>>();
    
    log!(info, "price_collector_executor", "run_price_collector_executor", 
        "加载币种列表", "assets" => format!("{:?}", enabled_assets));
    
    // 创建价格收集器
    let mut collector = PriceCollector::new(enabled_assets).await?;
    
    // 启动收集服务
    collector.start().await?;
    
    Ok(())
} 