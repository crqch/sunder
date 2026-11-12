use crate::api::ImportKind;
use crate::app::{AuthMode, Boot, CategoryCountFilter, Screen, SortField, SunderApp, SyncState};
use crate::assets::{Icon, icon, logo};
use crate::format;
use crate::model::{self, Entry, MonthFlow};
use crate::search::SearchKind;
use crate::store::Snapshot;
use crate::theme::{Theme, space, text as type_scale, theme};
use crate::ui::components::{
    BadgeKind, SunderTooltip, amount, badge, card, chip, danger_button, divider, dot, empty_state,
    field_label, ghost_button, icon_badge, icon_button, icon_el, keycap, label, link_button,
    page_title, primary_button, secondary_button, section_title, shortcut, spinner,
};
use gpui::{
    AnyElement, ClickEvent, Div, ElementId, IntoElement, SharedString, Stateful,
    StatefulInteractiveElement, Window, div, prelude::*, px, relative,
};

pub fn render_body(
    app: &mut SunderApp,
    window: &mut Window,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    match app.boot {
        Boot::Login => login_screen(app, window, cx),
        Boot::Ready => app_shell(app, window, cx),
    }
}

fn login_screen(
    app: &mut SunderApp,
    window: &mut Window,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    if !app.auth.focused {
        let handle = match app.auth.mode {
            AuthMode::Login => app.auth.login.read(cx).focus(),
            AuthMode::Register => app.auth.invite.read(cx).focus(),
        };
        window.focus(&handle);
        app.auth.focused = true;
    }

    let palette = theme(cx).clone();
    let register = app.auth.mode == AuthMode::Register;

    let mut form = div().flex().flex_col().gap(px(14.0));
    form = form.child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .child(logo(24.0, palette.primary))
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(type_scale::BOLD)
                            .text_color(palette.foreground)
                            .child("Sunder"),
                    ),
            )
            .child(
                icon_button(
                    &palette,
                    "auth-theme",
                    if palette.mode == crate::theme::ThemeMode::Dark {
                        Icon::Sun
                    } else {
                        Icon::Moon
                    },
                    true,
                )
                .on_click(cx.listener(|this, _, _, cx| this.toggle_theme(cx)))
                .sunder_tooltip("Toggle theme", &["Alt", "T"]),
            ),
    );
    form = form.child(
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(25.0))
                    .line_height(px(30.0))
                    .font_weight(type_scale::SEMIBOLD)
                    .text_color(palette.foreground)
                    .child(if register {
                        "Create your account"
                    } else {
                        "Sign in"
                    }),
            )
            .child(
                div()
                    .text_size(px(type_scale::BODY))
                    .text_color(palette.muted)
                    .child(if register {
                        "Sunder is invite-only. Enter the code you were given."
                    } else {
                        "Pick up where you left off."
                    }),
            ),
    );
    form = form.child(
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(field_label(&palette, "Server"))
            .child(app.auth.server.clone()),
    );

    if register {
        form = form.child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(&palette, "Invite code"))
                        .child(app.auth.invite.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(&palette, "Email"))
                        .child(app.auth.email.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(&palette, "Username"))
                        .child(app.auth.username.clone()),
                ),
        );
    } else {
        form = form.child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(field_label(&palette, "Email or username"))
                .child(app.auth.login.clone()),
        );
    }

    form = form.child(
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(field_label(&palette, "Password"))
            .child(app.auth.password.clone()),
    );

    if let Some(message) = &app.auth.message {
        let (fg, bg, border, icon_name) = if message.success {
            (
                palette.income,
                palette.income_tint,
                palette.income.opacity(0.25),
                Icon::Check,
            )
        } else {
            (
                palette.expense,
                palette.expense_tint,
                palette.expense.opacity(0.25),
                Icon::Alert,
            )
        };
        form = form.child(
            div()
                .flex()
                .items_start()
                .gap(px(8.0))
                .p(px(10.0))
                .rounded_lg()
                .bg(bg)
                .border_1()
                .border_color(border)
                .child(icon(icon_name, 14.0, fg))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .line_height(px(17.0))
                        .text_color(fg)
                        .child(message.text.clone()),
                ),
        );
    }

    let submit_label = if app.auth.busy {
        if register {
            "Creating account…"
        } else {
            "Signing in…"
        }
    } else if register {
        "Create account"
    } else {
        "Sign in"
    };

    let mut submit = div()
        .id("auth-submit")
        .flex()
        .items_center()
        .justify_center()
        .gap(px(8.0))
        .h(px(44.0))
        .w_full()
        .rounded_lg()
        .bg(palette.primary)
        .text_color(palette.on_primary)
        .text_size(px(14.0))
        .font_weight(type_scale::MEDIUM)
        .relative()
        .cursor_pointer()
        .hover({
            let hover = palette.primary_hover;
            move |style| style.bg(hover)
        })
        .active({
            let pressed = palette.primary_pressed;
            move |style| style.bg(pressed).opacity(0.9)
        });

    if app.auth.busy {
        submit = submit
            .opacity(0.78)
            .child(spinner(&Theme::for_mode(palette.mode), 15.0));
    }
    submit = submit.child(submit_label);
    if !app.auth.busy {
        submit = submit.on_click(cx.listener(|this, _, _, cx| this.submit_auth(cx)));
    }
    form = form.child(submit);

    form = form
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(4.0))
                .pt(px(2.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.muted)
                        .child(if register {
                            "Already have an account?"
                        } else {
                            "New to Sunder?"
                        }),
                )
                .child(
                    link_button(
                        &palette,
                        "auth-mode-toggle",
                        if register {
                            "Sign in"
                        } else {
                            "Create an account"
                        },
                        None,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        let next = if this.auth.mode == AuthMode::Login {
                            AuthMode::Register
                        } else {
                            AuthMode::Login
                        };
                        this.set_auth_mode(next, cx);
                    })),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .pt(px(2.0))
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child("Press Enter to continue"),
                )
                .child(keycap(&palette, "↵")),
        );

    let form = crate::motion::tab_in(
        ElementId::Name(format!("auth-form-{:?}", app.auth.mode).into()),
        form,
        crate::motion::reduce_motion(cx),
    );
    let content = card(&palette)
        .w(px(440.0))
        .p(px(26.0))
        .flex()
        .flex_col()
        .gap(px(16.0))
        .child(form);
    let content = crate::motion::page_in(
        ElementId::Name("login-in".into()),
        content,
        crate::motion::reduce_motion(cx),
    );

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(palette.bg)
        .p(px(24.0))
        .child(content)
        .into_any_element()
}

fn app_shell(
    app: &mut SunderApp,
    _window: &mut Window,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    let palette = theme(cx).clone();
    let screen = app.screen;

    let sidebar = sidebar(app, &palette, cx);

    let content = if let Some(detail) = app.detail.clone() {
        match detail {
            crate::app::DetailView::Entry(id) => entry_detail(app, &palette, &id, cx),
            crate::app::DetailView::Account(id) => account_detail(app, &palette, &id, cx),
            crate::app::DetailView::Category(id) => category_detail(app, &palette, &id, cx),
        }
    } else {
        match screen {
            Screen::Dashboard => dashboard(app, &palette, cx),
            Screen::Entries => entries(app, &palette, cx),
            Screen::Accounts => accounts(app, &palette, cx),
            Screen::Categories => categories(app, &palette, cx),
            Screen::Settings => settings(app, &palette, cx),
            Screen::Admin => admin(app, &palette, cx),
        }
    };

    let reduce_motion = crate::motion::reduce_motion(cx);
    let content = crate::motion::page_in(
        ElementId::Name(format!("page-in-{}", app.page_generation).into()),
        content,
        reduce_motion,
    );

    let scroll_id = ElementId::named_usize("page-scroll", screen as usize);
    let body = div()
        .id(scroll_id)
        .flex_1()
        .w_full()
        .min_h(px(0.0))
        .overflow_y_scroll()
        .child(
            div()
                .w_full()
                .max_w(px(space::CONTENT_MAX))
                .mx_auto()
                .px(px(space::PAGE_PAD))
                .pt(px(26.0))
                .pb(px(48.0))
                .child(content),
        );

    let unsynced = app.unsynced_count();
    let show_sync_bar = should_show_sync_bar(
        app.settings.sync_bar_minimized,
        matches!(app.sync_state, SyncState::Syncing | SyncState::Error(_)),
        unsynced,
    );
    let sync_bar = if show_sync_bar {
        Some(sync_status_bar(app, &palette, unsynced, cx))
    } else {
        None
    };

    let main = div()
        .flex_1()
        .h_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .child(body)
        .children(sync_bar);

    div()
        .size_full()
        .flex()
        .flex_row()
        .bg(palette.bg)
        .child(sidebar)
        .child(main)
        .into_any_element()
}

