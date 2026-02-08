//! # Hyperliquid WebSocket 订阅示例：所有交易对中间价 (AllMids)
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的**实时 WebSocket 数据流**。
//!
//! 本示例的核心功能是订阅 `AllMids` 数据流。这会使服务器在**任何一个**永续合约交易对的中间价（买一价和卖一价的平均值）
//! 发生变化时，立即向客户端推送包含所有交易对最新中间价的完整列表。
//!
//! 这对于构建需要监控整个市场概览的应用程序非常有用，例如：
//! -   市场行情仪表盘
//! -   寻找套利机会的扫描器
//! -   需要全局市场价格作为参考的复杂交易策略
//!
//! 其工作流程与上一个示例类似：
//! 1.  **创建通道 (Channel)**: 建立一个内存中的消息队列。
//! 2.  **订阅数据流**: 订阅 `AllMids` 数据流。
//! 3.  **启动取消任务**: 在后台启动一个独立的异步任务，30秒后自动取消订阅。
//! 4.  **处理数据**: 主任务进入一个循环，持续地从通道中接收并打印服务器推送过来的 `AllMids` 数据。
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
    println!("程序启动：开始执行 Hyperliquid WebSocket 订阅与接收流程...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端与通信通道 ----
    
    // 中文输出：告知用户正在初始化客户端
    println!("[步骤 1/4] 正在初始化 InfoClient...");
    // 创建 InfoClient，用于发起订阅请求。
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    // 中文输出：告知用户正在创建通道
    println!("[步骤 2/4] 正在创建异步消息通道...");
    // 创建一个无界消息通道（Channel），用于在后台WebSocket连接和我们的主处理逻辑之间安全地传递消息。
    let (sender, mut receiver) = unbounded_channel();
    

    // ---- Part 2: 发起订阅并设置自动取消 ----
    
    // 中文输出：告知用户正在发起订阅
    println!("[步骤 3/4] 正在向服务器订阅 'AllMids' (所有交易对中间价) 数据流...");
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(Subscription::AllMids, sender) // 参数1: 订阅类型。这里是 AllMids，不需要额外参数。
        .await
        .unwrap();
    println!("    - 订阅成功！获取到的订阅ID: {}", subscription_id);

    // 在后台启动一个新的异步任务，用于在30秒后取消订阅。
    spawn(async move {
        // 让这个后台任务先休眠30秒。
        sleep(Duration::from_secs(30)).await;
        
        // 30秒后，执行取消订阅的操作。
        info!("Unsubscribing from mids data");
        println!("\n[后台任务] 30秒计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个30秒后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时数据... (将持续30秒)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    // This loop ends when we unsubscribe
    while let Some(Message::AllMids(all_mids)) = receiver.recv().await {
        info!("Received mids data: {all_mids:?}");
        println!("【实时数据】收到 AllMids 更新: {:?}", all_mids);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}