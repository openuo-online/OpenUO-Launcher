use serde::{Deserialize, Serialize};

// 国际化配置
rust_i18n::i18n!("locales", fallback = "en");

/// 语言信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub file: String,
}

/// 语言配置
#[derive(Debug, Deserialize)]
struct LanguagesConfig {
    languages: Vec<LanguageInfo>,
    default: String,
}

/// 获取所有可用语言
pub fn available_languages() -> Vec<LanguageInfo> {
    // 嵌入语言配置文件
    let config_json = include_str!("../locales/languages.json");
    
    match serde_json::from_str::<LanguagesConfig>(config_json) {
        Ok(config) => config.languages,
        Err(_e) => {
            tracing::warn!("{}", t!("log.language_config_failed"));
            // 降级方案：返回硬编码的语言列表
            vec![
                LanguageInfo {
                    code: "zh-CN".to_string(),
                    name: "简体中文".to_string(),
                    native_name: "简体中文".to_string(),
                    file: "zh-CN.yml".to_string(),
                },
                LanguageInfo {
                    code: "en".to_string(),
                    name: "English".to_string(),
                    native_name: "English".to_string(),
                    file: "en.yml".to_string(),
                },
            ]
        }
    }
}

/// 获取默认语言代码
pub fn default_language() -> String {
    let config_json = include_str!("../locales/languages.json");
    
    match serde_json::from_str::<LanguagesConfig>(config_json) {
        Ok(config) => config.default,
        Err(_) => "en".to_string(),
    }
}

/// 获取当前语言
pub fn current_locale() -> String {
    rust_i18n::locale().to_string()
}

/// 设置语言
pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

/// 根据系统语言自动初始化
/// 优先级：保存的语言设置 > 系统语言 > 默认语言
pub fn init_locale() {
    init_locale_with_saved(None)
}

/// 使用保存的语言设置初始化
pub fn init_locale_with_saved(saved_language: Option<String>) {
    let system_locale = detect_system_locale();
    
    // 获取可用语言列表
    let available = available_languages();
    let default = default_language();
    
    // 优先使用保存的语言设置
    let locale = if let Some(ref saved) = saved_language {
        // 验证保存的语言是否有效
        if available.iter().any(|lang| lang.code == *saved) {
            tracing::info!("{}: {}", t!("log.using_saved_language"), saved);
            saved.clone()
        } else {
            // 保存的语言无效，使用系统语言
            match_system_locale(&system_locale, &available, &default)
        }
    } else {
        // 没有保存的语言，使用系统语言
        match_system_locale(&system_locale, &available, &default)
    };
    
    rust_i18n::set_locale(&locale);
    if saved_language.is_none() {
        tracing::info!("{}: {}, {}: {}", t!("log.system_language"), system_locale, t!("log.using_language"), locale);
    }
}

/// 匹配系统语言
fn match_system_locale(system_locale: &str, available: &[LanguageInfo], default: &str) -> String {
    available
        .iter()
        .find(|lang| {
            // 精确匹配
            if system_locale == lang.code {
                return true;
            }
            // 前缀匹配（例如 "zh-TW" 匹配 "zh-CN"）
            if let Some(prefix) = system_locale.split('-').next() {
                if let Some(lang_prefix) = lang.code.split('-').next() {
                    return prefix == lang_prefix;
                }
            }
            false
        })
        .map(|lang| lang.code.clone())
        .unwrap_or_else(|| default.to_string())
}

/// 检测系统语言（带回退机制）
fn detect_system_locale() -> String {
    // 方法 1: 使用 sys-locale（读取环境变量）
    if let Some(locale) = sys_locale::get_locale() {
        // 如果检测到的不是 "C" 或 "POSIX"（这些是默认值，不代表真实语言）
        if locale != "C" && locale != "POSIX" {
            return locale;
        }
    }
    
    // 方法 2: 在 macOS 上，尝试读取系统偏好设置
    #[cfg(target_os = "macos")]
    {
        if let Some(locale) = detect_macos_locale() {
            return locale;
        }
    }
    
    // 方法 3: 在 Windows 上，使用系统 API
    #[cfg(target_os = "windows")]
    {
        if let Some(locale) = detect_windows_locale() {
            return locale;
        }
    }
    
    // 默认英语
    "en".to_string()
}

/// macOS 特定：从系统偏好设置读取语言
#[cfg(target_os = "macos")]
fn detect_macos_locale() -> Option<String> {
    use std::process::Command;
    
    // 使用 defaults 命令读取系统语言
    let output = Command::new("defaults")
        .args(&["read", "-g", "AppleLocale"])
        .output()
        .ok()?;
    
    if output.status.success() {
        let locale = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !locale.is_empty() {
            return Some(locale);
        }
    }
    
    None
}

/// Windows 特定：从系统 API 读取语言
#[cfg(target_os = "windows")]
fn detect_windows_locale() -> Option<String> {
    // Windows 上 sys-locale 已经很准确了，这里只是占位
    // 如果需要更精确的检测，可以使用 Windows API
    None
}

// 重新导出 t! 宏，方便使用
pub use rust_i18n::t;