fn sync_status_bar(
    app: &SunderApp,
    palette: &Theme,
    unsynced: usize,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    let (icon_name, text, tone) = match &app.sync_state {
        SyncState::Syncing => (
            Icon::Refresh,
            "Syncing your data…".to_string(),
            palette.primary,
        ),
        SyncState::Error(message) if message.to_lowercase().contains("reach") => (
            Icon::CloudOff,
            "Offline — showing your local data.".to_string(),
            palette.warning,
        ),
        SyncState::Error(_) => (
            Icon::Alert,
            "Sync failed. Your changes are safe locally.".to_string(),
            palette.expense,
        ),
        _ => (
            Icon::Database,
            format!(
                "{} waiting to sync.",
                format::plural(unsynced, "change", "changes")
            ),
            palette.warning,
        ),
    };

    let syncing = matches!(app.sync_state, SyncState::Syncing);

    let mut header = div()
        .h(px(46.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(9.0))
        .px(px(space::PAGE_PAD))
        .bg(palette.surface)
        .border_t_1()
        .border_color(palette.border)
        .child(if syncing {
            spinner(palette, 15.0).into_any_element()
        } else {
            icon(icon_name, 15.0, tone).into_any_element()
        })
        .child(
            div()
                .text_size(px(type_scale::SMALL))
                .font_weight(type_scale::MEDIUM)
                .text_color(if syncing { palette.muted } else { tone })
                .child(text),
        )
        .child(div().flex_1());

    if !syncing && unsynced > 0 {
        header = header
            .child(
                link_button(palette, "sync-bar-discard", "Discard", Some(Icon::Trash))
                    .on_click(cx.listener(|this, _, _, cx| this.request_discard_unsynced(cx)))
                    .sunder_tooltip("Discard unsynced changes", &[]),
            )
            .child(
                link_button(
                    palette,
                    "sync-bar-details",
                    if app.sync_details_open {
                        "Hide changes"
                    } else {
                        "View changes"
                    },
                    Some(if app.sync_details_open {
                        Icon::ChevronUp
                    } else {
                        Icon::ChevronDown
                    }),
                )
                .debug_selector(|| "sync-bar-details-toggle".to_string())
                .on_click(cx.listener(|this, _, _, cx| this.toggle_sync_details(cx)))
                .sunder_tooltip("Pending changes", &[]),
            );
    }

    header = header
        .child(
            secondary_button(palette, "sync-bar-sync", "Sync now", Some(Icon::Refresh))
                .on_click(cx.listener(|this, _, _, cx| this.sync_now(true, cx))),
        )
        .child(
            icon_button(palette, "sync-bar-minimize", Icon::ChevronDown, true)
                .debug_selector(|| "sync-bar-minimize".to_string())
                .on_click(cx.listener(|this, _, _, cx| this.set_sync_bar_minimized(true, cx))),
        );

    let details = if app.sync_details_open && !syncing && unsynced > 0 {
        let dirty = app.snapshot.dirty_since(app.settings.last_sync.as_deref());
        let last_sync_value = app.settings.last_sync.as_deref();
        let mut groups = div().flex().flex_col().gap(px(8.0));
        if !dirty.accounts.is_empty() {
            groups = groups.child(label(palette, "Accounts"));
            for account in &dirty.accounts {
                let id = account.id.clone();
                let mut row = pending_row(
                    palette,
                    "sync",
                    &account.id,
                    &account.name,
                    "Account",
                    &account.deleted_at,
                    &account.inserted_at,
                    last_sync_value,
                );
                if account.deleted_at.is_none() {
                    row = row.on_click(cx.listener(move |this, _, _, cx| {
                        this.sync_details_open = false;
                        this.open_account_detail(&id, cx);
                    }));
                }
                groups = groups.child(row);
            }
        }
        if !dirty.categories.is_empty() {
            groups = groups.child(label(palette, "Categories"));
            for category in &dirty.categories {
                let id = category.id.clone();
                let mut row = pending_row(
                    palette,
                    "sync",
                    &category.id,
                    &category.title,
                    "Category",
                    &category.deleted_at,
                    &category.inserted_at,
                    last_sync_value,
                );
                if category.deleted_at.is_none() {
                    row = row.on_click(cx.listener(move |this, _, _, cx| {
                        this.sync_details_open = false;
                        this.open_category_detail(&id, cx);
                    }));
                }
                groups = groups.child(row);
            }
        }
        if !dirty.entries.is_empty() {
            groups = groups.child(label(palette, "Entries"));
            for entry in &dirty.entries {
                let id = entry.id.clone();
                let mut row = pending_row(
                    palette,
                    "sync",
                    &entry.id,
                    &entry.title,
                    "Entry",
                    &entry.deleted_at,
                    &entry.inserted_at,
                    last_sync_value,
                );
                if entry.deleted_at.is_none() {
                    row = row.on_click(cx.listener(move |this, _, _, cx| {
                        this.sync_details_open = false;
                        this.open_entry_detail(&id, cx);
                    }));
                }
                groups = groups.child(row);
            }
        }
        let panel = div()
            .id("sync-details-panel")
            .debug_selector(|| "sync-details-panel".to_string())
            .flex()
            .flex_col()
            .gap(px(8.0))
            .w_full()
            .max_h(px(240.0))
            .overflow_y_scroll()
            .px(px(space::PAGE_PAD))
            .py(px(10.0))
            .bg(palette.surface_muted)
            .border_t_1()
            .border_color(palette.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(section_title(palette, "Pending changes"))
                    .child(
                        danger_button(
                            palette,
                            "sync-details-discard",
                            "Discard all",
                            Some(Icon::Trash),
                        )
                        .debug_selector(|| "sync-details-discard".to_string())
                        .on_click(cx.listener(|this, _, _, cx| this.request_discard_unsynced(cx))),
                    ),
            )
            .child(groups);
        Some(
            crate::motion::panel_in(
                ElementId::named_usize("sync-details-in", app.sync_details_seq as usize),
                panel,
                crate::motion::reduce_motion(cx),
            )
            .into_any_element(),
        )
    } else {
        None
    };

    div()
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .child(header)
        .children(details)
        .into_any_element()
}

fn sidebar(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let mut nav = vec![
        (Screen::Dashboard, "Dashboard", Icon::Home, "1"),
        (Screen::Entries, "Entries", Icon::Entries, "2"),
        (Screen::Accounts, "Accounts", Icon::Wallet, "3"),
        (Screen::Categories, "Categories", Icon::Tags, "4"),
        (Screen::Settings, "Settings", Icon::Settings, "5"),
    ];
    if app.user.as_ref().is_some_and(|user| user.is_admin()) {
        nav.push((Screen::Admin, "Admin", Icon::Shield, ""));
    }

    let mut nav_list = div().flex().flex_col().gap(px(2.0)).px(px(10.0));
    for (screen, title, icon_name, shortcut_label) in nav {
        let selected = app.screen == screen;
        let hover = palette.surface_muted;
        let mut item = div()
            .id(("nav", screen as usize))
            .flex()
            .items_center()
            .gap(px(10.0))
            .h(px(36.0))
            .px(px(10.0))
            .rounded_lg()
            .cursor_pointer()
            .text_size(px(type_scale::BODY))
            .font_weight(if selected {
                type_scale::SEMIBOLD
            } else {
                type_scale::MEDIUM
            })
            .when(selected, |style| {
                style.bg(palette.primary_tint).text_color(palette.primary)
            })
            .when(!selected, |style| {
                style
                    .text_color(palette.muted)
                    .hover(move |style| style.bg(hover).text_color(palette.foreground))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.go(screen, window, cx);
            }))
            .child(icon_el(
                icon_name,
                17.0,
                if selected {
                    palette.primary
                } else {
                    palette.muted
                },
            ))
            .child(div().flex_1().child(title));
        if selected && !shortcut_label.is_empty() {
            item = item.child(keycap(palette, shortcut_label));
        }
        item = if shortcut_label.is_empty() {
            item.sunder_tooltip(title, &[])
        } else {
            item.sunder_tooltip(title, &[shortcut_label])
        };
        nav_list = nav_list.child(item);
    }

    let (sync_icon, sync_title, sync_detail, sync_color) = sync_presentation(app, palette);
    let sync_hover = palette.surface_muted;
    let sync_active = matches!(app.sync_state, SyncState::Syncing | SyncState::Error(_))
        || app.unsynced_count() > 0;

    let sync_card = div()
        .id("sidebar-sync")
        .flex()
        .items_center()
        .gap(px(9.0))
        .p(px(10.0))
        .rounded_lg()
        .cursor_pointer()
        .hover(move |style| style.bg(sync_hover))
        .on_click(cx.listener(|this, _, _, cx| this.sync_now(true, cx)))
        .sunder_tooltip("Sync now", &[format::modifier_label(), "R"])
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(30.0))
                .rounded_lg()
                .bg(sync_color.opacity(0.12))
                .child(icon(sync_icon, 15.0, sync_color)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .min_w(px(0.0))
                .flex_1()
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(palette.foreground)
                        .truncate()
                        .child(sync_title),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(palette.subtle)
                        .truncate()
                        .child(sync_detail),
                ),
        )
        .when(app.settings.sync_bar_minimized && sync_active, |element| {
            element.child(
                icon_button(palette, "sidebar-sync-expand", Icon::ChevronUp, true)
                    .debug_selector(|| "sidebar-sync-expand".to_string())
                    .on_click(cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        this.set_sync_bar_minimized(false, cx);
                    })),
            )
        });

    let theme_icon = if palette.mode == crate::theme::ThemeMode::Dark {
        Icon::Sun
    } else {
        Icon::Moon
    };
    let theme_label = if palette.mode == crate::theme::ThemeMode::Dark {
        "Light mode"
    } else {
        "Dark mode"
    };

    let user_name = app.display_name();
    let user_email = app.user_email();
    let avatar_text = format::initials(&user_name);
    let avatar_bg = palette.primary_tint;
    let avatar_fg = palette.primary;

    div()
        .w(px(space::SIDEBAR))
        .h_full()
        .flex()
        .flex_col()
        .bg(palette.sidebar)
        .border_r_1()
        .border_color(palette.border)
        .child(
            div()
                .id("sidebar-logo")
                .debug_selector(|| "sidebar-logo".to_string())
                .flex()
                .items_center()
                .gap(px(9.0))
                .h(px(62.0))
                .px(px(20.0))
                .cursor_pointer()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.go(Screen::Dashboard, window, cx);
                }))
                .child(logo(23.0, palette.primary))
                .child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(palette.foreground)
                        .child("Sunder"),
                ),
        )
        .child(div().h(px(1.0)).bg(palette.border))
        .child(div().pt(px(10.0)).child(nav_list))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(10.0))
                .border_t_1()
                .border_color(palette.border)
                .child(sync_card)
                .child(
                    div()
                        .id("sidebar-theme")
                        .debug_selector(|| "sidebar-theme".to_string())
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .h(px(32.0))
                        .px(px(10.0))
                        .rounded_lg()
                        .cursor_pointer()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.muted)
                        .hover({
                            let hover = palette.surface_muted;
                            let text = palette.foreground;
                            move |style| style.bg(hover).text_color(text)
                        })
                        .on_click(cx.listener(|this, _, _, cx| this.toggle_theme(cx)))
                        .sunder_tooltip("Toggle theme", &["Alt", "T"])
                        .child(crate::motion::fade_quick(
                            ElementId::Name(format!("theme-icon-{:?}", palette.mode).into()),
                            div().child(icon(theme_icon, 15.0, palette.muted)),
                        ))
                        .child(theme_label),
                )
                .child(div().h(px(1.0)).bg(palette.border))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .h(px(44.0))
                        .px(px(6.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(28.0))
                                .rounded_full()
                                .bg(avatar_bg)
                                .text_size(px(10.5))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(avatar_fg)
                                .child(avatar_text),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w(px(0.0))
                                .flex_1()
                                .child(
                                    div()
                                        .text_size(px(type_scale::SMALL))
                                        .font_weight(type_scale::MEDIUM)
                                        .text_color(palette.foreground)
                                        .truncate()
                                        .child(user_name),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(palette.subtle)
                                        .truncate()
                                        .child(if user_email.is_empty() {
                                            app.settings.normalized_base_url()
                                        } else {
                                            user_email
                                        }),
                                ),
                        )
                        .child(
                            icon_button(palette, "sidebar-sign-out", Icon::Logout, true)
                                .on_click(cx.listener(|this, _, _, cx| this.begin_logout(cx))),
                        ),
                ),
        )
}

fn sync_presentation(app: &SunderApp, palette: &Theme) -> (Icon, String, String, gpui::Hsla) {
    let unsynced = app.unsynced_count();
    let last = app
        .settings
        .last_sync
        .as_deref()
        .map(format::relative_from_now)
        .unwrap_or_else(|| "never synced".into());

    match &app.sync_state {
        SyncState::Syncing => (
            Icon::Refresh,
            "Syncing…".into(),
            "Talking to the server".into(),
            palette.primary,
        ),
        SyncState::Error(message) => {
            let offline = message.to_lowercase().contains("reach");
            (
                if offline { Icon::CloudOff } else { Icon::Alert },
                if offline { "Offline" } else { "Sync failed" }.into(),
                last,
                if offline {
                    palette.warning
                } else {
                    palette.expense
                },
            )
        }
        SyncState::Success | SyncState::Idle if unsynced > 0 => (
            Icon::Database,
            format::plural(unsynced, "change waiting", "changes waiting"),
            last,
            palette.warning,
        ),
        SyncState::Success => (Icon::Cloud, "In sync".into(), last, palette.income),
        SyncState::Idle => (Icon::Cloud, "Sync now".into(), last, palette.muted),
    }
}

fn page_header(
    palette: &Theme,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
    actions: Vec<AnyElement>,
) -> Div {
    div()
        .flex()
        .items_start()
        .justify_between()
        .gap(px(16.0))
        .w_full()
        .child(page_title(palette, title, subtitle))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .flex_shrink_0()
                .children(actions),
        )
}

fn sync_action(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> AnyElement {
    let syncing = matches!(app.sync_state, SyncState::Syncing);
    if syncing {
        let mut button = secondary_button(palette, "dashboard-sync", "Syncing", None);
        button = button.child(spinner(palette, 14.0));
        return button.into_any_element();
    }
    secondary_button(palette, "dashboard-sync", "Sync", Some(Icon::Refresh))
        .on_click(cx.listener(|this, _, _, cx| this.sync_now(true, cx)))
        .sunder_tooltip("Sync now", &[format::modifier_label(), "R"])
        .into_any_element()
}

fn new_entry_button(palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> AnyElement {
    primary_button(palette, "new-entry", "New entry", Some(Icon::Plus))
        .on_click(
            cx.listener(|this, _, window, cx| this.open_entry_editor(None, None, None, window, cx)),
        )
        .into_any_element()
}

fn dashboard(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let name = app.display_name();
    let title = format!("{}, {}", model::greeting(), name);
    let subtitle = format!(
        "{}  ·  Here's where your money stands.",
        format::today_label()
    );

    let (month_income, month_expenses) = app.current_month_flow();
    let account_count = app.snapshot.active_accounts().len();

    let stats = div()
        .flex()
        .gap(px(12.0))
        .w_full()
        .child(stat_card(
            palette,
            "Net balance",
            &app.signed_money(app.snapshot.balance()),
            &format!(
                "All time  ·  {}",
                format::plural(account_count, "account", "accounts")
            ),
            Icon::Wallet,
            BadgeKind::Primary,
            palette.foreground,
        ))
        .child(stat_card(
            palette,
            "Total income",
            &app.signed_money(app.snapshot.income()),
            &format!("{} this month", app.signed_money(month_income)),
            Icon::ArrowDownLeft,
            BadgeKind::Success,
            palette.income,
        ))
        .child(stat_card(
            palette,
            "Total expenses",
            &format!("-{}", app.money(app.snapshot.expenses())),
            &format!("-{} this month", app.money(month_expenses)),
            Icon::ArrowUpRight,
            BadgeKind::Danger,
            palette.expense,
        ));

    let flows = model::monthly_flow(&app.snapshot.entries, 6);
    let chart = cash_flow_card(app, palette, &flows);
    let top = top_categories_card(app, palette, cx);

    let recent_entries = app
        .snapshot
        .active_entries()
        .into_iter()
        .take(10)
        .collect::<Vec<_>>();
    let mut recent = card(palette).p(px(6.0)).flex().flex_col();
    if recent_entries.is_empty() {
        recent = recent.child(
            empty_state(
                palette,
                Icon::Sparkles,
                "Your ledger is waiting",
                "Add your first entry and Sunder will start painting the picture.",
            )
            .child(
                div().pt(px(6.0)).child(
                    primary_button(palette, "empty-new-entry", "Add an entry", Some(Icon::Plus))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_entry_editor(None, None, None, window, cx)
                        })),
                ),
            ),
        );
    } else {
        for entry in recent_entries {
            recent = recent.child(entry_row(
                palette,
                &app.snapshot,
                entry,
                app.symbol(),
                app.currency_position(),
                false,
                false,
                false,
                false,
                None,
                cx.listener({
                    let id = entry.id.clone();
                    move |this, _, _, cx| this.open_entry_detail(&id, cx)
                }),
            ));
        }
    }

    let recent_header = div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .child(section_title(palette, "Recent activity"))
        .child(
            link_button(
                palette,
                "view-all-entries",
                "View all",
                Some(Icon::ArrowRight),
            )
            .on_click(cx.listener(|this, _, window, cx| this.go(Screen::Entries, window, cx))),
        );

    div()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .w_full()
        .child(page_header(
            palette,
            title,
            subtitle,
            vec![
                secondary_button(
                    palette,
                    "dashboard-add-account",
                    "Add account",
                    Some(Icon::Wallet),
                )
                .debug_selector(|| "dashboard-add-account".to_string())
                .on_click(
                    cx.listener(|this, _, window, cx| this.open_account_editor(None, window, cx)),
                )
                .sunder_tooltip("New account", &["Alt", "N", "A"])
                .into_any_element(),
                sync_action(app, palette, cx),
                new_entry_button(palette, cx),
            ],
        ))
        .child(stats)
        .child(
            div()
                .flex()
                .gap(px(12.0))
                .w_full()
                .child(div().flex_1().child(chart))
                .child(div().w(px(300.0)).flex_shrink_0().child(top)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .w_full()
                .child(recent_header)
                .child(recent),
        )
}

