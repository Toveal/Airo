//! Interface language selection.
//!
//! The language is not a frontend-only concern: Rust draws the tray menu, and
//! it has to match the language of the window. That is why the setting lives in
//! the config rather than in `localStorage` like the theme does.

use serde::{Deserialize, Serialize};
use windows::Win32::Globalization::GetUserDefaultLocaleName;

/// The language the interface is actually rendered in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    Ru,
}

/// The language setting. `System` is resolved into a [`Locale`] on every read,
/// so changing the Windows language takes effect without editing the config.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    System,
    En,
    Ru,
}

impl Language {
    pub fn resolve(self) -> Locale {
        match self {
            Self::En => Locale::En,
            Self::Ru => Locale::Ru,
            Self::System => system_locale(),
        }
    }
}

impl Locale {
    /// The "show the window" tray entry.
    pub const fn tray_show(self) -> &'static str {
        match self {
            Self::En => "Show Airo",
            Self::Ru => "Показать Airo",
        }
    }

    /// The "quit" tray entry.
    pub const fn tray_quit(self) -> &'static str {
        match self {
            Self::En => "Quit",
            Self::Ru => "Выход",
        }
    }
}

/// The Windows interface language. Anything but Russian is treated as English.
fn system_locale() -> Locale {
    match user_locale_name() {
        Some(name) => {
            let locale = from_bcp47(&name);
            tracing::info!(%name, ?locale, "system language");
            locale
        }
        None => {
            tracing::warn!("could not detect the system language, falling back to English");
            Locale::En
        }
    }
}

/// Parses tags such as `ru-RU`, `ru` or `en-GB`.
///
/// Kept separate from the Win32 call so the behaviour can be unit tested.
fn from_bcp47(tag: &str) -> Locale {
    let primary = tag.split(['-', '_']).next().unwrap_or_default();
    if primary.eq_ignore_ascii_case("ru") {
        Locale::Ru
    } else {
        Locale::En
    }
}

fn user_locale_name() -> Option<String> {
    // LOCALE_NAME_MAX_LENGTH from winnls.h; windows-rs does not export it.
    let mut buffer = [0u16; 85];

    // SAFETY: the buffer outlives the call and the API takes its length from
    // the slice.
    let len = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    if len <= 0 {
        return None;
    }

    // The returned length includes the trailing NUL.
    let chars = &buffer[..(len as usize).saturating_sub(1)];
    Some(String::from_utf16_lossy(chars))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn russian_tags_map_to_ru() {
        for tag in ["ru", "ru-RU", "ru-BY", "RU-ru", "ru_RU"] {
            assert_eq!(from_bcp47(tag), Locale::Ru, "tag {tag}");
        }
    }

    #[test]
    fn everything_else_falls_back_to_en() {
        for tag in ["en-US", "de-DE", "zh-Hans-CN", "", "rus"] {
            assert_eq!(from_bcp47(tag), Locale::En, "tag {tag}");
        }
    }

    #[test]
    fn explicit_language_ignores_system() {
        assert_eq!(Language::En.resolve(), Locale::En);
        assert_eq!(Language::Ru.resolve(), Locale::Ru);
    }

    #[test]
    fn language_serializes_as_lowercase() {
        assert_eq!(
            serde_json::to_string(&Language::System).unwrap(),
            "\"system\""
        );
        assert_eq!(serde_json::to_string(&Locale::Ru).unwrap(), "\"ru\"");
    }

    #[test]
    fn default_language_is_system() {
        assert_eq!(Language::default(), Language::System);
    }
}
