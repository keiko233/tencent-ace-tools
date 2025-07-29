use iced::{Font, Theme};

/// Segoe UI font family
pub const SEGOE_UI: Font = Font::with_name("Segoe UI");
pub const SEGOE_UI_SEMIBOLD: Font = Font {
    family: iced::font::Family::Name("Segoe UI Semibold"),
    weight: iced::font::Weight::Semibold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

/// Monospace fonts for logs and process status
pub const SEGOE_UI_MONO: Font = Font::with_name("Cascadia Mono");
pub const CONSOLAS: Font = Font::with_name("Consolas");

/// Create a default theme
pub fn create_custom_theme() -> Theme {
    Theme::default()
}

/// Get the best available font for the system
pub fn get_default_font() -> Font {
    // On Windows, prefer Segoe UI
    #[cfg(target_os = "windows")]
    {
        SEGOE_UI
    }
    
    // Fallback for other systems
    #[cfg(not(target_os = "windows"))]
    {
        Font::DEFAULT
    }
}

/// Get the best available monospace font for the system
pub fn get_monospace_font() -> Font {
    // On Windows, prefer Cascadia Mono or Consolas
    #[cfg(target_os = "windows")]
    {
        CONSOLAS
    }
    
    // Fallback for other systems
    #[cfg(not(target_os = "windows"))]
    {
        Font::MONOSPACE
    }
}

/// Get font for headers (larger, semibold)
pub fn get_header_font() -> Font {
    #[cfg(target_os = "windows")]
    {
        SEGOE_UI_SEMIBOLD
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Font::DEFAULT
    }
}
