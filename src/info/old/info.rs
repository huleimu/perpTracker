//! # Hyperliquid 信息查询客户端 (InfoClient) 示例
//!
//! 该文件是一个全面的示例，展示了如何使用 `hyperliquid-rust-sdk` 中的 `InfoClient`
//! 来查询 Hyperliquid 测试网上的各种公开信息。
//!
//! 与需要私钥进行签名操作的 `ExchangeClient` 不同，`InfoClient` 用于只读访问，
//! 不需要任何身份验证。它可以查询两大类信息：
//!
//! 1.  **市场全局信息**: 如所有交易对的中间价、元数据、L2订单簿快照、K线数据等。
//! 2.  **特定用户信息**: 如某个地址的未结订单、账户状态、历史成交、资金费率历史等。
//!
//! 这个文件通过一系列独立的函数，逐一演示了 `InfoClient` 的各种常用查询功能。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin       info 
//! ```

use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use log::info;

// 定义一个常量，用于查询特定用户的公开信息。
const ADDRESS: &str = "0xc64cc00b46101bd40aa1c3121195e85c0b0918d8";

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();

    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 信息查询...");
    println!("将要查询的测试地址为: {}", ADDRESS);
    println!("-------------------------------------------------");
    // =========================================================

    // 创建一个 InfoClient 实例，用于后续的所有信息查询。
    // 注意：它不需要钱包或私钥。
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    // 依次调用下方的各个示例函数来演示不同的查询功能。
    // 为了清晰，每个查询前都会有中文标题输出。
    
    println!("\n--- 正在查询: 用户未结订单 ---");
    open_orders_example(&info_client).await;

    println!("\n--- 正在查询: 单个用户状态 ---");
    user_state_example(&info_client).await;
    
    println!("\n--- 正在查询: 批量用户状态 ---");
    user_states_example(&info_client).await;
    
    println!("\n--- 正在查询: ETH 交易对的近期成交记录 ---");
    recent_trades(&info_client).await;
    
    println!("\n--- 正在查询: 交易所元数据 ---");
    meta_example(&info_client).await;
    
    println!("\n--- 正在查询: 所有交易对的中间价 ---");
    all_mids_example(&info_client).await;
    
    println!("\n--- 正在查询: 用户的历史成交记录 ---");
    user_fills_example(&info_client).await;
    
    println!("\n--- 正在查询: ETH 交易对的资金费率历史 ---");
    funding_history_example(&info_client).await;
    
    println!("\n--- 正在查询: ETH 交易对的 L2 订单簿快照 ---");
    l2_snapshot_example(&info_client).await;
    
    println!("\n--- 正在查询: ETH 交易对的 K 线快照 ---");
    candles_snapshot_example(&info_client).await;
    
    println!("\n--- 正在查询: 用户的代币余额 ---");
    user_token_balances_example(&info_client).await;

    println!("\n--- 正在查询: 用户的手续费率 ---");
    user_fees_example(&info_client).await;
    
    println!("\n--- 正在查询: 用户的资金费率收取历史 ---");
    user_funding_example(&info_client).await;
    
    println!("\n--- 正在查询: 现货市场的元数据 ---");
    spot_meta_example(&info_client).await;
    
    println!("\n--- 正在查询: 现货市场的元数据和资产背景 ---");
    spot_meta_and_asset_contexts_example(&info_client).await;
    
    println!("\n--- 正在查询: 通过订单ID查询订单状态 ---");
    query_order_by_oid_example(&info_client).await;

    println!("\n--- 正在查询: 用户的推荐关系状态 ---");
    query_referral_state_example(&info_client).await;
    
    println!("\n--- 正在查询: 用户的历史订单 ---");
    historical_orders_example(&info_client).await;

    println!("\n-------------------------------------------------");
    println!("所有信息查询演示完毕。");
}

// 辅助函数：将字符串地址转换为 SDK 需要的 H160 类型。
fn address() -> H160 {
    ADDRESS.to_string().parse().unwrap()
}

