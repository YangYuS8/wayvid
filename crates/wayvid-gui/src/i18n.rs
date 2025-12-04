//! Internationalization (i18n) module for wayvid-gui
//!
//! This module provides multi-language support using rust-i18n.
//! Supported languages:
//! - English (en)
//! - Simplified Chinese (zh-CN)

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// Note: rust_i18n::i18n!("locales") is called in main.rs (crate root)

/// Available languages in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Auto-detect from system locale
    #[default]
    System,
    /// English
    #[serde(alias = "en", alias = "en-US")]
    English,
    /// Simplified Chinese
    #[serde(alias = "zh-CN", alias = "zh")]
    Chinese,
}

impl Language {
    /// Get the locale code for this language
    pub fn locale_code(&self) -> &'static str {
        match self {
            Language::System => detect_system_locale(),
            Language::English => "en",
            Language::Chinese => "zh-CN",
        }
    }

    /// Get the display name for this language (in its own language)
    pub fn display_name(&self) -> String {
        match self {
            Language::System => rust_i18n::t!("settings.language_auto").to_string(),
            Language::English => "English".to_string(),
            Language::Chinese => "简体中文".to_string(),
        }
    }

    /// Get all available languages
    pub fn all() -> &'static [Language] {
        &[Language::System, Language::English, Language::Chinese]
    }

    /// Parse from locale code string
    #[allow(dead_code)] // Reserved for settings persistence
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Language::English,
            "zh-cn" | "zh" | "zh-hans" => Language::Chinese,
            "system" | "auto" | "" => Language::System,
            _ => Language::English, // Fallback to English
        }
    }

    /// Convert to serializable string
    #[allow(dead_code)] // Reserved for settings persistence
    pub fn to_code(&self) -> &'static str {
        match self {
            Language::System => "system",
            Language::English => "en",
            Language::Chinese => "zh-CN",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Cached system locale
static SYSTEM_LOCALE: OnceLock<String> = OnceLock::new();

/// Detect system locale using sys-locale crate
pub fn detect_system_locale() -> &'static str {
    SYSTEM_LOCALE.get_or_init(|| {
        sys_locale::get_locale()
            .map(|locale| {
                let locale = locale.to_lowercase();
                if locale.starts_with("zh") {
                    "zh-CN".to_string()
                } else {
                    "en".to_string()
                }
            })
            .unwrap_or_else(|| "en".to_string())
    })
}

/// Set the application language
pub fn set_language(language: Language) {
    let locale = language.locale_code();
    rust_i18n::set_locale(locale);
    tracing::info!("Language set to: {} ({})", language.display_name(), locale);
}

/// Initialize i18n with system locale detection
pub fn init() {
    let system_locale = detect_system_locale();
    rust_i18n::set_locale(system_locale);
    tracing::info!("Initialized i18n with locale: {}", system_locale);
}

/// Get current locale code
#[allow(dead_code)] // Reserved for settings display
pub fn current_locale() -> String {
    rust_i18n::locale().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_locale_codes() {
        assert_eq!(Language::English.locale_code(), "en");
        assert_eq!(Language::Chinese.locale_code(), "zh-CN");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Language::English);
        assert_eq!(Language::from_code("en-US"), Language::English);
        assert_eq!(Language::from_code("zh-CN"), Language::Chinese);
        assert_eq!(Language::from_code("zh"), Language::Chinese);
        assert_eq!(Language::from_code("system"), Language::System);
        assert_eq!(Language::from_code("unknown"), Language::English);
    }

    #[test]
    fn test_language_display_name() {
        // Note: display_name() returns String now, and System uses i18n
        assert_eq!(Language::English.display_name(), "English");
        assert_eq!(Language::Chinese.display_name(), "简体中文");
    }
}
