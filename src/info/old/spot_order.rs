//! # Hyperliquid 现货市场下单与按`oid`取消（含逻辑错误）示例
//!
//! 该文件演示了在 Hyperliquid 测试网上下一个**现货市场 (Spot Market)** 的限价单，
//! 然后尝试用获取到的订单ID (`oid`) 去取消它。
//!
//! 然而，这个示例**故意包含了一个关键的逻辑错误**，用于教学目的：
//! -   下单时，资产是 `"XYZTWO/USDC"`。
//! -   取消时，指定的资产是 `"HFUN/USDC"`。
//!
//! 尽管取消时使用的 `oid` 是正确的，但由于资产不匹配，**取消操作注定会失败**。
//! 这强调了一个核心原则：订单的 `oid` 仅在其所属的资产（交易对）上下文中是唯一的和有效的。
//!
//! 最终，程序会在执行取消操作的 `.unwrap()` 时因 API 返回错误而**崩溃 (panic)**。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示。
//!
//! ### 如何运行并查看输出：
//! ```bash
//!  $env:RUST_LOG="info"; cargo run --bin      spot_order
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
    println!("程序启动：开始执行一个【包含逻辑错误】的订单取消流程...");
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


    // ---- Part 2: 在一个资产上下单并提取 oid ----
    
    // 中文输出：告知用户正在下单
    println!("\n[步骤 3/6] 正在为 'XYZTWO/USDC' 资产发送限价单...");

    // 定义限价单请求
    let order = ClientOrderRequest {
        asset: "XYZTWO/USDC".to_string(), // **注意这里的资产**
        is_buy: true,
        reduce_only: false,
        limit_px: 0.00002378,
        sz: 1000000.0,
        cloid: None,
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
    
    // 从订单状态中提取出 oid
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("错误: 出现了未预料的订单状态: {status:?}"),
    };
    println!("    - 成功提取到 oid: {}", oid);


    // ---- Part 3: 等待观察 ----

    // 中文输出：告知用户正在等待
    println!("\n[步骤 5/6] 下单成功，程序将等待 10 秒...");
    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));


    // ---- Part 4: 尝试在另一个资产上取消订单（注定失败）----
    
    // 中文输出：告知用户即将取消订单
    println!("\n[步骤 6/6] 等待结束，正在尝试使用错误的资产名称 'HFUN/USDC' 取消该订单...");
    println!("    !! 逻辑错误 !! 这里的资产与下单时的资产不匹配，因此操作将会失败。");
    
    // 构建一个按 oid 取消的请求
    let cancel = ClientCancelRequest {
        asset: "HFUN/USDC".to_string(), // !! 错误 !!: 这里的资产与下单时的资产 'XYZTWO/USDC' 不匹配
        oid,                            // oid 是正确的，但它只在 'XYZTWO/USDC' 资产下有效
    };

    // 发送取消请求。
    // 因为资产不匹配，交易所会找不到这个订单，API会返回一个错误。
    // `.unwrap()` 会在接收到错误时导致程序崩溃 (panic)。
    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel(cancel, None).await.unwrap();
    info!("Order potentially cancelled: {response:?}");

    // ==================== 中文流程输出结束 ====================
    // 这部分代码实际上不会被执行，因为上面的 .unwrap() 会使程序提前崩溃。
    println!("-------------------------------------------------");
    println!("如果能看到这条消息，说明发生了意外。正常情况下程序应该已经因错误而终止。");
    println!("{response:?}");
    // =========================================================
}