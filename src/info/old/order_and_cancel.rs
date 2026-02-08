//! # Hyperliquid 限价单下单与按`oid`取消示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 完成一个传统的订单管理流程：
//! 1.  **下单**: 发送一个常规的限价单（不指定客户端ID）。
//! 2.  **解析响应，获取`oid`**: 核心步骤！程序必须等待并解析交易所的响应，从中提取出由交易所分配的唯一订单ID (`oid`)。
//! 3.  **等待**: 程序暂停一段时间，以便用户可以在交易所界面上看到这个挂单。
//! 4.  **按`oid`取消**: 使用上一步获取的 `oid`，精确地取消指定的那个挂单。
//!
//! 这演示了与上一个 `cloid` 示例不同的、更经典的交互模式。在这种模式下，程序需要依赖交易所的实时反馈来获取下一步操作所需的标识符。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//!  $env:RUST_LOG="info"; cargo run --bin    order_and_cancel
//! ```

use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 按交易所ID(oid)下单与取消的流程...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端 ----

    // 中文输出：告知用户正在初始化钱包
    println!("[步骤 1/6] 正在使用测试私钥初始化钱包...");
    
    // 定义一个主钱包。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    // 创建 ExchangeClient，用于执行需要签名的交易操作。
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // 中文输出：告知用户已成功连接
    println!("[步骤 2/6] 成功连接到 Hyperliquid 测试网。");


    // ---- Part 2: 下单并从响应中提取 oid ----
    
    // 中文输出：告知用户正在下单
    println!("\n[步骤 3/6] 正在发送限价单（不带cloid）...");

    // 定义限价单请求，注意 `cloid` 为 `None`
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0, // 挂一个远离市价的价格，确保它不会立即成交
        sz: 0.01,
        cloid: None, // 不使用客户端ID
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    // 发送下单请求
    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");
    println!("下单请求已发送，正在解析交易所响应以获取订单ID (oid)...");

    // 中文输出：告知用户正在解析响应
    println!("[步骤 4/6] 正在解析响应以提取 oid...");

    // 解析交易所返回的详细状态
    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("交易所响应错误: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    
    // **核心**: 从订单状态中提取出交易所分配的 oid
    let oid = match status {
        // 无论订单是已成交还是在挂单中，响应里都会包含 oid
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("错误: 出现了未预料的订单状态: {status:?}"),
    };
    println!("    - 成功提取到 oid: {}", oid);


    // ---- Part 3: 等待观察 ----

    // 中文输出：告知用户正在等待
    println!("\n[步骤 5/6] 下单成功，程序将等待 10 秒以便您在交易所界面观察到此挂单...");
    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));


    // ---- Part 4: 使用 oid 取消订单 ----
    
    // 中文输出：告知用户即将取消订单
    println!("\n[步骤 6/6] 等待结束，正在使用刚才获取的 oid 取消该订单...");
    
    // 构建一个按 oid 取消的请求
    let cancel = ClientCancelRequest {
        asset: "ETH".to_string(),
        oid, // **核心**: 使用我们从响应中提取的 oid
    };

    // 发送取消请求
    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel(cancel, None).await.unwrap();
    info!("Order potentially cancelled: {response:?}");
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("取消请求已成功发送！");
    println!("\n以下是交易所返回的详细响应（如果订单未成交，则此响应表示取消成功）：");
    println!("{response:?}");
    // =========================================================
}