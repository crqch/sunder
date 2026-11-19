//! Interaction-level regression tests.
//!
//! These drive the real widgets through GPUI's test platform: clicks, key
//! presses, hover and focus are dispatched through the same event handlers the
//! desktop app uses at runtime.

use crate::api::AdminUser;
use crate::app::{Boot, CloseOverlay, Screen, ShowDashboard, SortField, SunderApp, SyncState};
use crate::format::CurrencyPosition;
use crate::model::{self, Account, Category, Entry, User};
use crate::store::{Paths, Settings};
use crate::theme::ThemeMode;
use crate::ui::components::SunderTooltip;
use gpui::{
    Context, Entity, KeyBinding, Modifiers, MouseButton, NavigationDirection, Render,
    TestAppContext, VisualTestContext, Window, div, point, prelude::*, px,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static NEXT_TEST_DIR: AtomicUsize = AtomicUsize::new(0);

fn test_paths(name: &str) -> Paths {
    let id = NEXT_TEST_DIR.fetch_add(1, Ordering::SeqCst);
    let root: PathBuf = std::env::temp_dir().join(format!(
        "sunder-gpui-test-{name}-{}-{id}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("test data directory");
    Paths {
        data_file: root.join("data.json"),
        settings_file: root.join("settings.json"),
        root,
    }
}

fn test_user() -> User {
    User {
        username: "tester".into(),
        email: "tester@example.com".into(),
        flags: vec!["is_admin".into()],
        default_currency_code: crate::model::default_currency_code(),
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.refresh().expect("refresh windows");
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
    cx.run_until_parked();
}

fn ready_entries_app<'a>(
    cx: &'a mut TestAppContext,
    name: &str,
) -> (Entity<SunderApp>, &'a mut VisualTestContext, Paths) {
    let paths = test_paths(name);
    let (view, cx) = cx.add_window_view({
        let paths = paths.clone();
        move |_, cx| SunderApp::for_test(paths, cx)
    });
    view.update(cx, |app, cx| {
        app.boot = Boot::Ready;
        app.user = Some(test_user());
        app.screen = Screen::Entries;
        cx.notify();
    });
    draw(cx);
    (view, cx, paths)
}

#[gpui::test]
fn sync_bar_minimize_and_expand_click_round_trip(cx: &mut TestAppContext) {
    let (view, cx, paths) = ready_entries_app(cx, "sync-bar");
    view.update(cx, |app, cx| {
        app.sync_state = SyncState::Error("Offline".into());
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("sync-bar-minimize")
        .expect("sync bar minimize button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(
        view.update(cx, |app, _| app.settings.sync_bar_minimized),
        "clicking minimize must update the in-memory preference"
    );
    assert!(
        Settings::load(&paths.settings_file).sync_bar_minimized,
        "clicking minimize must persist the preference"
    );
    draw(cx);

    let bounds = cx
        .debug_bounds("sidebar-sync-expand")
        .expect("collapsed sync card shows an expand button");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(
        !view.update(cx, |app, _| app.settings.sync_bar_minimized),
        "clicking expand must clear the preference"
    );
    assert!(
        !Settings::load(&paths.settings_file).sync_bar_minimized,
        "clicking expand must persist the cleared preference"
    );
}

#[gpui::test]
fn admin_password_reveal_click_toggles_masking(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "password-reveal");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_admin_editor(
                AdminUser {
                    id: "user-1".into(),
                    username: "alex".into(),
                    email: "alex@example.com".into(),
                    flags: vec!["is_admin".into()],
                },
                window,
                cx,
            );
        });
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("admin-password-reveal")
        .expect("password reveal button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(
        !view.update(cx, |app, cx| {
            let editor = app.admin_editor.as_ref().expect("admin editor stays open");
            assert!(!editor.password_masked);
            editor.password.read(cx).is_masked()
        }),
        "revealing must unmask the password input"
    );

    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(
        view.update(cx, |app, cx| {
            let editor = app.admin_editor.as_ref().expect("admin editor stays open");
            assert!(editor.password_masked);
            editor.password.read(cx).is_masked()
        }),
        "clicking again must re-mask the password input"
    );
}

#[gpui::test]
fn calendar_day_click_updates_the_entry_date(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "calendar");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    view.update(cx, |app, cx| {
        app.open_entry_calendar(cx);
        let calendar = app.calendar.as_mut().expect("calendar is open");
        calendar.year = 2026;
        calendar.month = 11;
        calendar.start = None;
        calendar.end = None;
        calendar.view = crate::app::CalendarView::Days;
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("calendar-day-15")
        .expect("calendar day cell is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert!(
            app.calendar.is_none(),
            "single-date pick closes the popover"
        );
        let date = app
            .editor
            .as_ref()
            .expect("entry editor stays open")
            .date
            .read(cx)
            .text()
            .to_string();
        assert_eq!(date, "2026-11-15");
    });
}

#[gpui::test]
fn escape_on_a_dirty_editor_blurs_before_closing(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "dirty-escape");
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new("escape", CloseOverlay, None)]);
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .title
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("Draft title");
    cx.simulate_keystrokes("escape");
    assert!(
        view.update(cx, |app, _| app.editor.is_some()),
        "the first Escape on a dirty form must not discard the draft"
    );
    cx.update(|window, cx| assert!(window.focused(cx).is_some()));

    cx.simulate_keystrokes("escape");
    assert!(
        view.update(cx, |app, _| app.editor.is_none()),
        "a second Escape closes the now-unfocused dirty form"
    );
}

