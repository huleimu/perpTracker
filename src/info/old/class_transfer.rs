//! # Hyperliquid 内部账户划转示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 在 Hyperliquid 交易所的测试网账户内部进行资金划转。
//!
//! 这个操作通过 `class_transfer` 函数完成。在 Hyperliquid 中，用户的资金可能存放在两个不同的“类别”或“子账户”中：
//! 1.  **主账户/现货账户 (Spot Account)**: 用于存放入金和保管资金。
//! 2.  **永续合约账户 (Perpetuals Account)**: 用于交易永续合约的保证金。
//!
//! 这段代码演示了如何将资金从**永续合约账户**划转回**主账户**。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 如果不设置，程序依然会成功执行所有网络请求，但你不会在控制台看到任何日志。
//!
//! ### 如何运行并查看输出：
//! ```bash
//!    $env:RUST_LOG="info"; cargo run --bin      class_transfer
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
    println!("程序启动：开始执行 Hyperliquid 内部账户划转操作...");
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


    // ---- Part 2: 定义并执行划转 ----

    // 定义要划转的 USDC 金额。
    let usdc = 1.0; // 1 USD

    // 定义划转方向。`to_perp` 意为 "to perpetuals" (到永续合约账户)。
    // `false` 表示方向是反向的，即从永续合约账户划转到主账户。
    // `true` 则表示从主账户划转到永续合约账户。
    let to_perp = false;

    // 中文输出：告知用户划转参数
    println!("[步骤 3/4] 划转参数设置完毕：");
    println!("    - 划转金额: {} USDC", usdc);
    let direction_desc = if to_perp { "从 主账户 划转到 永续合约账户" } else { "从 永续合约账户 划转到 主账户" };
    println!("    - 划转方向: {}", direction_desc);

    // 中文输出：告知用户即将发送请求
    println!("[步骤 4/4] 正在向交易所发送内部划转请求，请稍候...");


    // 调用 `class_transfer` 方法，向 Hyperliquid 发起一个内部资金划转请求。
    let res = exchange_client
        .class_transfer(
            usdc,     // 参数1: 要划转的金额
            to_perp,  // 参数2: 划转方向的布尔标志
            None      // 参数3: 可选参数，这里为 None
        )
        .await    // 异步等待网络请求完成。
        .unwrap();  // 如果请求出错则程序会恐慌(panic)。

    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("内部划转请求已成功发送！");
    println!("\n以下是交易所返回的详细响应：");
    // =========================================================

    // 记录划转请求的响应结果。
    info!("Class transfer result: {res:?}");

    // 添加一个 println! 以确保无论是否设置 RUST_LOG 都能看到最终结果。
    println!("{res:?}");
}