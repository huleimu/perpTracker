//! # Hyperliquid WebSocket 订阅示例：用户非资金费率账本更新
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 的 `InfoClient` 订阅来自 Hyperliquid 的**特定用户的非资金费率相关的实时账本更新**。
//!
//! `UserNonFundingLedgerUpdates` 是一个高度专业化的数据流。它提供了用户账户上**所有影响其现金余额的金融交易流水**，但**明确排除了**由资金费率（Funding Rate）引起的收支。
//!
//! 这意味着你会收到以下类型的事件通知：
//! -   **平仓盈亏 (PnL)**: 当一个仓位被关闭时，已实现的利润或亏损被记入账户余额。
//! -   **交易手续费 (Fees)**: 每一笔交易产生的手续费。
//! -   **存提款 (Deposits/Withdrawals)**: 资金的转入或转出。
//! -   **内部划转**: 在不同账户类别（如现货、合约）之间的资金移动。
//! -   **强平惩罚**等。
//!
//! 对于需要构建精确、实时的会计分类账或审计跟踪的系统来说，这个数据流非常有价值，因为它已经帮你过滤掉了周期性的资金费率事件。
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
    println!("程序启动：开始执行 Hyperliquid WebSocket 用户非资金费率账本更新订阅流程...");
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
    println!("[步骤 3/4] 正在向服务器订阅该用户的 'UserNonFundingLedgerUpdates' 数据流...");
    // 调用 `subscribe` 方法，发起订阅请求。
    let subscription_id = info_client
        .subscribe(
            // 定义订阅类型为 UserNonFundingLedgerUpdates，并提供目标用户地址。
            Subscription::UserNonFundingLedgerUpdates { user },
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
        info!("Unsubscribing from user non funding ledger update data");
        println!("\n[后台任务] 30秒计时结束，正在发送取消订阅请求...");
        info_client.unsubscribe(subscription_id).await.unwrap();
        println!("[后台任务] 取消订阅请求已发送。主程序的数据接收循环即将结束。");
    });
    println!("    - 已在后台启动一个30秒后自动取消订阅的计时器。");


    // ---- Part 3: 循环接收并处理消息 ----

    // 中文输出：告知用户进入接收循环
    println!("\n[步骤 4/4] 进入主循环，开始接收并打印实时账本更新... (将持续30秒)");
    
    // `while let` 循环会持续从 `receiver` 端接收消息。
    // 当订阅被取消后，`sender` 端会被关闭，`receiver.recv()` 会返回 `None`，循环自动结束。
    // this loop ends when we unsubscribe
    while let Some(Message::UserNonFundingLedgerUpdates(user_non_funding_ledger_update)) =
        receiver.recv().await
    {
        info!("Received user non funding ledger update data: {user_non_funding_ledger_update:?}");
        println!("【非资金费率账本更新】收到新消息: {:?}", user_non_funding_ledger_update);
    }
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("数据流已关闭，程序正常结束。");
    // =========================================================
}