#[gpui::test]
fn calendar_month_and_year_views_drill_down(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "calendar-views");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
            app.open_entry_calendar(cx);
            let calendar = app.calendar.as_mut().expect("calendar is open");
            calendar.year = 2026;
            calendar.month = 11;
            cx.notify();
        });
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("calendar-month-label")
        .expect("month label is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| {
        app.calendar.as_ref().map(|calendar| calendar.view)
            == Some(crate::app::CalendarView::Months)
    }));

    let bounds = cx
        .debug_bounds("calendar-month-3")
        .expect("month cell is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| {
        app.calendar.as_ref().is_some_and(|calendar| {
            calendar.view == crate::app::CalendarView::Days && calendar.month == 3
        })
    }));

    let bounds = cx
        .debug_bounds("calendar-year-label")
        .expect("year label is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    let bounds = cx
        .debug_bounds("calendar-year-2024")
        .expect("year cell is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| {
        app.calendar.as_ref().is_some_and(|calendar| {
            calendar.view == crate::app::CalendarView::Months && calendar.year == 2024
        })
    }));
}

#[gpui::test]
fn filter_calendar_quick_pick_and_done_write_the_range(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "calendar-range");
    view.update(cx, |app, cx| {
        app.show_filters = true;
        app.open_filter_calendar(cx);
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("calendar-quick-3M")
        .expect("three-month quick pick is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    let (start, end) = view.update(cx, |app, _| {
        let calendar = app.calendar.as_ref().expect("range calendar stays open");
        (calendar.start, calendar.end)
    });
    let today = chrono::Local::now().date_naive();
    assert_eq!(
        start,
        Some(crate::app::months_before(today, 3)),
        "quick pick subtracts calendar months, not 30-day blocks"
    );
    assert_eq!(end, Some(today));

    let bounds = cx
        .debug_bounds("calendar-done")
        .expect("done button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert!(app.calendar.is_none());
        let start_text = app.filter_start.read(cx).text().to_string();
        let end_text = app.filter_end.read(cx).text().to_string();
        assert_eq!(
            start_text,
            start.expect("start").format("%Y-%m-%d").to_string()
        );
        assert_eq!(end_text, end.expect("end").format("%Y-%m-%d").to_string());
    });
}

#[gpui::test]
fn entry_editor_can_create_an_account_and_category_inline(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "inline-create");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("entry-new-account")
        .expect("new account button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| app.account_editor.is_some()));
    draw(cx);
    cx.simulate_input("Checking");
    let bounds = cx
        .debug_bounds("account-save")
        .expect("account save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(app.account_editor.is_none());
        assert_eq!(app.snapshot.active_accounts().len(), 1);
        assert_eq!(
            app.editor
                .as_ref()
                .and_then(|editor| editor.account_id.clone()),
            Some(app.snapshot.active_accounts()[0].id.clone()),
            "the newly created account is selected in the entry form"
        );
    });

    let bounds = cx
        .debug_bounds("entry-category-select")
        .expect("category dropdown is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    let bounds = cx
        .debug_bounds("entry-new-category")
        .expect("new category button is rendered inside the dropdown");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| app.category_editor.is_some()));
    draw(cx);
    cx.simulate_input("Groceries");
    let bounds = cx
        .debug_bounds("category-save")
        .expect("category save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(app.category_editor.is_none());
        assert_eq!(app.snapshot.active_categories().len(), 1);
        assert_eq!(
            app.editor
                .as_ref()
                .and_then(|editor| editor.category_id.clone()),
            Some(app.snapshot.active_categories()[0].id.clone()),
            "the newly created category is selected in the entry form"
        );
    });
}

#[gpui::test]
fn theme_toggle_updates_and_persists(cx: &mut TestAppContext) {
    let (view, cx, paths) = ready_entries_app(cx, "theme");
    let bounds = cx
        .debug_bounds("sidebar-theme")
        .expect("sidebar theme toggle is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert_eq!(
        view.update(cx, |app, _| app.settings.theme),
        ThemeMode::Dark
    );
    assert_eq!(Settings::load(&paths.settings_file).theme, ThemeMode::Dark);
    cx.update(|_, cx| {
        assert_eq!(crate::theme::theme(cx).mode, ThemeMode::Dark);
    });

    cx.simulate_click(bounds.center(), Modifiers::none());
    assert_eq!(
        view.update(cx, |app, _| app.settings.theme),
        ThemeMode::Light
    );
    assert_eq!(Settings::load(&paths.settings_file).theme, ThemeMode::Light);
}

#[gpui::test]
fn category_editor_creates_and_opens_its_detail(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "create-category");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| app.open_category_editor(None, window, cx));
    });
    draw(cx);
    cx.simulate_input("Groceries");
    let bounds = cx
        .debug_bounds("category-save")
        .expect("category save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());

    view.update(cx, |app, _| {
        assert!(app.category_editor.is_none());
        let categories = app.snapshot.active_categories();
        assert_eq!(categories.len(), 1);
        assert_eq!(app.screen, Screen::Categories);
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Category(categories[0].id.clone())),
            "creating a category opens its detail page like the web client"
        );
    });
}

#[gpui::test]
fn search_filters_the_entry_list(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "search");
    view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let category = Category::new("Food", "#22c55e");
        let account_id = account.id.clone();
        let category_id = category.id.clone();
        app.snapshot.accounts.push(account);
        app.snapshot.categories.push(category);
        for (id, title) in [("ent-1", "Coffee"), ("ent-2", "Rent")] {
            app.snapshot.entries.push(Entry {
                id: id.into(),
                account_id: account_id.clone(),
                category_id: category_id.clone(),
                title: title.into(),
                description: None,
                location: None,
                amount: -10.0,
                currency_code: model::default_currency_code(),
                date: model::now(),
                deleted_at: None,
                inserted_at: model::now(),
                updated_at: model::now(),
            });
        }
        cx.notify();
    });
    draw(cx);

    let handle = view.update(cx, |app, cx| app.search.read(cx).focus());
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("coffee");
    view.update(cx, |app, cx| {
        let visible = app.visible_entries(cx);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Coffee");
    });
}

