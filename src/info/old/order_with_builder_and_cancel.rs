//! # Hyperliquid 限价单下单（使用构建者）与按`oid`取消示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 完成一个集成了**构建者（Builder）**的传统订单管理流程。
//!
//! 它结合了多个核心概念：
//! 1.  **带构建者下单**: 使用 `order_with_builder` 方法发送一个限价单，同时将此操作归功于一个构建者并为其指定费用。
//! 2.  **解析响应，获取`oid`**: 程序等待并解析交易所的响应，从中提取出由交易所分配的唯一订单ID (`oid`)。
//! 3.  **等待**: 程序暂停一段时间。
//! 4.  **按`oid`取消**: 使用上一步获取的 `oid`，精确地取消指定的那个挂单。
//!
//! 这个流程演示了构建者机制的灵活性——它不仅可以用于市价单，也可以用于限价单，非常适合那些提供挂单策略的社交/跟单交易场景。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//!  $env:RUST_LOG="info"; cargo run --bin       order_with_builder_and_cancel
//! ```

use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, BuilderInfo, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest,
    ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 按交易所ID(oid)下单（使用构建者）与取消的流程...");
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


    // ---- Part 2: 使用构建者下单并提取 oid ----
    
    // 中文输出：告知用户正在下单
    println!("\n[步骤 3/6] 正在发送限价单（并指定构建者）...");

    // 定义限价单请求
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

    // 定义构建者信息
    let fee = 1u64; // 支付给构建者的费用，单位是基点 (basis points)。1 表示 0.01%。
    let builder = "0x1ab189B7801140900C711E458212F9c76F8dAC79";

    // 使用 `order_with_builder` 方法发送下单请求
    let response = exchange_client
        .order_with_builder(
            order,      // 参数1: 订单详情
            None,       // 参数2: 可选的签名钱包（默认为客户端钱包）
            BuilderInfo { // 参数3: 构建者信息
                builder: builder.to_string(),
                fee,
            },
        )
        .await
        .unwrap();
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
    
    // 从订单状态中提取出交易所分配的 oid
    let oid = match status {
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
        oid, // 使用我们从响应中提取的 oid
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