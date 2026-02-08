//! 日志配置模块
//! 负责初始化日志系统，配置控制台和文件输出

use tracing_appender::rolling;
use tracing_subscriber::{fmt, Registry, prelude::*};

/// 初始化日志系统
/// 
/// 配置包括：
/// - 控制台输出：彩色格式，便于开发调试
/// - 文件输出：JSON格式，按天轮转，便于生产环境分析
/// - 自动创建日志目录
/// - 支持 log crate 和 tracing crate 的统一输出
pub fn init_logging() {
    // 创建日志目录
    if let Err(e) = std::fs::create_dir_all("logs") {
        eprintln!("创建日志目录失败: {}", e);
        return;
    }
    
    // 根据可执行文件名确定日志文件
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("perpTracker"));
    let exe_name = exe_path.file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
        .to_string_lossy();
    let log_file = format!("{}.log", exe_name);
    
    // 配置文件输出（按天轮转，JSON格式）
    let file_appender = rolling::daily("logs", &log_file);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    // 保存 guard 到静态变量，防止被提前释放
    std::mem::forget(guard);
    
    // 配置订阅者
    let subscriber = Registry::default()
        // 添加环境变量过滤器支持
        .with(tracing_subscriber::EnvFilter::from_default_env())
        // 控制台输出层 - 彩色格式，便于开发调试
        .with(fmt::layer()
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false))
        // 文件输出层 - JSON格式，便于生产环境分析
        .with(fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .json()
            .with_current_span(true)
            .with_span_list(false));
    
    // 设置全局订阅者
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("设置全局日志订阅者失败: {}", e);
        return;
    }
    
    // 配置 log crate 到 tracing 的桥接
    tracing_log::LogTracer::init().expect("初始化 log 到 tracing 桥接失败");
    
    // 记录日志系统初始化成功
    tracing::info!(
        service = "perpTracker",
        module = "logging_config",
        function = "init_logging",
        message = "日志系统初始化成功",
        log_dir = "logs",
        log_file = log_file
    );
}

 
