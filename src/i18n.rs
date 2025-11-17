// 国际化配置
rust_i18n::i18n!("locales", fallback = "en");

/// 获取当前语言
pub fn current_locale() -> String {
    rust_i18n::locale().to_string()
}

/// 设置语言
pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

/// 根据系统语言自动设置
pub fn init_locale() {
    let system_locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    
    // 简化语言代码（例如 "zh-CN" -> "zh-CN", "zh-TW" -> "zh-CN", "en-US" -> "en"）
    let locale = if system_locale.starts_with("zh") {
        "zh-CN"
    } else {
        "en"
    };
    
    set_locale(locale);
    tracing::info!("系统语言: {}, 使用语言: {}", system_locale, locale);
}

// 重新导出 t! 宏，方便使用
pub use rust_i18n::t;
