//! # Hyperliquid 杠杆与保证金管理示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 在 Hyperliquid 的测试网上管理一个已有仓位的风险参数。
//!
//! 其核心流程包括三个部分：
//! 1.  **更新杠杆 (Update Leverage)**: 为指定的交易对（如 "ETH"）设置新的杠杆倍数。
//! 2.  **更新独立保证金 (Update Isolated Margin)**: 为一个已存在的独立仓位增加或减少保证金。
//! 3.  **验证状态 (Verify State)**: 在执行操作后，使用 `InfoClient` 查询用户的账户状态，以确认更改是否生效。
//!
//! **重要前提**: 这个示例假设你的账户中**已经有一个 ETH 的持仓**，因为调整保证金的操作是针对现有仓位的。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin       leverage
//! ```

use ethers::signers::{LocalWallet, Signer};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, InfoClient};
use log::info;

#[tokio::main]
async fn main() {
    // 示例假设你的 ETH 已经有仓位，所以你可以更新保证金
    env_logger::init();

    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 杠杆与保证金管理操作...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 初始化客户端 ----

    // 定义一个主钱包。此钱包拥有账户的完全控制权。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    // 中文输出：告知用户正在初始化钱包
    println!("[步骤 1/5] 正在使用测试私钥初始化钱包...");
    
    let address = wallet.address();

    // 创建 ExchangeClient，用于执行需要签名的写操作（如更新杠杆和保证金）。
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // 创建 InfoClient，用于执行只读的查询操作（如获取用户状态）。
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    // 中文输出：告知用户已成功连接
    println!("[步骤 2/5] 成功创建 ExchangeClient 和 InfoClient，并连接到测试网。");
    

    // ---- Part 2: 修改风险参数 ----

    // 中文输出：告知用户即将更新杠杆
    println!("[步骤 3/5] 正在发送请求：将 ETH 的杠杆更新为 5x (非全仓模式)...");
    
    // 调用 `update_leverage` 方法，设置杠杆。
    let response = exchange_client
        .update_leverage(
            5,       // 参数1: 新的杠杆倍数
            "ETH",   // 参数2: 应用此杠杆的资产
            false,   // 参数3: is_cross (是否为全仓)，false 表示这是为独立仓位模式设置杠杆
            None     // 参数4: 可选参数
        )
        .await
        .unwrap();
    
    info!("Update leverage response: {response:?}");
    println!("更新杠杆成功！交易所响应: {:?}", response);

    // 中文输出：告知用户即将更新保证金
    println!("\n[步骤 4/5] 正在发送请求：为 ETH 的独立仓位增加 1.0 USDC 的保证金...");

    // 调用 `update_isolated_margin` 方法，调整独立仓位的保证金。
    // 注意：这要求你必须已经有一个 ETH 的独立仓位。
    let response = exchange_client
        .update_isolated_margin(
            1.0,     // 参数1: 要增加(正数)或减少(负数)的保证金金额
            "ETH",   // 参数2: 目标仓位的资产
            None     // 参数3: 可选参数
        )
        .await
        .unwrap();

    info!("Update isolated margin response: {response:?}");
    println!("更新独立保证金成功！交易所响应: {:?}", response);


    // ---- Part 3: 验证结果 ----

    // 中文输出：告知用户正在查询最终状态
    println!("\n[步骤 5/5] 正在查询操作完成后的最终用户账户状态...");

    // 使用 InfoClient 查询当前用户的账户状态，以验证上述操作是否生效。
    let user_state = info_client.user_state(address).await.unwrap();
    info!("User state: {user_state:?}");

    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("所有操作已成功发送！");
    println!("\n以下是最终的用户账户状态：");
    println!("{user_state:#?}");
    // =========================================================
}