// 示例函数：获取指定用户的当前未结订单。
async fn open_orders_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.open_orders(user).await.unwrap();
    info!("Open order data for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取单个用户的账户状态（如保证金、仓位等）。
async fn user_state_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.user_state(user).await.unwrap();
    info!("User state data for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：批量获取多个用户的账户状态。
async fn user_states_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.user_states(vec![user]).await.unwrap();
    info!("User state data for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取用户的代币余额。
async fn user_token_balances_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.user_token_balances(user).await.unwrap();
    info!("User token balances data for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取用户的手续费等级和费率。
async fn user_fees_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.user_fees(user).await.unwrap();
    info!("User fees data for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取某个交易对的近期市场成交记录。
async fn recent_trades(info_client: &InfoClient) {
    let coin = "ETH";
    let result = info_client.recent_trades(coin.to_string()).await.unwrap();
    info!("Recent trades for {coin}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取交易所的元数据（包含所有可交易资产的信息）。
async fn meta_example(info_client: &InfoClient) {
    let result = info_client.meta().await.unwrap();
    info!("Metadata: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取所有交易对的当前中间价。
async fn all_mids_example(info_client: &InfoClient) {
    let result = info_client.all_mids().await.unwrap();
    info!("All mids: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取某个用户的历史成交记录。
async fn user_fills_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.user_fills(user).await.unwrap();
    info!("User fills data for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取某个交易对在指定时间范围内的资金费率历史。
async fn funding_history_example(info_client: &InfoClient) {
    let coin = "ETH";
    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    let result = info_client.funding_history(coin.to_string(), start_timestamp, Some(end_timestamp)).await.unwrap();
    info!("Funding data history for {coin} between timestamps {start_timestamp} and {end_timestamp}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取某个交易对的 Level 2 订单簿快照。
async fn l2_snapshot_example(info_client: &InfoClient) {
    let coin = "ETH";
    let result = info_client.l2_snapshot(coin.to_string()).await.unwrap();
    info!("L2 snapshot data for {coin}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取某个交易对在指定时间范围和间隔的 K 线数据。
async fn candles_snapshot_example(info_client: &InfoClient) {
    let coin = "ETH";
    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    let interval = "1h";
    let result = info_client
            .candles_snapshot(coin.to_string(), interval.to_string(), start_timestamp, end_timestamp)
            .await
            .unwrap();
    info!("Candles snapshot data for {coin} between timestamps {start_timestamp} and {end_timestamp} with interval {interval}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取某个用户在指定时间范围内的资金费率收取/支付历史。
async fn user_funding_example(info_client: &InfoClient) {
    let user = address();
    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    let result = info_client.user_funding_history(user, start_timestamp, Some(end_timestamp)).await.unwrap();
    info!("Funding data history for {user} between timestamps {start_timestamp} and {end_timestamp}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取现货市场的元数据。
async fn spot_meta_example(info_client: &InfoClient) {
    let result = info_client.spot_meta().await.unwrap();
    info!("SpotMeta: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：获取现货市场的元数据和资产背景信息。
async fn spot_meta_and_asset_contexts_example(info_client: &InfoClient) {
    let result = info_client.spot_meta_and_asset_contexts().await.unwrap();
    info!("SpotMetaAndAssetContexts: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：通过用户地址和订单ID查询特定订单的状态。
async fn query_order_by_oid_example(info_client: &InfoClient) {
    let user = address();
    let oid = 26342632321;
    let result = info_client.query_order_by_oid(user, oid).await.unwrap();
    info!("Order status for {user} for oid {oid}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：查询用户的推荐关系状态。
async fn query_referral_state_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.query_referral_state(user).await.unwrap();
    info!("Referral state for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}

// 示例函数：查询用户的历史订单（包括已完成和已取消的）。
async fn historical_orders_example(info_client: &InfoClient) {
    let user = address();
    let result = info_client.historical_orders(user).await.unwrap();
    info!("Historical orders for {user}: {:?}", result);
    println!("查询结果: {:?}", result);
}