//! # Hyperliquid 金库 (Vault) 存取款示例
//!
//! 该文件展示了如何使用 `hyperliquid-rust-sdk` 与 Hyperliquid 的**金库 (Vaults)** 进行交互。
//!
//! 金库是 Hyperliquid 上的自动化投资策略（例如，做市策略）。用户可以将资金存入金库，
//! 由金库的智能合约代为执行交易，用户则分享策略产生的收益。
//!
//! 本示例的核心功能是调用 `vault_transfer` 方法，它演示了如何将资金从用户的**主账户**
//! **存入 (Deposit)** 到一个指定的金库中。
//!
//! **特别注意**:
//! 1.  `is_deposit` 参数控制是存入还是取出。
//! 2.  金额 `usd` 的单位是**微美元** (1,000,000 微美元 = 1 美元)。
//!
//! ## 关于输出的说明
//!
//! 这段代码使用了 `log::info!` 来打印信息。要看到这些输出，你必须在运行时设置 `RUST_LOG` 环境变量。
//! 同时，为了方便观察，代码中也加入了 `println!` 用于直接在控制台输出中文提示和查询结果。
//!
//! ### 如何运行并查看输出：
//! ```bash
//! RUST_LOG=info cargo run
//! ```

use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    // 初始化 env_logger 日志记录器。
    env_logger::init();

    // ==================== 中文流程输出开始 ====================
    println!("程序启动：开始执行 Hyperliquid 金库(Vault)转账操作...");
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


    // ---- Part 2: 定义并执行金库转账 ----

    // 中文输出：告知用户正在设置转账参数
    println!("\n[步骤 3/4] 金库转账参数设置完毕：");
    
    // 定义要存入的金额。
    // !! 重要 !!: 这里的单位是微美元 (micros)。1,000,000 微美元 = 1 美元。
    // 所以 5,000,000 代表 5 USDC。
    let usd = 5_000_000; // at least 5 USD
    
    // 定义转账方向。`true` 表示存入 (Deposit) 金库，`false` 表示从金库取出 (Withdraw)。
    let is_deposit = true;

    println!("    - 转账方向: {}", if is_deposit { "存入金库 (Deposit)" } else { "从金库取出 (Withdraw)" });
    println!("    - 转账金额: {} 微美元 (即 {} USDC)", usd, usd / 1_000_000);
    println!("    - 目标金库地址: 0x1962905b0a2d0ce7907ae1a0d17f3e4a1f63dfb7");
    
    // 中文输出：告知用户即将发送请求
    println!("[步骤 4/4] 正在向交易所发送金库【存款】请求，请稍候...");

    // 调用 `vault_transfer` 方法，发起一个向指定金库存款的请求。
    let res = exchange_client
        .vault_transfer(
            is_deposit, // 参数1: 转账方向 (true = 存入)
            usd,        // 参数2: 金额（微美元单位）
            Some(       // 参数3: 目标金库的地址。`Some` 表示我们明确指定了一个金库。
                "0x1962905b0a2d0ce7907ae1a0d17f3e4a1f63dfb7"
                    .parse()
                    .unwrap(),
            ),
            None,       // 参数4: 可选参数
        )
        .await
        .unwrap();
    info!("Vault transfer result: {res:?}");
    
    // ==================== 中文流程输出结束 ====================
    println!("-------------------------------------------------");
    println!("金库转账请求已成功发送！");
    println!("\n以下是交易所返回的详细响应：");
    println!("{res:?}");
    // =========================================================
}