fn stat_card(
    palette: &Theme,
    title: &str,
    value: &str,
    detail: &str,
    icon_name: Icon,
    tone: BadgeKind,
    value_color: gpui::Hsla,
) -> Div {
    card(palette)
        .flex_1()
        .h(px(118.0))
        .p(px(17.0))
        .flex()
        .flex_col()
        .justify_between()
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.muted)
                        .child(title.to_string()),
                )
                .child(icon_badge(palette, icon_name, tone)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(
                    div()
                        .text_size(px(26.0))
                        .line_height(px(30.0))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(value_color)
                        .child(value.to_string()),
                )
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child(detail.to_string()),
                ),
        )
}

fn cash_flow_card(app: &SunderApp, palette: &Theme, flows: &[MonthFlow]) -> Div {
    let max = flows
        .iter()
        .map(|flow| flow.income.max(flow.expenses))
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let total_income: f64 = flows.iter().map(|flow| flow.income).sum();
    let total_expenses: f64 = flows.iter().map(|flow| flow.expenses).sum();

    let mut chart = div().flex().items_end().gap(px(8.0)).h(px(150.0)).w_full();
    for flow in flows {
        let income_height = ((flow.income / max) * 126.0).max(2.0);
        let expense_height = ((flow.expenses / max) * 126.0).max(2.0);
        chart = chart.child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_end()
                .gap(px(6.0))
                .h_full()
                .child(
                    div()
                        .flex()
                        .items_end()
                        .gap(px(3.0))
                        .h(px(126.0))
                        .child(
                            div()
                                .w(px(9.0))
                                .h(px(income_height as f32))
                                .rounded_t_md()
                                .bg(palette.income.opacity(if flow.income > 0.0 {
                                    0.9
                                } else {
                                    0.15
                                })),
                        )
                        .child(
                            div()
                                .w(px(9.0))
                                .h(px(expense_height as f32))
                                .rounded_t_md()
                                .bg(palette.expense.opacity(if flow.expenses > 0.0 {
                                    0.9
                                } else {
                                    0.15
                                })),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.subtle)
                        .child(format::month_short(flow)),
                ),
        );
    }

    let month_count = flows.len().max(1) as f64;
    let average_income = total_income / month_count;
    let average_expenses = total_expenses / month_count;

    card(palette)
        .flex()
        .flex_col()
        .gap(px(14.0))
        .p(px(18.0))
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(section_title(palette, "Cash flow"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("Last six months"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(legend(
                            palette,
                            palette.income,
                            format!("+{}", app.compact_money(total_income)),
                        ))
                        .child(legend(
                            palette,
                            palette.expense,
                            format!("-{}", app.compact_money(total_expenses)),
                        )),
                ),
        )
        .child(chart)
        .child(div().flex_1())
        .child(divider(palette))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(24.0))
                .child(metric(
                    palette,
                    "Average income",
                    &format!("+{}", app.money(average_income)),
                    palette.income,
                ))
                .child(metric(
                    palette,
                    "Average spend",
                    &format!("-{}", app.money(average_expenses)),
                    palette.expense,
                )),
        )
}

fn metric(palette: &Theme, title: &str, value: &str, color: gpui::Hsla) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(
            div()
                .text_size(px(10.0))
                .text_color(palette.subtle)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(type_scale::SEMIBOLD)
                .text_color(color)
                .child(value.to_string()),
        )
}

fn legend(palette: &Theme, color: gpui::Hsla, value: String) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(5.0))
        .child(dot(color, 7.0))
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(type_scale::MEDIUM)
                .text_color(palette.muted)
                .child(value),
        )
}

fn top_categories_card(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let categories = app.top_categories(5);
    let max = categories
        .iter()
        .map(|category| category.spend)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let total = app.snapshot.expenses().max(1.0);

    let mut list = div().flex().flex_col().gap(px(12.0));
    if categories.is_empty() {
        list = list.child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child("No expenses yet. Your categories will show up here."),
        );
    } else {
        for category in categories {
            let share = category.spend / max;
            let percent = (category.spend / total * 100.0).round() as i64;
            list = list.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(dot(crate::theme::parse_color(&category.color), 8.0))
                                    .child(
                                        div()
                                            .text_size(px(type_scale::SMALL))
                                            .font_weight(type_scale::MEDIUM)
                                            .text_color(palette.foreground)
                                            .child(category.title),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(type_scale::SMALL))
                                    .font_weight(type_scale::MEDIUM)
                                    .text_color(palette.muted)
                                    .child(app.money(category.spend)),
                            ),
                    )
                    .child(
                        div()
                            .h(px(5.0))
                            .w_full()
                            .rounded_full()
                            .bg(palette.surface_muted)
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(share.max(0.02) as f32))
                                    .rounded_full()
                                    .bg(crate::theme::parse_color(&category.color)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(palette.subtle)
                            .child(format!("{percent}% of spending")),
                    ),
            );
        }
    }

    card(palette)
        .flex()
        .flex_col()
        .gap(px(16.0))
        .p(px(18.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(section_title(palette, "Top categories"))
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child("Where the money goes"),
                ),
        )
        .child(list)
        .child(div().flex_1())
        .child(
            link_button(
                palette,
                "view-categories",
                "Manage categories",
                Some(Icon::ArrowRight),
            )
            .on_click(cx.listener(|this, _, window, cx| this.go(Screen::Categories, window, cx))),
        )
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static>;

#[allow(clippy::too_many_arguments)]
fn entry_row(
    palette: &Theme,
    snapshot: &Snapshot,
    entry: &Entry,
    symbol: &str,
    position: crate::format::CurrencyPosition,
    boxed: bool,
    show_date: bool,
    selectable: bool,
    selected: bool,
    on_select: Option<ClickHandler>,
    handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> Stateful<Div> {
    let category = snapshot.category_name(&entry.category_id);
    let account = snapshot.account_name(&entry.account_id);
    let color = crate::theme::parse_color(&snapshot.category_color(&entry.category_id));
    let meta = if show_date {
        format!(
            "{}  ·  {}  ·  {}",
            category,
            account,
            format::month_day(entry.occurred_at())
        )
    } else {
        format!("{category}  ·  {account}")
    };
    let title = entry.title.clone();

    let hover_bg = palette.surface_hover;
    let border_hover = palette.border_strong;
    let mut row = div()
        .id(ElementId::Name(entry.id.clone().into()))
        .flex()
        .items_center()
        .gap(px(11.0))
        .h(px(58.0))
        .px(px(11.0))
        .rounded_lg()
        .cursor_pointer()
        .when(boxed, |style| {
            style
                .bg(palette.surface)
                .border_1()
                .border_color(palette.border)
        })
        .when(!boxed, |style| style.bg(gpui::transparent_black()))
        .hover(move |style| {
            if boxed {
                style.bg(hover_bg).border_color(border_hover)
            } else {
                style.bg(hover_bg)
            }
        })
        .on_click(handler);

    if selectable && let Some(on_select) = on_select {
        row = row.child(
            crate::ui::components::selection_checkbox(
                palette,
                ElementId::Name(format!("select-entry-{}", entry.id).into()),
                selected,
            )
            .on_click(on_select),
        );
    }

    row.child(
        div()
            .w(px(3.0))
            .h(px(28.0))
            .rounded_full()
            .bg(color)
            .flex_shrink_0(),
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .flex_1()
            .min_w(px(0.0))
            .child(
                div()
                    .text_size(px(13.5))
                    .font_weight(type_scale::MEDIUM)
                    .text_color(palette.foreground)
                    .truncate()
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(type_scale::MICRO))
                    .text_color(palette.subtle)
                    .truncate()
                    .child(meta),
            ),
    )
    .child(amount(palette, entry.amount, symbol, position, 14.0))
}

