//! # Hyperliquid WebSocket 订阅示例：用户实时通知 (Notification)
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的**特定用户的实时事件通知**。
//!
//! 与之前订阅公开市场数据（如订单簿、K线）不同，`Notification` 订阅是针对一个**具体的用户地址**的。
//! 当这个被监控的账户发生任何重要的活动时，服务器会主动推送一条通知消息。这些活动可能包括：
//! -   订单被成交 (Fill)
//! -   仓位被强平 (Liquidation)
//! -   资金转账完成
//! -   ... 等等其他与账户状态变更相关的事件。
//!
//! 这对于构建需要实时响应账户变化的应用程序至关重要，例如：
//! -   交易机器人的实时成交回报处理。
//! -   风险管理系统，用于即时发送强平警报。
//! -   用户交易活动的可视化动态流。
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

use std::str::FromStr;

use ethers::types::H160;
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
    println!("程序启动：开始执行 Hyperliquid WebSocket 用户通知订阅流程...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端与通信通道 ----
    
    // 中文输出：告知用户正在初始化客户端
    println!("[步骤 1/4] 正在初始化 InfoClient (连接到测试网)...");
    // 创建 InfoClient，并连接到测试网。
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    
    // 定义要监控的用户地址。
    // H160 是 ethers-rs 库中代表一个160位（20字节）以太坊地址的类型。
    let user = H160::from_str("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8").unwrap();
    println!("    - 将要监控的用户地址: {:?}", user);

    // 中文输出：告知用户正在创建通道
    println!("[步骤 2/4] 正在创建异步消息通道...");
    // 创建一个无界消息通道（Channel）。
    let (sender, mut receiver) = unbounded_channel();
    

    // ---- Part 2: 发起订阅并设置自动取消 ----
    
    // 中文输出：告知用户正在发起订阅
    println!("[步骤 3/4] 正在向服务器订阅该用户的 'Notification' (事件通知) 数据流...");
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(
            // 定义订阅类型为 Notification，并提供目标用户地址。
            Subscription::Notification { user },
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
        info!("Unsubscribing from notification data");
        println!("\n[后台任务] 30秒计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个30秒后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时用户通知... (将持续30秒)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    // this loop ends when we unsubscribe
    while let Some(Message::Notification(notification)) = receiver.recv().await {
        info!("Received notification data: {notification:?}");
        println!("【用户通知】收到新消息: {:?}", notification);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}