#[gpui::test]
fn power_search_filters_entries_and_removes_chips(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "power-search");
    view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let category = Category::new("Food", "#22c55e");
        let account_id = account.id.clone();
        let category_id = category.id.clone();
        app.snapshot.accounts.push(account);
        app.snapshot.categories.push(category);
        for (id, title, amount) in [("ent-1", "Coffee", -10.0), ("ent-2", "Salary", 900.0)] {
            app.snapshot.entries.push(Entry {
                id: id.into(),
                account_id: account_id.clone(),
                category_id: category_id.clone(),
                title: title.into(),
                description: None,
                location: None,
                amount,
                currency_code: model::default_currency_code(),
                date: model::now(),
                deleted_at: None,
                inserted_at: model::now(),
                updated_at: model::now(),
            });
        }
        app.search.update(cx, |input, cx| {
            input.set_text("title:coffee amount:>5 type:expense ", cx)
        });
        cx.notify();
    });
    draw(cx);

    view.update(cx, |app, cx| {
        assert_eq!(
            app.search_tokens.len(),
            3,
            "enter commits every filter term"
        );
        assert!(app.search.read(cx).text().is_empty());
        let visible = app.visible_entries(cx);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Coffee");
    });
    assert!(
        cx.debug_bounds("power-term-0").is_some(),
        "committed terms render as chips"
    );
    assert!(cx.debug_bounds("power-term-2").is_some());

    let bounds = cx
        .debug_bounds("power-term-1")
        .expect("amount chip is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert_eq!(
            app.search_tokens.len(),
            2,
            "removing a chip drops that term"
        );
        assert_eq!(app.visible_entries(cx).len(), 1);
    });

    let bounds = cx
        .debug_bounds("power-search-clear")
        .expect("clear-all button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert!(app.search.read(cx).text().is_empty());
        assert_eq!(app.visible_entries(cx).len(), 2);
    });
}

#[gpui::test]
fn power_search_autocomplete_completes_on_tab(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "autocomplete");
    cx.update(|_, cx| crate::ui::input::bind_text_input_keys(cx));
    view.update(cx, |app, _| {
        app.snapshot
            .categories
            .push(Category::new("Eating out", "#22c55e"));
    });
    let handle = view.update(cx, |app, cx| app.search.read(cx).focus());
    cx.update(|window, _| window.focus(&handle));

    cx.simulate_keystrokes("ctrl-space");
    draw(cx);
    assert!(
        cx.debug_bounds("search-suggestion-0").is_some(),
        "ctrl+space opens the completion menu on an empty search"
    );

    cx.simulate_input("tit");
    draw(cx);
    assert!(
        cx.debug_bounds("search-suggestion-0").is_some(),
        "a key prefix opens the suggestion menu"
    );
    cx.simulate_keystrokes("tab");
    view.update(cx, |app, cx| {
        assert_eq!(app.search.read(cx).text(), "title:");
    });

    view.update(cx, |app, cx| {
        app.search.update(cx, |input, cx| input.set_text("", cx));
    });
    cx.simulate_input("category:Ea");
    draw(cx);
    assert!(
        cx.debug_bounds("search-suggestion-0").is_some(),
        "category values are suggested"
    );
    cx.simulate_keystrokes("tab");
    view.update(cx, |app, cx| {
        assert_eq!(app.search_tokens.len(), 1);
        assert_eq!(app.search_tokens[0].display, "Eating out");
        assert!(app.search.read(cx).text().is_empty());
    });

    view.update(cx, |app, cx| {
        app.search
            .update(cx, |input, cx| input.set_text("type:", cx));
    });
    draw(cx);
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("tab");
    view.update(cx, |app, _| {
        assert_eq!(app.search_tokens.len(), 2);
        assert_eq!(app.search_tokens[1].display, "Expense");
    });
}

#[gpui::test]
fn entries_sort_menu_reorders_the_list(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "sort-menu");
    view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let category = Category::new("Food", "#22c55e");
        let account_id = account.id.clone();
        let category_id = category.id.clone();
        app.snapshot.accounts.push(account);
        app.snapshot.categories.push(category);
        for (id, title, amount, date) in [
            ("ent-1", "Alpha", -5.0, "2026-01-01"),
            ("ent-2", "Zulu", -900.0, "2026-01-02"),
        ] {
            app.snapshot.entries.push(Entry {
                id: id.into(),
                account_id: account_id.clone(),
                category_id: category_id.clone(),
                title: title.into(),
                description: None,
                location: None,
                amount,
                currency_code: model::default_currency_code(),
                date: date.into(),
                deleted_at: None,
                inserted_at: date.into(),
                updated_at: date.into(),
            });
        }
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("entries-sort")
        .expect("sort button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("sort-amount-desc")
        .expect("sort menu lists amount order");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert_eq!(app.filter.sort, SortField::Amount);
        assert!(app.filter.descending);
        assert_eq!(app.visible_entries(cx)[0].title, "Zulu");
    });

    draw(cx);
    let bounds = cx
        .debug_bounds("entries-sort")
        .expect("sort button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("sort-title-asc")
        .expect("sort menu lists title order");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert_eq!(app.filter.sort, SortField::Title);
        assert!(!app.filter.descending);
        assert_eq!(app.visible_entries(cx)[0].title, "Alpha");
    });
}

#[gpui::test]
fn entry_editor_category_dropdown_picks_a_category(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "category-dropdown");
    let category = Category {
        id: "cat-1".into(),
        title: "Groceries".into(),
        color: "#22c55e".into(),
        ..Default::default()
    };
    view.update(cx, |app, _| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.snapshot.categories.push(category);
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    assert!(
        cx.debug_bounds("entry-category-panel").is_none(),
        "the category list is collapsed by default"
    );
    let bounds = cx
        .debug_bounds("entry-category-select")
        .expect("category dropdown control is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    assert!(
        cx.debug_bounds("entry-category-panel").is_some(),
        "clicking the control opens the category list"
    );

    let bounds = cx
        .debug_bounds("entry-category-cat-1")
        .expect("category option is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let editor = app.editor.as_ref().expect("editor stays open");
        assert_eq!(editor.category_id.as_deref(), Some("cat-1"));
        assert!(!editor.category_picker_open);
    });
    assert!(!view.update(cx, |app, _| {
        app.editor
            .as_ref()
            .expect("editor stays open")
            .category_picker_open
    }));
}

