use anyhow::Result;
use gpui::{Hsla, SharedString, Svg, prelude::*, px, svg};
use std::borrow::Cow;

macro_rules! embedded_icons {
    ($($name:literal),* $(,)?) => {
        fn icon_bytes(path: &str) -> Option<Cow<'static, [u8]>> {
            match path {
                $(
                    concat!("icons/", $name, ".svg") => {
                        Some(Cow::Borrowed(&include_bytes!(concat!("../assets/icons/", $name, ".svg"))[..]))
                    }
                )*
                _ => None,
            }
        }
    };
}

embedded_icons!(
    "alert",
    "arrow-down-left",
    "arrow-left",
    "arrow-right",
    "arrow-up-right",
    "calendar",
    "chart",
    "check",
    "chevron-down",
    "chevron-right",
    "chevron-up",
    "cloud",
    "cloud-off",
    "command",
    "database",
    "download",
    "entries",
    "eye",
    "eye-off",
    "eye",
    "filter",
    "globe",
    "home",
    "key",
    "logout",
    "minus",
    "moon",
    "pencil",
    "plus",
    "refresh",
    "search",
    "server",
    "settings",
    "shield",
    "sparkles",
    "sun",
    "tags",
    "trash",
    "undo",
    "upload",
    "user",
    "wallet",
    "x",
);

pub struct Assets;

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == "logo.svg" {
            return Ok(Some(Cow::Borrowed(
                &include_bytes!("../assets/logo.svg")[..],
            )));
        }
        Ok(icon_bytes(path))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Alert,
    ArrowDownLeft,
    ArrowLeft,
    ArrowRight,
    ArrowUpRight,
    Calendar,
    Check,
    ChevronDown,
    ChevronRight,
    ChevronUp,
    Cloud,
    CloudOff,
    Database,
    Download,
    Entries,
    Eye,
    EyeOff,
    Filter,
    Globe,
    Home,
    Key,
    Logout,
    Minus,
    Moon,
    Pencil,
    Plus,
    Refresh,
    Search,
    Server,
    Settings,
    Shield,
    Sparkles,
    Sun,
    Tags,
    Trash,
    Undo,
    Upload,
    User,
    Wallet,
    X,
}

impl Icon {
    pub fn path(self) -> &'static str {
        match self {
            Icon::Alert => "icons/alert.svg",
            Icon::ArrowDownLeft => "icons/arrow-down-left.svg",
            Icon::ArrowLeft => "icons/arrow-left.svg",
            Icon::ArrowRight => "icons/arrow-right.svg",
            Icon::ArrowUpRight => "icons/arrow-up-right.svg",
            Icon::Calendar => "icons/calendar.svg",
            Icon::Check => "icons/check.svg",
            Icon::ChevronDown => "icons/chevron-down.svg",
            Icon::ChevronRight => "icons/chevron-right.svg",
            Icon::ChevronUp => "icons/chevron-up.svg",
            Icon::Cloud => "icons/cloud.svg",
            Icon::CloudOff => "icons/cloud-off.svg",
            Icon::Database => "icons/database.svg",
            Icon::Download => "icons/download.svg",
            Icon::Entries => "icons/entries.svg",
            Icon::Eye => "icons/eye.svg",
            Icon::EyeOff => "icons/eye-off.svg",
            Icon::Filter => "icons/filter.svg",
            Icon::Globe => "icons/globe.svg",
            Icon::Home => "icons/home.svg",
            Icon::Key => "icons/key.svg",
            Icon::Logout => "icons/logout.svg",
            Icon::Minus => "icons/minus.svg",
            Icon::Moon => "icons/moon.svg",
            Icon::Pencil => "icons/pencil.svg",
            Icon::Plus => "icons/plus.svg",
            Icon::Refresh => "icons/refresh.svg",
            Icon::Search => "icons/search.svg",
            Icon::Server => "icons/server.svg",
            Icon::Settings => "icons/settings.svg",
            Icon::Shield => "icons/shield.svg",
            Icon::Sparkles => "icons/sparkles.svg",
            Icon::Sun => "icons/sun.svg",
            Icon::Tags => "icons/tags.svg",
            Icon::Trash => "icons/trash.svg",
            Icon::Undo => "icons/undo.svg",
            Icon::Upload => "icons/upload.svg",
            Icon::User => "icons/user.svg",
            Icon::Wallet => "icons/wallet.svg",
            Icon::X => "icons/x.svg",
        }
    }
}

pub fn icon(name: Icon, size: f32, color: Hsla) -> Svg {
    svg()
        .path(name.path())
        .size(px(size))
        .flex_shrink_0()
        .text_color(color)
}

pub fn logo(size: f32, color: Hsla) -> Svg {
    svg()
        .path("logo.svg")
        .size(px(size))
        .flex_shrink_0()
        .text_color(color)
}
