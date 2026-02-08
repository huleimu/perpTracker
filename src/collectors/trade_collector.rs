use anyhow::Result;
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription, UserFills};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::time::Duration;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;
use crate::types::MonitorEvent;
use crate::database::{Database, TradeEvent};
use alloy::primitives::Address;


/// 订阅组 - 管理最多10个地址，每个组独立处理消息
pub struct SubscriptionGroup {
    pub client: InfoClient,
    pub addresses: Vec<H160>,
    pub address_subscriptions: HashMap<H160, u32>, // 地址 -> subscription_id 映射
    pub message_sender: mpsc::UnboundedSender<Message>,
    pub max_capacity: usize,
    pub group_id: usize, // 用于标识订阅组
    pub database: Database, // 每个组有自己的数据库引用
    pub address_activity: Arc<Mutex<HashMap<H160, Instant>>>, // 地址活动时间跟踪
    pub trade_sender: mpsc::Sender<TradeEvent>, //将交易事件发送到中心写入器
}

impl SubscriptionGroup {
    pub fn new(
        client: InfoClient, 
        group_id: usize, 
        database: Database,
        address_activity: Arc<Mutex<HashMap<H160, Instant>>>,
        trade_sender: mpsc::Sender<TradeEvent>,
    ) -> (Self, mpsc::UnboundedReceiver<Message>) {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        let group = Self {
            client,
            addresses: Vec::new(),
            address_subscriptions: HashMap::new(),
            message_sender,
            max_capacity: 10, //Hyperliquid限制：每个订阅最多10个用户
            group_id,
            database,
            address_activity,
            trade_sender,
        };
        
        (group, message_receiver)
    }
    
    pub fn has_capacity(&self) -> bool {
        self.addresses.len() < self.max_capacity
    }
    
    pub fn add_address(&mut self, address: H160) -> Result<(), String> {
        if !self.has_capacity() {
            return Err("订阅组已达到最大容量".to_string());
        }
        if self.addresses.contains(&address) {
            return Err("地址已存在于订阅组中".to_string());
        }
        self.addresses.push(address);
        Ok(())
    }

    pub fn add_subscription(&mut self, address: H160, subscription_id: u32) {
        self.address_subscriptions.insert(address, subscription_id);
    }
    
    pub fn remove_subscription(&mut self, address: &H160) -> Option<u32> {
        self.address_subscriptions.remove(address)
    }
    
    pub fn get_subscription_id(&self, address: &H160) -> Option<&u32> {
        self.address_subscriptions.get(address)
    }
    
    pub fn remove_address(&mut self, address: &H160) -> bool {
        if let Some(pos) = self.addresses.iter().position(|x| x == address) {
            self.addresses.remove(pos);
            // 同时移除订阅记录
            self.address_subscriptions.remove(address);
            true
        } else {
            false
        }
    }
    
    pub fn address_count(&self) -> usize {
        self.addresses.len()
    }
    
    
}

/// 订阅池管理器 - 管理多个订阅组
pub struct SubscriptionPool {
    pub groups: HashMap<usize, SubscriptionGroup>,
    pub address_mappings: HashMap<H160, usize>, // 地址 -> 订阅组ID
    pub next_group_id: usize,
    pub database: Database, // 添加数据库属性
    pub trade_sender: mpsc::Sender<TradeEvent>, //中心写入器 sender
}

impl SubscriptionPool {
    pub fn new(database: Database, trade_sender: mpsc::Sender<TradeEvent>) -> Self {
        Self {
            groups: HashMap::new(),
            address_mappings: HashMap::new(),
            next_group_id: 0,
            database,
            trade_sender,
        }
    }
    
