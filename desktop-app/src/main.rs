mod api;
mod app;
mod assets;
mod format;
mod model;
mod motion;
mod search;
mod store;
mod theme;
mod ui;

use app::{
    CloseOverlay, FocusSearch, NewAccount, NewCategory, NewEntry, SaveOverlay, ShowAccounts,
    ShowCategories, ShowDashboard, ShowEntries, ShowSettings, ShowShortcuts, SunderApp, SyncNow,
    ToggleTheme,
};
use assets::Assets;
use gpui::{
    App, Application, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions, prelude::*,
    px, size,
};
use std::borrow::Cow;

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        load_fonts(cx);
        bind_keys(cx);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Sunder".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                window_min_size: Some(size(px(880.0), px(620.0))),
                app_id: Some("dev.crqch.sunder".into()),
                ..Default::default()
            },
            |_, cx| cx.new(SunderApp::new),
        )
        .expect("failed to open the Sunder window");

        cx.activate(true);
    });
}

fn load_fonts(cx: &mut App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(&include_bytes!("../assets/fonts/InterTight-Regular.ttf")[..]),
        Cow::Borrowed(&include_bytes!("../assets/fonts/InterTight-Medium.ttf")[..]),
        Cow::Borrowed(&include_bytes!("../assets/fonts/InterTight-SemiBold.ttf")[..]),
        Cow::Borrowed(&include_bytes!("../assets/fonts/InterTight-Bold.ttf")[..]),
    ];
    if let Err(error) = cx.text_system().add_fonts(fonts) {
        eprintln!("Sunder could not load its bundled fonts: {error}");
    }
}

fn bind_keys(cx: &mut App) {
    ui::input::bind_text_input_keys(cx);
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewEntry, None),
        KeyBinding::new("secondary-r", SyncNow, None),
        KeyBinding::new("secondary-1", ShowDashboard, None),
        KeyBinding::new("secondary-2", ShowEntries, None),
        KeyBinding::new("secondary-3", ShowAccounts, None),
        KeyBinding::new("secondary-4", ShowCategories, None),
        KeyBinding::new("secondary-5", ShowSettings, None),
        KeyBinding::new("secondary-f", FocusSearch, None),
        KeyBinding::new("secondary-enter", SaveOverlay, None),
        KeyBinding::new("escape", CloseOverlay, None),
        KeyBinding::new("secondary-shift-t", ToggleTheme, None),
        // Web-client shortcuts, kept for muscle memory.
        KeyBinding::new("alt-space", NewEntry, None),
        KeyBinding::new("alt-n e", NewEntry, None),
        KeyBinding::new("alt-n a", NewAccount, None),
        KeyBinding::new("alt-n c", NewCategory, None),
        KeyBinding::new("alt-1", ShowDashboard, None),
        KeyBinding::new("alt-2", ShowEntries, None),
        KeyBinding::new("alt-e", ShowEntries, None),
        KeyBinding::new("alt-3", ShowAccounts, None),
        KeyBinding::new("alt-a", ShowAccounts, None),
        KeyBinding::new("alt-4", ShowCategories, None),
        KeyBinding::new("alt-c", ShowCategories, None),
        KeyBinding::new("alt-5", ShowSettings, None),
        KeyBinding::new("alt-t", ToggleTheme, None),
        KeyBinding::new("alt-/", ShowShortcuts, None),
        KeyBinding::new("alt-?", ShowShortcuts, None),
        KeyBinding::new("alt-shift-/", ShowShortcuts, None),
        KeyBinding::new("secondary-/", ShowShortcuts, None),
    ]);
}