#[gpui::test]
fn mouse_back_and_forward_navigate_history(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "mouse-history");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| app.go(Screen::Dashboard, window, cx));
    });
    draw(cx);
    view.update(cx, |app, _| assert_eq!(app.screen, Screen::Dashboard));

    cx.simulate_mouse_down(
        point(px(700.0), px(700.0)),
        MouseButton::Navigate(NavigationDirection::Back),
        Modifiers::none(),
    );
    view.update(cx, |app, _| assert_eq!(app.screen, Screen::Entries));

    cx.simulate_mouse_down(
        point(px(700.0), px(700.0)),
        MouseButton::Navigate(NavigationDirection::Forward),
        Modifiers::none(),
    );
    view.update(cx, |app, _| assert_eq!(app.screen, Screen::Dashboard));

    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx)
        });
    });
    draw(cx);
    cx.simulate_mouse_down(
        point(px(700.0), px(700.0)),
        MouseButton::Navigate(NavigationDirection::Back),
        Modifiers::none(),
    );
    view.update(cx, |app, _| {
        assert!(app.editor.is_none(), "mouse back closes the top overlay");
        assert_eq!(
            app.screen,
            Screen::Dashboard,
            "closing an overlay does not move through history"
        );
    });
}

#[gpui::test]
fn backspace_on_an_empty_search_removes_the_last_chip(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "backspace-chip");
    cx.update(|_, cx| crate::ui::input::bind_text_input_keys(cx));
    view.update(cx, |app, cx| {
        app.search
            .update(cx, |input, cx| input.set_text("title:coffee ", cx));
        cx.notify();
    });
    draw(cx);
    let handle = view.update(cx, |app, cx| app.search.read(cx).focus());
    cx.update(|window, _| window.focus(&handle));
    view.update(cx, |app, _| {
        assert_eq!(app.search_tokens.len(), 1, "the term committed to a chip");
    });

    cx.simulate_keystrokes("backspace");
    view.update(cx, |app, _| {
        assert!(
            app.search_tokens.is_empty(),
            "backspace on an empty search removes the last chip"
        );
    });
}

#[gpui::test]
fn ctrl_backspace_and_ctrl_delete_remove_words(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "word-delete");
    cx.update(|_, cx| crate::ui::input::bind_text_input_keys(cx));

    let handle = view.update(cx, |app, cx| app.search.read(cx).focus());
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("coffee rent");
    cx.simulate_keystrokes("ctrl-backspace");
    view.update(cx, |app, cx| {
        assert_eq!(app.search.read(cx).text(), "coffee ");
    });

    view.update(cx, |app, cx| {
        app.search
            .update(cx, |input, cx| input.set_text("coffee rent", cx));
    });
    cx.simulate_keystrokes("home");
    cx.simulate_keystrokes("ctrl-delete");
    view.update(cx, |app, cx| {
        assert_eq!(app.search.read(cx).text(), " rent");
    });
}

#[gpui::test]
fn logout_with_unsynced_changes_asks_first(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "logout-unsynced");
    view.update(cx, |app, cx| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.screen = Screen::Settings;
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("settings-sign-out")
        .expect("settings sign-out button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(app.logout_dialog);
        assert_eq!(app.logout_unsynced, 1);
        assert_eq!(app.boot, Boot::Ready, "still signed in until confirmed");
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("logout-cancel")
        .expect("logout dialog cancel is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(!app.logout_dialog);
        assert_eq!(app.boot, Boot::Ready);
        assert_eq!(app.snapshot.active_accounts().len(), 1);
    });
}

#[gpui::test]
fn bulk_delete_entries_removes_the_selection(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "bulk-delete");
    view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let category = Category::new("Food", "#22c55e");
        let account_id = account.id.clone();
        let category_id = category.id.clone();
        app.snapshot.accounts.push(account);
        app.snapshot.categories.push(category);
        for id in ["bulk-1", "bulk-2"] {
            app.snapshot.entries.push(Entry {
                id: id.into(),
                account_id: account_id.clone(),
                category_id: category_id.clone(),
                title: id.into(),
                description: None,
                location: None,
                amount: -10.0,
                currency_code: model::default_currency_code(),
                date: model::now(),
                deleted_at: None,
                inserted_at: model::now(),
                updated_at: model::now(),
            });
            app.selected_entries.insert(id.into());
        }
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("entries-delete-selected")
        .expect("bulk delete button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("confirm-run")
        .expect("bulk delete confirmation is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(app.selected_entries.is_empty());
        assert_eq!(app.snapshot.active_entries().len(), 0);
        assert!(
            app.snapshot
                .entries
                .iter()
                .all(|entry| entry.deleted_at.is_some())
        );
    });
}

#[gpui::test]
fn account_detail_edit_and_delete_round_trip(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "account-edit-delete");
    let account_id = view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let id = account.id.clone();
        app.snapshot.accounts.push(account);
        app.screen = Screen::Accounts;
        app.detail = Some(crate::app::DetailView::Account(id.clone()));
        cx.notify();
        id
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("account-detail-edit")
        .expect("account edit button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| app.account_editor.is_some()));
    view.update(cx, |app, cx| {
        app.account_editor
            .as_ref()
            .expect("account editor open")
            .name
            .update(cx, |input, cx| input.set_text("Savings", cx));
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("account-save")
        .expect("account save is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert_eq!(app.snapshot.account("dummy").map(|_| ()), None);
        assert_eq!(app.snapshot.account(&account_id).unwrap().name, "Savings");
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Account(account_id.clone()))
        );
    });

    draw(cx);
    let bounds = cx
        .debug_bounds("account-detail-delete")
        .expect("account delete button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("confirm-run")
        .expect("account delete confirmation is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(
            app.snapshot
                .account(&account_id)
                .unwrap()
                .deleted_at
                .is_some()
        );
        assert!(app.detail.is_none());
    });
}