    // 为地址分配订阅组
    pub async fn assign_address_to_subscription(&mut self, address: H160, address_activity: Arc<Mutex<HashMap<H160, Instant>>>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // 如果地址已经分配，返回现有的订阅组ID
        if let Some(&group_id) = self.address_mappings.get(&address) {
            return Ok(group_id);
        }
        
        // 查找有容量的现有订阅组
        for (group_id, group) in &mut self.groups {
            if group.has_capacity() {
                group.add_address(address)?;
                self.address_mappings.insert(address, *group_id);
                return Ok(*group_id);
            }
        }
        
        // 没有可用的订阅组，创建新的
        let new_group_id = self.create_subscription_group(address_activity).await?;
        if let Some(group) = self.groups.get_mut(&new_group_id) {
            group.add_address(address)?;
            self.address_mappings.insert(address, new_group_id);
        }
        
        Ok(new_group_id)
    }
    
    // 创建新的订阅组
    pub async fn create_subscription_group(&mut self, address_activity: Arc<Mutex<HashMap<H160, Instant>>>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let group_id = self.next_group_id;
        
        // 创建新的WebSocket订阅
        let client = InfoClient::with_reconnect(None, Some(BaseUrl::Mainnet)).await?;
        
        // 创建订阅组，获取组和消息接收器
        let (group, mut message_receiver) = SubscriptionGroup::new(client, group_id, self.database.clone(), address_activity.clone(), self.trade_sender.clone());
        
        // 启动独立的消息处理任务
        let group_id_clone = group_id;
        let database = self.database.clone();
        let address_activity_clone = Arc::clone(&address_activity);
        let trade_sender_clone = self.trade_sender.clone();
        // 提前注册到可见结构，消除“组不可见”窗口
        let address_count_snapshot = group.address_count();
        self.groups.insert(group_id, group);
        self.next_group_id += 1;
                
        // 验证订阅组状态
        crate::log!(info, "trade_collector", "create_subscription_group", "订阅组创建完成", "group_id" => group_id, "address_count" => address_count_snapshot);
                
                tokio::spawn(async move {
            crate::log!(info, "trade_collector", "create_subscription_group", "订阅组启动独立消息处理任务", "group_id" => group_id_clone);
            let mut saw_no_data = false;
            let mut subscription_span = tracing::info_span!(
                "subscription_cycle",
                group_id = group_id_clone,
                subscription_trace_id = %uuid::Uuid::new_v4()
            );
            
            let mut message_count = 0;
            let mut last_message_time = Instant::now();
            
            // 独立的消息处理循环
            while let Some(message) = message_receiver.recv().await {
                let mut refresh_span = false;
                {
                    // 将本次迭代内的日志附着到当前订阅周期 span
                    let _guard = subscription_span.enter();
                    message_count += 1;
                    last_message_time = Instant::now();
                    
                    // 每10条消息打印一次统计
                    if message_count % 10 == 0 {
                        crate::log!(info, "trade_collector", "create_subscription_group", "订阅组统计", "group_id" => group_id_clone, "processed" => message_count, "last_msg_secs_ago" => last_message_time.elapsed().as_secs());
                    }
                        
                        match message {
                            Message::UserFills(user_fills) => {
                            crate::log!(debug, "trade_collector", "create_subscription_group", "处理UserFills消息", "group_id" => group_id_clone, "user" => format!("{:?}", user_fills.data.user), "fills" => user_fills.data.fills.len());
                            
                            if let Err(e) = Self::process_message(
                                    user_fills, 
                                group_id_clone, 
                                    &database, 
                                    &address_activity_clone,
                                    &trade_sender_clone
                                ).await {
                                crate::error!("trade_collector", "create_subscription_group", "处理交易失败", e, "group_id" => group_id_clone);
                            }
                        }
                        Message::NoData => {
                            crate::log!(warn, "trade_collector", "create_subscription_group", "收到断线通知(NoData)", "group_id" => group_id_clone);
                            saw_no_data = true;  // 标记断线，等待重连
                                                    }
                        Message::HyperliquidError(err) => {
                            crate::error!("trade_collector", "create_subscription_group", "收到服务端错误(HyperliquidError)", err, "group_id" => group_id_clone);
                        }
                        Message::SubscriptionResponse => {
                            // 若刚经历断线恢复，标记为刷新订阅周期 trace_id
                            if saw_no_data { refresh_span = true; }
                            crate::log!(debug, "trade_collector", "cr   eate_subscription_group", "收到订阅回执(SubscriptionResponse)", "group_id" => group_id_clone);
                        }
                        other => {
                            crate::log!(debug, "trade_collector", "create_subscription_group", "收到其他消息", "group_id" => group_id_clone, "payload" => format!("{:?}", other));
                        }
                    }
                }
                // 在本次迭代结束时（guard 释放后）安全刷新订阅周期 span
                if refresh_span {
                    saw_no_data = false;
                    subscription_span = tracing::info_span!(
                        "subscription_cycle",
                        group_id = group_id_clone,
                        subscription_trace_id = %uuid::Uuid::new_v4()
                    );
                    crate::log!(info, "trade_collector", "create_subscription_group", "重连完成，更新订阅周期", "group_id" => group_id_clone);
                }
            }
            
            crate::log!(info, "trade_collector", "create_subscription_group", "订阅组消息处理任务结束", "group_id" => group_id_clone, "processed" => message_count);
        });
         
        crate::log!(info, "trade_collector", "create_subscription_group", "创建新的订阅组", "group_id" => group_id);
        Ok(group_id)
    }
    
