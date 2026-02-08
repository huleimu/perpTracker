//! 日志宏模块
//! 提供统一的日志格式和调用方式

/// 日志宏
/// 
/// # 参数
/// - `$level`: 日志级别 (info, warn, error, debug)
/// - `$module`: 模块名
/// - `$function`: 函数名
/// - `$message`: 日志消息
/// - `$($key:expr => $value:expr),*`: 可选的键值对数据
/// 
#[macro_export]
macro_rules! log {
    ($level:ident, $module:expr, $function:expr, $message:expr) => {
        tracing::$level!(
            service = "perpTracker",
            module = $module,
            function = $function,
            file = file!(),
            line = line!(),
            message = $message
        );
    };
    ($level:ident, $module:expr, $function:expr, $message:expr, $($key:expr => $value:expr),*) => {
        tracing::$level!(
            service = "perpTracker",
            module = $module,
            function = $function,
            file = file!(),
            line = line!(),
            $($key = $value,)*
            message = $message
        )
    };
}

/// 错误日志宏
/// 
/// # 参数
/// - `$module`: 模块名
/// - `$function`: 函数名
/// - `$message`: 日志消息
/// - `$error`: 错误对象
/// - `$($key:expr => $value:expr),*`: 可选的键值对数据
/// 
/// # 示例
/// ```rust
/// error!(
///     "copy_trading_strategy",
///     "execute_copy_trade_inline",
///     "跟单交易失败",
///     e,
///     "coin" => coin,
///     "action" => dir
/// );
/// ```
#[macro_export]
macro_rules! error {
    ($module:expr, $function:expr, $message:expr, $error:expr) => {
        tracing::error!(
            service = "perpTracker",
            module = $module,
            function = $function,
            file = file!(),
            line = line!(),
            error = %$error,
            message = $message
        );
    };
    ($module:expr, $function:expr, $message:expr, $error:expr, $($key:expr => $value:expr),*) => {
        tracing::error!(
            service = "perpTracker",
            module = $module,
            function = $function,
            file = file!(),
            line = line!(),
            error = %$error,
            $($key = $value,)*
            message = $message
        )
    };
}

 