#[gpui::test]
fn category_detail_edit_and_delete_round_trip(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "category-edit-delete");
    let category_id = view.update(cx, |app, cx| {
        let category = Category::new("Food", "#22c55e");
        let id = category.id.clone();
        app.snapshot.categories.push(category);
        app.screen = Screen::Categories;
        app.detail = Some(crate::app::DetailView::Category(id.clone()));
        cx.notify();
        id
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("category-detail-edit")
        .expect("category edit button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| app.category_editor.is_some()));
    view.update(cx, |app, cx| {
        app.category_editor
            .as_ref()
            .expect("category editor open")
            .title
            .update(cx, |input, cx| input.set_text("Dining", cx));
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("category-save")
        .expect("category save is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert_eq!(app.snapshot.category(&category_id).unwrap().title, "Dining");
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Category(category_id.clone()))
        );
    });

    draw(cx);
    let bounds = cx
        .debug_bounds("category-detail-delete")
        .expect("category delete button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("confirm-run")
        .expect("category delete confirmation is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(
            app.snapshot
                .category(&category_id)
                .unwrap()
                .deleted_at
                .is_some()
        );
        assert!(app.detail.is_none());
    });
}

#[gpui::test]
fn amount_date_and_sort_filters_narrow_the_entry_list(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "entry-filters");
    view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let category = Category::new("Food", "#22c55e");
        let account_id = account.id.clone();
        let category_id = category.id.clone();
        app.snapshot.accounts.push(account);
        app.snapshot.categories.push(category);
        for (id, title, amount, date) in [
            ("filter-1", "Coffee", -10.0, "2026-11-01T12:00:00"),
            ("filter-2", "Rent", -20.0, "2026-11-10T12:00:00"),
        ] {
            app.snapshot.entries.push(Entry {
                id: id.into(),
                account_id: account_id.clone(),
                category_id: category_id.clone(),
                title: title.into(),
                description: None,
                location: None,
                amount,
                currency_code: model::default_currency_code(),
                date: date.into(),
                deleted_at: None,
                inserted_at: model::now(),
                updated_at: model::now(),
            });
        }
        app.filter_min
            .update(cx, |input, cx| input.set_text("5", cx));
        app.filter_max
            .update(cx, |input, cx| input.set_text("15", cx));
        cx.notify();
    });
    view.update(cx, |app, cx| {
        let visible = app.visible_entries(cx);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Coffee");
    });

    view.update(cx, |app, cx| {
        app.filter_min.update(cx, |input, cx| input.clear(cx));
        app.filter_max.update(cx, |input, cx| input.clear(cx));
        app.filter_start
            .update(cx, |input, cx| input.set_text("2026-11-05", cx));
        app.filter_end
            .update(cx, |input, cx| input.set_text("2026-11-15", cx));
        cx.notify();
    });
    view.update(cx, |app, cx| {
        let visible = app.visible_entries(cx);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Rent");
    });

    view.update(cx, |app, cx| {
        app.filter_start.update(cx, |input, cx| input.clear(cx));
        app.filter_end.update(cx, |input, cx| input.clear(cx));
        app.filter.sort = SortField::Amount;
        app.filter.descending = false;
        cx.notify();
    });
    view.update(cx, |app, cx| {
        let titles = app
            .visible_entries(cx)
            .iter()
            .map(|entry| entry.title.as_str())
            .collect::<Vec<_>>();
        assert_eq!(titles, vec!["Coffee", "Rent"]);
    });
}

#[gpui::test]
fn sync_bar_details_open_a_pending_change(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "sync-details");
    let account_id = view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let id = account.id.clone();
        app.snapshot.accounts.push(account);
        app.sync_state = SyncState::Error("Offline".into());
        cx.notify();
        id
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("sync-bar-details-toggle")
        .expect("sync bar details toggle is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| app.sync_details_open));
    draw(cx);

    let bounds = cx
        .debug_bounds("pending-account-row")
        .expect("pending account row is rendered in the sync bar");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(!app.sync_details_open);
        assert_eq!(app.screen, Screen::Accounts);
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Account(account_id))
        );
    });
}

#[gpui::test]
fn bulk_delete_accounts_removes_the_selection(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "bulk-accounts");
    let ids = view.update(cx, |app, cx| {
        let mut ids = Vec::new();
        for name in ["Checking", "Savings"] {
            let account = Account::new(name);
            ids.push(account.id.clone());
            app.selected_accounts.insert(account.id.clone());
            app.snapshot.accounts.push(account);
        }
        app.screen = Screen::Accounts;
        cx.notify();
        ids
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("accounts-delete-selected")
        .expect("bulk account delete button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("confirm-run")
        .expect("account delete confirmation is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(app.selected_accounts.is_empty());
        assert!(app.snapshot.active_accounts().is_empty());
        assert!(ids.iter().all(|id| {
            app.snapshot
                .account(id)
                .is_some_and(|account| account.deleted_at.is_some())
        }));
    });
}

#[gpui::test]
fn bulk_delete_categories_removes_the_selection(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "bulk-categories");
    let ids = view.update(cx, |app, cx| {
        let mut ids = Vec::new();
        for title in ["Food", "Travel"] {
            let category = Category::new(title, "#22c55e");
            ids.push(category.id.clone());
            app.selected_categories.insert(category.id.clone());
            app.snapshot.categories.push(category);
        }
        app.screen = Screen::Categories;
        cx.notify();
        ids
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("categories-delete-selected")
        .expect("bulk category delete button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    draw(cx);
    let bounds = cx
        .debug_bounds("confirm-run")
        .expect("category delete confirmation is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(app.selected_categories.is_empty());
        assert!(app.snapshot.active_categories().is_empty());
        assert!(ids.iter().all(|id| {
            app.snapshot
                .category(id)
                .is_some_and(|category| category.deleted_at.is_some())
        }));
    });
}

#[gpui::test]
fn sync_bar_details_discard_all_opens_confirmation(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "sync-details-discard");
    view.update(cx, |app, cx| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.sync_state = SyncState::Error("Offline".into());
        app.sync_details_open = true;
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("sync-details-discard")
        .expect("discard-all action is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert!(matches!(
            app.confirm.as_ref().map(|confirm| &confirm.action),
            Some(crate::app::ConfirmAction::DiscardUnsynced)
        ));
    });
}

