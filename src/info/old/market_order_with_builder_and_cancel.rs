//! # Hyperliquid 市价开仓（使用构建者）与平仓示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 在 Hyperliquid 的测试网上完成一个使用了**构建者（Builder）**的交易流程。
//!
//! 它结合了之前两个示例的核心功能：
//! 1.  **带构建者开仓**: 使用 `market_open_with_builder` 方法，在通过市价单开仓的同时，将这次交易归功于一个“构建者”，并为其指定一笔费用。
//! 2.  **等待**: 程序暂停一段时间。
//! 3.  **常规平仓**: 使用标准的 `market_close` 方法平掉仓位。
//!
//! 这段代码演示了社交/跟单交易功能的实际应用：一个用户（我们）发起交易，但通过 `BuilderInfo` 参数，让系统知道这笔交易是由某个构建者（如一个策略提供者或信号源）促成的，并愿意为此支付费用。
//!
//! **重要前提**: 在真实场景中，为了让这笔费用能成功支付给构建者，用户需要**预先**调用 `approve_builder_fee` 方法授权过该构建者。
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

use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, BuilderInfo, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus,
    MarketCloseParams, MarketOrderParams,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();

    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 市价开仓（使用构建者）与平仓的交易周期...");
    println!("-------------------------------------------------");
    // =========================================================
    
    // ---- Part 1: 初始化客户端 ----
    
    // 中文输出：告知用户正在初始化钱包
    println!("[步骤 1/5] 正在使用测试私钥初始化钱包...");

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
    println!("[步骤 2/5] 成功连接到 Hyperliquid 测试网。");


    // ---- Part 2: 市价开仓 (使用构建者) ----
    
    // 中文输出：告知用户即将开仓
    println!("\n[步骤 3/5] 正在发送市价单请求以开仓（并指定构建者）...");

    // 定义市价开仓订单的参数，与常规开仓相同。
    let market_open_params = MarketOrderParams {
        asset: "ETH",
        is_buy: true,
        sz: 0.01,
        px: None,
        slippage: Some(0.01), // 1% slippage
        cloid: None,
        wallet: None,
    };

    // 定义构建者信息
    let fee = 1; // 支付给构建者的费用，单位是基点 (basis points)。1 表示 0.01%。
    let builder = "0x1ab189B7801140900C711E458212F9c76F8dAC79";

    // 使用 `market_open_with_builder` 方法发送开仓请求。
    // 这个方法除了包含订单本身，还附加了构建者的信息。
    let response = exchange_client
        .market_open_with_builder(
            market_open_params, // 参数1: 订单详情
            BuilderInfo {       // 参数2: 构建者信息
                builder: builder.to_string(),
                fee,
            },
        )
        .await
        .unwrap();
    info!("Market open order placed: {response:?}");
    println!("开仓请求已发送，正在解析交易所响应...");

    // 解析交易所返回的详细状态
    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("交易所响应错误: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => {
            info!("Order filled: {order:?}");
            println!("开仓订单已完全成交（通过构建者）！成交详情: {:?}", order);
        },
        ExchangeDataStatus::Resting(order) => {
            info!("Order resting: {order:?}");
            println!("开仓订单已进入订单簿等待成交（通过构建者）: {:?}", order);
        },
        _ => panic!("出现未预料的订单状态: {status:?}"),
    };

    // ---- Part 3: 持仓等待 ----
    
    // 中文输出：告知用户正在等待
    println!("\n[步骤 4/5] 开仓成功，程序将等待 10 秒后进行平仓...");
    // 等待 10 秒
    sleep(Duration::from_secs(10));


    // ---- Part 4: 市价平仓 ----
    
    // 中文输出：告知用户即将平仓
    println!("\n[步骤 5/5] 等待结束，正在发送【常规】市价单请求以平仓...");

    // 定义市价平仓订单的参数。
    // 注意：这里使用的是标准的平仓流程，没有指定构建者。
    let market_close_params = MarketCloseParams {
        asset: "ETH",
        sz: None, // 平掉该资产的全部仓位
        px: None,
        slippage: Some(0.01), // 1% slippage
        cloid: None,
        wallet: None,
    };

    // 发送常规的市价平仓请求
    let response = exchange_client
        .market_close(market_close_params)
        .await
        .unwrap();
    info!("Market close order placed: {response:?}");
    println!("平仓请求已发送，正在解析交易所响应...");

    // 再次解析响应
    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("交易所响应错误: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => {
            info!("Close order filled: {order:?}");
            println!("平仓订单已完全成交！成交详情: {:?}", order);
        },
        ExchangeDataStatus::Resting(order) => {
            info!("Close order resting: {order:?}");
            println!("平仓订单已进入订单簿等待成交: {:?}", order);
        },
        _ => panic!("出现未预料的订单状态: {status:?}"),
    };

    // ==================== 中文流程输出结束 ====================
    println!("\n-------------------------------------------------");
    println!("完整的（使用构建者开仓）交易周期演示完毕。");
    // =========================================================
}