fn admin(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    if !app.user.as_ref().is_some_and(|user| user.is_admin()) {
        return div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .w_full()
            .child(page_header(
                palette,
                "Admin",
                "This area is only available to administrators.",
                Vec::new(),
            ))
            .child(empty_state(
                palette,
                Icon::Shield,
                "Admins only",
                "Ask an administrator to grant your account the is_admin flag.",
            ));
    }

    let refresh = secondary_button(palette, "admin-refresh", "Refresh", Some(Icon::Refresh))
        .on_click(cx.listener(|this, _, _, cx| this.load_admin(cx)))
        .sunder_tooltip("Reload users and invites", &[]);

    let mut content = div().flex().flex_col().gap(px(18.0)).w_full();
    if app.admin.loading && !app.admin.loaded {
        content = content.child(
            card(palette)
                .flex()
                .items_center()
                .justify_center()
                .gap(px(10.0))
                .p(px(30.0))
                .child(spinner(palette, 18.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.muted)
                        .child("Loading admin data…"),
                ),
        );
    } else {
        if let Some(error) = &app.admin.error {
            content = content.child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(8.0))
                    .p(px(12.0))
                    .rounded_lg()
                    .bg(palette.expense_tint)
                    .border_1()
                    .border_color(palette.expense.opacity(0.25))
                    .child(icon(Icon::Alert, 15.0, palette.expense))
                    .child(
                        div()
                            .text_size(px(type_scale::SMALL))
                            .text_color(palette.expense)
                            .child(error.clone()),
                    ),
            );
        }

        let user_rows = if app.admin.users.is_empty() {
            div()
                .flex()
                .items_center()
                .justify_center()
                .p(px(24.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.subtle)
                        .child("No users found."),
                )
        } else {
            let mut rows = div().flex().flex_col();
            for user in &app.admin.users {
                let edit_user = crate::api::AdminUser {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    email: user.email.clone(),
                    flags: user.flags.clone(),
                };
                rows = rows.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .px(px(14.0))
                        .py(px(11.0))
                        .border_b_1()
                        .border_color(palette.border)
                        .child(
                            div()
                                .w(px(110.0))
                                .flex_shrink_0()
                                .text_size(px(10.0))
                                .font_family("Obel Mono")
                                .text_color(palette.subtle)
                                .truncate()
                                .child(user.id.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.0))
                                .flex_1()
                                .min_w(px(0.0))
                                .child(
                                    div()
                                        .text_size(px(type_scale::SMALL))
                                        .font_weight(type_scale::SEMIBOLD)
                                        .text_color(palette.foreground)
                                        .truncate()
                                        .child(user.username.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.muted)
                                        .truncate()
                                        .child(user.email.clone()),
                                ),
                        )
                        .child(div().flex().items_center().gap(px(4.0)).children(
                            if user.flags.is_empty() {
                                vec![
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child("none")
                                        .into_any_element(),
                                ]
                            } else {
                                user.flags
                                    .iter()
                                    .map(|flag| badge(palette, flag.clone(), BadgeKind::Primary))
                                    .map(IntoElement::into_any_element)
                                    .collect()
                            },
                        ))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .child(
                                    icon_button(
                                        palette,
                                        ElementId::Name(format!("admin-edit-{}", user.id).into()),
                                        Icon::Pencil,
                                        true,
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            this.open_admin_editor(edit_user.clone(), window, cx)
                                        },
                                    )),
                                )
                                .child(
                                    icon_button(
                                        palette,
                                        ElementId::Name(format!("admin-delete-{}", user.id).into()),
                                        Icon::Trash,
                                        true,
                                    )
                                    .on_click(cx.listener({
                                        let id = user.id.clone();
                                        move |this, _, _, cx| {
                                            this.request_admin_delete_user(&id, cx)
                                        }
                                    }))
                                    .sunder_tooltip("Delete user", &[]),
                                ),
                        ),
                );
            }
            rows
        };

        content = content.child(
            card(palette)
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .py(px(14.0))
                        .border_b_1()
                        .border_color(palette.border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(section_title(palette, "Users"))
                                .child(badge(
                                    palette,
                                    app.admin.users.len().to_string(),
                                    BadgeKind::Neutral,
                                )),
                        )
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("ID · account · flags"),
                        ),
                )
                .child(user_rows),
        );

        let mut invite_rows = div().flex().flex_col();
        if app.admin.invites.is_empty() {
            invite_rows = invite_rows.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(px(24.0))
                    .child(
                        div()
                            .text_size(px(type_scale::SMALL))
                            .text_color(palette.subtle)
                            .child("No invite codes yet."),
                    ),
            );
        } else {
            for invite in &app.admin.invites {
                let used = invite.used_by.clone();
                invite_rows = invite_rows.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .px(px(14.0))
                        .py(px(11.0))
                        .border_b_1()
                        .border_color(palette.border)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.0))
                                .flex_1()
                                .min_w(px(0.0))
                                .child(
                                    div()
                                        .text_size(px(type_scale::SMALL))
                                        .font_family("Obel Mono")
                                        .font_weight(type_scale::MEDIUM)
                                        .text_color(palette.foreground)
                                        .truncate()
                                        .child(invite.token.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .truncate()
                                        .child(match invite.used_by.clone() {
                                            Some(used_by) => {
                                                format!("{}  ·  used by {used_by}", invite.id)
                                            }
                                            None => invite.id.clone(),
                                        }),
                                ),
                        )
                        .child(if used.is_some() {
                            badge(palette, "Used", BadgeKind::Neutral)
                        } else {
                            badge(palette, "Available", BadgeKind::Success)
                        })
                        .child(
                            icon_button(
                                palette,
                                ElementId::Name(
                                    format!("admin-invite-delete-{}", invite.id).into(),
                                ),
                                Icon::Trash,
                                true,
                            )
                            .on_click(cx.listener({
                                let id = invite.id.clone();
                                move |this, _, _, cx| this.request_admin_delete_invite(&id, cx)
                            }))
                            .sunder_tooltip("Delete invite", &[]),
                        ),
                );
            }
        }

        content = content.child(
            card(palette)
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .py(px(14.0))
                        .border_b_1()
                        .border_color(palette.border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(section_title(palette, "Invite codes"))
                                .child(badge(
                                    palette,
                                    app.admin.invites.len().to_string(),
                                    BadgeKind::Neutral,
                                )),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .p(px(14.0))
                        .border_b_1()
                        .border_color(palette.border)
                        .child(div().flex_1().child(app.admin_invite.clone()))
                        .child(
                            primary_button(
                                palette,
                                "admin-create-invite",
                                "Create invite",
                                Some(Icon::Plus),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.create_admin_invite(cx))),
                        ),
                )
                .child(invite_rows),
        );
    }

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .child(page_header(
            palette,
            "Admin",
            "Manage users and invite codes.",
            vec![refresh.into_any_element()],
        ))
        .child(content)
}

fn power_search_chip(
    palette: &Theme,
    id: String,
    label: &'static str,
    display: String,
    on_remove: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> Stateful<Div> {
    let debug = id.clone();
    div()
        .id(ElementId::Name(id.into()))
        .debug_selector(move || debug.clone())
        .flex()
        .items_center()
        .gap(px(4.0))
        .h(px(26.0))
        .px(px(9.0))
        .rounded_full()
        .bg(palette.surface_muted)
        .cursor_pointer()
        .hover({
            let hover = palette.surface_hover;
            move |style| style.bg(hover)
        })
        .on_click(on_remove)
        .child(
            div()
                .text_size(px(type_scale::MICRO))
                .text_color(palette.muted)
                .child(format!("{label} ")),
        )
        .child(
            div()
                .text_size(px(type_scale::MICRO))
                .font_weight(type_scale::SEMIBOLD)
                .text_color(palette.foreground)
                .child(display),
        )
        .child(icon(Icon::X, 10.0, palette.subtle))
}

fn entries(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let visible = app.visible_entries(cx);
    let total_entries = app.snapshot.active_entries().len();
    let showing = visible.len();
    let query = app.search.read(cx).text().to_string();
    let power_query = app.entry_query(cx);
    let filter_count = app.active_filter_count(cx);
    let filter_active = filter_count > 0;
    let selected_count = app.selected_entries.len();
    let all_selected = !visible.is_empty()
        && visible
            .iter()
            .all(|entry| app.selected_entries.contains(&entry.id));

    let mut filter_button =
        secondary_button(palette, "toggle-filters", "Filters", Some(Icon::Filter));
    if app.show_filters || filter_active {
        filter_button = filter_button
            .bg(palette.primary_tint)
            .border_color(palette.primary.opacity(0.3))
            .text_color(palette.primary);
    }
    if filter_count > 0 {
        filter_button =
            filter_button.child(badge(palette, filter_count.to_string(), BadgeKind::Primary));
    }
    let filter_button = filter_button
        .debug_selector(|| "toggle-filters".to_string())
        .on_click(cx.listener(|this, _, _, cx| {
            this.show_filters = !this.show_filters;
            cx.notify();
        }))
        .sunder_tooltip("Filter entries", &[]);

    let mut header_actions = vec![filter_button.into_any_element()];
    header_actions.push(
        primary_button(palette, "entries-new", "New entry", Some(Icon::Plus))
            .on_click(cx.listener(|this, _, window, cx| {
                let account = this.filter.account_ids.first().cloned();
                let category = this.filter.category_ids.first().cloned();
                this.open_entry_editor(None, account, category, window, cx);
            }))
            .sunder_tooltip("New entry", &[format::modifier_label(), "N"])
            .into_any_element(),
    );

    let search_active = !query.trim().is_empty() || !app.search_tokens.is_empty();
    let subtitle = if filter_active || search_active {
        format!(
            "{} of {} — filtered",
            format::plural(showing, "entry", "entries"),
            total_entries
        )
    } else {
        format!(
            "{} across your accounts",
            format::plural(showing, "entry", "entries")
        )
    };

    let structured_count = power_query
        .terms
        .iter()
        .filter(|term| term.kind != SearchKind::Text)
        .count();
    let suggestions = if app.search_menu_open {
        app.search_suggestions(cx)
    } else {
        Vec::new()
    };
    let selected_suggestion = if suggestions.is_empty() {
        0
    } else {
        app.search_suggestion.min(suggestions.len() - 1)
    };

    let search_focus = app.search.read(cx).focus();
    let mut search_bar = div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap(px(4.0))
        .min_h(px(34.0))
        .px(px(6.0))
        .py(px(3.0))
        .rounded_lg()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_muted)
        .track_focus(&search_focus)
        .focus(move |style| style.border_color(palette.primary).bg(palette.surface));
    for (index, term) in app.search_tokens.iter().enumerate() {
        search_bar = search_bar.child(power_search_chip(
            palette,
            format!("power-term-{index}"),
            term.label,
            term.display.clone(),
            cx.listener(move |this, _, _, cx| this.remove_search_term(index, cx)),
        ));
    }
    search_bar = search_bar.child(div().flex_1().min_w(px(150.0)).child(app.search.clone()));
    if !query.trim().is_empty() || !app.search_tokens.is_empty() || filter_active {
        search_bar = search_bar.child(
            icon_button(palette, "power-search-clear", Icon::X, true)
                .debug_selector(|| "power-search-clear".to_string())
                .sunder_tooltip("Clear search and filters", &[])
                .on_click(cx.listener(|this, _, _, cx| this.clear_entry_filters(cx))),
        );
    }

    let mut power_search = div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(search_bar);

    if !suggestions.is_empty() {
        let mut menu = div()
            .flex()
            .flex_col()
            .gap(px(1.0))
            .p(px(4.0))
            .rounded_lg()
            .border_1()
            .border_color(palette.border)
            .bg(palette.surface)
            .shadow(vec![gpui::BoxShadow {
                color: palette.shadow.opacity(0.9),
                offset: gpui::point(px(0.0), px(10.0)),
                blur_radius: px(28.0),
                spread_radius: px(0.0),
            }]);
        for (index, suggestion) in suggestions.iter().enumerate() {
            let selected = index == selected_suggestion;
            menu = menu.child(
                div()
                    .id(ElementId::Name(format!("search-suggestion-{index}").into()))
                    .debug_selector(move || format!("search-suggestion-{index}"))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .h(px(30.0))
                    .px(px(8.0))
                    .rounded_md()
                    .cursor_pointer()
                    .when(selected, |element| element.bg(palette.primary_tint))
                    .hover({
                        let hover = palette.surface_hover;
                        move |style| style.bg(hover)
                    })
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.apply_search_suggestion(index, cx)),
                    )
                    .child(keycap(palette, suggestion.badge.clone()))
                    .child(
                        div()
                            .text_size(px(type_scale::SMALL))
                            .text_color(if selected {
                                palette.primary
                            } else {
                                palette.foreground
                            })
                            .truncate()
                            .child(suggestion.label.clone()),
                    ),
            );
        }
        menu = menu.child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .pt(px(5.0))
                .border_t_1()
                .border_color(palette.border)
                .child(keycap(palette, "Tab"))
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child("to complete"),
                )
                .child(keycap(palette, "Up/Down"))
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child("to choose"),
                ),
        );
        power_search = power_search.child(gpui::deferred(menu).with_priority(20));
    }

    power_search = power_search.child(
        div()
            .text_size(px(type_scale::MICRO))
            .text_color(palette.subtle)
            .child(if app.search_tokens.is_empty() && query.trim().is_empty() {
                "Power search · title: · amount:>50 · type: · category: · after:".to_string()
            } else if structured_count == 0 {
                format!("{} match{}", showing, if showing == 1 { "" } else { "es" })
            } else {
                format!(
                    "{} filter{} · {} match{}",
                    structured_count,
                    if structured_count == 1 { "" } else { "s" },
                    showing,
                    if showing == 1 { "" } else { "es" }
                )
            }),
    );

    let sort_field_label = match app.filter.sort {
        SortField::Date => "Date",
        SortField::Amount => "Amount",
        SortField::Title => "Title",
    };
    let sort_arrow = if app.filter.descending { "↓" } else { "↑" };
    let mut sort_control = div().relative().child(
        secondary_button(
            palette,
            "entries-sort",
            format!("Sort: {sort_field_label} {sort_arrow}"),
            Some(Icon::ChevronDown),
        )
        .debug_selector(|| "entries-sort".to_string())
        .sunder_tooltip("Sort entries", &[])
        .on_click(cx.listener(|this, _, _, cx| {
            this.sort_menu_open = !this.sort_menu_open;
            cx.notify();
        })),
    );
    if app.sort_menu_open {
        let options = [
            (SortField::Date, true, "Newest first", "sort-date-desc"),
            (SortField::Date, false, "Oldest first", "sort-date-asc"),
            (
                SortField::Amount,
                true,
                "Amount: high to low",
                "sort-amount-desc",
            ),
            (
                SortField::Amount,
                false,
                "Amount: low to high",
                "sort-amount-asc",
            ),
            (SortField::Title, false, "Title: A to Z", "sort-title-asc"),
            (SortField::Title, true, "Title: Z to A", "sort-title-desc"),
        ];
        let mut menu = div()
            .absolute()
            .top(px(38.0))
            .right_0()
            .w(px(200.0))
            .flex()
            .flex_col()
            .gap(px(1.0))
            .p(px(4.0))
            .rounded_lg()
            .border_1()
            .border_color(palette.border)
            .bg(palette.surface)
            .shadow(vec![gpui::BoxShadow {
                color: palette.shadow.opacity(0.9),
                offset: gpui::point(px(0.0), px(10.0)),
                blur_radius: px(28.0),
                spread_radius: px(0.0),
            }]);
        for (field, descending, label, id) in options {
            let active = app.filter.sort == field && app.filter.descending == descending;
            let debug = id.to_string();
            menu =
                menu.child(
                    div()
                        .id(id)
                        .debug_selector(move || debug.clone())
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(8.0))
                        .h(px(30.0))
                        .px(px(8.0))
                        .rounded_md()
                        .cursor_pointer()
                        .when(active, |element| element.bg(palette.primary_tint))
                        .hover({
                            let hover = palette.surface_hover;
                            move |style| style.bg(hover)
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.set_entry_sort(field, descending, cx)
                        }))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .text_color(if active {
                                    palette.primary
                                } else {
                                    palette.foreground
                                })
                                .child(label),
                        )
                        .when(active, |element| {
                            element.child(icon(Icon::Check, 12.0, palette.primary))
                        }),
                );
        }
        sort_control = sort_control.child(gpui::deferred(menu).with_priority(30));
    }

    let toolbar = div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .w_full()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .flex_1()
                .min_w(px(0.0))
                .child(power_search),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(sort_control)
                .child(shortcut(palette, &[format::modifier_label(), "F"]))
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child("to search"),
                ),
        );

    let mut list = div().flex().flex_col().gap(px(14.0));

    if visible.is_empty() {
        if total_entries == 0 {
            list = list.child(
                empty_state(
                    palette,
                    Icon::Entries,
                    "No entries yet",
                    "Log what you earn and spend. Sunder keeps it local, then syncs when you're online.",
                )
                .child(
                    div().pt(px(6.0)).child(
                        primary_button(palette, "entries-empty-new", "Add your first entry", Some(Icon::Plus))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_entry_editor(None, None, None, window, cx)
                            })),
                    ),
                ),
            );
        } else {
            list = list.child(
                empty_state(
                    palette,
                    Icon::Search,
                    "Nothing matches",
                    "Try a different search, or clear the filters to see everything again.",
                )
                .child(
                    div().pt(px(6.0)).child(
                        secondary_button(
                            palette,
                            "entries-clear",
                            "Clear search & filters",
                            Some(Icon::X),
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.clear_entry_filters(cx))),
                    ),
                ),
            );
        }
    } else {
        let selection = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.0))
            .px(px(2.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        crate::ui::components::selection_checkbox(
                            palette,
                            "entries-select-all",
                            all_selected,
                        )
                        .on_click(
                            cx.listener(|this, _, _, cx| this.toggle_all_visible_entries(cx)),
                        ),
                    )
                    .child(
                        div()
                            .text_size(px(type_scale::SMALL))
                            .font_weight(type_scale::MEDIUM)
                            .text_color(palette.muted)
                            .child(format!("Select all ({showing})")),
                    ),
            )
            .when(selected_count > 0, |element| {
                element.child(
                    danger_button(
                        palette,
                        "entries-delete-selected",
                        format!("Delete selected ({selected_count})"),
                        Some(Icon::Trash),
                    )
                    .debug_selector(|| "entries-delete-selected".to_string())
                    .on_click(
                        cx.listener(|this, _, _, cx| this.request_delete_selected_entries(cx)),
                    ),
                )
            });
        list = list.child(selection);

        if app.filter.sort == SortField::Date {
            let mut groups: Vec<(String, Vec<&Entry>, f64)> = Vec::new();
            for entry in visible {
                let label = entry
                    .date_naive()
                    .map(model::day_label)
                    .unwrap_or_else(|| "Undated".to_string());
                if let Some((_, rows, total)) =
                    groups.iter_mut().find(|(group, _, _)| *group == label)
                {
                    rows.push(entry);
                    *total += entry.amount;
                } else {
                    groups.push((label, vec![entry], entry.amount));
                }
            }

            for (label, rows, total) in groups {
                let mut group = div().flex().flex_col().gap(px(6.0));
                group = group.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(2.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child(label),
                        )
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .font_weight(type_scale::MEDIUM)
                                .text_color(palette.subtle)
                                .child(app.signed_money(total)),
                        ),
                );
                for entry in rows {
                    let id = entry.id.clone();
                    let selected = app.selected_entries.contains(&id);
                    let on_select: ClickHandler = Box::new(cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.toggle_selected_entry(&id, cx);
                        }
                    }));
                    group = group.child(entry_row(
                        palette,
                        &app.snapshot,
                        entry,
                        app.symbol(),
                        app.currency_position(),
                        true,
                        false,
                        true,
                        selected,
                        Some(on_select),
                        cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| {
                                if this.selected_entries.is_empty() {
                                    this.open_entry_detail(&id, cx);
                                } else {
                                    this.toggle_selected_entry(&id, cx);
                                }
                            }
                        }),
                    ));
                }
                list = list.child(group);
            }
        } else {
            let mut flat = div().flex().flex_col().gap(px(6.0));
            for entry in visible {
                let id = entry.id.clone();
                let selected = app.selected_entries.contains(&id);
                let on_select: ClickHandler = Box::new(cx.listener({
                    let id = id.clone();
                    move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.toggle_selected_entry(&id, cx);
                    }
                }));
                flat = flat.child(entry_row(
                    palette,
                    &app.snapshot,
                    entry,
                    app.symbol(),
                    app.currency_position(),
                    true,
                    true,
                    true,
                    selected,
                    Some(on_select),
                    cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| {
                            if this.selected_entries.is_empty() {
                                this.open_entry_detail(&id, cx);
                            } else {
                                this.toggle_selected_entry(&id, cx);
                            }
                        }
                    }),
                ));
            }
            list = list.child(flat);
        }
    }

    let filters = if app.show_filters {
        Some(filter_panel(app, palette, cx))
    } else {
        None
    };

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .child(page_header(palette, "Entries", subtitle, header_actions))
        .child(toolbar)
        .children(filters)
        .child(list)
}

