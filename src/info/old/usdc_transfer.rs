//! # Hyperliquid 内部 USDC 转账示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 在 Hyperliquid 平台内部，将 USDC 从当前账户
//! 转移到另一个指定的 Hyperliquid 用户账户。
//!
//! 它的核心功能是调用 `usdc_transfer` 方法。这个功能与之前的几种转账有所不同：
//! -   `withdraw_from_bridge`: 是**提现**，将资金从交易所提到外部钱包。
//! -   `class_transfer`: 是**内部划转**，在自己的现货和合约账户之间移动资金。
//! -   `usdc_transfer`: 是**用户间转账**，在 Hyperliquid 平台内部从一个用户转给另一个用户，通常是即时到账的。
//!
//! 这模拟了一个“站内转账”或“内部转账”的用户场景。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin            usdc_transfer
//! ```

use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 内部USDC转账操作...");
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


    // ---- Part 2: 定义并执行 USDC 转账 ----

    // 中文输出：告知用户正在设置转账参数
    println!("\n[步骤 3/4] 转账参数设置完毕：");
    
    // 定义要转账的 USDC 金额。
    let amount = "1"; // 1 USD
    // 定义接收资金的目标用户地址。
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";
    
    println!("    - 转账数量: {} USDC", amount);
    println!("    - 目标用户地址: {}", destination);
    
    // 中文输出：告知用户即将发送请求
    println!("[步骤 4/4] 正在向交易所发送【平台内部】用户间转账请求，请稍候...");

    // 调用 `usdc_transfer` 方法，发起一个平台内部的用户间转账请求。
    let res = exchange_client
        .usdc_transfer(
            amount,           // 参数1: 要转账的数量 (字符串形式)
            destination,      // 参数2: 接收资金的【另一个Hyperliquid用户】的地址
            None              // 参数3: 可选参数
        )
        .await
        .unwrap();
    info!("Usdc transfer result: {res:?}");
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("内部转账请求已成功发送！");
    println!("\n以下是交易所返回的详细响应：");
    println!("{res:?}");
    // =========================================================
}