    async fn process_message(
        user_fills: UserFills,
        group_id: usize,
        database: &Database,
        address_activity: &Arc<Mutex<HashMap<H160, Instant>>>,
        trade_sender: &mpsc::Sender<TradeEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let user = user_fills.data.user;
        let fills = user_fills.data.fills;
        
        // 获取当前时间，用于时间过滤
        let now = chrono::Utc::now();
        let mut saved_count = 0;
        let mut skipped_count = 0;
        
        for fill in fills {
            // 时间过滤：只保存最近30分钟内的交易
            let trade_time = chrono::DateTime::from_timestamp_millis(fill.time as i64)
                .unwrap_or(now);
            
            let time_diff = now.signed_duration_since(trade_time);
            let max_age = chrono::Duration::minutes(30); // 可配置的时间窗口
            
            if time_diff > max_age {
                skipped_count += 1;
                continue; // 跳过过期的交易
            }
            
            let coin = fill.coin.clone();
            let px = fill.px.clone();
            let sz = fill.sz.clone();
            
            let size_f64 = sz.parse::<f64>().unwrap_or(0.0);
            let price_f64 = px.parse::<f64>().unwrap_or(0.0);
            let value = size_f64 * price_f64;
            
            let monitor_event = MonitorEvent {
                source_address: H160::from_slice(&user.as_slice()),
                timestamp: trade_time.to_rfc3339(), // 使用交易的实际时间
                coin: coin.clone(),
                action: if fill.side == "B" { "Buy".to_string() } else { "Sell".to_string() },
                direction: fill.side,
                closed_pnl: fill.closed_pnl.parse().unwrap_or(0.0),
                size: size_f64,
                price: price_f64,
                value: value,
                trade_type: "Fill".to_string(),
                order_id: fill.oid.to_string(),
            };
            
            let trade_event = TradeEvent {
                id: None,
                addr: format!("{:?}", monitor_event.source_address),
                coin: monitor_event.coin,
                action: monitor_event.action,
                direction: monitor_event.direction,
                size: monitor_event.size,
                price: monitor_event.price,
                value: monitor_event.value,
                closed_pnl: monitor_event.closed_pnl,
                trade_type: monitor_event.trade_type,
                order_id: monitor_event.order_id,
                trade_time: trade_time, // 使用交易的实际时间
                created_at: Some(chrono::Utc::now()),
            };
            
            // 通过中心通道发送，避免在网络 I/O 回调里执行慢 I/O
            if let Err(e) = trade_sender.send(trade_event).await {
                crate::error!("trade_collector", "process_message", "发送交易到处理队列失败", e, "group_id" => group_id);
            } else {
                saved_count += 1; 
                crate::log!(debug, "trade_collector", "process_message", format!("订阅组 {} 发送交易到队列: {:?} -> {:?} (价格: {}, 数量: {}, 时间: {})", group_id, user, coin, price_f64, size_f64, trade_time.format("%H:%M:%S")));
            }
        }
        
        // 打印过滤统计
        if saved_count > 0 || skipped_count > 0 {
              crate::log!(info, "trade_collector", "process_message", "地址统计", "group_id" => group_id, "user" => format!("{:?}", user), "queued" => saved_count, "skipped" => skipped_count);
        }
        
        // 更新地址活动时间
        let user_h160 = H160::from_slice(&user.as_slice());
        {
            let mut activity_map = address_activity.lock().await;
            activity_map.insert(user_h160, Instant::now());
        }
        
        Ok(())
    }
    