fn filter_panel(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> AnyElement {
    let accounts = app
        .snapshot
        .active_accounts()
        .into_iter()
        .map(|account| (account.id.clone(), account.name.clone()))
        .collect::<Vec<_>>();
    let categories = app
        .snapshot
        .active_categories()
        .into_iter()
        .map(|category| {
            (
                category.id.clone(),
                category.title.clone(),
                category.color.clone(),
            )
        })
        .collect::<Vec<_>>();

    let mut account_chips = div().flex().flex_wrap().gap(px(6.0));
    account_chips = account_chips.child(
        chip(
            palette,
            "filter-account-all",
            "All accounts",
            app.filter.account_ids.is_empty(),
            None,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.filter.account_ids.clear();
            cx.notify();
        })),
    );
    for (id, name) in accounts {
        account_chips = account_chips.child(
            chip(
                palette,
                ElementId::Name(format!("filter-account-{id}").into()),
                name,
                app.filter.account_ids.contains(&id),
                None,
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.filter.toggle_account(&id);
                cx.notify();
            })),
        );
    }

    let mut category_chips = div().flex().flex_wrap().gap(px(6.0));
    category_chips = category_chips.child(
        chip(
            palette,
            "filter-category-all",
            "All categories",
            app.filter.category_ids.is_empty(),
            None,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.filter.category_ids.clear();
            cx.notify();
        })),
    );
    for (id, title, color) in categories {
        category_chips = category_chips.child(
            chip(
                palette,
                ElementId::Name(format!("filter-category-{id}").into()),
                title,
                app.filter.category_ids.contains(&id),
                Some(crate::theme::parse_color(&color)),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.filter.toggle_category(&id);
                cx.notify();
            })),
        );
    }

    let mut sort_chips = div().flex().items_center().gap(px(6.0));
    for (field, label) in [
        (SortField::Date, "Date"),
        (SortField::Amount, "Amount"),
        (SortField::Title, "Title"),
    ] {
        sort_chips = sort_chips.child(
            chip(
                palette,
                ElementId::Name(format!("sort-{label}").into()),
                label,
                app.filter.sort == field,
                None,
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.filter.sort = field;
                cx.notify();
            })),
        );
    }
    sort_chips = sort_chips
        .child(
            chip(
                palette,
                "sort-desc",
                "Descending",
                app.filter.descending,
                None,
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.filter.descending = true;
                cx.notify();
            })),
        )
        .child(
            chip(
                palette,
                "sort-asc",
                "Ascending",
                !app.filter.descending,
                None,
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.filter.descending = false;
                cx.notify();
            })),
        );

    card(palette)
        .flex()
        .flex_col()
        .gap(px(14.0))
        .p(px(16.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(section_title(palette, "Filters"))
                .child(
                    link_button(palette, "filter-clear", "Clear all", Some(Icon::X))
                        .on_click(cx.listener(|this, _, _, cx| this.clear_entry_filters(cx))),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label(palette, "Accounts"))
                .child(account_chips),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label(palette, "Categories"))
                .child(category_chips),
        )
        .child(divider(palette))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(18.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(label(palette, "Amount range"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().w(px(96.0)).child(app.filter_min.clone()))
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child("to"),
                                )
                                .child(div().w(px(96.0)).child(app.filter_max.clone())),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(label(palette, "Date range"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().w(px(142.0)).child(app.filter_start.clone()))
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child("to"),
                                )
                                .child(div().w(px(142.0)).child(app.filter_end.clone()))
                                .child(
                                    ghost_button(
                                        palette,
                                        "filter-calendar",
                                        "Calendar",
                                        Some(Icon::Calendar),
                                    )
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.open_filter_calendar(cx)),
                                    ),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label(palette, "Sort"))
                .child(sort_chips),
        )
        .into_any_element()
}

fn detail_header(
    palette: &Theme,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
    actions: Vec<AnyElement>,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .w_full()
        .child(
            icon_button(palette, "detail-back", Icon::ArrowLeft, false)
                .on_click(cx.listener(|this, _, _, cx| this.close_detail(cx)))
                .sunder_tooltip("Back", &[]),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .flex_1()
                .min_w(px(0.0))
                .child(
                    div()
                        .text_size(px(type_scale::TITLE))
                        .line_height(px(31.0))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(palette.foreground)
                        .truncate()
                        .child(title.into()),
                )
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.muted)
                        .child(subtitle.into()),
                ),
        )
        .child(div().flex().items_center().gap(px(8.0)).children(actions))
}