#[gpui::test]
fn calendar_clear_and_cancel_actions(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "calendar-actions");
    view.update(cx, |app, cx| {
        app.show_filters = true;
        app.open_filter_calendar(cx);
        if let Some(calendar) = app.calendar.as_mut() {
            calendar.start = Some(chrono::Local::now().date_naive());
            calendar.end = Some(chrono::Local::now().date_naive());
        }
        cx.notify();
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("calendar-clear")
        .expect("calendar clear button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let calendar = app.calendar.as_ref().expect("calendar stays open");
        assert!(calendar.start.is_none());
        assert!(calendar.end.is_none());
    });

    view.update(cx, |app, cx| {
        app.calendar = None;
        app.show_filters = false;
        cx.notify();
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
            app.open_entry_calendar(cx);
        });
    });
    draw(cx);
    let before = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .date
            .read(cx)
            .text()
            .to_string()
    });
    let bounds = cx
        .debug_bounds("calendar-cancel")
        .expect("calendar cancel button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert!(app.calendar.is_none());
        assert_eq!(
            app.editor
                .as_ref()
                .expect("entry editor stays open")
                .date
                .read(cx)
                .text(),
            before
        );
    });
}

#[gpui::test]
fn sync_bar_details_opens_category_and_entry_rows(cx: &mut TestAppContext) {
    let (view, cx, ids) = ready_entries_app(cx, "sync-details-rows");
    let (account_id, category_id, entry_id) = view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let category = Category::new("Food", "#22c55e");
        let entry = Entry {
            id: "sync-entry".into(),
            account_id: account.id.clone(),
            category_id: category.id.clone(),
            title: "Lunch".into(),
            description: None,
            location: None,
            amount: -12.0,
            currency_code: model::default_currency_code(),
            date: model::now(),
            deleted_at: None,
            inserted_at: model::now(),
            updated_at: model::now(),
        };
        let ids = (account.id.clone(), category.id.clone(), entry.id.clone());
        app.snapshot.accounts.push(account);
        app.snapshot.categories.push(category);
        app.snapshot.entries.push(entry);
        app.sync_state = SyncState::Error("Offline".into());
        app.sync_details_open = true;
        cx.notify();
        ids
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("pending-category-row")
        .expect("pending category row is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert_eq!(app.screen, Screen::Categories);
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Category(category_id.clone()))
        );
    });

    view.update(cx, |app, cx| {
        app.screen = Screen::Entries;
        app.detail = None;
        app.sync_details_open = true;
        cx.notify();
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("pending-entry-row")
        .expect("pending entry row is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert_eq!(app.screen, Screen::Entries);
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Entry(entry_id.clone()))
        );
        let _ = (account_id, ids);
    });
}

#[gpui::test]
fn category_count_filter_narrows_the_table(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "category-count-filter");
    view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let account_id = account.id.clone();
        app.snapshot.accounts.push(account);
        for (title, count) in [("Empty", 0usize), ("Few", 2), ("Several", 6)] {
            let category = Category::new(title, "#22c55e");
            let category_id = category.id.clone();
            app.snapshot.categories.push(category);
            for n in 0..count {
                app.snapshot.entries.push(Entry {
                    id: format!("{title}-{n}"),
                    account_id: account_id.clone(),
                    category_id: category_id.clone(),
                    title: "Line".into(),
                    description: None,
                    location: None,
                    amount: -1.0,
                    currency_code: model::default_currency_code(),
                    date: model::now(),
                    deleted_at: None,
                    inserted_at: model::now(),
                    updated_at: model::now(),
                });
            }
        }
        app.screen = Screen::Categories;
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("category-filter-Empty")
        .expect("empty category filter is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        let visible = app.visible_categories(cx);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Empty");
    });

    let bounds = cx
        .debug_bounds("category-filter-5-9")
        .expect("5-9 category filter is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        let visible = app.visible_categories(cx);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Several");
    });

    let bounds = cx
        .debug_bounds("category-filter-All")
        .expect("all category filter is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert_eq!(app.visible_categories(cx).len(), 3)
    });
}

