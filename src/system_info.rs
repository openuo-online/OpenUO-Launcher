use std::sync::OnceLock;

/// System info helpers.
static SYSTEM_INFO: OnceLock<String> = OnceLock::new();

pub fn os_name() -> String {
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

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

pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

/// Cached system info string so we don't shell out every frame.
pub fn system_info_string() -> String {
    SYSTEM_INFO
        .get_or_init(|| format!("{} {}", os_name_version(), arch()))
        .clone()
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> String {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;
    
    if let Ok(output) = Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW.0)
        .args(&["/C", "ver"])
        .output()
    {
        if let Ok(version_str) = String::from_utf8(output.stdout) {
            if version_str.contains("Windows") {
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
    
    // ʹ�� sw_vers ��ȡ macOS �汾
    if let Ok(output) = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
    {
        if output.status.success() {
            if let Ok(version) = String::from_utf8(output.stdout) {
                let version = version.trim();
                // �����汾�ţ����� "14.1.1" -> "macOS 14"
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
