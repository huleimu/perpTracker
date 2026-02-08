//! # Hyperliquid 自动化做市商 (Market Maker) 示例
//!
//! 该文件展示了 `hyperliquid-rust-sdk` 中一个高级功能：一个基础的**自动化做市商 (Market Maker)** 机器人。
//!
//! 与之前执行单个操作的示例不同，这段代码会启动一个**持续运行的进程**。它的工作逻辑如下：
//! 1.  **订阅价格**: 机器人会实时监控指定资产（如 "ETH"）的中间价。
//! 2.  **提供流动性**: 它会根据配置，在中间价的两侧挂上买单（Bid）和卖单（Ask），从而为市场提供流动性。
//! 3.  **动态更新**: 当市场中间价发生变化，导致之前挂的订单“过时”时，机器人会自动取消旧订单，并围绕新的中间价挂上新订单。
//! 4.  **风险管理**: 它会控制自己的总持仓大小，避免承担过多风险。
//!
//! 本质上，这是一个试图通过赚取买卖价差（Spread）来盈利的自动化交易策略。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印机器人的内部运行状态。要看到这些详细输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出启动流程。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! $env:RUST_LOG="info"; cargo run --bin   market_maker
//! ```

use ethers::signers::LocalWallet;

use hyperliquid_rust_sdk::{MarketMaker, MarketMakerInput};

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();
    
    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始配置并启动 Hyperliquid 自动化做市商机器人...");
    println!("-------------------------------------------------");
    // =========================================================

    // ---- Part 1: 配置做市商参数 ----

    // 中文输出：告知用户正在初始化钱包
    println!("[步骤 1/2] 正在使用测试私钥初始化钱包...");

    // 定义一个钱包。机器人将使用这个钱包进行签名和交易。
    // !! 安全警告 !!: 这是一个公开的测试私钥，绝不应该用于存储任何真实资金。
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    // 中文输出：告知用户正在配置机器人
    println!("\n[步骤 2/2] 正在配置做市商机器人的运行参数...");

    // `MarketMakerInput` 结构体包含了机器人的所有核心配置。
    let market_maker_input = MarketMakerInput {
        // 目标资产：机器人将为 "ETH" 市场提供流动性。
        asset: "ETH".to_string(),
        // 目标流动性：每个订单的大小（以资产为单位）。这里是 0.25 ETH。
        target_liquidity: 0.25,
        // 最大基点差异：当市场中间价与机器人报价的中间价差异超过 2 个基点 (0.02%) 时，触发订单更新。
        max_bps_diff: 2,
        // 半价差：挂单价格与中间价的距离。这里是 1 美元。
        // 例如，如果中间价是 $2000，机器人会挂一个买单在 $1999，一个卖单在 $2001。
        half_spread: 1,
        // 最大绝对持仓大小：机器人的净头寸（多仓或空仓）不会超过 0.5 ETH，这是一个风险控制参数。
        max_absolute_position_size: 0.5,
        // 价格小数位数：挂单价格的精度。
        decimals: 1,
        // 用于交易的钱包。
        wallet,
    };
    
    // 中文输出：打印所有配置项
    println!("    - 目标资产: {}", market_maker_input.asset);
    println!("    - 目标流动性 (每单大小): {} {}", market_maker_input.target_liquidity, market_maker_input.asset);
    println!("    - 订单更新阈值 (基点): {}", market_maker_input.max_bps_diff);
    println!("    - 半价差 (挂单距离): {} USD", market_maker_input.half_spread);
    println!("    - 最大持仓限制: {} {}", market_maker_input.max_absolute_position_size, market_maker_input.asset);


    // ---- Part 2: 启动做市商 ----

    println!("\n-------------------------------------------------");
    println!("配置完成！正在启动做市商机器人...");
    println!("机器人将持续运行，请观察日志输出（需设置 RUST_LOG=info）。按 Ctrl+C 停止。");
    println!("-------------------------------------------------");

    // 创建一个 MarketMaker 实例并启动它。
    // `.start().await` 会启动一个无限循环，程序将在这里持续运行，直到被手动停止。
    MarketMaker::new(market_maker_input).await.start().await
}