fn entry_detail(
    app: &SunderApp,
    palette: &Theme,
    id: &str,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let Some(entry) = app.snapshot.entry(id).filter(|entry| entry.is_active()) else {
        return missing_detail(palette, "Entry not found or deleted.", cx);
    };
    let account = app.snapshot.account_name(&entry.account_id);
    let category = app.snapshot.category_name(&entry.category_id);
    let color = crate::theme::parse_color(&app.snapshot.category_color(&entry.category_id));
    let date = entry
        .date_naive()
        .map(|date| date.format("%A, %B %-d, %Y").to_string())
        .unwrap_or_else(|| "Undated".into());

    let account_id = entry.account_id.clone();
    let category_id = entry.category_id.clone();
    let entry_id = entry.id.clone();
    let title = entry.title.clone();
    let location = entry.location.clone();
    let description = entry.description.clone();

    let row = |palette: &Theme, name: &str, value: AnyElement| {
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(16.0))
            .child(
                div()
                    .text_size(px(type_scale::SMALL))
                    .text_color(palette.muted)
                    .child(name.to_string()),
            )
            .child(value)
    };

    div()
        .flex()
        .flex_col()
        .gap(px(18.0))
        .w_full()
        .max_w(px(720.0))
        .mx_auto()
        .child(detail_header(
            palette,
            "Entry details",
            date.clone(),
            vec![
                secondary_button(palette, "entry-detail-edit", "Edit", Some(Icon::Pencil))
                    .sunder_tooltip("Edit entry", &[])
                    .on_click(cx.listener({
                        let id = entry_id.clone();
                        move |this, _, window, cx| {
                            this.open_entry_editor(Some(id.clone()), None, None, window, cx)
                        }
                    }))
                    .into_any_element(),
                crate::ui::components::danger_ghost_button(
                    palette,
                    "entry-detail-delete",
                    "Delete",
                    Some(Icon::Trash),
                )
                .debug_selector(|| "entry-detail-delete".to_string())
                .sunder_tooltip("Delete entry", &[])
                .on_click(cx.listener({
                    let id = entry_id.clone();
                    move |this, _, _, cx| this.request_delete_entry(&id, cx)
                }))
                .into_any_element(),
            ],
            cx,
        ))
        .child(
            card(palette)
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(7.0))
                        .px(px(24.0))
                        .py(px(28.0))
                        .border_b_1()
                        .border_color(palette.border)
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .text_color(palette.subtle)
                                .child(date.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(24.0))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(42.0))
                                .line_height(px(48.0))
                                .font_weight(type_scale::BOLD)
                                .text_color(palette.money_color(entry.amount))
                                .child(app.signed_money(entry.amount)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(15.0))
                        .p(px(24.0))
                        .child(row(
                            palette,
                            "Account",
                            div()
                                .id("entry-detail-account")
                                .cursor_pointer()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::MEDIUM)
                                .text_color(palette.primary)
                                .hover({
                                    let hover = palette.primary_hover;
                                    move |style| style.text_color(hover)
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.open_account_detail(&account_id, cx)
                                }))
                                .child(account)
                                .into_any_element(),
                        ))
                        .child(row(
                            palette,
                            "Category",
                            div()
                                .id("entry-detail-category")
                                .flex()
                                .items_center()
                                .gap(px(7.0))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.open_category_detail(&category_id, cx)
                                }))
                                .child(dot(color, 8.0))
                                .child(
                                    div()
                                        .text_size(px(type_scale::SMALL))
                                        .font_weight(type_scale::MEDIUM)
                                        .text_color(palette.primary)
                                        .child(category),
                                )
                                .into_any_element(),
                        ))
                        .when_some(location, |element, location| {
                            element.child(row(
                                palette,
                                "Location",
                                div()
                                    .text_size(px(type_scale::SMALL))
                                    .font_weight(type_scale::MEDIUM)
                                    .text_color(palette.foreground)
                                    .child(location)
                                    .into_any_element(),
                            ))
                        })
                        .when_some(description, |element, description| {
                            element.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .pt(px(14.0))
                                    .border_t_1()
                                    .border_color(palette.border)
                                    .child(
                                        div()
                                            .text_size(px(type_scale::SMALL))
                                            .text_color(palette.muted)
                                            .child("Note"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(type_scale::BODY))
                                            .line_height(px(19.0))
                                            .text_color(palette.foreground)
                                            .child(description),
                                    ),
                            )
                        }),
                ),
        )
}

fn account_detail(
    app: &SunderApp,
    palette: &Theme,
    id: &str,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let Some(account) = app
        .snapshot
        .account(id)
        .filter(|account| account.is_active())
    else {
        return missing_detail(palette, "Account not found or deleted.", cx);
    };
    let name = account.name.clone();
    let entries: Vec<&Entry> = app
        .snapshot
        .active_entries()
        .into_iter()
        .filter(|entry| entry.account_id == id)
        .collect();
    let balance = app.snapshot.account_balance(id);
    let account_id = id.to_string();

    let mut list = div().flex().flex_col().gap(px(6.0));
    if entries.is_empty() {
        list = list.child(empty_state(
            palette,
            Icon::Wallet,
            "No transactions yet",
            "Add an entry to start building this account's history.",
        ));
    } else {
        for entry in entries {
            list = list.child(entry_row(
                palette,
                &app.snapshot,
                entry,
                app.symbol(),
                app.currency_position(),
                true,
                true,
                false,
                false,
                None,
                cx.listener({
                    let id = entry.id.clone();
                    move |this, _, _, cx| this.open_entry_detail(&id, cx)
                }),
            ));
        }
    }

    div()
        .flex()
        .flex_col()
        .gap(px(18.0))
        .w_full()
        .child(detail_header(
            palette,
            name.clone(),
            format!(
                "{}  ·  {}",
                format::plural(app.snapshot.account_entry_count(id), "entry", "entries"),
                app.signed_money(balance)
            ),
            vec![
                secondary_button(palette, "account-detail-add", "Add entry", Some(Icon::Plus))
                    .sunder_tooltip("Add entry to this account", &[])
                    .on_click(cx.listener({
                        let id = account_id.clone();
                        move |this, _, window, cx| {
                            this.open_entry_editor(None, Some(id.clone()), None, window, cx)
                        }
                    }))
                    .into_any_element(),
                secondary_button(palette, "account-detail-edit", "Edit", Some(Icon::Pencil))
                    .debug_selector(|| "account-detail-edit".to_string())
                    .sunder_tooltip("Edit account", &[])
                    .on_click(cx.listener({
                        let id = account_id.clone();
                        move |this, _, window, cx| {
                            this.open_account_editor(Some(id.clone()), window, cx)
                        }
                    }))
                    .into_any_element(),
                crate::ui::components::danger_ghost_button(
                    palette,
                    "account-detail-delete",
                    "Delete",
                    Some(Icon::Trash),
                )
                .debug_selector(|| "account-detail-delete".to_string())
                .sunder_tooltip("Delete account", &[])
                .on_click(cx.listener({
                    let id = account_id.clone();
                    move |this, _, _, cx| this.request_delete_account(&id, cx)
                }))
                .into_any_element(),
            ],
            cx,
        ))
        .child(
            card(palette)
                .flex()
                .flex_col()
                .gap(px(2.0))
                .p(px(20.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.muted)
                        .child("Current balance"),
                )
                .child(
                    div()
                        .text_size(px(36.0))
                        .line_height(px(42.0))
                        .font_weight(type_scale::BOLD)
                        .text_color(if balance < 0.0 {
                            palette.expense
                        } else {
                            palette.income
                        })
                        .child(app.signed_money(balance)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(section_title(palette, "Transactions"))
                .child(list),
        )
}

fn category_detail(
    app: &SunderApp,
    palette: &Theme,
    id: &str,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let Some(category) = app
        .snapshot
        .category(id)
        .filter(|category| category.is_active())
    else {
        return missing_detail(palette, "Category not found or deleted.", cx);
    };
    let title = category.title.clone();
    let description = category.description.clone().unwrap_or_default();
    let color = crate::theme::parse_color(&category.color);
    let entries: Vec<&Entry> = app
        .snapshot
        .active_entries()
        .into_iter()
        .filter(|entry| entry.category_id == id)
        .collect();
    let category_id = id.to_string();
    let spend = app.snapshot.category_spend(id);

    let mut list = div().flex().flex_col().gap(px(6.0));
    if entries.is_empty() {
        list = list.child(empty_state(
            palette,
            Icon::Tags,
            "No transactions yet",
            "Add an entry to see it counted under this category.",
        ));
    } else {
        for entry in entries {
            list = list.child(entry_row(
                palette,
                &app.snapshot,
                entry,
                app.symbol(),
                app.currency_position(),
                true,
                true,
                false,
                false,
                None,
                cx.listener({
                    let id = entry.id.clone();
                    move |this, _, _, cx| this.open_entry_detail(&id, cx)
                }),
            ));
        }
    }

    div()
        .flex()
        .flex_col()
        .gap(px(18.0))
        .w_full()
        .child(detail_header(
            palette,
            title,
            if description.is_empty() {
                format!(
                    "{} spent  ·  {}",
                    app.money(spend),
                    format::plural(app.snapshot.category_entry_count(id), "entry", "entries")
                )
            } else {
                description
            },
            vec![
                secondary_button(
                    palette,
                    "category-detail-add",
                    "Add entry",
                    Some(Icon::Plus),
                )
                .on_click(cx.listener({
                    let id = category_id.clone();
                    move |this, _, window, cx| {
                        this.open_entry_editor(None, None, Some(id.clone()), window, cx)
                    }
                }))
                .into_any_element(),
                secondary_button(palette, "category-detail-edit", "Edit", Some(Icon::Pencil))
                    .debug_selector(|| "category-detail-edit".to_string())
                    .sunder_tooltip("Edit category", &[])
                    .on_click(cx.listener({
                        let id = category_id.clone();
                        move |this, _, window, cx| {
                            this.open_category_editor(Some(id.clone()), window, cx)
                        }
                    }))
                    .into_any_element(),
                crate::ui::components::danger_ghost_button(
                    palette,
                    "category-detail-delete",
                    "Delete",
                    Some(Icon::Trash),
                )
                .debug_selector(|| "category-detail-delete".to_string())
                .sunder_tooltip("Delete category", &[])
                .on_click(cx.listener({
                    let id = category_id.clone();
                    move |this, _, _, cx| this.request_delete_category(&id, cx)
                }))
                .into_any_element(),
            ],
            cx,
        ))
        .child(
            card(palette)
                .flex()
                .items_center()
                .gap(px(12.0))
                .p(px(18.0))
                .child(div().size(px(16.0)).rounded_full().bg(color))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::MEDIUM)
                                .text_color(palette.muted)
                                .child("Spent in this category"),
                        )
                        .child(
                            div()
                                .text_size(px(24.0))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.expense)
                                .child(app.money(spend)),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(section_title(palette, "Transactions"))
                .child(list),
        )
}

fn missing_detail(
    palette: &Theme,
    message: &'static str,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    icon_button(palette, "detail-back-missing", Icon::ArrowLeft, false)
                        .on_click(cx.listener(|this, _, _, cx| this.close_detail(cx))),
                )
                .child(
                    div()
                        .text_size(px(type_scale::HEADING))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(palette.foreground)
                        .child("Not found"),
                ),
        )
        .child(
            empty_state(palette, Icon::Search, "Nothing here", message).child(
                div().pt(px(6.0)).child(
                    secondary_button(
                        palette,
                        "detail-back-button",
                        "Go back",
                        Some(Icon::ArrowLeft),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.close_detail(cx))),
                ),
            ),
        )
}

fn accounts(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let query = app.account_search.read(cx).text().trim().to_lowercase();
    let accounts = app
        .snapshot
        .active_accounts()
        .into_iter()
        .filter(|account| query.is_empty() || account.name.to_lowercase().contains(&query))
        .map(|account| {
            (
                account.id.clone(),
                account.name.clone(),
                app.snapshot.account_balance(&account.id),
                app.snapshot.account_entry_count(&account.id),
            )
        })
        .collect::<Vec<_>>();
    let total_accounts = app.snapshot.active_accounts().len();
    let selected_count = app.selected_accounts.len();
    let all_selected = !accounts.is_empty()
        && accounts
            .iter()
            .all(|(id, ..)| app.selected_accounts.contains(id));

    let add = primary_button(palette, "accounts-add", "Add account", Some(Icon::Plus))
        .on_click(cx.listener(|this, _, window, cx| this.open_account_editor(None, window, cx)));

    let subtitle = if query.is_empty() {
        format!(
            "{} holding {}",
            format::plural(total_accounts, "account", "accounts"),
            app.signed_money(app.snapshot.balance())
        )
    } else {
        format!("{} of {} accounts", accounts.len(), total_accounts)
    };

    let toolbar = div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .w_full()
        .child(div().w(px(320.0)).child(app.account_search.clone()))
        .when(selected_count > 0, |element| {
            element.child(
                danger_button(
                    palette,
                    "accounts-delete-selected",
                    format!("Delete selected ({selected_count})"),
                    Some(Icon::Trash),
                )
                .debug_selector(|| "accounts-delete-selected".to_string())
                .on_click(cx.listener(|this, _, _, cx| this.request_delete_selected_accounts(cx))),
            )
        });

    let body = if total_accounts == 0 {
        empty_state(
            palette,
            Icon::Wallet,
            "No accounts yet",
            "An account is where entries live — a checking account, a card, or a savings pot.",
        )
        .child(
            div().pt(px(6.0)).child(
                primary_button(
                    palette,
                    "accounts-empty-add",
                    "Create your first account",
                    Some(Icon::Plus),
                )
                .on_click(
                    cx.listener(|this, _, window, cx| this.open_account_editor(None, window, cx)),
                ),
            ),
        )
        .into_any_element()
    } else if accounts.is_empty() {
        empty_state(
            palette,
            Icon::Search,
            "No accounts match",
            "Try a different search to find the account you're looking for.",
        )
        .into_any_element()
    } else {
        let selection = div()
            .flex()
            .items_center()
            .justify_between()
            .px(px(2.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        crate::ui::components::selection_checkbox(
                            palette,
                            "accounts-select-all",
                            all_selected,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.toggle_all_accounts(cx))),
                    )
                    .child(
                        div()
                            .text_size(px(type_scale::SMALL))
                            .font_weight(type_scale::MEDIUM)
                            .text_color(palette.muted)
                            .child(format!("Select all ({})", accounts.len())),
                    ),
            );

        let mut grid = div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .w_full()
            .child(selection);
        let mut cards = div().grid().grid_cols(2).gap(px(12.0)).w_full();
        for (id, name, balance, count) in accounts {
            let account_name = name.clone();
            let card_id = ElementId::Name(format!("account-card-{id}").into());
            let checkbox_id = ElementId::Name(format!("account-select-{id}").into());
            let selected = app.selected_accounts.contains(&id);
            let click_id = id.clone();
            let select_id = id.clone();
            cards = cards.child(
                card(palette)
                    .id(card_id)
                    .p(px(17.0))
                    .h(px(128.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover({
                        let hover = palette.surface_hover;
                        let border = palette.border_strong;
                        move |style| style.bg(hover).border_color(border)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if this.selected_accounts.is_empty() {
                            this.open_account_detail(&click_id, cx);
                        } else {
                            this.toggle_selected_account(&click_id, cx);
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(10.0))
                                    .min_w(px(0.0))
                                    .child(icon_badge(palette, Icon::Wallet, BadgeKind::Primary))
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(type_scale::SEMIBOLD)
                                            .text_color(palette.foreground)
                                            .truncate()
                                            .child(account_name),
                                    ),
                            )
                            .child(
                                crate::ui::components::selection_checkbox(
                                    palette,
                                    checkbox_id,
                                    selected,
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.toggle_selected_account(&select_id, cx);
                                    },
                                )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.0))
                                    .child(
                                        div()
                                            .text_size(px(25.0))
                                            .line_height(px(29.0))
                                            .font_weight(type_scale::SEMIBOLD)
                                            .text_color(if balance < 0.0 {
                                                palette.expense
                                            } else {
                                                palette.foreground
                                            })
                                            .child(app.signed_money(balance)),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(type_scale::MICRO))
                                            .text_color(palette.subtle)
                                            .child(format!(
                                                "Current balance  ·  {}",
                                                format::plural(count, "entry", "entries")
                                            )),
                                    ),
                            )
                            .child(icon_el(Icon::ChevronRight, 15.0, palette.subtle)),
                    ),
            );
        }
        cards = cards.child(div());
        grid = grid.child(cards);
        grid.into_any_element()
    };

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .child(page_header(
            palette,
            "Accounts",
            subtitle,
            vec![add.into_any_element()],
        ))
        .child(toolbar)
        .child(body)
}

