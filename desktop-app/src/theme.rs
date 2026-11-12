use gpui::{App, Global, Hsla, SharedString, rgb};

/// The visual language of Sunder.
///
/// The web app is the source of truth for the brand: cool indigo, mint income,
/// coral expenses, tinted neutrals rather than pure black and white. The desktop
/// app keeps that language but leans into native desktop conventions: slightly
/// denser type, softer elevation and a calmer chrome.
#[derive(Clone)]
pub struct Theme {
    pub mode: ThemeMode,

    pub bg: Hsla,
    pub sidebar: Hsla,
    pub surface: Hsla,
    pub surface_muted: Hsla,
    pub surface_hover: Hsla,

    pub foreground: Hsla,
    pub muted: Hsla,
    pub subtle: Hsla,
    pub on_primary: Hsla,

    pub border: Hsla,
    pub border_strong: Hsla,

    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_pressed: Hsla,
    pub primary_tint: Hsla,

    pub income: Hsla,
    pub income_tint: Hsla,
    pub expense: Hsla,
    pub expense_tint: Hsla,
    pub warning: Hsla,

    pub overlay: Hsla,
    pub shadow: Hsla,
    pub selection: Hsla,

    pub font: SharedString,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl Global for Theme {}

impl Theme {
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            bg: hex(0xF5F6FA),
            sidebar: hex(0xFBFBFD),
            surface: hex(0xFFFFFF),
            surface_muted: hex(0xEFF1F7),
            surface_hover: hex(0xF1F3F9),
            foreground: hex(0x151824),
            muted: hex(0x656C80),
            subtle: hex(0x767D91),
            on_primary: hex(0xFFFFFF),
            border: hex(0xE4E7F0),
            border_strong: hex(0xD4D8E5),
            primary: hex(0x4F46E5),
            primary_hover: hex(0x4338CA),
            primary_pressed: hex(0x3730A3),
            primary_tint: hex(0xEEF0FF),
            income: hex(0x059669),
            income_tint: hex(0xE8F8F1),
            expense: hex(0xE11D48),
            expense_tint: hex(0xFFEDF1),
            warning: hex(0xB45309),
            overlay: hex(0x0B0D14).opacity(0.42),
            shadow: hex(0x1B2030).opacity(0.08),
            selection: hex(0x4F46E5).opacity(0.16),
            font: "Inter Tight".into(),
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            bg: hex(0x0D0F15),
            sidebar: hex(0x12141B),
            surface: hex(0x171A22),
            surface_muted: hex(0x20242F),
            surface_hover: hex(0x1E222C),
            foreground: hex(0xF1F3FA),
            muted: hex(0xA2A9BC),
            subtle: hex(0x8A91A6),
            on_primary: hex(0xFFFFFF),
            border: hex(0x252A36),
            border_strong: hex(0x333949),
            primary: hex(0x7C78FF),
            primary_hover: hex(0x8D89FF),
            primary_pressed: hex(0x6B66F0),
            primary_tint: hex(0x1D1C3A),
            income: hex(0x34D399),
            income_tint: hex(0x102A23),
            expense: hex(0xFB7185),
            expense_tint: hex(0x2E151E),
            warning: hex(0xFBBF24),
            overlay: hex(0x000000).opacity(0.58),
            shadow: hex(0x000000).opacity(0.28),
            selection: hex(0x7C78FF).opacity(0.24),
            font: "Inter Tight".into(),
        }
    }

    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
        }
    }

    pub fn money_color(&self, amount: f64) -> Hsla {
        if amount > 0.0 {
            self.income
        } else if amount < 0.0 {
            self.expense
        } else {
            self.muted
        }
    }
}

/// Read the active theme from the GPUI global state.
pub fn theme(cx: &App) -> &Theme {
    cx.global::<Theme>()
}

/// A neutral type ramp; keeping it in one place makes the hierarchy obvious.
pub mod text {
    use gpui::FontWeight;

    pub const TITLE: f32 = 26.0;
    pub const HEADING: f32 = 17.0;
    pub const BODY: f32 = 13.0;
    pub const SMALL: f32 = 12.0;
    pub const MICRO: f32 = 11.0;

    pub const MEDIUM: FontWeight = FontWeight::MEDIUM;
    pub const SEMIBOLD: FontWeight = FontWeight::SEMIBOLD;
    pub const BOLD: FontWeight = FontWeight::BOLD;
}

/// Layout constants shared by the shell and the screens.
pub mod space {
    pub const SIDEBAR: f32 = 228.0;
    pub const CONTENT_MAX: f32 = 1080.0;
    pub const PAGE_PAD: f32 = 30.0;
}

pub fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

/// Parse category colors coming from the server (`#22c55e`) with a graceful
/// fallback for malformed values.
pub fn parse_color(value: &str) -> Hsla {
    let value = value.trim().trim_start_matches('#');
    if value.len() == 6
        && let Ok(rgb) = u32::from_str_radix(value, 16)
    {
        return hex(rgb);
    }
    hex(0x8D93A6)
}

pub fn set_theme(cx: &mut App, mode: ThemeMode) {
    cx.set_global(Theme::for_mode(mode));
    cx.refresh_windows();
}
