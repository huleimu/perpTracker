use anyhow::Result;
use ethers::types::H160;
use std::str::FromStr;

/// 验证以太坊地址格式
pub fn is_valid_eth_address(address: &str) -> bool {
    address.len() == 42 && address.starts_with("0x") && address.chars().skip(2).all(|c| c.is_ascii_hexdigit())
}

/// 标准化以太坊地址（转换为小写并确保有0x前缀）
pub fn normalize_eth_address(address: &str) -> String {
    let addr = address.trim().to_lowercase();
    if addr.starts_with("0x") { addr } else { format!("0x{}", addr) }
}

/// 安全解析字符串为 H160 地址
/// 
/// # 参数
/// * `address_str` - 地址字符串
/// 
/// # 返回
/// * `Result<H160>` - 解析后的地址，失败时返回错误
/// 
/// # 示例
/// ```
/// let addr = safe_parse_h160("0x1234567890123456789012345678901234567890");
/// assert!(addr.is_ok());
/// ```
pub fn safe_parse_h160(address_str: &str) -> Result<H160> {
    let normalized = normalize_eth_address(address_str);
    if !is_valid_eth_address(&normalized) {
        return Err(anyhow::anyhow!("无效的以太坊地址格式: {}", address_str));
    }
    H160::from_str(&normalized).map_err(|e| anyhow::anyhow!("地址解析失败: {}", e))
}

/// 验证并解析地址字符串列表
/// 
/// # 参数
/// * `address_strings` - 地址字符串列表
/// 
/// # 返回
/// * `Result<Vec<H160>>` - 解析后的地址列表
/// 
/// # 示例
/// ```
/// let addresses = vec!["0x1234...", "0x5678..."];
/// let parsed = parse_address_list(&addresses);
/// ```
pub fn parse_address_list(address_strings: &[String]) -> Result<Vec<H160>> {
    let mut addresses = Vec::new();
    
    for address_str in address_strings {
        match safe_parse_h160(address_str) {
            Ok(address) => {
                // 检查是否重复
                if addresses.iter().any(|addr| *addr == address) {
                    continue;
                }
                addresses.push(address);
            }
            Err(_) => {
                // 跳过无效地址
                continue;
            }
        }
    }
    
    Ok(addresses)
} 