#[gpui::test]
fn currency_position_and_symbol_are_customizable(cx: &mut TestAppContext) {
    let (view, cx, paths) = ready_entries_app(cx, "currency");
    view.update(cx, |app, cx| {
        app.screen = Screen::Settings;
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("currency-position-after")
        .expect("after-position control is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert_eq!(app.settings.currency_position, CurrencyPosition::After);
        assert_eq!(app.money(1234.5), "1,234.50 $");
    });
    assert_eq!(
        Settings::load(&paths.settings_file).currency_position,
        CurrencyPosition::After
    );

    view.update(cx, |app, cx| {
        app.currency_input
            .update(cx, |input, cx| input.set_text("kr", cx));
    });
    cx.run_until_parked();
    view.update(cx, |app, _| {
        assert_eq!(app.settings.currency_symbol, "kr");
        assert_eq!(app.money(1234.5), "1,234.50 kr");
    });
}

#[gpui::test]
fn motion_preference_toggles_and_persists(cx: &mut TestAppContext) {
    let (view, cx, paths) = ready_entries_app(cx, "motion-preference");
    view.update(cx, |app, cx| {
        app.screen = Screen::Settings;
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("motion-reduced")
        .expect("reduced motion control is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| assert!(app.settings.reduce_motion));
    assert!(Settings::load(&paths.settings_file).reduce_motion);
    cx.update(|_, cx| assert!(crate::motion::reduce_motion(cx)));

    let bounds = cx
        .debug_bounds("motion-full")
        .expect("full motion control is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| assert!(!app.settings.reduce_motion));
    assert!(!Settings::load(&paths.settings_file).reduce_motion);
    cx.update(|_, cx| assert!(!crate::motion::reduce_motion(cx)));
}

#[gpui::test]
fn dashboard_add_account_opens_the_account_editor(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "dashboard-add-account");
    view.update(cx, |app, cx| {
        app.screen = Screen::Dashboard;
        cx.notify();
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("dashboard-add-account")
        .expect("dashboard exposes an add-account action");
    cx.simulate_click(bounds.center(), Modifiers::none());
    assert!(view.update(cx, |app, _| app.account_editor.is_some()));
}

#[gpui::test]
fn pending_change_row_opens_its_detail(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "pending-row");
    let account_id = view.update(cx, |app, cx| {
        let account = Account::new("Checking");
        let id = account.id.clone();
        app.snapshot.accounts.push(account);
        app.screen = Screen::Settings;
        cx.notify();
        id
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("pending-account-row")
        .expect("pending account row is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        assert_eq!(app.screen, Screen::Accounts);
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Account(account_id)),
            "clicking a pending change opens the matching detail page"
        );
    });
}

#[gpui::test]
fn modal_backdrop_blocks_clicks_to_the_page_beneath(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "modal-occlude");
    view.update(cx, |app, _| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.snapshot
            .categories
            .push(Category::new("Groceries", "#22c55e"));
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("toggle-filters")
        .expect("the page button under the modal is still laid out");
    cx.simulate_click(bounds.center(), Modifiers::none());

    view.update(cx, |app, cx| {
        assert!(app.editor.is_none(), "the backdrop click closes the modal");
        assert!(
            !app.show_filters,
            "the click must not reach the filter button underneath the backdrop"
        );
        assert!(
            app.search.read(cx).text().is_empty(),
            "the click must not reach the search field underneath"
        );
    });
}

#[gpui::test]
fn entry_editor_saves_a_new_expense(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "create-entry");
    view.update(cx, |app, _| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.snapshot
            .categories
            .push(Category::new("Groceries", "#22c55e"));
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .amount
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("12.50");

    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .title
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("Coffee");

    let bounds = cx
        .debug_bounds("entry-save")
        .expect("save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());

    view.update(cx, |app, _| {
        assert!(app.editor.is_none(), "saving closes the editor");
        let entries = app.snapshot.active_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Coffee");
        assert!((entries[0].amount + 12.5).abs() < 1e-9);
        assert_eq!(
            entries[0].account_id,
            app.snapshot.active_accounts()[0].id,
            "the default account is attached"
        );
        assert_eq!(
            app.detail,
            Some(crate::app::DetailView::Entry(entries[0].id.clone())),
            "creating an entry follows the web client to its detail page"
        );
    });
}

#[gpui::test]
fn calendar_prev_next_navigates_months_and_years(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "calendar-nav");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    view.update(cx, |app, cx| {
        app.open_entry_calendar(cx);
        if let Some(calendar) = app.calendar.as_mut() {
            calendar.year = 2026;
            calendar.month = 1;
        }
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("calendar-prev")
        .expect("previous month button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let calendar = app.calendar.as_ref().expect("calendar stays open");
        assert_eq!(calendar.year, 2025);
        assert_eq!(calendar.month, 12);
    });

    let bounds = cx
        .debug_bounds("calendar-next")
        .expect("next month button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let calendar = app.calendar.as_ref().expect("calendar stays open");
        assert_eq!(calendar.year, 2026);
        assert_eq!(calendar.month, 2);
    });
}

#[gpui::test]
fn entry_income_toggle_saves_a_positive_amount(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "income-entry");
    view.update(cx, |app, _| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.snapshot
            .categories
            .push(Category::new("Salary", "#6366f1"));
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("entry-income")
        .expect("income toggle is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .amount
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("42");
    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .title
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("Bonus");
    draw(cx);
    let bounds = cx
        .debug_bounds("entry-save")
        .expect("save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let entries = app.snapshot.active_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].amount, 42.0);
    });
}

#[gpui::test]
fn entry_validation_blocks_an_empty_draft(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "entry-validation");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("entry-save")
        .expect("save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let editor = app.editor.as_ref().expect("editor stays open");
        assert!(
            editor
                .error
                .as_deref()
                .is_some_and(|error| error.to_lowercase().contains("amount")),
            "an empty draft reports the missing amount"
        );
    });
}

#[gpui::test]
fn calendar_today_sets_the_editor_and_range_dates(cx: &mut TestAppContext) {
    let today = chrono::Local::now().date_naive();
    let (view, cx, _) = ready_entries_app(cx, "calendar-today");
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    view.update(cx, |app, cx| {
        app.open_entry_calendar(cx);
        if let Some(calendar) = app.calendar.as_mut() {
            calendar.year = 2020;
            calendar.month = 1;
        }
        cx.notify();
    });
    draw(cx);

    let bounds = cx
        .debug_bounds("calendar-today")
        .expect("calendar Today button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, cx| {
        assert!(app.calendar.is_none());
        assert_eq!(
            app.editor
                .as_ref()
                .expect("entry editor stays open")
                .date
                .read(cx)
                .text(),
            today.format("%Y-%m-%d").to_string()
        );
    });

    view.update(cx, |app, cx| {
        app.show_filters = true;
        app.open_filter_calendar(cx);
        if let Some(calendar) = app.calendar.as_mut() {
            calendar.start = Some(today - chrono::Duration::days(10));
            calendar.end = Some(today - chrono::Duration::days(5));
        }
        cx.notify();
    });
    draw(cx);
    let bounds = cx
        .debug_bounds("calendar-today")
        .expect("range calendar Today button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());
    view.update(cx, |app, _| {
        let calendar = app.calendar.as_ref().expect("range calendar stays open");
        assert_eq!(calendar.start, Some(today));
        assert_eq!(calendar.end, Some(today));
    });
}

#[gpui::test]
fn entry_note_keeps_line_breaks_when_saved(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "entry-note-save");
    cx.update(|_, cx| {
        crate::ui::input::bind_text_input_keys(cx);
    });
    view.update(cx, |app, _| {
        app.snapshot.accounts.push(Account::new("Checking"));
        app.snapshot
            .categories
            .push(Category::new("Food", "#22c55e"));
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);
    for (field, value) in [("amount", "12"), ("title", "Lunch")] {
        let handle = view.update(cx, |app, cx| {
            let editor = app.editor.as_ref().expect("entry editor open");
            match field {
                "amount" => editor.amount.read(cx).focus(),
                _ => editor.title.read(cx).focus(),
            }
        });
        cx.update(|window, _| window.focus(&handle));
        cx.simulate_input(value);
    }
    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .description
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("Line one");
    cx.simulate_keystrokes("enter");
    cx.simulate_input("Line two");
    draw(cx);
    let bounds = cx
        .debug_bounds("entry-save")
        .expect("save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());

    view.update(cx, |app, _| {
        let entries = app.snapshot.active_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].description.as_deref(),
            Some("Line one\nLine two")
        );
    });
}

#[gpui::test]
fn category_description_keeps_line_breaks_when_saved(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "category-note-save");
    cx.update(|_, cx| {
        crate::ui::input::bind_text_input_keys(cx);
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_category_editor(None, window, cx);
        });
    });
    draw(cx);
    let handle = view.update(cx, |app, cx| {
        app.category_editor
            .as_ref()
            .expect("category editor open")
            .title
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("Groceries");
    let handle = view.update(cx, |app, cx| {
        app.category_editor
            .as_ref()
            .expect("category editor open")
            .description
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("Line one");
    cx.simulate_keystrokes("enter");
    cx.simulate_input("Line two");
    draw(cx);
    let bounds = cx
        .debug_bounds("category-save")
        .expect("category save button is rendered");
    cx.simulate_click(bounds.center(), Modifiers::none());

    view.update(cx, |app, _| {
        let categories = app.snapshot.active_categories();
        assert_eq!(categories.len(), 1);
        assert_eq!(
            categories[0].description.as_deref(),
            Some("Line one\nLine two")
        );
    });
}

#[gpui::test]
fn multiline_note_keeps_line_breaks(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "multiline");
    cx.update(|_, cx| {
        crate::ui::input::bind_text_input_keys(cx);
    });
    cx.update(|window, cx| {
        view.update(cx, |app, cx| {
            app.open_entry_editor(None, None, None, window, cx);
        });
    });
    draw(cx);

    let handle = view.update(cx, |app, cx| {
        app.editor
            .as_ref()
            .expect("entry editor open")
            .description
            .read(cx)
            .focus()
    });
    cx.update(|window, _| window.focus(&handle));
    cx.simulate_input("First line");
    cx.simulate_keystrokes("enter");
    cx.simulate_input("Second line");

    view.update(cx, |app, cx| {
        let text = app
            .editor
            .as_ref()
            .expect("entry editor open")
            .description
            .read(cx)
            .text()
            .to_string();
        assert_eq!(text, "First line\nSecond line");
    });
}

#[gpui::test]
fn alt_shortcuts_are_suppressed_while_the_search_field_is_focused(cx: &mut TestAppContext) {
    let (view, cx, _) = ready_entries_app(cx, "shortcut-suppression");
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new("alt-1", ShowDashboard, None)]);
    });
    let handle = view.update(cx, |app, cx| app.search.read(cx).focus());
    cx.update(|window, _| window.focus(&handle));

    cx.simulate_keystrokes("alt-1");
    assert_eq!(
        view.update(cx, |app, _| app.screen),
        Screen::Entries,
        "navigation shortcuts must not fire while a text field is focused"
    );

    let root_focus = view.update(cx, |app, _| app.root_focus.clone());
    cx.update(|window, _| window.focus(&root_focus));
    cx.simulate_keystrokes("alt-1");
    assert_eq!(
        view.update(cx, |app, _| app.screen),
        Screen::Dashboard,
        "the same shortcut works once the text field is blurred"
    );
}

struct TooltipHarness;

impl Render for TooltipHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            div()
                .id("tooltip-target")
                .debug_selector(|| "tooltip-target".to_string())
                .size(px(140.0))
                .sunder_tooltip("Delete entry", &["Del"]),
        )
    }
}

#[gpui::test]
fn first_launch_opens_the_login_screen(cx: &mut TestAppContext) {
    let paths = test_paths("login-default");
    let (view, cx) = cx.add_window_view(move |_, cx| SunderApp::for_test(paths, cx));
    cx.run_until_parked();
    view.update(cx, |app, _| assert_eq!(app.boot, Boot::Login));
}

#[gpui::test]
fn tooltip_appears_after_hovering(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_global(crate::theme::Theme::light()));
    let (_, cx) = cx.add_window_view(|_, _| TooltipHarness);
    draw(cx);

    let bounds = cx
        .debug_bounds("tooltip-target")
        .expect("tooltip target is rendered");
    cx.simulate_mouse_move(bounds.center(), None, Modifiers::none());
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();
    draw(cx);

    assert!(
        cx.debug_bounds("sunder-tooltip").is_some(),
        "hovering the target must render the tooltip view"
    );
}