    // 获取地址所在的订阅组ID
    pub fn get_group_id(&self, address: &H160) -> Option<usize> {
        self.address_mappings.get(address).copied()
    }
    
    // 获取订阅组
    pub fn get_group(&self, group_id: usize) -> Option<&SubscriptionGroup> {
        self.groups.get(&group_id)
    }
    
    // 获取可变订阅组
    pub fn get_group_mut(&mut self, group_id: usize) -> Option<&mut SubscriptionGroup> {
        self.groups.get_mut(&group_id)
    }
    
    // // 获取当前订阅组数量
    // pub fn group_count(&self) -> usize {
    //     self.groups.len()
    // }
    

    
    // 获取订阅池状态摘要
    pub fn get_pool_status_summary(&self) -> String {
        let mut summary = format!("订阅池状态: {} 个订阅组, {} 个地址\n", self.groups.len(), self.address_mappings.len());
        
        for (group_id, group) in &self.groups {
            let address_count = group.address_count();
            let capacity = group.max_capacity;
            summary.push_str(&format!("  订阅组 {}: {}/{} 地址\n", group_id, address_count, capacity));
        }
        
        summary
    }
    
}

/// 交易收集器 - 使用订阅池管理多个地址
pub struct TradeCollector {
    database: Database,
    subscription_pool: SubscriptionPool,
    address_activity: Arc<Mutex<HashMap<H160, Instant>>>, // 地址 -> 最后活动时间
    resubscribe_timeout: Duration,  // 多久没收到交易就重订阅
    resubscribe_batch_size: usize,  // 每轮最多重订阅多少个地址
    trade_sender: mpsc::Sender<TradeEvent>, //中心写入器 sender
    trade_receiver: Option<mpsc::Receiver<TradeEvent>>, //中心写入器 receiver（在 run 中消费）
}

impl TradeCollector {
    pub fn new(database: Database) -> Self {
        let (trade_sender, trade_receiver) = mpsc::channel::<TradeEvent>(8192);
        Self {
            database: database.clone(),
            subscription_pool: SubscriptionPool::new(database, trade_sender.clone()),
            address_activity: Arc::new(Mutex::new(HashMap::new())),
            resubscribe_timeout: Duration::from_secs(600),  // 3分钟没收到交易就重订阅
            resubscribe_batch_size: 10,  // 每轮最多重订阅10个地址
            trade_sender,
            trade_receiver: Some(trade_receiver),
        }
    }
    
    // 订阅函数
    async fn subscribe_address_with_retry(
        address: H160,
        group: &mut SubscriptionGroup,
        group_id: usize,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let alloy_address = Address::from_slice(&address.as_bytes());
        
        // 重试策略：立即一次，随后 200/400/800ms 退避
        let delays_ms: [u64; 4] = [0, 200, 400, 800];
        
        for (attempt, delay) in delays_ms.iter().enumerate() {
            if *delay > 0 { 
                tokio::time::sleep(Duration::from_millis(*delay)).await; 
            }
            
            crate::log!(info, "trade_collector", "subscribe_address_with_retry", "尝试订阅", "address" => format!("{:?}", address), "attempt" => attempt + 1, "group_id" => group_id);
            
            match group.client.subscribe(
                Subscription::UserFills { user: alloy_address },
                group.message_sender.clone()
            ).await {
                Ok(subscription_id) => {
                    crate::log!(info, "trade_collector", "subscribe_address_with_retry", "订阅成功", "address" => format!("{:?}", address), "attempt" => attempt + 1, "subscription_id" => subscription_id);
                    return Ok(subscription_id);
                }
                Err(e) => {
                    crate::error!("trade_collector", "subscribe_address_with_retry", "订阅失败", e, "address" => format!("{:?}", address), "attempt" => attempt + 1);
                    if attempt == delays_ms.len() - 1 {
                        return Err(e.into());
                    }
                }
            }
        }
        
        Err("所有重试都失败了".into())
    }
    
