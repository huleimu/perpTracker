//! # Hyperliquid WebSocket 订阅示例：实时资产上下文 (ActiveAssetCtx)
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的**实时 WebSocket 数据流**。
//!
//! 与之前“请求-响应”模式不同，WebSocket 允许服务器在数据更新时**主动推送**信息给客户端，
//! 实现了真正的实时通信。这对于构建实时仪表盘、警报系统或高频交易机器人至关重要。
//!
//! 本示例的核心流程如下：
//! 1.  **创建通道 (Channel)**: 建立一个内存中的消息队列，用于在不同的异步任务之间安全地传递数据。
//! 2.  **订阅数据流**: 向服务器订阅特定资产（"BTC"）的“活动资产上下文”(`ActiveAssetCtx`)，这包含了该资产的实时状态，如标记价格、资金费率等。
//! 3.  **启动取消任务**: 在后台启动一个独立的异步任务，该任务在等待30秒后会自动取消此次订阅。
//! 4.  **处理数据**: 主任务进入一个循环，持续地从通道中接收并打印服务器推送过来的实时数据，直到订阅被取消。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin                 ws_active_asset_ctx
//! ```

use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use log::info;
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
    println!("程序启动：开始执行 Hyperliquid WebSocket 订阅与接收流程...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端与通信通道 ----

    // 中文输出：告知用户正在初始化客户端
    println!("[步骤 1/4] 正在初始化 InfoClient...");
    // 创建 InfoClient，用于发起订阅请求。
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    let coin = "BTC".to_string();

    // 中文输出：告知用户正在创建通道
    println!("[步骤 2/4] 正在创建异步消息通道...");
    // 创建一个无界消息通道（Channel）。
    // `sender` 端用于发送数据（SDK内部会用它），`receiver` 端用于接收数据（我们的代码会用它）。
    // 这就像一个内存中的邮箱，用于在后台的WebSocket连接和我们的主处理逻辑之间安全地传递消息。
    let (sender, mut receiver) = unbounded_channel();
    

    // ---- Part 2: 发起订阅并设置自动取消 ----
    
    // 中文输出：告知用户正在发起订阅
    println!("[步骤 3/4] 正在向服务器订阅 '{}' 资产的 ActiveAssetCtx 数据流...", coin);
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(
            Subscription::ActiveAssetCtx {   coin }, // 参数1: 订阅类型和目标
            sender                                 // 参数2: 将收到的消息发送到这个 `sender`
        )
        .await
        .unwrap();
    println!("    - 订阅成功！获取到的订阅ID: {}", subscription_id);

    // `spawn` 会在后台启动一个新的异步任务。
    // 这个任务独立运行，不会阻塞主程序的执行。
    spawn(async move {
        // 让这个后台任务先休眠30秒。
        sleep(Duration::from_secs(30)).await;
        
        // 30秒后，执行取消订阅的操作。
        info!("30秒计时结束，正在取消订阅...");
        println!("\n[后台任务] 30秒计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个30秒后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时数据... (将持续30秒)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // `receiver.recv().await` 会异步地等待，直到有新消息到达通道。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    while let Some(Message::ActiveAssetCtx(active_asset_ctx)) = receiver.recv().await {
        info!("Received active asset ctx: {active_asset_ctx:?}");
        println!("【实时数据】收到 ActiveAssetCtx 更新: {:?}", active_asset_ctx);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}