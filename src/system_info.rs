/// 系统信息辅助模块

/// 获取操作系统名称和版本
pub fn os_name_version() -> String {
    #[cfg(target_os = "windows")]
    {
        get_windows_version()
    }
    
    #[cfg(target_os = "macos")]
    {
        get_macos_version()
    }
    
    #[cfg(target_os = "linux")]
    {
        "Linux".to_string()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Unknown".to_string()
    }
}

/// 获取 CPU 架构
pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

/// 获取完整的系统信息字符串
pub fn system_info_string() -> String {
    format!("{} {}", os_name_version(), arch())
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> String {
    // 尝试获取 Windows 版本
    // 注意：在 Windows 10+ 上，可能需要特殊处理
    use std::process::Command;
    
    if let Ok(output) = Command::new("cmd")
        .args(&["/C", "ver"])
        .output()
    {
        if let Ok(version_str) = String::from_utf8(output.stdout) {
            // 解析版本字符串
            if version_str.contains("Windows") {
                // 简化版本显示
                if version_str.contains("10.0") {
                    return "Windows 10/11".to_string();
                }
            }
        }
    }
    
    "Windows".to_string()
}

#[cfg(target_os = "macos")]
fn get_macos_version() -> String {
    use std::process::Command;
    
    // 使用 sw_vers 获取 macOS 版本
    if let Ok(output) = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
    {
        if output.status.success() {
            if let Ok(version) = String::from_utf8(output.stdout) {
                let version = version.trim();
                // 解析版本号，例如 "14.1.1" -> "macOS 14"
                if let Some(major) = version.split('.').next() {
                    return format!("macOS {}", major);
                }
            }
        }
    }
    
    "macOS".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info() {
        println!("OS: {}", os_name_version());
        println!("Arch: {}", arch());
        println!("Full: {}", system_info_string());
    }
}
