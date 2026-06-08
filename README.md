# perpTracker

`perpTracker` 是一个基于 Rust 的 Hyperliquid 永续合约交易监控与跟单工具集，用于追踪地址交易行为、汇总盈亏、管理监控钱包、执行跟单策略，以及采集指定资产价格数据。

项目采用多二进制入口设计，既支持通过交互式主程序统一调度，也支持按服务独立运行。

## Features

- 实时监控指定地址的永续合约交易事件
- 将交易事件、当前盈亏和历史收益写入 MySQL
- 管理钱包监控列表，支持手动维护与文件批量导入
- 根据配置策略执行跟单交易
- 采集指定资产的价格数据并持久化
- 输出控制台日志和结构化 JSON 日志，便于排查和分析

## Use Cases

- 跟踪某一批地址的交易活跃度与行为模式
- 建立地址级别的收益分析数据集
- 验证简单跟单策略的执行链路
- 为后续的策略研究、数据分析或可视化系统提供底层数据源

## Tech Stack

- Rust 2021
- Tokio
- `hyperliquid_rust_sdk`
- MySQL
- SQLx
- Serde / TOML
- Tracing

## Binaries

仓库中当前包含以下可执行入口：

| Binary | 作用 |
| --- | --- |
| `perpTrader` | 交互式主程序，用于统一选择服务入口 |
| `trade_monitor` | 监控数据库中的地址列表并采集交易事件 |
| `pnl_calculator` | 计算当前仓位盈亏与历史收益区间 |
| `wallet_manager` | 管理监控地址，支持导入、查看、删除 |
| `copy_trading` | 按配置执行跟单策略 |
| `price_collector` | 采集配置资产的价格数据 |

## How It Works

应用启动后会执行以下主流程：

1. 读取 `config/config.toml`
2. 初始化 MySQL 连接
3. 自动创建所需表结构
4. 启动对应服务逻辑
5. 将日志输出到控制台和 `logs/` 目录

自动创建的数据表包括：

- `wallet_addresses`
- `trade_events`
- `user_profit`
- `user_history_profit`
- `hl_prices`

## Project Structure

```text
perpTracker/
├── config/
│   └── config.toml
├── src/
│   ├── analyzers/
│   ├── bin/
│   ├── collectors/
│   ├── database/
│   ├── executors/
│   ├── strategies/
│   ├── tg_bot/
│   ├── utils/
│   ├── lib.rs
│   └── main.rs
├── addresses.txt
├── Cargo.toml
└── README.md
```

模块职责概览：

- `src/executors`：服务入口与任务编排
- `src/collectors`：交易、收益、价格等采集逻辑
- `src/analyzers`：收益分析逻辑
- `src/database`：数据库连接、建表与仓储方法
- `src/strategies`：跟单策略与止盈止损逻辑
- `src/utils`：配置读取、日志、地址处理等辅助逻辑
- `src/tg_bot`：Telegram 发送能力

## Requirements

- Rust 稳定版工具链
- Cargo
- MySQL 8.x 或兼容版本
- 可访问 Hyperliquid 接口的网络环境
- Linux、macOS 或 WSL

## Quick Start

### 1. 安装 Rust

确保本机已安装 Rust 与 Cargo。

### 2. 配置 `config/config.toml`

程序当前直接从 `config/config.toml` 读取数据库连接与跟单配置。公开仓库建议只保留示例值，不要提交真实密钥、真实数据库密码或真实生产地址。

示例配置：

```toml
database_url = "mysql://username:password@localhost:3306/perp_tracker"
network = "mainnet"

[copy_trading]
target_user = "0x0000000000000000000000000000000000000000"
private_key = "replace_with_your_private_key"
wallet = "0x0000000000000000000000000000000000000000"
copy_ratio = 0.1
max_position_value = 1000
enabled_assets = ["BTC", "ETH"]
take_profit_percentage = 0.05
stop_loss_percentage = 0.03
strategy_type = "conservative"
order_type = "Limit"
margin_type = "isolated"
leverage = 5
```

### 3. 准备地址文件

如果需要批量导入监控地址，可编辑根目录下的 `addresses.txt`。格式为每行一个地址；空行和以 `#` 开头的注释行会被忽略。

### 4. 启动

启动交互式主程序：

```bash
cargo run
```

直接启动指定服务：

```bash
cargo run --bin trade_monitor
cargo run --bin pnl_calculator
cargo run --bin wallet_manager
cargo run --bin copy_trading
cargo run --bin price_collector
```

## Configuration

### Top-level

- `database_url`：MySQL 连接字符串
- `network`：网络环境标识

### `copy_trading`

- `target_user`：跟单目标地址
- `private_key`：执行跟单的钱包私钥
- `wallet`：执行交易的钱包地址
- `copy_ratio`：跟单比例
- `max_position_value`：最大持仓价值
- `enabled_assets`：允许交易或采集的资产列表
- `take_profit_percentage`：止盈比例
- `stop_loss_percentage`：止损比例
- `strategy_type`：当前支持 `conservative` / `aggressive`
- `order_type`：订单类型
- `margin_type`：当前支持 `isolated` / `cross`
- `leverage`：杠杆倍数

## Logging

日志输出位置：

- 控制台
- `logs/<binary-name>.log`

文件日志使用 JSON 格式并按天轮转，便于后续接入日志分析或审计流程。

## Development

建议使用以下命令做本地检查：

```bash
cargo fmt
cargo check
cargo test
```

当前仓库需要注意：

- 部分代码路径依赖外部网络和真实配置
- 某些测试可能产生外部副作用
- 在未完成脱敏前，不建议直接在公开副本上执行真实交易相关流程



## Risks and Current Limitations

- 当前配置文件读取路径固定为 `config/config.toml`
- 项目暂未展示独立的公开示例配置文件
- 代码中存在外部消息发送与外部交易接口调用路径
- 当前仓库还需要在公开前完成脱敏整理

## Disclaimer

- 本项目仅用于开发、研究和自动化实验用途
- 本项目不构成投资建议、交易建议或收益承诺
- 使用者应自行遵守所在地法律法规、平台条款及税务要求
- 因使用本项目产生的交易风险、账户风险或合规风险，由使用者自行承担

