//! # Hyperliquid WebSocket 订阅示例：实时3分钟K线数据 (Candle)
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的**实时 WebSocket K线数据流**。
//!
//! 本示例与上一个非常相似，核心功能是订阅特定资产（"ETH"）在特定时间周期（"3m"，即3分钟）的K线数据。
//!
//! 每当一个3分钟的周期结束，服务器就会主动推送一条包含该周期开盘价(o)、最高价(h)、最低价(l)、收盘价(c)和成交量(v)
//! 的完整K线数据。这展示了 `Subscription::Candle` 的可配置性，您可以根据策略需求选择不同的时间粒度。
//!
//! ## !! 重要提示：连接到主网 (Mainnet) !!
//!
//! 请注意，此示例中的 `InfoClient` 被配置为连接到 `BaseUrl::Mainnet`，即**真实交易环境**。
//! 虽然 `InfoClient` 本身是只读的，不会产生资金风险，但在您复制或修改此代码用于其他目的
//! （尤其是使用 `ExchangeClient`）时，务必清楚您正在与真实资金环境交互。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin                   ws_candles
//! ```

use log::info;

use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid WebSocket K线数据订阅流程...");
    println!("!! 警告 !! 本程序将连接到【主网 Mainnet】读取实时数据。");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端与通信通道 ----
    
    // 中文输出：告知用户正在初始化客户端
    println!("[步骤 1/4] 正在初始化 InfoClient (连接到【主网】)...");
    // 创建 InfoClient，并明确指定连接到主网。
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();

    // 中文输出：告知用户正在创建通道
    println!("[步骤 2/4] 正在创建异步消息通道...");
    // 创建一个无界消息通道（Channel），用于在后台WebSocket连接和我们的主处理逻辑之间安全地传递消息。
    let (sender, mut receiver) = unbounded_channel();
    

    // ---- Part 2: 发起订阅并设置自动取消 ----
    
    // 中文输出：告知用户正在发起订阅
    println!("[步骤 3/4] 正在向服务器订阅 'ETH' 资产的 '3m' (3分钟) K线数据流...");
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(
            // 定义订阅类型为 Candle，并提供必要的参数。
            Subscription::Candle {
                coin: "ETH".to_string(),      // 参数1: 目标资产
                interval: "1m".to_string(),   // 参数2: K线的时间间隔 (这里是3分钟)
            },
            sender, // 将收到的消息发送到这个 `sender`
        )
        .await
        .unwrap();
    println!("    - 订阅成功！获取到的订阅ID: {}", subscription_id);

    // 在后台启动一个新的异步任务，用于在1分钟（60秒）后取消订阅。
    spawn(async move {
        // 让这个后台任务先休眠300秒。
        sleep(Duration::from_secs(30)).await;
        
        // 300秒后，执行取消订阅的操作。
        info!("Unsubscribing from candle data");
        println!("\n[后台任务] 5分钟计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个5分钟后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时K线数据... (将持续5分钟)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    // This loop ends when we unsubscribe
    while let Some(Message::Candle(candle)) = receiver.recv().await {
        info!("Received candle data: {candle:?}");
        println!("【实时K线】收到新的 3m K线数据: {:?}", candle);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}
