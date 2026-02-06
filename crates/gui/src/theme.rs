use gpui::{App, Global, Rgba};

/// Theme mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            ThemeMode::System => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}

/// All theme colors grouped in a struct.
#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub is_dark: bool,

    pub bg: Rgba,
    pub sidebar_bg: Rgba,
    pub sidebar_hover: Rgba,
    pub sidebar_active: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub user_bubble: Rgba,
    pub assistant_bubble: Rgba,
    pub error_color: Rgba,
    pub input_bg: Rgba,
    pub stop_button: Rgba,
}

impl Global for Theme {}

fn rgba(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

impl Theme {
    pub fn dark() -> Self {
        Theme {
            mode: ThemeMode::System,
            is_dark: true,
            bg: rgba(0x1e, 0x1e, 0x1e),
            sidebar_bg: rgba(0x25, 0x25, 0x26),
            sidebar_hover: rgba(0x2d, 0x2d, 0x30),
            sidebar_active: rgba(0x37, 0x37, 0x3d),
            border: rgba(0x33, 0x33, 0x33),
            accent: rgba(0x00, 0x7a, 0xcc),
            accent_hover: rgba(0x1c, 0x8c, 0xd9),
            text: rgba(0xcc, 0xcc, 0xcc),
            text_muted: rgba(0x80, 0x80, 0x80),
            user_bubble: rgba(0x26, 0x4f, 0x78),
            assistant_bubble: rgba(0x2d, 0x2d, 0x2d),
            error_color: rgba(0xf4, 0x43, 0x36),
            input_bg: rgba(0x2a, 0x2a, 0x2a),
            stop_button: rgba(0xd3, 0x2f, 0x2f),
        }
    }

    pub fn light() -> Self {
        Theme {
            mode: ThemeMode::System,
            is_dark: false,
            bg: rgba(0xfa, 0xfa, 0xfa),
            sidebar_bg: rgba(0xf0, 0xf0, 0xf0),
            sidebar_hover: rgba(0xe4, 0xe4, 0xe4),
            sidebar_active: rgba(0xd8, 0xd8, 0xd8),
            border: rgba(0xd0, 0xd0, 0xd0),
            accent: rgba(0x00, 0x7a, 0xcc),
            accent_hover: rgba(0x00, 0x6b, 0xb3),
            text: rgba(0x1e, 0x1e, 0x1e),
            text_muted: rgba(0x6e, 0x6e, 0x6e),
            user_bubble: rgba(0x00, 0x7a, 0xcc),
            assistant_bubble: rgba(0xe8, 0xe8, 0xe8),
            error_color: rgba(0xd3, 0x2f, 0x2f),
            input_bg: rgba(0xf5, 0xf5, 0xf5),
            stop_button: rgba(0xd3, 0x2f, 0x2f),
        }
    }

    pub fn for_appearance(appearance: gpui::WindowAppearance, mode: ThemeMode) -> Self {
        let is_dark = match mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => matches!(
                appearance,
                gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
            ),
        };
        let mut theme = if is_dark { Self::dark() } else { Self::light() };
        theme.mode = mode;
        theme.is_dark = is_dark;
        theme
    }
}

// --- Convenience accessor functions (read from global) ---

fn current(cx: &App) -> &Theme {
    cx.global::<Theme>()
}

pub fn bg(cx: &App) -> Rgba { current(cx).bg }
pub fn sidebar_bg(cx: &App) -> Rgba { current(cx).sidebar_bg }
pub fn sidebar_hover(cx: &App) -> Rgba { current(cx).sidebar_hover }
pub fn sidebar_active(cx: &App) -> Rgba { current(cx).sidebar_active }
pub fn border(cx: &App) -> Rgba { current(cx).border }
pub fn accent(cx: &App) -> Rgba { current(cx).accent }
pub fn accent_hover(cx: &App) -> Rgba { current(cx).accent_hover }
pub fn text(cx: &App) -> Rgba { current(cx).text }
pub fn text_muted(cx: &App) -> Rgba { current(cx).text_muted }
pub fn user_bubble(cx: &App) -> Rgba { current(cx).user_bubble }
pub fn assistant_bubble(cx: &App) -> Rgba { current(cx).assistant_bubble }
pub fn error_color(cx: &App) -> Rgba { current(cx).error_color }
pub fn input_bg(cx: &App) -> Rgba { current(cx).input_bg }
pub fn stop_button(cx: &App) -> Rgba { current(cx).stop_button }
pub fn is_dark(cx: &App) -> bool { current(cx).is_dark }
pub fn mode(cx: &App) -> ThemeMode { current(cx).mode }
