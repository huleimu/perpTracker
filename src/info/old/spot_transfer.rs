//! # Hyperliquid 现货资产转账示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 从 Hyperliquid 交易所的测试网账户中，
//! 将一个**特定的现货资产**（Spot Asset）转移到外部的钱包地址。
//!
//! 它的核心功能是调用 `spot_transfer` 方法。与之前用于提取主要抵押品（USDC）的 `withdraw_from_bridge` 不同，
//! `spot_transfer` 专门用于提取在 Hyperliquid 现货市场上交易的其他代币，如本例中的 "PURR"。
//!
//! 这模拟了一个真实的用户场景：将在现货市场买入的某个山寨币或特定代币，从交易所提取到自己的个人钱包中。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin         spot_transfer
//! ```

use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 现货资产转账操作...");
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


    // ---- Part 2: 定义并执行现货转账 ----

    // 中文输出：告知用户正在设置转账参数
    println!("\n[步骤 3/4] 转账参数设置完毕：");
    
    // 定义要转账的现货资产数量。
    let amount = "1";
    // 定义接收资金的目标地址。
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";
    // **核心**: 定义要转账的代币。
    // 格式为 "TICKER:CONTRACT_ADDRESS"，用于唯一标识一个特定的现货资产。
    let token = "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2";
    
    println!("    - 转账数量: {}", amount);
    println!("    - 转账代币: {}", token);
    println!("    - 目标地址: {}", destination);
    
    // 中文输出：告知用户即将发送请求
    println!("[步骤 4/4] 正在向交易所发送现货转账请求，请稍候...");

    // 调用 `spot_transfer` 方法，发起一个特定现货资产的提现请求。
    let res = exchange_client
        .spot_transfer(
            amount,           // 参数1: 要转账的数量 (字符串形式)
            destination,      // 参数2: 接收资金的外部钱包地址
            token,            // 参数3: 要转账的代币及其合约地址
            None              // 参数4: 可选参数
        )
        .await
        .unwrap();
    info!("Spot transfer result: {res:?}");
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("现货转账请求已成功发送！");
    println!("\n以下是交易所返回的详细响应：");
    println!("{res:?}");
    // =========================================================
}