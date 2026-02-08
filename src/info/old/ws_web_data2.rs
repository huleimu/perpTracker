//! # Hyperliquid WebSocket 订阅示例：网页端综合数据 (WebData2)
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的一个**高度优化的、为网页前端设计的综合数据流**。
//!
//! `WebData2` 订阅是一个“瑞士军刀”式的数据源。它不是提供单一类型的原始数据（如L2订单簿或单笔成交），
//! 而是将一个特定用户在网页交易界面上需要看到的**所有关键信息打包在一起**，通过一次订阅高效地推送过来。
//!
//! 当被监控的账户或其相关的市场状态发生变化时，服务器会推送一条 `WebData2` 消息，其中通常包含了：
//! -   **用户状态 (User State)**: 如保证金、杠杆、未实现盈亏等。
//! -   **资产上下文 (Asset Contexts)**: 用户持有仓位的那些资产的实时市场信息（如标记价格、资金费率等）。
//! -   **未结订单 (Open Orders)**: 该用户的所有活动订单列表。
//! -   可能还包括其他用于UI渲染的聚合信息。
//!
//! 这对于想要构建一个功能齐全、类似官方交易界面的应用程序来说，是最高效、最方便的订阅选项，因为它避免了同时维护多个独立订阅的复杂性。
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
    println!("程序启动：开始执行 Hyperliquid WebSocket 网页端综合数据订阅流程...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端与通信通道 ----
    
    // 中文输出：告知用户正在初始化客户端
    println!("[步骤 1/4] 正在初始化 InfoClient (连接到测试网)...");
    // 创建 InfoClient，并连接到测试网。
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    
    // 定义要监控的用户地址。
    let user = H160::from_str("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8").unwrap();
    println!("    - 将要监控的用户地址: {:?}", user);

    // 中文输出：告知用户正在创建通道
    println!("[步骤 2/4] 正在创建异步消息通道...");
    // 创建一个无界消息通道（Channel）。
    let (sender, mut receiver) = unbounded_channel();
    

    // ---- Part 2: 发起订阅并设置自动取消 ----
    
    // 中文输出：告知用户正在发起订阅
    println!("[步骤 3/4] 正在向服务器订阅该用户的 'WebData2' (网页端综合数据) 数据流...");
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(
            // 定义订阅类型为 WebData2，并提供目标用户地址。
            Subscription::WebData2 { user },
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
        info!("Unsubscribing from web data2");
        println!("\n[后台任务] 30秒计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个30秒后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时综合数据... (将持续30秒)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    // this loop ends when we unsubscribe
    while let Some(Message::WebData2(web_data2)) = receiver.recv().await {
        info!("Received web data: {web_data2:?}");
        println!("【网页端综合数据】收到新消息: {:?}", web_data2);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}