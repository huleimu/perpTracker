use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::interval;
use anyhow::Result;
use crate::database::Database;
use crate::types::PriceData;
use crate::utils::{safe_parse_f64, database_init::init_database};
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};

// 配置常量
const NETWORK: BaseUrl = BaseUrl::Mainnet;
const WRITE_INTERVAL_MS: u64 = 1000;  // 写库间隔（毫秒）
const HEARTBEAT_INTERVAL_MS: u64 = 900000;  // 心跳日志间隔（毫秒）

pub struct PriceCollector {
    database: Database,
    info_client: InfoClient,
    enabled_assets: Vec<String>,  // 要收集的币种列表
    price_cache: HashMap<String, PriceData>,
    sender: UnboundedSender<Message>,
    receiver: UnboundedReceiver<Message>,
}

impl PriceCollector {
    pub async fn new(enabled_assets: Vec<String>) -> Result<Self> {
        // 初始化数据库
        let database = init_database().await?;
        
        // 创建 HL Info 客户端
        let info_client = InfoClient::new(None, Some(NETWORK)).await?;
        
        // 创建消息通道
        let (sender, receiver) = unbounded_channel();
        
        Ok(Self {
            database,
            info_client,
            enabled_assets,      // 使用传入的币种列表
            price_cache: HashMap::new(),
            sender,
            receiver,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        crate::log!(info, "price_collector", "start", 
            "启动价格收集服务", 
            "enabled_assets" => format!("{:?}", self.enabled_assets)
        );
        
        // 订阅所有币种的价格数据
        for coin in &self.enabled_assets {
            // 订阅 ActiveAssetCtx（获取指数价）
            crate::log!(debug, "price_collector", "start", 
                "订阅币种数据", 
                "coin" => coin.clone(),
                "subscription_type" => "ActiveAssetCtx"
            );
            let _subscription_id = self.info_client
                .subscribe(
                    Subscription::ActiveAssetCtx { 
                        coin: coin.to_string() 
                    },
                    self.sender.clone(),
                )
                .await?;
            
            // 订阅 BBO（获取最优买卖价）
            crate::log!(debug, "price_collector", "start", 
                "订阅币种数据", 
                "coin" => coin.clone(),
                "subscription_type" => "Bbo"
            );
            let _subscription_id = self.info_client
                .subscribe(
                    Subscription::Bbo { 
                        coin: coin.to_string() 
                    },
                    self.sender.clone(),
                )
                .await?;
        }
        
        crate::log!(info, "price_collector", "start", 
            "所有订阅完成，开始消息循环"
        );
        
        // 启动消息处理循环
        self.run_message_loop().await?;
        
        Ok(())
    }
    
    async fn run_message_loop(&mut self) -> Result<()> {
        crate::log!(info, "price_collector", "run_message_loop", 
            "开始消息处理循环", 
            "write_interval_ms" => WRITE_INTERVAL_MS
        );
        
        let mut write_timer = interval(Duration::from_millis(WRITE_INTERVAL_MS));
        let mut heartbeat_timer = interval(Duration::from_millis(HEARTBEAT_INTERVAL_MS));
        
        loop {
            tokio::select! {
                // 处理 WebSocket 消息
                message = self.receiver.recv() => {
                    if let Some(msg) = message {
                        crate::log!(debug, "price_collector", "run_message_loop", 
                            "收到WebSocket消息", 
                            "message_type" => format!("{:?}", std::mem::discriminant(&msg))
                        );
                        self.handle_message(msg).await?;
                    }
                }
                
                // 定时写库
                _ = write_timer.tick() => {
                    crate::log!(debug, "price_collector", "run_message_loop", 
                        "定时器触发，准备保存数据到数据库"
                    );
                    self.save_prices_to_db().await?;
                }

                // 心跳日志
                _ = heartbeat_timer.tick() => {
                    crate::log!(info, "price_collector", "heartbeat", 
                        "价格收集中..", 
                        "cache_size" => self.price_cache.len(),
                        "enabled_assets" => self.enabled_assets.len()
                    );
                }
            }
        }
    }
    
    async fn handle_message(&mut self, message: Message) -> Result<()> {
        match message {
            // 处理 ActiveAssetCtx（指数价）
            Message::ActiveAssetCtx(ctx) => {
                let coin = ctx.data.coin.clone();
                let index_price = match ctx.data.ctx {
                    hyperliquid_rust_sdk::AssetCtx::Perps(perps) => {
                        // 永续：使用 oracle_px 作为指数价
                        let price = safe_parse_f64(&perps.oracle_px, 0.0);
                        crate::log!(debug, "price_collector", "handle_message", 
                            "合约指数价格", 
                            "coin" => coin.clone(),
                            "oracle_px" => perps.oracle_px.clone(),
                            "parsed_price" => price
                        );
                        Some(price)
                    }
                    hyperliquid_rust_sdk::AssetCtx::Spot(_) => {
                        // 现货：跳过，不处理
                        crate::log!(debug, "price_collector", "handle_message", 
                            "跳过现货数据", 
                            "coin" => coin.clone()
                        );
                        None
                    }
                };
                
                if let Some(price) = index_price {
                    // 更新缓存
                    if let Some(price_data) = self.price_cache.get_mut(&coin) {
                        price_data.index_price = Some(price);
                        price_data.last_update = Instant::now();
                        crate::log!(debug, "price_collector", "handle_message", 
                            "更新现有价格数据", 
                            "coin" => coin.clone(),
                            "index_price" => price
                        );
                    } else {
                        self.price_cache.insert(coin.clone(), PriceData {
                            coin: coin.clone(),
                            best_bid: None,
                            best_ask: None,
                            index_price: Some(price),
                            last_update: Instant::now(),
                        });
                        crate::log!(info, "price_collector", "handle_message", 
                            "创建新币种价格数据", 
                            "coin" => coin.clone(),
                            "index_price" => price
                        );
                    }
                }
            }
            
            // 处理 BBO（最优买卖价）
            Message::Bbo(bbo) => {
                let coin = bbo.data.coin.clone();
                let best_bid = bbo.data.bbo[0].as_ref()
                    .and_then(|level| Some(safe_parse_f64(&level.px, 0.0)));
                let best_ask = bbo.data.bbo[1].as_ref()
                    .and_then(|level| Some(safe_parse_f64(&level.px, 0.0)));
                
                crate::log!(debug, "price_collector", "handle_message", 
                    "收到BBO数据", 
                    "coin" => coin.clone(),
                    "best_bid" => best_bid,
                    "best_ask" => best_ask
                );
                
                // 更新缓存
                if let Some(price_data) = self.price_cache.get_mut(&coin) {
                    price_data.best_bid = best_bid;
                    price_data.best_ask = best_ask;
                    price_data.last_update = Instant::now();
                    crate::log!(debug, "price_collector", "handle_message", 
                        "更新现有BBO数据", 
                        "coin" => coin.clone(),
                        "best_bid" => best_bid,
                        "best_ask" => best_ask
                    );
                } else {
                    self.price_cache.insert(coin.clone(), PriceData {
                        coin: coin.clone(),
                        best_bid,
                        best_ask,
                        index_price: None,
                        last_update: Instant::now(),
                    });
                    crate::log!(info, "price_collector", "handle_message", 
                        "创建新币种BBO数据", 
                        "coin" => coin.clone(),
                        "best_bid" => best_bid,
                        "best_ask" => best_ask
                    );
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    async fn save_prices_to_db(&mut self) -> Result<()> {
        let mut saved_count = 0;
        for (coin, price_data) in &self.price_cache {
            // 只保存有完整数据的记录
            if let (Some(bid), Some(ask)) = (price_data.best_bid, price_data.best_ask) {
                self.database.save_hl_prices(coin, bid, ask, price_data.index_price).await?;
                saved_count += 1;
                crate::log!(debug, "price_collector", "save_prices_to_db", 
                    "保存价格数据到数据库", 
                    "coin" => coin.clone(),
                    "bid" => bid,
                    "ask" => ask,
                    "index_price" => price_data.index_price
                );
            }
        }
        
        if saved_count > 0 {
            crate::log!(debug, "price_collector", "save_prices_to_db", 
                "保存数据完成", 
                "saved_count" => saved_count
            );
        }
        Ok(())
    }
} 