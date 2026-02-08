//! # Hyperliquid 设置推荐人 (Referrer) 示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 为当前账户设置一个**推荐人（Referrer）**。
//!
//! 它的核心功能是调用 `set_referrer` 方法，将当前用户的账户与一个推荐码关联起来。
//! 这在交易所的**推荐计划**中非常常见：
//! -   当一个新用户（或未设置过推荐人的用户）设置了推荐码后，推荐人（即推荐码的所有者）
//!     可以在该用户未来的交易中获得一定比例的手续费返佣。
//!
//! 与之前的示例不同，这段代码还展示了如何**安全地处理 API 的响应**，
//! 使用 `if let Ok(res) = res` 结构来分别处理成功和失败的情况，而不是直接使用 `.unwrap()`，
//! 这是一种更稳健的编程实践。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 和 `println!` 来打印信息。要看到 `info!` 的输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin    set_referrer
//! ```

use ethers::signers::LocalWallet;

use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 设置推荐人操作...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端 ----

    // 中文输出：告知用户正在初始化钱包
    println!("[步骤 1/4] 正在使用测试私钥初始化钱包...");

    // 定义一个主钱包。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    // 创建 ExchangeClient，用于执行需要签名的操作。
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // 中文输出：告知用户已成功连接
    println!("[步骤 2/4] 成功连接到 Hyperliquid 测试网。");


    // ---- Part 2: 设置推荐人 ----
    
    // 中文输出：告知用户正在设置推荐码
    println!("\n[步骤 3/4] 准备设置推荐码...");

    // 定义要设置的推荐码。
    let code = "TESTNET".to_string();
    println!("    - 目标推荐码: {}", code);

    // 中文输出：告知用户即将发送请求
    println!("[步骤 4/4] 正在向交易所发送设置推荐人的请求...");
    
    // 调用 `set_referrer` 方法。
    // 注意：这个操作通常只能成功执行一次。如果该账户已经设置过推荐人，API会返回错误。
    let res = exchange_client.set_referrer(code, None).await;


    // ---- Part 3: 处理并打印响应 ----
    
    // 这是一个更健壮的错误处理方式。
    // 我们检查 `res` 变量（它是一个 `Result` 类型）是成功 (`Ok`) 还是失败 (`Err`)。
    if let Ok(res) = res {
        // 如果成功
        info!("Exchange response: {res:#?}");
        
        // ==================== 中文流程输出 ====================
        println!("-------------------------------------------------");
        println!("设置推荐人成功！");
        println!("\n以下是交易所返回的详细响应：");
        println!("{res:#?}");
        // =======================================================
        
    } else {
        // 如果失败
        let error_details = res.err().unwrap();
        info!("Got error: {:#?}", error_details);

        // ==================== 中文流程输出 ====================
        println!("-------------------------------------------------");
        println!("设置推荐人失败！这通常是因为该账户已经设置过推荐人，或者推荐码无效。");
        println!("\n以下是交易所返回的错误详情：");
        println!("{:#?}", error_details);
        // =======================================================
    }
}