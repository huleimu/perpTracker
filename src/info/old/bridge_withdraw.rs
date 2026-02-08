//! # Hyperliquid 跨链桥提现示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 从 Hyperliquid 交易所的测试网账户中提现资金。
//!
//! 这个操作通过 Hyperliquid 的内置**跨链桥**完成，它将账户中的美元稳定币 (USDC)
//! 从 Hyperliquid 的 L1 区块链转移到外部的 EVM 兼容链（通常是 Arbitrum）上的一个指定钱包地址。
//!
//! 这模拟了一个真实的用户场景：将交易利润从交易所提取到自己的个人钱包（如 MetaMask）。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 如果不设置，程序依然会成功执行所有网络请求，但你不会在控制台看到任何日志。
//!
//! ### 如何运行并查看输出：
//! ```bash
//!  $env:RUST_LOG="info"; cargo run --bin      bridge_withdraw
//! ```

use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    // 它会根据 RUST_LOG 环境变量来决定显示哪些级别的日志。
    env_logger::init();

    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 跨链桥提现操作...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端 ----

    // 定义一个主钱包。此钱包拥有账户的完全控制权。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();
    
    // 中文输出：告知用户正在初始化钱包
    println!("[步骤 1/4] 正在使用测试私钥初始化钱包...");

    // 使用钱包创建一个 ExchangeClient 实例，并连接到 Hyperliquid 的测试网 (Testnet)。
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // 中文输出：告知用户已成功连接
    println!("[步骤 2/4] 成功连接到 Hyperliquid 测试网。");


    // ---- Part 2: 定义并执行提现 ----

    // 定义提现金额。这里是 "5" 美元。
    let usd = "5"; // 5 USD

    // 定义接收资金的目标地址。
    // 这必须是你希望接收资金的那个钱包地址，通常位于 Arbitrum 网络上。
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";

    // 中文输出：告知用户提现参数
    println!("[步骤 3/4] 提现参数设置完毕：");
    println!("    - 提现金额: {} 美元", usd);
    println!("    - 目标地址: {}", destination);

    // 中文输出：告知用户即将发送请求
    println!("[步骤 4/4] 正在向交易所发送提现请求，请稍候...");


    // 调用 `withdraw_from_bridge` 方法，向 Hyperliquid 发起一个提现请求。
    // 这相当于在链上签署一份声明，允许 `builder` 地址代表你进行交易，并约定了费用上限。
    let res = exchange_client
        .withdraw_from_bridge(
            usd,          // 参数1: 要提现的金额 (字符串形式)
            destination,  // 参数2: 接收资金的外部钱包地址
            None          // 参数3: 可选参数，这里为 None
        )
        .await        // 异步等待网络请求完成。
        .unwrap();    // 如果请求出错则程序会恐慌(panic)。

    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("提现请求已成功发送！");
    println!("\n以下是交易所返回的详细响应：");
    // =========================================================

    // 记录提现请求的响应结果
    // `res` 变量会包含 Hyperliquid API 的返回信息，确认提现请求已被系统接收和处理。
    info!("Withdraw from bridge result: {res:?}");

    // 添加一个 println! 以确保无论是否设置 RUST_LOG 都能看到最终结果。
    println!("{res:?}");
}