    async fn update_address_activity(
        address: H160,
        address_activity: &Arc<Mutex<HashMap<H160, Instant>>>
    ) {
        let mut activity_map = address_activity.lock().await;
        activity_map.insert(address, Instant::now());
    }
    
    async fn handle_successful_subscription(
        address: H160,
        subscription_id: u32,
        group: &mut SubscriptionGroup,
        address_activity: &Arc<Mutex<HashMap<H160, Instant>>>
    ) {
        // 保存subscription_id
        group.add_subscription(address, subscription_id);
        
        // 记录地址活动时间
        Self::update_address_activity(address, address_activity).await;
        
        crate::log!(info, "trade_collector", "handle_successful_subscription", "订阅处理完成", "address" => format!("{:?}", address), "subscription_id" => subscription_id);
    }
    
    // 主运行函数
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        crate::log!(info, "trade_collector", "run", "启动简化交易收集器（订阅复用模式）...");
        
        if let Some(mut receiver) = self.trade_receiver.take() {
            let db_clone = self.database.clone();
            tokio::spawn(async move {
                let mut batch: Vec<TradeEvent> = Vec::with_capacity(50);
                let batch_timeout = Duration::from_millis(500);
                loop {
                    tokio::select! {
                        Some(trade) = receiver.recv() => {
                            batch.push(trade);
                            if batch.len() >= 50 {
                                let to_write = std::mem::take(&mut batch);
                                let count = to_write.len();
                                let start = std::time::Instant::now();
                                if let Err(e) = db_clone.save_trade_events_batch(&to_write).await {
                                    crate::error!("database_writer", "run", "批量保存交易失败", e);
                                } else {
                                    crate::log!(info, "database_writer", "run", "批量写入交易", "count" => count, "ms" => start.elapsed().as_millis());
                                }
                            }
                        }
                        _ = tokio::time::sleep(batch_timeout), if !batch.is_empty() => {
                            let to_write = std::mem::take(&mut batch);
                            let count = to_write.len();
                            let start = std::time::Instant::now();
                            if let Err(e) = db_clone.save_trade_events_batch(&to_write).await {
                                crate::error!("database_writer", "run", "超时批量保存交易失败", e);
                            } else {
                                crate::log!(info, "database_writer", "run", "定时写入交易", "count" => count, "ms" => start.elapsed().as_millis());
                            }
                        }
                    }
                }
            });
        }
        
