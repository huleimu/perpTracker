use reqwest::Error as ReqwestError;

/// Requests will be sent according to bot instance.
/// So users can use this library interacting with multiple bot APIs by having
/// multiple of `BotInstance`.
#[derive(Clone)]
pub struct BotInstance {
    pub bot_token: String,
    pub chat_id: String,
}

/// ErrorResult usually returned to indicate result from calling APIs related
/// functions.
#[derive(Debug, Clone)]
pub struct ErrorResult {
    pub code: u16,       // error returned code
    pub msg: String,     // error string description
}

/// Telegram's error result.
/// In case of error occurred as part of telegram API calling, then this struct
/// will be formed and returned.
#[derive(Debug, serde::Deserialize)]
pub struct TelegramErrorResult {
    pub ok: bool,
    pub error_code: i32,
    pub description: String,
}

/// Status code indicating the result of APIs related function call.
#[derive(Debug, Clone)]
pub enum StatusCode {
    /// Success
    Success = 0,

    /// Internal error due to various internal operations.
    /// Whenever Telegram's related operations occurred with error, then this
    /// value will be used.
    ErrorInternalError,
}

/// Parse mode for `sendMessage` API
#[derive(Clone, Debug)]
pub enum SendMessageParseMode {
    /// MarkdownV2 style
    MarkdownV2,

    /// HTML style
    HTML,
}

/// Options which can be used with `sendMessage` API
pub struct SendMessageOption {
    /// Parse mode
    pub parse_mode: Option<SendMessageParseMode>,
}

/// Create an `ErrorResult` from a `reqwest::Error`.
/// 
/// # Arguments
/// 
/// * `error` - `reqwest::Error` represents the error occurred during the HTTP request
pub fn create_error_result_from_reqwest(error: ReqwestError) -> Result<(), ErrorResult> {
    Err(ErrorResult {
        code: StatusCode::ErrorInternalError as u16,
        msg: error.to_string(),
    })
}

/// Create an `ErrorResult` from input of string slice.
/// 
/// # Arguments
/// 
/// * `code` - `StatusCode`
/// * `msg` - message string to add as an error description
pub fn create_error_result_str(code: StatusCode, msg: &str) -> Result<(), ErrorResult> {
    Err(ErrorResult { code: code as u16, msg: msg.to_string() })
}

/// Get a string representing of specified parse mode.
///
/// This function returns static string as it is expected to be used across
/// the lifetime of the application. So it has no need to return new instance
/// of `String` every time.
///
/// # Arguments
/// * `mode` - parse mode
pub fn get_send_message_parse_mode_str(mode: SendMessageParseMode) -> &'static str {
    match mode {
        SendMessageParseMode::MarkdownV2 => "MarkdownV2",
        SendMessageParseMode::HTML => "HTML",
    }
}


impl From<reqwest::Error> for ErrorResult {
    fn from(error: reqwest::Error) -> Self {
        ErrorResult {
            code: StatusCode::ErrorInternalError as u16,
            msg: error.to_string(),
        }
    }
}

/// Struct to hold detailed token information
#[derive(Debug, Clone)]
pub struct TokenDetails {
    pub mint_address: String,
    pub name: String,
    pub symbol: String,
    pub url: String,
    pub ai_analysis: String,
    pub ai_from_x_url: String,
    pub market_cap: String,
    pub creator: String, 
    pub launch_time: String,
}