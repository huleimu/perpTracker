//! # Hyperliquid 限价单下单与按`cloid`取消示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 完成一个更高级的订单管理流程：
//! 1.  **生成客户端ID**: 在下单前，程序使用 `uuid` 库生成一个全球唯一的标识符（`cloid`）。
//! 2.  **带ID下单**: 发送一个限价单，并将这个 `cloid` 附加到订单上。
//! 3.  **等待**: 程序暂停一段时间，以便用户可以在交易所界面上看到这个挂单。
//! 4.  **按ID取消**: 使用之前生成的 `cloid`，精确地取消指定的那个挂单。
//!
//! 使用 `cloid` 是一个非常强大的功能，它让交易机器人可以独立地、可靠地管理自己的订单，而无需依赖交易所返回的订单号（`oid`），大大简化了复杂策略的实现。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//!  $env:RUST_LOG="info"; cargo run --bin       order_and_cancel_cloid
//! ```

use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequestCloid, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
};
use std::{thread::sleep, time::Duration};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 按客户端ID下单与取消的流程...");
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


    // ---- Part 2: 使用自定义 cloid 下单 ----
    
    // 中文输出：告知用户正在生成cloid并下单
    println!("\n[步骤 3/5] 正在生成客户端订单ID (cloid) 并发送限价单...");
    
    // 生成一个版本4的UUID作为客户端订单ID (cloid)。
    // 这个ID由我们自己控制，用于唯一标识这个订单。
    let cloid = Uuid::new_v4();
    println!("    - 生成的 cloid: {}", cloid);
    
    // 定义限价单请求
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0, // 挂一个远离市价的价格，确保它不会立即成交
        sz: 0.01,
        cloid: Some(cloid), // **核心**: 将我们生成的 cloid 附加到订单上
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(), // Gtc: Good 'Til Canceled
        }),
    };

    // 发送下单请求
    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");
    println!("下单请求已发送，交易所响应: {:?}", response);


    // ---- Part 3: 等待观察 ----

    // 中文输出：告知用户正在等待
    println!("\n[步骤 4/5] 下单成功，程序将等待 10 秒以便您在交易所界面观察到此挂单...");
    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));


    // ---- Part 4: 使用 cloid 取消订单 ----
    
    // 中文输出：告知用户即将取消订单
    println!("\n[步骤 5/5] 等待结束，正在使用刚才的 cloid 取消该订单...");

    // 构建一个按 cloid 取消的请求
    let cancel = ClientCancelRequestCloid {
        asset: "ETH".to_string(), // 要取消的订单所在的资产
        cloid,                    // **核心**: 使用我们之前保存的 cloid
    };

    // 发送取消请求
    // 注意：如果在这10秒内订单被市场意外成交了，那么取消操作会返回一个错误，因为已成交的订单无法取消。
    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel_by_cloid(cancel, None).await.unwrap();
    info!("Order potentially cancelled: {response:?}");
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("取消请求已成功发送！");
    println!("\n以下是交易所返回的详细响应（如果订单未成交，则此响应表示取消成功）：");
    println!("{response:?}");
    // =========================================================
}