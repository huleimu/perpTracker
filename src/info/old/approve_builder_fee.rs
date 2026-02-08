//! # Hyperliquid 构建者授权与费用设置示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 在 Hyperliquid 交易所的测试网上执行一个**授权操作**。
//!
//! 具体来说，它授权一个指定的钱包地址（称为“构建者”或 “Builder”）未来可以代表你的账户进行交易。
//! 同时，它还为你愿意支付给该构建者的服务设定了一个**最高手续费率**。
//!
//! 这是实现**社交交易 (Social Trading)** 或**跟单交易 (Copy Trading)** 的关键准备步骤，
//! 允许用户在不暴露私钥的情况下，安全地委托专业交易员或策略进行操作。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 如果不设置，程序依然会成功执行所有网络请求，但你不会在控制台看到任何日志。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin approve_builder_fee
//! ```

use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    // 它会根据 RUST_LOG 环境变量来决定显示哪些级别的日志。
    env_logger::init();

    // ---- Part 1: 初始化客户端 ----

    // 定义一个主钱包。此钱包拥有账户的完全控制权。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    // 使用钱包创建一个 ExchangeClient 实例，并连接到 Hyperliquid 的测试网 (Testnet)。
    // `wallet.clone()` 创建了钱包的一个副本，用于初始化客户端，同时保留原始钱包的所有权以备后用。
    let exchange_client =
        ExchangeClient::new(None, wallet.clone(), Some(BaseUrl::Testnet), None, None)
            .await
            .unwrap();


    // ---- Part 2: 定义并执行授权 ----

    // 设置你愿意支付给“构建者”的最高费用比率。
    // "0.1%" 意味着对于构建者为你执行的每笔交易，你最多支付其名义价值的0.1%作为报酬。
    let max_fee_rate = "0.1%";

    // 定义“构建者”(Builder)的钱包地址。
    // 这是你希望授权的那个交易员、策略或机器人的地址。
    // `.to_lowercase()` 是一个很好的实践，可以确保地址格式一致，避免因大小写问题导致错误。
    let builder = "0x1ab189B7801140900C711E458212F9c76F8dAC79".to_lowercase();


    // 调用 `approve_builder_fee` 方法，向 Hyperliquid 的智能合约发送一个授权交易。
    // 这相当于在链上签署一份声明，允许 `builder` 地址代表你进行交易，并约定了费用上限。
    let resp = exchange_client
        .approve_builder_fee(
            builder.to_string(),      // 参数1: 你要授权的构建者地址。
            max_fee_rate.to_string(), // 参数2: 你设定的最高费用比率。
            Some(&wallet)             // 参数3: 签名钱包。这个操作必须由你自己的钱包签名，以证明你同意此授权。
        )
        .await; // 异步等待网络请求完成。

    // 记录授权操作完成后，交易所API返回的响应信息。
    // 使用 `{resp:#?}` 可以让输出的结构体格式化，更易于阅读。
    info!("resp: {resp:#?}");
}