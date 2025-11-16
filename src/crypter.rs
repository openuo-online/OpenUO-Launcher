

/// 加密字符串，使用机器名作为密钥
pub fn encrypt(source: &str) -> String {
    if source.is_empty() {
        return String::new();
    }

    let key = calculate_key();
    if key.is_empty() {
        return String::new();
    }

    let buff = source.as_bytes();
    let key_bytes = key.as_bytes();
    let mut result = String::from("1-");
    let mut kidx = 0;

    for &byte in buff {
        let encrypted = byte ^ key_bytes[kidx];
        result.push_str(&format!("{:02X}", encrypted));
        kidx += 1;
        if kidx >= key_bytes.len() {
            kidx = 0;
        }
    }

    result
}

/// 解密字符串，使用机器名作为密钥
pub fn decrypt(source: &str) -> String {
    if source.is_empty() {
        return String::new();
    }

    // 新格式：以 "1-" 或 "1+" 开头
    if source.len() > 2 && source.starts_with("1-") || source.starts_with("1+") {
        let key = calculate_key();
        if key.is_empty() {
            return String::new();
        }

        let key_bytes = key.as_bytes();
        let hex_str = &source[2..];
        let mut result = Vec::new();
        let mut kidx = 0;

        let mut i = 0;
        while i < hex_str.len() {
            if i + 2 <= hex_str.len() {
                if let Ok(byte) = u8::from_str_radix(&hex_str[i..i + 2], 16) {
                    let decrypted = byte ^ key_bytes[kidx];
                    result.push(decrypted);
                    kidx += 1;
                    if kidx >= key_bytes.len() {
                        kidx = 0;
                    }
                }
            }
            i += 2;
        }

        String::from_utf8_lossy(&result).to_string()
    } else {
        // 旧格式
        let key = (source.len() >> 1) as u8;
        let mut result = Vec::new();

        let mut i = 0;
        while i < source.len() {
            if i + 2 <= source.len() {
                if let Ok(byte) = u8::from_str_radix(&source[i..i + 2], 16) {
                    let decrypted = byte ^ key.wrapping_add((i >> 1) as u8);
                    result.push(decrypted);
                }
            }
            i += 2;
        }

        String::from_utf8_lossy(&result).to_string()
    }
}

fn calculate_key() -> String {
    // 使用机器名作为密钥
    hostname::get()
        .ok()
        .and_then(|name| name.into_string().ok())
        .unwrap_or_else(|| "default".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "test123";
        let encrypted = encrypt(original);
        let decrypted = decrypt(&encrypted);
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(encrypt(""), "");
        assert_eq!(decrypt(""), "");
    }
}