fn categories(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let query = app.category_search.read(cx).text().trim().to_lowercase();
    let categories = app
        .visible_categories(cx)
        .into_iter()
        .map(|category| {
            (
                category.id.clone(),
                category.title.clone(),
                category.description.clone().unwrap_or_default(),
                category.color.clone(),
                app.snapshot.category_spend(&category.id),
                app.snapshot.category_entry_count(&category.id),
            )
        })
        .collect::<Vec<_>>();
    let total_categories = app.snapshot.active_categories().len();
    let selected_count = app.selected_categories.len();
    let all_selected = !categories.is_empty()
        && categories
            .iter()
            .all(|(id, ..)| app.selected_categories.contains(id));
    let filters_active = !query.is_empty() || app.category_count_filter != CategoryCountFilter::Any;
    let reduce_motion = crate::motion::reduce_motion(cx);

    let add = primary_button(palette, "categories-add", "Add category", Some(Icon::Plus))
        .on_click(cx.listener(|this, _, window, cx| this.open_category_editor(None, window, cx)));

    let subtitle = if filters_active {
        format!("{} of {} categories", categories.len(), total_categories)
    } else {
        format!(
            "{} for the way you spend",
            format::plural(total_categories, "category", "categories")
        )
    };

    let filter_chips = div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap(px(4.0))
        .children(
            [
                CategoryCountFilter::Any,
                CategoryCountFilter::Empty,
                CategoryCountFilter::OneToFour,
                CategoryCountFilter::FiveToNine,
                CategoryCountFilter::TenPlus,
            ]
            .into_iter()
            .map(|filter| {
                chip(
                    palette,
                    ElementId::Name(format!("category-filter-{}", filter.label()).into()),
                    filter.label(),
                    app.category_count_filter == filter,
                    None,
                )
                .debug_selector(move || format!("category-filter-{}", filter.label()))
                .on_click(
                    cx.listener(move |this, _, _, cx| this.set_category_count_filter(filter, cx)),
                )
                .into_any_element()
            }),
        );

    let toolbar =
        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .flex_wrap()
                            .gap(px(10.0))
                            .child(div().w(px(260.0)).child(app.category_search.clone()))
                            .child(filter_chips),
                    )
                    .when(selected_count > 0, |element| {
                        element.child(
                            danger_button(
                                palette,
                                "categories-delete-selected",
                                format!("Delete selected ({selected_count})"),
                                Some(Icon::Trash),
                            )
                            .debug_selector(|| "categories-delete-selected".to_string())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.request_delete_selected_categories(cx)
                            })),
                        )
                    }),
            )
            .when(filters_active, |element| {
                element.child(
                    div().flex().justify_end().child(
                        link_button(
                            palette,
                            "categories-clear-filters",
                            "Clear filters",
                            Some(Icon::X),
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.clear_category_filters(cx))),
                    ),
                )
            });

    let body = if total_categories == 0 {
        empty_state(
            palette,
            Icon::Tags,
            "No categories yet",
            "Categories give your spending shape — groceries, rent, travel, or whatever matters to you.",
        )
        .child(
            div().pt(px(6.0)).child(
                primary_button(
                    palette,
                    "categories-empty-add",
                    "Create your first category",
                    Some(Icon::Plus),
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.open_category_editor(None, window, cx)
                })),
            ),
        )
        .into_any_element()
    } else if categories.is_empty() {
        empty_state(
            palette,
            Icon::Search,
            "No categories match",
            "Try a different search or item count.",
        )
        .child(
            div().pt(px(6.0)).child(
                secondary_button(palette, "categories-empty-clear", "Clear filters", None)
                    .on_click(cx.listener(|this, _, _, cx| this.clear_category_filters(cx))),
            ),
        )
        .into_any_element()
    } else {
        let header = div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .px(px(14.0))
            .py(px(9.0))
            .bg(palette.surface_muted.opacity(0.55))
            .border_b_1()
            .border_color(palette.border)
            .child(
                div().w(px(26.0)).child(
                    crate::ui::components::selection_checkbox(
                        palette,
                        "categories-select-all",
                        all_selected,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_all_categories(cx))),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(150.0))
                    .child(label(palette, "Category")),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(170.0))
                    .child(label(palette, "Description")),
            )
            .child(div().w(px(72.0)).child(label(palette, "Entries")))
            .child(div().w(px(110.0)).child(label(palette, "Spending")))
            .child(div().w(px(64.0)));

        let mut rows = div().flex().flex_col();
        for (index, (id, title, description, color, spend, count)) in
            categories.into_iter().enumerate()
        {
            let dot_color = crate::theme::parse_color(&color);
            let selected = app.selected_categories.contains(&id);
            let click_id = id.clone();
            let select_id = id.clone();
            let edit_id = id.clone();
            let delete_id = id.clone();
            let row = div()
                .id(ElementId::Name(format!("category-row-{id}").into()))
                .flex()
                .items_center()
                .gap(px(12.0))
                .px(px(14.0))
                .py(px(10.0))
                .border_b_1()
                .border_color(palette.border)
                .cursor_pointer()
                .when(selected, |style| style.bg(palette.primary_tint))
                .hover({
                    let hover = palette.surface_hover;
                    move |style| style.bg(hover)
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    if this.selected_categories.is_empty() {
                        this.open_category_detail(&click_id, cx);
                    } else {
                        this.toggle_selected_category(&click_id, cx);
                    }
                }))
                .child(
                    crate::ui::components::selection_checkbox(
                        palette,
                        ElementId::Name(format!("category-select-{id}").into()),
                        selected,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.toggle_selected_category(&select_id, cx);
                    })),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .size(px(10.0))
                                .rounded_full()
                                .bg(dot_color)
                                .flex_shrink_0(),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_weight(type_scale::MEDIUM)
                                .text_color(palette.foreground)
                                .truncate()
                                .child(title),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.muted)
                        .truncate()
                        .child(if description.is_empty() {
                            "—".to_string()
                        } else {
                            description
                        }),
                )
                .child(div().w(px(72.0)).child(badge(
                    palette,
                    count.to_string(),
                    if count == 0 {
                        BadgeKind::Neutral
                    } else {
                        BadgeKind::Primary
                    },
                )))
                .child(
                    div()
                        .w(px(110.0))
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.foreground)
                        .child(app.money(spend)),
                )
                .child(
                    div()
                        .w(px(64.0))
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(2.0))
                        .child(
                            icon_button(
                                palette,
                                ElementId::Name(format!("category-edit-{edit_id}").into()),
                                Icon::Pencil,
                                true,
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.open_category_editor(Some(edit_id.clone()), window, cx);
                            }))
                            .sunder_tooltip("Edit category", &[]),
                        )
                        .child(
                            icon_button(
                                palette,
                                ElementId::Name(format!("category-delete-{delete_id}").into()),
                                Icon::Trash,
                                true,
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.request_delete_category(&delete_id, cx);
                            }))
                            .sunder_tooltip("Delete category", &[]),
                        ),
                );
            let row = crate::motion::row_in(
                ElementId::Name(
                    format!(
                        "category-row-in-{}-{}-{id}",
                        app.page_generation,
                        app.category_count_filter.label()
                    )
                    .into(),
                ),
                index,
                row,
                reduce_motion,
            );
            rows = rows.child(row);
        }

        card(palette)
            .flex()
            .flex_col()
            .overflow_hidden()
            .w_full()
            .child(header)
            .child(rows)
            .into_any_element()
    };

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .child(page_header(
            palette,
            "Categories",
            subtitle,
            vec![add.into_any_element()],
        ))
        .child(toolbar)
        .child(body)
}

