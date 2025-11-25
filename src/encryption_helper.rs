/// 根据客户端版本号推荐是否使用加密
/// 返回值：0 = 不加密，1 = 加密
pub fn suggest_encryption_from_version(version: &str) -> u8 {
    // 解析版本号
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() {
        return 0; // 无法解析，默认无加密
    }
    
    // 获取主版本号和次版本号
    let major = parts[0].parse::<u32>().unwrap_or(0);
    let minor = if parts.len() > 1 {
        parts[1].parse::<u32>().unwrap_or(0)
    } else {
        0
    };
    
    // 根据版本号推荐是否加密
    // 参考 ClassicUO/OpenUO 的实现
    
    // 1.26.0 及以上版本通常需要加密（官服）
    if major > 1 || (major == 1 && minor >= 26) {
        return 1; // 加密
    }
    
    // 6.x.x 及以上版本可能需要加密
    if major >= 6 {
        return 1; // 加密
    }
    
    // 更老的版本或私服通常不需要加密
    0 // 不加密
}

/// 获取加密类型的显示名称
pub fn encryption_type_name(encryption: u8) -> &'static str {
    match encryption {
        0 => "不加密",
        1 => "加密",
        _ => "未知",
    }
}

/// 获取加密类型的详细说明
pub fn encryption_type_description(encryption: u8) -> &'static str {
    match encryption {
        0 => "明文通信，适用于私服和旧版本客户端",
        1 => "使用客户端内置加密，适用于官服和部分私服",
        _ => "未知的加密类型",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_encryption() {
        assert_eq!(suggest_encryption_from_version("7.0.102"), 1); // 加密
        assert_eq!(suggest_encryption_from_version("7.0.10"), 1);  // 加密
        assert_eq!(suggest_encryption_from_version("6.0.0"), 1);   // 加密
        assert_eq!(suggest_encryption_from_version("1.26.0"), 1);  // 加密
        assert_eq!(suggest_encryption_from_version("1.25.0"), 0);  // 不加密
        assert_eq!(suggest_encryption_from_version("5.0.0"), 0);   // 不加密
    }
}
