//! # Hyperliquid 市价开仓与平仓完整交易周期示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 在 Hyperliquid 的测试网上完成一个完整的交易流程：
//! 1.  **市价开仓 (Market Open)**: 使用市价单（以当前市场最优价立即成交）建立一个头寸。
//! 2.  **等待 (Wait)**: 程序会暂停一小段时间，模拟真实交易中持仓的场景。
//! 3.  **市价平仓 (Market Close)**: 使用市价单平掉之前建立的全部头寸。
//!
//! 与之前的示例不同，本文件还详细展示了如何**解析交易所返回的响应 (Response)**，
//! 从而可以精确地判断订单是被立即填充 (Filled) 还是进入订单簿等待 (Resting)，这是编写可靠交易逻辑的关键一步。
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
    BaseUrl, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, MarketCloseParams,
    MarketOrderParams,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();

    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 市价开仓与平仓的完整交易周期...");
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


    // ---- Part 2: 市价开仓 ----

    // 中文输出：告知用户即将开仓
    println!("\n[步骤 3/5] 正在发送市价单请求以开仓...");

    // 定义市价开仓订单的参数
    let market_open_params = MarketOrderParams {
        asset: "ETH",         // 交易资产
        is_buy: true,         // 方向：买入
        sz: 0.01,             // 数量：0.01 ETH
        px: None,             // 价格：市价单不指定价格，所以为 None
        slippage: Some(0.01), // 允许的最大滑点：1%。这是一个安全措施，防止成交价与预期相差过大。
        cloid: None,          // 客户端自定义订单ID，可选
        wallet: None,         // 签名钱包，可选（默认使用客户端的钱包）
    };

    // 发送市价开仓请求
    let response = exchange_client
        .market_open(market_open_params)
        .await
        .unwrap();
    info!("Market open order placed: {response:?}");
    println!("开仓请求已发送，正在解析交易所响应...");

    // 解析交易所返回的详细状态
    let response = match response {
        // API 请求成功，我们拿到了具体的响应数据
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        // API 请求本身就失败了
        ExchangeResponseStatus::Err(e) => panic!("交易所响应错误: {e}"),
    };
    // 从响应数据中取出第一个订单的状态
    let status = response.data.unwrap().statuses[0].clone();
    // 判断订单的具体执行状态
    match status {
        ExchangeDataStatus::Filled(order) => {
            info!("Order filled: {order:?}");
            println!("开仓订单已完全成交！成交详情: {:?}", order);
        },
        ExchangeDataStatus::Resting(order) => {
            info!("Order resting: {order:?}");
            println!("开仓订单已进入订单簿等待成交: {:?}", order);
        },
        _ => panic!("出现未预料的订单状态: {status:?}"),
    };

    // ---- Part 3: 持仓等待 ----

    // 中文输出：告知用户正在等待
    println!("\n[步骤 4/5] 开仓成功，程序将等待 10 秒后进行平仓...");
    // 等待 10 秒，模拟持仓一段时间
    sleep(Duration::from_secs(10));


    // ---- Part 4: 市价平仓 ----
    
    // 中文输出：告知用户即将平仓
    println!("\n[步骤 5/5] 等待结束，正在发送市价单请求以平仓...");

    // 定义市价平仓订单的参数
    let market_close_params = MarketCloseParams {
        asset: "ETH",         // 要平仓的资产
        sz: None,             // 数量：为 None 表示平掉该资产的【全部】仓位
        px: None,             // 价格：市价单不指定价格，所以为 None
        slippage: Some(0.01), // 允许的最大滑点：1%
        cloid: None,          // 客户端自定义订单ID，可选
        wallet: None,         // 签名钱包，可选
    };

    // 发送市价平仓请求
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
    println!("完整的交易周期演示完毕。");
    // =========================================================
}