        loop {
            // 检查超时地址
            if let Err(e) = self.check_timeout_addresses().await {
                crate::error!("trade_collector", "run", "检查超时地址失败", e);
            }
            
            match self.database.get_active_wallets().await {
                Ok(active_addresses) => {
                    let h160_addresses: Vec<H160> = active_addresses
                        .iter()
                        .filter_map(|addr_str| {
                            H160::from_str(addr_str).ok()
                        })
                        .collect();
                    
                    crate::log!(info, "trade_collector", "run", "数据库中的活跃地址数量", "count" => h160_addresses.len());
                    
                    if let Err(e) = self.handle_address_changes(&h160_addresses).await {
                        crate::error!("trade_collector", "run", "处理地址变化失败", e);
                    }
                    
                    // 打印订阅池状态
                    crate::log!(info, "trade_collector", "run", "订阅池状态", "summary" => self.subscription_pool.get_pool_status_summary());
                    
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                Err(e) => {
                    crate::error!("trade_collector", "run", "获取活跃地址失败", e);
                }
            }
            
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
    
    // 处理地址变化
    async fn handle_address_changes(&mut self, active_addresses: &[H160]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        crate::log!(info, "trade_collector", "handle_address_changes", "开始处理地址变化...");

        let current_addresses: Vec<H160> = self.subscription_pool.address_mappings.keys().cloned().collect();

        // 限制每轮新增地址数量，避免瞬时订阅过多导致限流
        let mut added_this_tick: usize = 0;
        let add_limit: usize = 5;

        for address in active_addresses {
            if !current_addresses.contains(address) {
                if added_this_tick >= add_limit { break; }
                crate::log!(info, "trade_collector", "handle_address_changes", "添加新地址", "address" => format!("{:?}", address));
                if let Err(e) = self.add_address(*address).await {
                    crate::error!("trade_collector", "handle_address_changes", "添加地址失败", e, "address" => format!("{:?}", address));
                }
                added_this_tick += 1;
                // 订阅之间增加间隔，降低瞬时压力
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }

        for address in current_addresses {
            if !active_addresses.contains(&address) {
                crate::log!(info, "trade_collector", "handle_address_changes", "移除地址", "address" => format!("{:?}", address));
                if let Err(e) = self.remove_address(address).await {
                    crate::error!("trade_collector", "handle_address_changes", "移除地址失败", e, "address" => format!("{:?}", address));
                }
            }
        }

        crate::log!(info, "trade_collector", "handle_address_changes", "地址变化处理完成");
        Ok(())
    }

    // 添加新地址
    async fn add_address(&mut self, address: H160) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        let group_id = self.subscription_pool.assign_address_to_subscription(address, Arc::clone(&self.address_activity)).await?;
        crate::log!(info, "trade_collector", "add_address", "地址分配到订阅组", "address" => format!("{:?}", address), "group_id" => group_id);

        if let Some(group) = self.subscription_pool.get_group_mut(group_id) {
            crate::log!(info, "trade_collector", "add_address", "订阅组当前容量", "group_id" => group_id, "count" => group.address_count(), "capacity" => group.max_capacity);
            
            if group.address_subscriptions.is_empty() {
                tokio::time::sleep(Duration::from_millis(3000)).await;
            }
            //订阅函数
            match Self::subscribe_address_with_retry(address, group, group_id).await {
                Ok(subscription_id) => {
                    // 使用通用处理函数
                    Self::handle_successful_subscription(address, subscription_id, group, &self.address_activity).await;
                }
                Err(e) => {
                    crate::error!("trade_collector", "add_address", "订阅最终失败", e, "address" => format!("{:?}", address));
                }
            }
        } else {
            crate::error!("trade_collector", "add_address", "无法获取订阅组引用", format!("{}", group_id));
        }

        Ok(())
    }

    // 移除地址
    async fn remove_address(&mut self, address: H160) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let group_id = self.subscription_pool.get_group_id(&address);
        
        if let Some(group_id) = group_id {
            if let Some(group) = self.subscription_pool.get_group_mut(group_id) {
                group.remove_address(&address);
            }
            
            self.subscription_pool.address_mappings.remove(&address);
            crate::log!(info, "trade_collector", "remove_address", "地址已从订阅组移除", "address" => format!("{:?}", address), "group_id" => group_id);
            
            if let Some(group) = self.subscription_pool.get_group(group_id) {
                if group.address_count() == 0 {
                    crate::log!(info, "trade_collector", "remove_address", "订阅组现在为空，可以考虑关闭", "group_id" => group_id);
                }
            }
        }
        Ok(())
    }
    
    // 检查超时地址并重新订阅
    async fn check_timeout_addresses(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Instant::now();
        let mut timeout_addresses = Vec::new();
        
        // 检查哪些地址超时了
        {
            let mut activity_map = self.address_activity.lock().await;
            crate::log!(info, "trade_collector", "check_timeout_addresses", "检查地址活动状态", "monitored" => activity_map.len());
            
            // 获取所有当前监控的地址
            let current_addresses: Vec<H160> = self.subscription_pool.address_mappings.keys().cloned().collect();
            
            for address in &current_addresses {
                if let Some(last_activity) = activity_map.get(address) {
                    let duration = now.duration_since(*last_activity);
                    if duration > self.resubscribe_timeout {
                        crate::log!(info, "trade_collector", "check_timeout_addresses", "地址超时未收到交易", "address" => format!("{:?}", address), "idle_secs" => duration.as_secs(), "threshold_secs" => self.resubscribe_timeout.as_secs());
                        timeout_addresses.push(*address);
                    } else {
                        crate::log!(info, "trade_collector", "check_timeout_addresses", "地址正常", "address" => format!("{:?}", address), "last_trade_secs_ago" => duration.as_secs());
                    }
                } else {
                    // 地址从未收到过交易，给一个初始宽限期（比如5分钟）
                    let initial_grace_period = now - Duration::from_secs(300); // 5分钟前
                    crate::log!(info, "trade_collector", "check_timeout_addresses", "地址首次监控，设置初始宽限期", "address" => format!("{:?}", address), "grace_secs" => 300);
                    activity_map.insert(*address, initial_grace_period);
                }
            }
        }
        
        if timeout_addresses.is_empty() {
            crate::log!(info, "trade_collector", "check_timeout_addresses", "没有发现超时地址，所有地址都在正常接收交易");
            return Ok(());
        }
        
        crate::log!(info, "trade_collector", "check_timeout_addresses", "发现超时地址，开始重新订阅", "count" => timeout_addresses.len());
        
        // 限制每轮重订阅的数量
        let addresses_to_resubscribe: Vec<H160> = timeout_addresses
            .into_iter()
            .take(self.resubscribe_batch_size)
            .collect();
        
        for address in addresses_to_resubscribe {
            crate::log!(info, "trade_collector", "check_timeout_addresses", "重新订阅超时地址", "address" => format!("{:?}", address));
            
            // 获取订阅组
            let group_id = self.subscription_pool.assign_address_to_subscription(address, Arc::clone(&self.address_activity)).await?;
            if let Some(group) = self.subscription_pool.get_group_mut(group_id) {
                // 先取消现有订阅
                if let Some(subscription_id) = group.get_subscription_id(&address) {
                    crate::log!(info, "trade_collector", "check_timeout_addresses", "取消现有订阅", "address" => format!("{:?}", address), "subscription_id" => subscription_id);
                    match group.client.unsubscribe(*subscription_id).await {
                        Ok(_) => {
                            crate::log!(info, "trade_collector", "check_timeout_addresses", "取消订阅成功", "address" => format!("{:?}", address));
                            // 移除旧的订阅记录
                            group.remove_subscription(&address);
                        }
                        Err(e) => {
                            crate::error!("trade_collector", "check_timeout_addresses", "取消订阅失败", e, "address" => format!("{:?}", address));
                        }
                    }
                }
                
                // 等待一小段时间再重新订阅
                tokio::time::sleep(Duration::from_millis(1000)).await;
                
                // 使用通用订阅函数重新订阅
                match Self::subscribe_address_with_retry(address, group, group_id).await {
                    Ok(subscription_id) => {
                        crate::log!(info, "trade_collector", "check_timeout_addresses", "重新订阅成功", "address" => format!("{:?}", address), "subscription_id" => subscription_id);
                        // 使用通用处理函数
                        Self::handle_successful_subscription(address, subscription_id, group, &self.address_activity).await;
                    }
                    Err(e) => {
                        crate::error!("trade_collector", "check_timeout_addresses", "重新订阅失败", e, "address" => format!("{:?}", address));
                    }
                }
            }
        }
        
        Ok(())
    }
}


// 主函数 - 运行多地址监控
pub async fn run_multi_address_monitor(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    crate::log!(info, "trade_collector", "run_multi_address_monitor", "启动多地址交易监控...");
    
    let mut collector = TradeCollector::new(database.clone());
    
    collector.run().await?;
    
    Ok(())
}


