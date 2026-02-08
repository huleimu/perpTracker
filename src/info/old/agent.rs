//! # Hyperliquid 代理创建与下单示例
//!
//! 该文件展示了如何通过 `hyperliquid-rust-sdk` 与 Hyperliquid 交易所的测试网进行交互。
//! 其核心流程分为两个主要步骤：
//!
//! 1.  **创建代理账户**: 使用一个主账户的私钥，请求 Hyperliquid API 创建一个关联的“代理”账户。
//!     此代理账户拥有自己的私钥，但权限受限（例如，只能交易，不能提现），这是一种增强自动化策略安全性的推荐做法。
//!
//! 2.  **使用代理下单**: 获取到代理账户的私钥后，程序会使用该代理的身份，在测试网上提交一个限价买单。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 如果不设置，程序依然会成功执行所有网络请求，但你不会在控制台看到任何日志。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin agent
//! ```

use log::info;

use ethers::signers::{LocalWallet, Signer};
use hyperliquid_rust_sdk::{BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    // 它会根据 RUST_LOG 环境变量来决定显示哪些级别的日志。
    // 如果未设置该变量，默认情况下 `info!` 级别的日志不会被打印。
    env_logger::init();

    // ---- Part 1: 使用主账户创建代理 ----

    // 定义一个主钱包。此钱包拥有账户的完全控制权。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    // 在生产环境中，切勿硬编码私钥。
    // 从一个硬编码的私钥定义主钱包。这个钱包拥有账户的全部权限。
    // 警告：此私钥仅用于演示目的，不包含任何真实资产。
    // 切勿在生产代码中硬编码或暴露你的真实私钥。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"   // 主钱包的私钥
        .parse()
        .unwrap();

    // 使用主钱包创建一个 ExchangeClient 实例，并连接到 Hyperliquid 的测试网 (Testnet)。
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    /*
        创建一个新的代理钱包。
        这个代理不能转移或提取资金，但可以例如下订单。
    */

    // 调用 `approve_agent` 方法，为当前主账户创建一个新的代理。
    // API 会返回新代理的私钥和创建成功的响应信息。
    info!("正在请求创建代理账户...");
    let (private_key, response) = exchange_client.approve_agent(None).await.unwrap();
     info!("代理账户创建成功: {response:?}");

    // 使用 info! 宏记录代理创建的响应。
    info!("Agent creation response: {response:?}");

    // ---- Part 2: 使用新创建的代理进行下单 ----

    // 使用上一步获得的代理私钥，创建一个新的 LocalWallet 实例。
    // 注意：这里的 `wallet` 变量通过“遮蔽”(shadowing)被重新赋值。
    // 它现在代表的是代理钱包，而不是之前的主钱包。
    let wallet: LocalWallet = private_key.parse().unwrap();

    // 记录新代理钱包的以太坊地址。
    info!("Agent address: {:?}", wallet.address());

    // 再次创建一个 ExchangeClient 实例。
    // 注意：这里的 `exchange_client` 变量也被“遮蔽”了。
    // 这个新的客户端实例是使用代理钱包进行身份验证的，因此后续所有操作都将以代理的身份执行。
    // 创建一个新的 ExchangeClient 实例，这次使用代理钱包进行身份验证。
    // 此后，所有通过 `agent_client` 发出的操作都将由代理执行。
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // 构建一个下單请求的结构体。
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),      // 交易资产为 ETH
        is_buy: true,                  // 订单方向为买入
        reduce_only: false,            // 不是一个只减仓订单（意味着可以用来开仓）
        limit_px: 1795.0,              // 限价价格为 1795.0
        sz: 0.01,                      // 订单大小为 0.01
        cloid: None,                   // 客户端自定义订单ID，这里不设置
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),    // 订单有效期类型: "Good 'Til Canceled" (直到取消前一直有效)
        }),
    };

    // 使用代理身份的客户端，将上面定义的订单发送到交易所。
    // 使用代理客户端将订单请求发送到交易所。
    info!("正在以代理身份下单...");
    let response = exchange_client.order(order, None).await.unwrap();

    // 记录下单成功后，交易所返回的响应信息。
    info!("Order placed: {response:?}");
    info!("下单成功，交易所响应: {response:?}");
}