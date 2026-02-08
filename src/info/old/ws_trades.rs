//! # Hyperliquid WebSocket 订阅示例：实时市场成交记录 (Trades)
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的**实时市场成交数据流**。
//!
//! 与订阅订单簿（L2Book）或用户订单更新（OrderUpdates）不同，`Trades` 数据流提供了市场上**每一笔实际发生**的公开成交记录。
//! 每当指定资产（如 "ETH"）有一个新的成交时，服务器就会主动推送一条包含该笔交易价格、数量、方向和时间戳等详细信息的 `Trades` 消息。
//!
//! 这对于构建以下应用至关重要：
//! -   高频交易者分析市场动向，例如检测大单成交或市场买卖压力。
//! -   构建“Time and Sales”（时间与销售）工具，实时显示每一笔成交。
//! -   为需要精确成交量数据的量化策略提供实时输入。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! RUST_LOG=info cargo run
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
    println!("程序启动：开始执行 Hyperliquid WebSocket 实时成交记录订阅流程...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端与通信通道 ----
    
    // 中文输出：告知用户正在初始化客户端
    println!("[步骤 1/4] 正在初始化 InfoClient (连接到测试网)...");
    // 创建 InfoClient，并连接到测试网。
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    // 中文输出：告知用户正在创建通道
    println!("[步骤 2/4] 正在创建异步消息通道...");
    // 创建一个无界消息通道（Channel），用于在后台WebSocket连接和我们的主处理逻辑之间安全地传递消息。
    let (sender, mut receiver) = unbounded_channel();
    

    // ---- Part 2: 发起订阅并设置自动取消 ----
    
    // 中文输出：告知用户正在发起订阅
    println!("[步骤 3/4] 正在向服务器订阅 'ETH' 资产的 'Trades' (实时市场成交记录) 数据流...");
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(
            // 定义订阅类型为 Trades，并提供目标资产。
            Subscription::Trades {
                coin: "ETH".to_string(),
            },
            sender, // 将收到的消息发送到这个 `sender`
        )
        .await
        .unwrap();
    println!("    - 订阅成功！获取到的订阅ID: {}", subscription_id);

    // 在后台启动一个新的异步任务，用于在30秒后取消订阅。
    spawn(async move {
        // 让这个后台任务先休眠30秒。
        sleep(Duration::from_secs(30)).await;
        
        // 30秒后，执行取消订阅的操作。
        info!("Unsubscribing from trades data");
        println!("\n[后台任务] 30秒计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个30秒后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时成交记录... (将持续30秒)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    // This loop ends when we unsubscribe
    while let Some(Message::Trades(trades)) = receiver.recv().await {
        info!("Received trade data: {trades:?}");
        println!("【实时成交】收到新的成交记录: {:?}", trades);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}