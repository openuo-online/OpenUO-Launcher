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
        Err(e) => {
            tracing::warn!("解析语言配置失败: {}, 使用默认配置", e);
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
pub fn init_locale() {
    let system_locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    
    // 获取可用语言列表
    let available = available_languages();
    let default = default_language();
    
    // 尝试匹配系统语言
    let locale = available
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
        .unwrap_or(default);
    
    rust_i18n::set_locale(&locale);
    tracing::info!("系统语言: {}, 使用语言: {}", system_locale, locale);
}

// 重新导出 t! 宏，方便使用
pub use rust_i18n::t;