fn settings(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let account_count = app
        .snapshot
        .accounts
        .iter()
        .filter(|a| a.is_active())
        .count();
    let category_count = app
        .snapshot
        .categories
        .iter()
        .filter(|c| c.is_active())
        .count();
    let entry_count = app
        .snapshot
        .entries
        .iter()
        .filter(|e| e.is_active())
        .count();
    let last_sync = app
        .settings
        .last_sync
        .as_deref()
        .map(format::relative_from_now)
        .unwrap_or_else(|| "never".into());
    let unsynced = app.unsynced_count();

    let account_section = card(palette)
        .flex()
        .flex_col()
        .gap(px(16.0))
        .p(px(18.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(40.0))
                        .rounded_full()
                        .bg(palette.primary_tint)
                        .text_size(px(13.0))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(palette.primary)
                        .child(format::initials(&app.display_name())),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .text_size(px(15.0))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .truncate()
                                .child(app.display_name()),
                        )
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .text_color(palette.muted)
                                .truncate()
                                .child(if app.user_email().is_empty() {
                                    "Signed in".to_string()
                                } else {
                                    app.user_email()
                                }),
                        ),
                )
                .when(
                    app.user.as_ref().is_some_and(|user| user.is_admin()),
                    |element| element.child(badge(palette, "Admin", BadgeKind::Primary)),
                )
                .child(
                    secondary_button(palette, "settings-sign-out", "Sign out", Some(Icon::Logout))
                        .debug_selector(|| "settings-sign-out".to_string())
                        .on_click(cx.listener(|this, _, _, cx| this.begin_logout(cx)))
                        .sunder_tooltip("Sign out", &[]),
                ),
        )
        .child(divider(palette))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(icon_el(Icon::Server, 15.0, palette.muted))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .text_color(palette.muted)
                                .child("Server"),
                        ),
                )
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.foreground)
                        .max_w(px(460.0))
                        .truncate()
                        .child(app.settings.normalized_base_url()),
                ),
        );

    let mut theme_chips = div().flex().items_center().gap(px(6.0));
    for (mode, title) in [
        (crate::theme::ThemeMode::Light, "Light"),
        (crate::theme::ThemeMode::Dark, "Dark"),
    ] {
        let selected = app.settings.theme == mode;
        theme_chips = theme_chips.child(
            chip(
                palette,
                ElementId::Name(format!("theme-{title}").into()),
                title,
                selected,
                None,
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.settings.theme != mode {
                    this.toggle_theme(cx);
                }
            })),
        );
    }

    let mut currency_chips = div().flex().items_center().gap(px(4.0));
    for symbol in ["$", "€", "£", "¥"] {
        let selected = app.symbol() == symbol;
        currency_chips = currency_chips.child(
            chip(
                palette,
                ElementId::Name(format!("currency-{symbol}").into()),
                symbol,
                selected,
                None,
            )
            .on_click(cx.listener(move |this, _, _, cx| this.set_currency(symbol, cx))),
        );
    }
    let position_chips = div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .child(
            chip(
                palette,
                "currency-position-before",
                "Before",
                app.settings.currency_position == crate::format::CurrencyPosition::Before,
                None,
            )
            .debug_selector(|| "currency-position-before".to_string())
            .on_click(cx.listener(|this, _, _, cx| {
                this.set_currency_position(crate::format::CurrencyPosition::Before, cx)
            })),
        )
        .child(
            chip(
                palette,
                "currency-position-after",
                "After",
                app.settings.currency_position == crate::format::CurrencyPosition::After,
                None,
            )
            .debug_selector(|| "currency-position-after".to_string())
            .on_click(cx.listener(|this, _, _, cx| {
                this.set_currency_position(crate::format::CurrencyPosition::After, cx)
            })),
        );
    let currency_preview = format::money(
        1234.5,
        &app.settings.currency_symbol,
        app.settings.currency_position,
    );
    let motion_chips = div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .child(
            chip(
                palette,
                "motion-full",
                "Full",
                !app.settings.reduce_motion,
                None,
            )
            .debug_selector(|| "motion-full".to_string())
            .on_click(cx.listener(|this, _, _, cx| this.set_reduce_motion(false, cx))),
        )
        .child(
            chip(
                palette,
                "motion-reduced",
                "Reduced",
                app.settings.reduce_motion,
                None,
            )
            .debug_selector(|| "motion-reduced".to_string())
            .on_click(cx.listener(|this, _, _, cx| this.set_reduce_motion(true, cx))),
        );

    let auto_sync_chips = div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(
            chip(palette, "auto-sync-on", "On", app.settings.auto_sync, None)
                .on_click(cx.listener(|this, _, _, cx| this.set_auto_sync(true, cx))),
        )
        .child(
            chip(
                palette,
                "auto-sync-off",
                "Off",
                !app.settings.auto_sync,
                None,
            )
            .on_click(cx.listener(|this, _, _, cx| this.set_auto_sync(false, cx))),
        );

    let interval_chips =
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .children([1_u32, 5, 15, 60].into_iter().map(|minutes| {
                let label = if minutes == 60 {
                    "1 hour".to_string()
                } else {
                    format!("{minutes} min")
                };
                chip(
                    palette,
                    ElementId::Name(format!("sync-interval-{minutes}").into()),
                    label,
                    app.settings.auto_sync_interval_minutes == minutes,
                    None,
                )
                .on_click(
                    cx.listener(move |this, _, _, cx| this.set_auto_sync_interval(minutes, cx)),
                )
                .into_any_element()
            }));

    let appearance_section = card(palette)
        .flex()
        .flex_col()
        .gap(px(14.0))
        .p(px(18.0))
        .child(section_title(palette, "Appearance"))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(label(palette, "Theme"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("Saved on this device"),
                        ),
                )
                .child(theme_chips),
        )
        .child(divider(palette))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(label(palette, "Currency symbol"))
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child("Custom symbol, up to four characters"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(div().w(px(82.0)).child(app.currency_input.clone()))
                                .child(currency_chips),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(label(palette, "Number formatting"))
                        .child(position_chips),
                )
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child(format!("Preview  ·  {currency_preview}")),
                )
                .child(divider(palette))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(label(palette, "Motion"))
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child("Full motion or a gentler reduced-motion mode"),
                                ),
                        )
                        .child(motion_chips),
                ),
        );

    let sync_section = card(palette)
        .flex()
        .flex_col()
        .gap(px(14.0))
        .p(px(18.0))
        .child(section_title(palette, "Sync"))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(label(palette, "Sync automatically"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child(format!(
                                    "Last synced {last_sync}  ·  {} unsynced",
                                    unsynced
                                )),
                        ),
                )
                .child(auto_sync_chips),
        )
        .when(app.settings.auto_sync, |element| {
            element.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(label(palette, "Sync interval"))
                            .child(
                                div()
                                    .text_size(px(type_scale::MICRO))
                                    .text_color(palette.subtle)
                                    .child("How often Sunder checks for server changes"),
                            ),
                    )
                    .child(interval_chips),
            )
        })
        .child(divider(palette))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(label(palette, "Local snapshot"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .truncate()
                                .max_w(px(340.0))
                                .child(app.paths.location_label()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            secondary_button(
                                palette,
                                "settings-sync",
                                "Sync now",
                                Some(Icon::Refresh),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.sync_now(true, cx)))
                            .sunder_tooltip("Sync now", &[format::modifier_label(), "R"]),
                        )
                        .child(
                            ghost_button(
                                palette,
                                "settings-clear",
                                "Clear local data",
                                Some(Icon::Trash),
                            )
                            .on_click(
                                cx.listener(|this, _, _, cx| this.confirm_clear_local_data(cx)),
                            )
                            .sunder_tooltip("Remove the local snapshot", &[]),
                        ),
                ),
        )
        .child(divider(palette))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(16.0))
                .child(record_count(palette, account_count, "accounts"))
                .child(record_count(palette, category_count, "categories"))
                .child(record_count(palette, entry_count, "entries")),
        );

    let data_section = card(palette)
        .flex()
        .flex_col()
        .gap(px(14.0))
        .p(px(18.0))
        .child(section_title(palette, "Data management"))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(label(palette, "Export data"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("Download a pure Sunder JSON backup."),
                        ),
                )
                .child(
                    secondary_button(
                        palette,
                        "export-data",
                        "Download backup",
                        Some(Icon::Download),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.export_data(cx))),
                ),
        )
        .child(divider(palette))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(label(palette, "Import Sunder backup"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("Restore from a previous Sunder JSON export."),
                        ),
                )
                .child(
                    secondary_button(palette, "import-sunder", "Import JSON", Some(Icon::Upload))
                        .on_click(
                            cx.listener(|this, _, _, cx| this.import_data(ImportKind::Sunder, cx)),
                        ),
                ),
        )
        .child(divider(palette))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(label(palette, "Import Bagels database"))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("Migrate a Bagels SQLite database."),
                        ),
                )
                .child(
                    secondary_button(
                        palette,
                        "import-bagels",
                        "Import Bagels DB",
                        Some(Icon::Database),
                    )
                    .on_click(
                        cx.listener(|this, _, _, cx| this.import_data(ImportKind::Bagels, cx)),
                    ),
                ),
        );

    let dirty = app.snapshot.dirty_since(app.settings.last_sync.as_deref());
    let last_sync_value = app.settings.last_sync.as_deref();
    let mut pending_rows = div().flex().flex_col().gap(px(6.0));
    let mut pending_count = 0;
    for account in &dirty.accounts {
        pending_count += 1;
        let account_id = account.id.clone();
        pending_rows = pending_rows.child(
            pending_row(
                palette,
                "settings",
                &account_id,
                &account.name,
                "Account",
                &account.deleted_at,
                &account.inserted_at,
                last_sync_value,
            )
            .on_click(cx.listener(move |this, _, _, cx| this.open_account_detail(&account_id, cx))),
        );
    }
    for category in &dirty.categories {
        pending_count += 1;
        let category_id = category.id.clone();
        pending_rows = pending_rows.child(
            pending_row(
                palette,
                "settings",
                &category_id,
                &category.title,
                "Category",
                &category.deleted_at,
                &category.inserted_at,
                last_sync_value,
            )
            .on_click(
                cx.listener(move |this, _, _, cx| this.open_category_detail(&category_id, cx)),
            ),
        );
    }
    for entry in &dirty.entries {
        pending_count += 1;
        let entry_id = entry.id.clone();
        pending_rows = pending_rows.child(
            pending_row(
                palette,
                "settings",
                &entry_id,
                &entry.title,
                "Entry",
                &entry.deleted_at,
                &entry.inserted_at,
                last_sync_value,
            )
            .on_click(cx.listener(move |this, _, _, cx| this.open_entry_detail(&entry_id, cx))),
        );
    }

    let pending_section = if pending_count > 0 {
        Some(
            card(palette)
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(18.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(section_title(palette, "Pending changes"))
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child(format!(
                                            "{} waiting to sync",
                                            format::plural(pending_count, "change", "changes")
                                        )),
                                ),
                        )
                        .child(
                            danger_button(
                                palette,
                                "discard-unsynced",
                                "Discard all",
                                Some(Icon::Trash),
                            )
                            .on_click(
                                cx.listener(|this, _, _, cx| this.request_discard_unsynced(cx)),
                            ),
                        ),
                )
                .child(pending_rows),
        )
    } else {
        None
    };

    let shortcuts_section = card(palette)
        .flex()
        .flex_col()
        .gap(px(10.0))
        .p(px(18.0))
        .child(section_title(palette, "Keyboard"))
        .child(shortcut_row(
            palette,
            &[format::modifier_label(), "N"],
            "New entry",
        ))
        .child(shortcut_row(
            palette,
            &[format::modifier_label(), "R"],
            "Sync now",
        ))
        .child(shortcut_row(
            palette,
            &[format::modifier_label(), "F"],
            "Search entries",
        ))
        .child(shortcut_row(
            palette,
            &[format::modifier_label(), "1–5"],
            "Switch sections",
        ))
        .child(shortcut_row(
            palette,
            &[format::modifier_label(), "↵"],
            "Save an entry",
        ))
        .child(shortcut_row(palette, &["esc"], "Close a dialog"))
        .child(
            link_button(
                palette,
                "settings-shortcuts",
                "View all shortcuts",
                Some(Icon::ArrowRight),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_shortcuts = true;
                cx.notify();
            })),
        );

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .child(page_header(
            palette,
            "Settings",
            "Account, appearance, and your local data.",
            Vec::new(),
        ))
        .child(account_section)
        .child(appearance_section)
        .child(sync_section)
        .children(pending_section)
        .child(data_section)
        .child(shortcuts_section)
}

#[allow(clippy::too_many_arguments)]
fn pending_row(
    palette: &Theme,
    scope: &'static str,
    id: &str,
    title: &str,
    kind: &str,
    deleted_at: &Option<String>,
    inserted_at: &str,
    last_sync: Option<&str>,
) -> Stateful<Div> {
    let selector: &'static str = match kind {
        "Account" => "pending-account-row",
        "Category" => "pending-category-row",
        _ => "pending-entry-row",
    };
    let hover = palette.surface_muted;
    let (label, tone) = if deleted_at.is_some() {
        ("Deleted", BadgeKind::Danger)
    } else if last_sync.is_none_or(|last| crate::model::timestamp_is_newer(inserted_at, last)) {
        ("Created", BadgeKind::Success)
    } else {
        ("Updated", BadgeKind::Warning)
    };
    div()
        .id(ElementId::Name(
            format!("{scope}-pending-{kind}-{id}").into(),
        ))
        .debug_selector(move || selector.to_string())
        .flex()
        .items_center()
        .justify_between()
        .gap(px(10.0))
        .px(px(6.0))
        .py(px(5.0))
        .mx(px(-6.0))
        .rounded_md()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .min_w(px(0.0))
                .child(badge(palette, label, tone))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.foreground)
                        .truncate()
                        .child(title.to_string()),
                ),
        )
        .child(
            div()
                .text_size(px(type_scale::MICRO))
                .text_color(palette.subtle)
                .child(kind.to_string()),
        )
}

fn record_count(palette: &Theme, count: usize, label: &str) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(
            div()
                .text_size(px(15.0))
                .font_weight(type_scale::SEMIBOLD)
                .text_color(palette.foreground)
                .child(count.to_string()),
        )
        .child(
            div()
                .text_size(px(type_scale::MICRO))
                .text_color(palette.subtle)
                .child(label.to_string()),
        )
}

fn shortcut_row(palette: &Theme, keys: &[&str], description: &str) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child(description.to_string()),
        )
        .child(shortcut(palette, keys))
}

/// The web client hides the bottom sync bar while the compact sidebar form is
/// active. The bar only appears for active work while not minimized.
pub fn should_show_sync_bar(minimized: bool, sync_active: bool, unsynced: usize) -> bool {
    !minimized && (sync_active || unsynced > 0)
}

#[cfg(test)]
mod screen_tests {
    use super::should_show_sync_bar;

    #[test]
    fn minimized_sync_bar_stays_hidden() {
        assert!(!should_show_sync_bar(true, true, 2));
        assert!(!should_show_sync_bar(true, false, 0));
        assert!(should_show_sync_bar(false, true, 0));
        assert!(should_show_sync_bar(false, false, 3));
        assert!(!should_show_sync_bar(false, false, 0));
    }
}
