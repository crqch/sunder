use crate::api::{
    AdminCurrencyPatch, AdminInvite, AdminUser, ApiClient, ApiError, ImportKind, LoginResponse,
    ProfileResponse,
};
use crate::format;
use crate::model::{
    Account, Category, Currency, Entry, SyncPayload, SyncResponse, User, monthly_flow_converted,
    normalize_currency_code, now,
};
use crate::search::{EntryQuery, SearchKind, SearchSuggestion, SearchTerm};
use crate::store::{Paths, Settings, Snapshot};
use crate::theme::{Theme, set_theme, theme};
use crate::ui::input::{InputStyle, TextInput, TextInputEvent};
use chrono::{Datelike, NaiveDate};
use gpui::{
    AnyElement, App, Context, Entity, MouseButton, NavigationDirection, PathPromptOptions, Task,
    Window, actions, div, prelude::*,
};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

actions!(
    sunder,
    [
        NewEntry,
        NewAccount,
        NewCategory,
        SyncNow,
        ShowDashboard,
        ShowEntries,
        ShowAccounts,
        ShowCategories,
        ShowSettings,
        FocusSearch,
        SaveOverlay,
        CloseOverlay,
        ToggleTheme,
        ShowShortcuts,
        SearchComplete,
        SearchNext,
        SearchPrevious,
        SearchOpenSuggestions,
    ]
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boot {
    Login,
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Entries,
    Accounts,
    Categories,
    Settings,
    Admin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    Login,
    Register,
}

#[derive(Clone, Debug)]
pub struct AuthMessage {
    pub text: String,
    pub success: bool,
}

pub struct AuthForm {
    pub mode: AuthMode,
    pub server: Entity<TextInput>,
    pub login: Entity<TextInput>,
    pub password: Entity<TextInput>,
    pub invite: Entity<TextInput>,
    pub email: Entity<TextInput>,
    pub username: Entity<TextInput>,
    pub message: Option<AuthMessage>,
    pub busy: bool,
    pub focused: bool,
}

#[derive(Clone, Debug)]
pub enum SyncState {
    Idle,
    Syncing,
    Success,
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Clone, Debug)]
pub enum ToastAction {
    UndoEntry { id: String, previous: Option<Entry> },
    UndoAccount { account: Account, created: bool },
    UndoCategory { category: Category, created: bool },
    RetrySync,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
    pub action: Option<ToastAction>,
}

#[derive(Clone, Debug)]
pub enum ConfirmAction {
    ClearLocalData,
    DiscardUnsynced,
    DeleteEntry(String),
    DeleteAccount(String),
    DeleteCategory(String),
    DeleteEntries(Vec<String>),
    DeleteAccounts(Vec<String>),
    DeleteCategories(Vec<String>),
    AdminDeleteUser(String),
    AdminDeleteInvite(String),
}

#[derive(Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub action: ConfirmAction,
    pub seq: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CategoryCountFilter {
    #[default]
    Any,
    Empty,
    OneToFour,
    FiveToNine,
    TenPlus,
}

impl CategoryCountFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::Any => "All",
            Self::Empty => "Empty",
            Self::OneToFour => "1-4",
            Self::FiveToNine => "5-9",
            Self::TenPlus => "10+",
        }
    }

    pub fn matches(self, count: usize) -> bool {
        match self {
            Self::Any => true,
            Self::Empty => count == 0,
            Self::OneToFour => (1..=4).contains(&count),
            Self::FiveToNine => (5..=9).contains(&count),
            Self::TenPlus => count >= 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortField {
    #[default]
    Date,
    Amount,
    Title,
}

#[derive(Clone, Debug)]
pub struct EntryFilter {
    pub account_ids: Vec<String>,
    pub category_ids: Vec<String>,
    pub currency_codes: Vec<String>,
    pub sort: SortField,
    pub descending: bool,
}

impl Default for EntryFilter {
    fn default() -> Self {
        Self {
            account_ids: Vec::new(),
            category_ids: Vec::new(),
            currency_codes: Vec::new(),
            sort: SortField::Date,
            descending: true,
        }
    }
}

impl EntryFilter {
    pub fn toggle_account(&mut self, id: &str) {
        toggle_value(&mut self.account_ids, id);
    }

    pub fn toggle_category(&mut self, id: &str) {
        toggle_value(&mut self.category_ids, id);
    }

    pub fn toggle_currency(&mut self, code: &str) {
        let code = normalize_currency_code(code);
        toggle_value(&mut self.currency_codes, &code);
    }
}

fn toggle_value(values: &mut Vec<String>, id: &str) {
    if let Some(index) = values.iter().position(|value| value == id) {
        values.remove(index);
    } else {
        values.push(id.to_string());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarTarget {
    EntryDate,
    FilterRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarView {
    Days,
    Months,
    Years,
}

#[derive(Clone, Debug)]
pub struct CalendarState {
    pub target: CalendarTarget,
    pub view: CalendarView,
    pub year: i32,
    pub month: u32,
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    pub hover: Option<NaiveDate>,
    pub seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetailView {
    Account(String),
    Category(String),
    Entry(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub screen: Screen,
    pub detail: Option<DetailView>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdminCurrencyFilter {
    #[default]
    All,
    Active,
    Inactive,
}

#[derive(Clone, Debug, Default)]
pub struct AdminState {
    pub users: Vec<AdminUser>,
    pub invites: Vec<AdminInvite>,
    pub currencies: Vec<Currency>,
    pub currency_filter: AdminCurrencyFilter,
    pub syncing_rates: bool,
    pub last_sync_summary: Option<String>,
    pub loading: bool,
    pub loaded: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TopCategory {
    pub title: String,
    pub color: String,
    pub spend: f64,
}

pub struct SunderApp {
    pub paths: Paths,
    pub root_focus: gpui::FocusHandle,
    pub settings: Settings,
    pub snapshot: Snapshot,
    pub api: ApiClient,
    pub access_token: Option<String>,
    pub user: Option<User>,
    pub boot: Boot,
    pub screen: Screen,
    pub auth: AuthForm,
    pub sync_state: SyncState,
    pub sync_in_flight: bool,
    pub sync_again: bool,
    pub sync_details_open: bool,
    pub sync_details_seq: u64,
    pub auto_sync_started: bool,
    pub profile_loaded: bool,
    pub session_generation: u64,
    pub currencies_loading: bool,
    pub search: Entity<TextInput>,
    pub account_search: Entity<TextInput>,
    pub category_search: Entity<TextInput>,
    pub filter_min: Entity<TextInput>,
    pub filter_max: Entity<TextInput>,
    pub filter_start: Entity<TextInput>,
    pub filter_end: Entity<TextInput>,
    pub filter: EntryFilter,
    pub show_filters: bool,
    pub category_count_filter: CategoryCountFilter,
    pub page_generation: u64,
    pub search_tokens: Vec<SearchTerm>,
    pub search_suggestion: usize,
    pub search_menu_open: bool,
    pub search_menu_all: bool,
    pub sort_menu_open: bool,
    pub detail: Option<DetailView>,
    pub calendar: Option<CalendarState>,
    pub selected_entries: HashSet<String>,
    pub selected_accounts: HashSet<String>,
    pub selected_categories: HashSet<String>,
    pub history: Vec<HistoryEntry>,
    pub history_index: usize,
    pub admin: AdminState,
    pub admin_editor: Option<crate::ui::overlays::AdminEditor>,
    pub admin_currency_editor: Option<crate::ui::overlays::AdminCurrencyEditor>,
    pub admin_invite: Entity<TextInput>,
    pub admin_currency_search: Entity<TextInput>,
    pub admin_rate_date: Entity<TextInput>,
    pub admin_backfill_days: Entity<TextInput>,
    pub admin_manual_target: Entity<TextInput>,
    pub admin_manual_rate: Entity<TextInput>,
    pub admin_manual_date: Entity<TextInput>,
    pub currency_input: Entity<TextInput>,
    pub settings_currency_open: bool,
    pub settings_currency_search: Entity<TextInput>,
    pub show_shortcuts: bool,
    pub logout_dialog: bool,
    pub logout_unsynced: usize,
    pub logout_after_sync: bool,
    pub toasts: Vec<Toast>,
    pub next_toast_id: u64,
    pub modal_seq: u64,
    pub(crate) last_auto_sync: Option<std::time::Instant>,
    #[cfg(debug_assertions)]
    pub(crate) debug_overlay_opened: bool,
    pub editor: Option<crate::ui::overlays::EntryEditor>,
    pub account_editor: Option<crate::ui::overlays::AccountEditor>,
    pub category_editor: Option<crate::ui::overlays::CategoryEditor>,
    pub bulk_currency_editor: Option<crate::ui::overlays::BulkCurrencyEditor>,
    pub confirm: Option<ConfirmDialog>,
    pub(super) _auto_sync: Option<Task<()>>,
}

impl SunderApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let paths = Paths::discover().unwrap_or_else(|error| {
            eprintln!("Sunder could not find a data directory: {error}");
            Paths {
                data_file: std::path::PathBuf::from("sunder-data.json"),
                settings_file: std::path::PathBuf::from("sunder-settings.json"),
                root: std::path::PathBuf::from("."),
            }
        });

        Self::with_paths(paths, cx)
    }

    #[cfg(test)]
    pub fn for_test(paths: Paths, cx: &mut Context<Self>) -> Self {
        Self::with_paths(paths, cx)
    }

    fn with_paths(paths: Paths, cx: &mut Context<Self>) -> Self {
        let settings = Settings::load(&paths.settings_file);
        let snapshot = Snapshot::load(&paths.data_file);
        let api = ApiClient::new(settings.base_url.clone());
        cx.set_global(Theme::for_mode(settings.theme));
        cx.set_global(crate::motion::MotionPreference {
            reduce_motion: settings.reduce_motion,
        });

        let server = cx.new(|cx| {
            TextInput::new(
                "http://localhost:4000",
                settings.base_url.clone(),
                InputStyle::compact(),
                cx,
            )
        });
        let login = cx.new(|cx| {
            TextInput::new("Email or username", "", InputStyle::large(), cx)
                .with_prefix(crate::assets::Icon::User)
        });
        let password = cx.new(|cx| {
            TextInput::new("Password", "", InputStyle::large(), cx)
                .masked(true)
                .with_prefix(crate::assets::Icon::Shield)
        });
        let invite = cx.new(|cx| {
            TextInput::new("Invite code", "", InputStyle::default(), cx)
                .with_prefix(crate::assets::Icon::Sparkles)
        });
        let email = cx.new(|cx| {
            TextInput::new("Email address", "", InputStyle::default(), cx)
                .with_prefix(crate::assets::Icon::Globe)
        });
        let username = cx.new(|cx| {
            TextInput::new("Username", "", InputStyle::default(), cx)
                .with_prefix(crate::assets::Icon::User)
        });
        let search = cx.new(|cx| {
            let style = InputStyle {
                height: 30.0,
                key_context: "TextInput SearchInput",
                ..InputStyle::bare()
            };
            TextInput::new("Filter entries…", "", style, cx)
                .with_prefix(crate::assets::Icon::Search)
        });
        let account_search = cx.new(|cx| {
            let style = InputStyle {
                height: 38.0,
                ..InputStyle::default()
            };
            TextInput::new("Search accounts…", "", style, cx)
                .with_prefix(crate::assets::Icon::Search)
        });
        let category_search = cx.new(|cx| {
            let style = InputStyle {
                height: 38.0,
                ..InputStyle::default()
            };
            TextInput::new("Search categories…", "", style, cx)
                .with_prefix(crate::assets::Icon::Search)
        });
        let filter_min = cx.new(|cx| TextInput::new("Min", "", InputStyle::compact(), cx));
        let filter_max = cx.new(|cx| TextInput::new("Max", "", InputStyle::compact(), cx));
        let filter_start = cx.new(|cx| {
            TextInput::new("YYYY-MM-DD", "", InputStyle::compact(), cx)
                .with_prefix(crate::assets::Icon::Calendar)
        });
        let filter_end = cx.new(|cx| {
            TextInput::new("YYYY-MM-DD", "", InputStyle::compact(), cx)
                .with_prefix(crate::assets::Icon::Calendar)
        });
        let admin_invite = cx.new(|cx| {
            TextInput::new(
                "New invite code (e.g. BETA-2024)",
                "",
                InputStyle::default(),
                cx,
            )
        });
        let admin_currency_search = cx.new(|cx| {
            TextInput::new("Search currencies (code, name, symbol)…", "", InputStyle::compact(), cx)
                .with_prefix(crate::assets::Icon::Search)
        });
        let admin_rate_date = cx.new(|cx| {
            TextInput::new("YYYY-MM-DD", "", InputStyle::compact(), cx)
                .with_prefix(crate::assets::Icon::Calendar)
        });
        let admin_backfill_days =
            cx.new(|cx| TextInput::new("Days (e.g. 30)", "30", InputStyle::compact(), cx));
        let admin_manual_target =
            cx.new(|cx| TextInput::new("Target (e.g. EUR)", "", InputStyle::compact(), cx));
        let admin_manual_rate =
            cx.new(|cx| TextInput::new("Rate per 1 USD", "", InputStyle::compact(), cx));
        let admin_manual_date = cx.new(|cx| {
            TextInput::new("YYYY-MM-DD (optional)", "", InputStyle::compact(), cx)
                .with_prefix(crate::assets::Icon::Calendar)
        });
        let currency_input = cx.new(|cx| {
            TextInput::new(
                "$",
                settings.currency_symbol.clone(),
                InputStyle::compact(),
                cx,
            )
        });
        let settings_currency_search = cx.new(|cx| {
            TextInput::new("Search currency…", "", InputStyle::compact(), cx)
                .with_prefix(crate::assets::Icon::Search)
        });

        let initial_boot = Boot::Login;

        let auth = AuthForm {
            mode: AuthMode::Login,
            server,
            login,
            password,
            invite,
            email,
            username,
            message: None,
            busy: false,
            focused: false,
        };

        let mut app = Self {
            paths,
            root_focus: cx.focus_handle(),
            settings,
            snapshot,
            api,
            access_token: None,
            user: None,
            boot: initial_boot,
            screen: Screen::Dashboard,
            auth,
            sync_state: SyncState::Idle,
            sync_in_flight: false,
            sync_again: false,
            sync_details_open: false,
            sync_details_seq: 0,
            auto_sync_started: false,
            profile_loaded: false,
            session_generation: 0,
            currencies_loading: false,
            search,
            account_search,
            category_search,
            filter_min,
            filter_max,
            filter_start,
            filter_end,
            filter: EntryFilter::default(),
            show_filters: false,
            category_count_filter: CategoryCountFilter::Any,
            page_generation: 0,
            search_tokens: Vec::new(),
            search_suggestion: 0,
            search_menu_open: false,
            search_menu_all: false,
            sort_menu_open: false,
            detail: None,
            calendar: None,
            selected_entries: HashSet::new(),
            selected_accounts: HashSet::new(),
            selected_categories: HashSet::new(),
            history: Vec::new(),
            history_index: 0,
            admin: AdminState::default(),
            admin_editor: None,
            admin_currency_editor: None,
            admin_invite,
            admin_currency_search,
            admin_rate_date,
            admin_backfill_days,
            admin_manual_target,
            admin_manual_rate,
            admin_manual_date,
            currency_input,
            settings_currency_open: false,
            settings_currency_search,
            show_shortcuts: false,
            logout_dialog: false,
            logout_unsynced: 0,
            logout_after_sync: false,
            toasts: Vec::new(),
            next_toast_id: 1,
            modal_seq: 1,
            last_auto_sync: None,
            #[cfg(debug_assertions)]
            debug_overlay_opened: false,
            editor: None,
            account_editor: None,
            category_editor: None,
            bulk_currency_editor: None,
            confirm: None,
            _auto_sync: None,
        };

        app.wire_subscriptions(cx);
        app.start_boot(cx);
        app
    }

    fn wire_subscriptions(&mut self, cx: &mut Context<Self>) {
        let fields = [
            self.auth.login.clone(),
            self.auth.password.clone(),
            self.auth.invite.clone(),
            self.auth.email.clone(),
            self.auth.username.clone(),
        ];
        for field in fields {
            cx.subscribe(&field, |this, _, event, cx| {
                if matches!(event, TextInputEvent::Submit) {
                    this.submit_auth(cx);
                }
            })
            .detach();
        }

        cx.subscribe(&self.currency_input.clone(), |this, _, event, cx| {
            if matches!(event, TextInputEvent::Change) {
                let raw = this.currency_input.read(cx).text().to_string();
                let value = raw.chars().take(4).collect::<String>();
                if raw != value {
                    this.currency_input
                        .update(cx, |input, cx| input.set_text(value.clone(), cx));
                }
                if this.settings.currency_symbol != value {
                    this.settings.currency_symbol = value;
                    let _ = this.settings.save(&this.paths.settings_file);
                    cx.notify();
                }
            }
        })
        .detach();

        cx.subscribe(&self.search.clone(), |this, _, event, cx| match event {
            TextInputEvent::Change => {
                this.search_menu_open = true;
                this.search_menu_all = false;
                this.search_suggestion = 0;
                if this
                    .search
                    .read(cx)
                    .text()
                    .chars()
                    .last()
                    .is_some_and(char::is_whitespace)
                {
                    this.commit_all_search_terms(cx);
                }
                cx.notify();
            }
            TextInputEvent::BackspaceEmpty => {
                if this.search_tokens.pop().is_some() {
                    this.search_menu_all = false;
                    cx.notify();
                }
            }
            TextInputEvent::Submit => {
                if this.search_menu_open && !this.search_suggestions(cx).is_empty() {
                    let index = this.search_suggestion;
                    this.apply_search_suggestion(index, cx);
                } else {
                    this.commit_all_search_terms(cx);
                }
            }
        })
        .detach();

        for field in [
            self.account_search.clone(),
            self.category_search.clone(),
            self.filter_min.clone(),
            self.filter_max.clone(),
            self.filter_start.clone(),
            self.filter_end.clone(),
            self.admin_invite.clone(),
            self.admin_currency_search.clone(),
            self.admin_rate_date.clone(),
            self.admin_backfill_days.clone(),
            self.admin_manual_target.clone(),
            self.admin_manual_rate.clone(),
            self.admin_manual_date.clone(),
            self.settings_currency_search.clone(),
        ] {
            cx.subscribe(&field, |_, _, event, cx| {
                if matches!(event, TextInputEvent::Change) {
                    cx.notify();
                }
            })
            .detach();
        }
    }

    fn start_boot(&mut self, cx: &mut Context<Self>) {
        if self.settings.refresh_token.is_none() {
            self.boot = Boot::Login;
            return;
        }

        // Local-first: show the cached ledger immediately and let the token
        // refresh happen behind the scenes. If the session turns out to be
        // invalid, `finish_sync` moves us back to the sign-in screen.
        self.boot = Boot::Ready;
        self.user = Some(User {
            username: self.settings.username.clone(),
            email: self.settings.email.clone(),
            flags: self.settings.user_flags.clone(),
            default_currency_code: self.settings.default_currency_code.clone(),
        });
        self.start_auto_sync(cx);
        self.sync_now(false, cx);
        #[cfg(debug_assertions)]
        self.apply_debug_screen(cx);
    }

    #[cfg(debug_assertions)]
    fn apply_debug_screen(&mut self, cx: &mut Context<Self>) {
        if let Ok(detail) = std::env::var("SUNDER_START_DETAIL")
            && let Some((kind, id)) = detail.split_once(':')
        {
            self.detail = match kind.to_ascii_lowercase().as_str() {
                "account" => {
                    self.screen = Screen::Accounts;
                    Some(DetailView::Account(id.to_string()))
                }
                "category" => {
                    self.screen = Screen::Categories;
                    Some(DetailView::Category(id.to_string()))
                }
                "entry" => {
                    self.screen = Screen::Entries;
                    Some(DetailView::Entry(id.to_string()))
                }
                _ => None,
            };
            if self.detail.is_some() {
                cx.notify();
                return;
            }
        }
        let Ok(screen) = std::env::var("SUNDER_START_SCREEN") else {
            return;
        };
        self.screen = match screen.to_ascii_lowercase().as_str() {
            "entries" => Screen::Entries,
            "accounts" => Screen::Accounts,
            "categories" => Screen::Categories,
            "settings" => Screen::Settings,
            "admin" => {
                self.load_admin(cx);
                Screen::Admin
            }
            _ => return,
        };
        cx.notify();
    }

    #[cfg(debug_assertions)]
    fn maybe_open_debug_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.debug_overlay_opened {
            return;
        }
        let Ok(overlay) = std::env::var("SUNDER_START_OVERLAY") else {
            return;
        };
        match overlay.to_ascii_lowercase().as_str() {
            "entry" => {
                self.debug_overlay_opened = true;
                self.open_entry_editor(None, None, None, window, cx);
            }
            "account" => {
                self.debug_overlay_opened = true;
                self.open_account_editor(None, window, cx);
            }
            "category" => {
                self.debug_overlay_opened = true;
                self.open_category_editor(None, window, cx);
            }
            "admin" => {
                self.debug_overlay_opened = true;
                let user = AdminUser {
                    id: "debug-user".into(),
                    username: "alex".into(),
                    email: "alex@example.com".into(),
                    flags: vec!["is_admin".into()],
                };
                self.open_admin_editor(user, window, cx);
            }
            "shortcuts" => {
                self.debug_overlay_opened = true;
                self.show_shortcuts = true;
                cx.notify();
            }
            "logout" => {
                self.debug_overlay_opened = true;
                self.logout_unsynced = 3;
                self.logout_dialog = true;
                cx.notify();
            }
            "confirm" => {
                self.debug_overlay_opened = true;
                self.modal_seq += 1;
                self.confirm = Some(ConfirmDialog {
                    title: "Delete entry?".into(),
                    message: "A debug confirmation dialog.".into(),
                    confirm_label: "Delete entry".into(),
                    action: ConfirmAction::DeleteEntry("debug".into()),
                    seq: self.modal_seq,
                });
                cx.notify();
            }
            "calendar" => {
                self.debug_overlay_opened = true;
                self.open_entry_editor(None, None, None, window, cx);
                self.open_entry_calendar(cx);
            }
            "filter-calendar" => {
                self.debug_overlay_opened = true;
                self.show_filters = true;
                self.screen = Screen::Entries;
                self.open_filter_calendar(cx);
            }
            _ => {}
        }
    }

    fn complete_auth(&mut self, access_token: String, user: User, cx: &mut Context<Self>) {
        self.session_generation += 1;
        self.access_token = Some(access_token);
        let incoming_name = user.username.trim().to_string();
        let server = crate::store::normalize_base_url(&self.settings.base_url);
        if local_owner_mismatch(&self.settings, &incoming_name, &server) {
            self.snapshot = Snapshot::default();
            let _ = self.snapshot.save(&self.paths.data_file);
            self.settings.last_sync = None;
            self.filter = EntryFilter::default();
        }
        if !incoming_name.is_empty() {
            self.settings.owner = incoming_name.clone();
            self.settings.username = incoming_name;
        }
        self.settings.owner_base_url = server;
        if !user.email.is_empty() {
            self.settings.email = user.email.clone();
        }
        self.settings.user_flags = user.flags.clone();
        self.settings.default_currency_code = normalize_currency_code(&user.default_currency_code);
        self.user = Some(user);
        self.profile_loaded = true;
        self.boot = Boot::Ready;
        self.auth.busy = false;
        self.auth.message = None;
        let _ = self.settings.save(&self.paths.settings_file);
        self.start_auto_sync(cx);
        self.sync_now(false, cx);
        self.refresh_currencies_and_rates(false, cx);
        cx.notify();
    }

    /// Prefer the server-provided username, then the cached one, then the owner.
    pub fn display_name(&self) -> String {
        self.user
            .as_ref()
            .map(|user| user.username.clone())
            .filter(|name| !name.is_empty())
            .or_else(|| Some(self.settings.username.clone()).filter(|name| !name.is_empty()))
            .or_else(|| Some(self.settings.owner.clone()).filter(|name| !name.is_empty()))
            .unwrap_or_else(|| "there".into())
    }

    pub fn user_email(&self) -> String {
        self.user
            .as_ref()
            .map(|user| user.email.clone())
            .filter(|email| !email.is_empty())
            .unwrap_or_else(|| self.settings.email.clone())
    }

    pub(crate) fn submit_auth(&mut self, cx: &mut Context<Self>) {
        if self.auth.busy {
            return;
        }
        match self.auth.mode {
            AuthMode::Login => self.submit_login(cx),
            AuthMode::Register => self.submit_register(cx),
        }
    }

    fn submit_login(&mut self, cx: &mut Context<Self>) {
        let login = self.auth.login.read(cx).text().trim().to_string();
        let password = self.auth.password.read(cx).text().to_string();
        if login.is_empty() || password.is_empty() {
            self.auth.message = Some(AuthMessage {
                text: "Enter your email (or username) and password.".into(),
                success: false,
            });
            cx.notify();
            return;
        }

        self.use_current_server(cx);
        if login.contains('@') {
            self.settings.email = login.clone();
        }
        let api = self.api.clone();
        self.auth.busy = true;
        self.auth.message = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = login_and_load(api, login, password).await;
            this.update(cx, |app, cx| app.finish_auth_attempt(result, cx))
                .ok();
        })
        .detach();
    }

    fn submit_register(&mut self, cx: &mut Context<Self>) {
        let invite = self.auth.invite.read(cx).text().trim().to_string();
        let email = self.auth.email.read(cx).text().trim().to_string();
        let username = self.auth.username.read(cx).text().trim().to_string();
        let password = self.auth.password.read(cx).text().to_string();

        if invite.is_empty() || email.is_empty() || username.is_empty() || password.is_empty() {
            self.auth.message = Some(AuthMessage {
                text: "Fill in every field to create your account.".into(),
                success: false,
            });
            cx.notify();
            return;
        }
        if !email.contains('@') {
            self.auth.message = Some(AuthMessage {
                text: "That email address doesn't look right.".into(),
                success: false,
            });
            cx.notify();
            return;
        }

        self.use_current_server(cx);
        let api = self.api.clone();
        self.auth.busy = true;
        self.auth.message = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let registered = api
                .register(invite, email.clone(), username, password.clone())
                .await;
            let result = match registered {
                Ok(()) => login_and_load(api, email, password).await,
                Err(error) => Err(error),
            };
            this.update(cx, |app, cx| app.finish_register(result, cx))
                .ok();
        })
        .detach();
    }

    fn use_current_server(&mut self, cx: &Context<Self>) {
        let server = crate::store::normalize_base_url(self.auth.server.read(cx).text());
        self.settings.base_url = server.clone();
        self.api = ApiClient::new(server);
        let _ = self.settings.save(&self.paths.settings_file);
    }

    fn finish_auth_attempt(
        &mut self,
        result: Result<(String, User, String), ApiError>,
        cx: &mut Context<Self>,
    ) {
        self.auth.busy = false;
        match result {
            Ok((access, user, refresh_token)) => {
                self.settings.refresh_token = Some(refresh_token);
                let _ = self.settings.save(&self.paths.settings_file);
                self.complete_auth(access, user, cx);
            }
            Err(error) => {
                self.auth.message = Some(AuthMessage {
                    text: error.message,
                    success: false,
                });
                cx.notify();
            }
        }
    }

    fn finish_register(
        &mut self,
        result: Result<(String, User, String), ApiError>,
        cx: &mut Context<Self>,
    ) {
        self.auth.busy = false;
        match result {
            Ok((access, user, refresh_token)) => {
                self.settings.refresh_token = Some(refresh_token);
                let _ = self.settings.save(&self.paths.settings_file);
                self.complete_auth(access, user, cx);
            }
            Err(error) => {
                self.auth.message = Some(AuthMessage {
                    text: error.message,
                    success: false,
                });
                cx.notify();
            }
        }
    }

    pub fn set_auth_mode(&mut self, mode: AuthMode, cx: &mut Context<Self>) {
        if self.auth.busy {
            return;
        }
        self.auth.mode = mode;
        self.auth.message = None;
        self.auth.focused = false;
        if mode == AuthMode::Login {
            self.auth.password.update(cx, |input, cx| input.clear(cx));
            self.auth.email.update(cx, |input, cx| input.clear(cx));
            self.auth.username.update(cx, |input, cx| input.clear(cx));
            self.auth.invite.update(cx, |input, cx| input.clear(cx));
        }
        cx.notify();
    }

    pub fn begin_logout(&mut self, cx: &mut Context<Self>) {
        let unsynced = self.unsynced_count();
        if unsynced == 0 {
            self.complete_logout(cx);
        } else {
            self.modal_seq += 1;
            self.logout_unsynced = unsynced;
            self.logout_dialog = true;
            cx.notify();
        }
    }

    pub fn complete_logout(&mut self, cx: &mut Context<Self>) {
        self.snapshot = Snapshot::default();
        self.settings.last_sync = None;
        let _ = self.snapshot.save(&self.paths.data_file);
        let _ = self.settings.save(&self.paths.settings_file);
        self.logout_dialog = false;
        self.logout_after_sync = false;
        self.sign_out(cx);
    }

    pub fn discard_and_logout(&mut self, cx: &mut Context<Self>) {
        self.complete_logout(cx);
    }

    pub fn sync_and_logout(&mut self, cx: &mut Context<Self>) {
        self.logout_dialog = false;
        self.logout_after_sync = true;
        cx.notify();
        self.sync_now(true, cx);
    }

    pub fn sign_out(&mut self, cx: &mut Context<Self>) {
        self.session_generation += 1;
        self.sync_in_flight = false;
        self.sync_again = false;
        self.access_token = None;
        self.user = None;
        self.boot = Boot::Login;
        self.screen = Screen::Dashboard;
        self.detail = None;
        self.settings.refresh_token = None;
        self.settings.username.clear();
        self.settings.email.clear();
        self.settings.user_flags.clear();
        let _ = self.settings.save(&self.paths.settings_file);
        self.auth.mode = AuthMode::Login;
        self.auth.busy = false;
        self.auth.focused = false;
        self.auth.message = None;
        self.auth.password.update(cx, |input, cx| input.clear(cx));
        self.sync_state = SyncState::Idle;
        self.sync_details_open = false;
        self.profile_loaded = false;
        self.editor = None;
        self.account_editor = None;
        self.category_editor = None;
        self.admin_editor = None;
        self.admin_currency_editor = None;
        self.bulk_currency_editor = None;
        self.settings_currency_open = false;
        self.currencies_loading = false;
        self.confirm = None;
        self.calendar = None;
        self.show_filters = false;
        self.show_shortcuts = false;
        self.logout_dialog = false;
        self.logout_after_sync = false;
        self.logout_unsynced = 0;
        self.selected_entries.clear();
        self.selected_accounts.clear();
        self.selected_categories.clear();
        self.history.clear();
        self.history_index = 0;
        self.admin = AdminState::default();
        cx.notify();
    }

    pub(crate) fn session_expired(&mut self, cx: &mut Context<Self>) {
        self.session_generation += 1;
        self.sync_in_flight = false;
        self.sync_again = false;
        self.settings.refresh_token = None;
        let _ = self.settings.save(&self.paths.settings_file);
        self.access_token = None;
        self.boot = Boot::Login;
        self.sync_state = SyncState::Idle;
        self.sync_details_open = false;
        self.profile_loaded = false;
        self.detail = None;
        self.selected_entries.clear();
        self.selected_accounts.clear();
        self.selected_categories.clear();
        self.history.clear();
        self.history_index = 0;
        self.admin = AdminState::default();
        self.editor = None;
        self.account_editor = None;
        self.category_editor = None;
        self.admin_editor = None;
        self.admin_currency_editor = None;
        self.bulk_currency_editor = None;
        self.settings_currency_open = false;
        self.currencies_loading = false;
        self.confirm = None;
        self.calendar = None;
        self.show_filters = false;
        self.show_shortcuts = false;
        self.logout_dialog = false;
        self.logout_after_sync = false;
        self.logout_unsynced = 0;
        self.auth.focused = false;
        self.auth.message = Some(AuthMessage {
            text: "Your session expired. Please sign in again.".into(),
            success: false,
        });
        cx.notify();
    }

    pub fn load_admin(&mut self, cx: &mut Context<Self>) {
        self.admin.loading = true;
        self.admin.error = None;
        cx.notify();
        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, |api, token| {
                Box::pin(async move {
                    let users = api.admin_users(token.clone()).await?;
                    let invites = api.admin_invites(token.clone()).await?;
                    let currencies = api.admin_currencies(token).await?;
                    Ok((users, invites, currencies))
                })
            })
            .await;
            this.update(cx, |app, cx| {
                app.admin.loading = false;
                match result {
                    Ok(((users, invites, currencies), token)) => {
                        app.access_token = Some(token);
                        app.admin.users = users;
                        app.admin.invites = invites;
                        app.admin.currencies = currencies;
                        app.admin.loaded = true;
                        app.admin.error = None;
                    }
                    Err(error) => {
                        if error.status == Some(401) {
                            app.session_expired(cx);
                        } else {
                            app.admin.error = Some(error.message.clone());
                            app.toast(
                                ToastKind::Error,
                                format!("Couldn't load admin data.\n{}", error.message),
                                None,
                                cx,
                            );
                        }
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn create_admin_invite(&mut self, cx: &mut Context<Self>) {
        let token = self.admin_invite.read(cx).text().trim().to_string();
        if token.is_empty() {
            return;
        }
        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, |api, access| {
                let token = token.clone();
                Box::pin(async move { api.admin_create_invite(access, token).await })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok((invite, access)) => {
                    app.access_token = Some(access);
                    app.admin.invites.push(invite);
                    app.admin_invite.update(cx, |input, cx| input.clear(cx));
                    app.toast(ToastKind::Success, "Invite created.".into(), None, cx);
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't create the invite.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn admin_delete_user(&mut self, id: &str, cx: &mut Context<Self>) {
        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let id = id.to_string();
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, |api, access| {
                let id = id.clone();
                Box::pin(async move { api.admin_delete_user(access, id).await })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok(((), access)) => {
                    app.access_token = Some(access);
                    app.admin.users.retain(|user| user.id != id);
                    app.toast(ToastKind::Info, "User deleted.".into(), None, cx);
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't delete the user.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn admin_delete_invite(&mut self, id: &str, cx: &mut Context<Self>) {
        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let id = id.to_string();
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, |api, access| {
                let id = id.clone();
                Box::pin(async move { api.admin_delete_invite(access, id).await })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok(((), access)) => {
                    app.access_token = Some(access);
                    app.admin.invites.retain(|invite| invite.id != id);
                    app.toast(ToastKind::Info, "Invite deleted.".into(), None, cx);
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't delete the invite.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn request_admin_delete_user(&mut self, id: &str, cx: &mut Context<Self>) {
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete user?".into(),
            message: "This permanently removes the account and its tokens from the server.".into(),
            confirm_label: "Delete user".into(),
            action: ConfirmAction::AdminDeleteUser(id.to_string()),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn request_admin_delete_invite(&mut self, id: &str, cx: &mut Context<Self>) {
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete invite?".into(),
            message: "The code will stop working immediately.".into(),
            confirm_label: "Delete invite".into(),
            action: ConfirmAction::AdminDeleteInvite(id.to_string()),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn filtered_admin_currencies(&self, cx: &App) -> Vec<&Currency> {
        let query = self
            .admin_currency_search
            .read(cx)
            .text()
            .trim()
            .to_lowercase();
        let mut currencies: Vec<&Currency> = self
            .admin
            .currencies
            .iter()
            .filter(|currency| match self.admin.currency_filter {
                AdminCurrencyFilter::All => true,
                AdminCurrencyFilter::Active => currency.is_active,
                AdminCurrencyFilter::Inactive => !currency.is_active,
            })
            .filter(|currency| {
                query.is_empty()
                    || currency.code.to_lowercase().contains(&query)
                    || currency.name.to_lowercase().contains(&query)
                    || currency.symbol.to_lowercase().contains(&query)
            })
            .collect();
        currencies.sort_by(|a, b| a.code.cmp(&b.code));
        currencies
    }

    pub fn admin_toggle_currency_active(
        &mut self,
        code: String,
        is_active: bool,
        cx: &mut Context<Self>,
    ) {
        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let normalized = normalize_currency_code(&code);
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, move |api, token| {
                let code = normalized.clone();
                Box::pin(async move {
                    api.admin_update_currency(
                        token,
                        code,
                        AdminCurrencyPatch {
                            is_active: Some(is_active),
                            ..AdminCurrencyPatch::default()
                        },
                    )
                    .await
                })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok((currency, token)) => {
                    app.access_token = Some(token);
                    if let Some(existing) = app
                        .admin
                        .currencies
                        .iter_mut()
                        .find(|item| item.code.eq_ignore_ascii_case(&currency.code))
                    {
                        *existing = currency.clone();
                    } else {
                        app.admin.currencies.push(currency.clone());
                    }
                    app.refresh_currencies_and_rates(false, cx);
                    app.toast(
                        ToastKind::Success,
                        format!(
                            "{} {}.",
                            currency.code,
                            if currency.is_active {
                                "activated"
                            } else {
                                "deactivated"
                            }
                        ),
                        None,
                        cx,
                    );
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't update currency.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn admin_sync_latest_rates(&mut self, cx: &mut Context<Self>) {
        self.admin_sync_rates_impl(None, None, false, cx);
    }

    pub fn admin_sync_date_rates(&mut self, cx: &mut Context<Self>) {
        let raw_date = self.admin_rate_date.read(cx).text().trim().to_string();
        let Some(parsed) = crate::format::parse_date(&raw_date) else {
            self.toast(
                ToastKind::Error,
                "Enter a valid YYYY-MM-DD date to sync.".into(),
                None,
                cx,
            );
            return;
        };
        let date = parsed.format("%Y-%m-%d").to_string();
        self.admin_sync_rates_impl(Some(date), None, false, cx);
    }

    pub fn admin_backfill_rates(&mut self, cx: &mut Context<Self>) {
        let raw = self.admin_backfill_days.read(cx).text().trim().to_string();
        let Ok(days) = raw.parse::<u32>() else {
            self.toast(
                ToastKind::Error,
                "Enter a number of days between 1 and 366.".into(),
                None,
                cx,
            );
            return;
        };
        if !(1..=366).contains(&days) {
            self.toast(
                ToastKind::Error,
                "Backfill days must be between 1 and 366.".into(),
                None,
                cx,
            );
            return;
        }
        self.admin_sync_rates_impl(None, Some(days), false, cx);
    }

    fn admin_sync_rates_impl(
        &mut self,
        date: Option<String>,
        days: Option<u32>,
        force: bool,
        cx: &mut Context<Self>,
    ) {
        if self.admin.syncing_rates {
            return;
        }
        self.admin.syncing_rates = true;
        cx.notify();
        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, move |api, token| {
                let date = date.clone();
                Box::pin(async move { api.admin_sync_rates(token, date, days, force).await })
            })
            .await;
            this.update(cx, |app, cx| {
                app.admin.syncing_rates = false;
                match result {
                    Ok((resp, token)) => {
                        app.access_token = Some(token);
                        let summary = if let Some(stats) = resp.stats {
                            format!(
                                "Backfill finished ({} synced, {} skipped, {} failed).",
                                stats.synced, stats.skipped, stats.failed
                            )
                        } else if let Some(count) = resp.stored_count {
                            let label = resp
                                .date
                                .or(resp.fetched_date)
                                .unwrap_or_else(|| "latest".into());
                            format!("Stored {count} exchange rates for {label}.")
                        } else {
                            "Exchange rates synced.".into()
                        };
                        app.admin.last_sync_summary = Some(summary.clone());
                        app.toast(ToastKind::Success, summary, None, cx);
                        app.refresh_currencies_and_rates(false, cx);
                    }
                    Err(error) => {
                        if error.status == Some(401) {
                            app.session_expired(cx);
                        } else {
                            app.toast(
                                ToastKind::Error,
                                format!("Rate sync failed.\n{}", error.message),
                                None,
                                cx,
                            );
                        }
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn admin_save_manual_rate(&mut self, cx: &mut Context<Self>) {
        let target = normalize_currency_code(self.admin_manual_target.read(cx).text());
        let rate_raw = self.admin_manual_rate.read(cx).text().trim().to_string();
        let date_raw = self.admin_manual_date.read(cx).text().trim().to_string();
        if target.len() < 3 {
            self.toast(
                ToastKind::Error,
                "Enter a target currency code (e.g. EUR).".into(),
                None,
                cx,
            );
            return;
        }
        let Ok(rate) = rate_raw.replace(',', ".").parse::<f64>() else {
            self.toast(
                ToastKind::Error,
                "Enter a positive numeric exchange rate.".into(),
                None,
                cx,
            );
            return;
        };
        if !rate.is_finite() || rate <= 0.0 {
            self.toast(
                ToastKind::Error,
                "Exchange rate must be greater than zero.".into(),
                None,
                cx,
            );
            return;
        }
        let rate_date = if date_raw.is_empty() {
            None
        } else if let Some(parsed) = crate::format::parse_date(&date_raw) {
            Some(parsed.format("%Y-%m-%d").to_string())
        } else {
            self.toast(
                ToastKind::Error,
                "Use YYYY-MM-DD for the manual rate date.".into(),
                None,
                cx,
            );
            return;
        };

        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        cx.spawn(async move |this, cx| {
            let result = with_access(api, access, refresh, move |api, token| {
                let target = target.clone();
                let rate_date = rate_date.clone();
                Box::pin(async move {
                    api.admin_upsert_rate(token, Some("USD".into()), target, rate, rate_date)
                        .await
                })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok((saved, token)) => {
                    app.access_token = Some(token);
                    let mut map = HashMap::new();
                    map.insert(saved.target_currency_code.clone(), saved.rate);
                    app.snapshot.merge_rates(saved.rate_date.clone(), map);
                    app.persist(cx);
                    app.admin_manual_rate
                        .update(cx, |input, cx| input.clear(cx));
                    app.refresh_currencies_and_rates(false, cx);
                    app.toast(
                        ToastKind::Success,
                        format!(
                            "Saved 1 {} = {} {} for {}.",
                            saved.base_currency_code,
                            saved.rate,
                            saved.target_currency_code,
                            saved.rate_date
                        ),
                        None,
                        cx,
                    );
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't save manual rate.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    /// True when the currently focused element is one of Sunder's text fields.
    /// Mirrors the web client's KeybindListener, which ignores registered
    /// shortcuts while an input, textarea, or select has focus.
    pub fn text_input_focused(&self, window: &Window, cx: &App) -> bool {
        let Some(focused) = window.focused(cx) else {
            return false;
        };
        let mut handles = Vec::new();
        match self.boot {
            Boot::Login => {
                handles.extend([
                    self.auth.server.read(cx).focus(),
                    self.auth.login.read(cx).focus(),
                    self.auth.password.read(cx).focus(),
                    self.auth.invite.read(cx).focus(),
                    self.auth.email.read(cx).focus(),
                    self.auth.username.read(cx).focus(),
                ]);
            }
            Boot::Ready => {
                if self.detail.is_none() {
                    match self.screen {
                        Screen::Entries => {
                            handles.push(self.search.read(cx).focus());
                            if self.show_filters {
                                handles.extend([
                                    self.filter_min.read(cx).focus(),
                                    self.filter_max.read(cx).focus(),
                                    self.filter_start.read(cx).focus(),
                                    self.filter_end.read(cx).focus(),
                                ]);
                            }
                        }
                        Screen::Accounts => {
                            handles.push(self.account_search.read(cx).focus());
                        }
                        Screen::Categories => {
                            handles.push(self.category_search.read(cx).focus());
                        }
                        Screen::Admin => {
                            handles.extend([
                                self.admin_invite.read(cx).focus(),
                                self.admin_currency_search.read(cx).focus(),
                                self.admin_rate_date.read(cx).focus(),
                                self.admin_backfill_days.read(cx).focus(),
                                self.admin_manual_target.read(cx).focus(),
                                self.admin_manual_rate.read(cx).focus(),
                                self.admin_manual_date.read(cx).focus(),
                            ]);
                        }
                        Screen::Settings => {
                            handles.push(self.currency_input.read(cx).focus());
                            if self.settings_currency_open {
                                handles.push(self.settings_currency_search.read(cx).focus());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(editor) = &self.editor {
            handles.extend([
                editor.amount.read(cx).focus(),
                editor.title.read(cx).focus(),
                editor.date.read(cx).focus(),
                editor.description.read(cx).focus(),
                editor.location.read(cx).focus(),
                editor.currency_search.read(cx).focus(),
            ]);
        }
        if let Some(editor) = &self.account_editor {
            handles.extend([
                editor.name.read(cx).focus(),
                editor.currency_search.read(cx).focus(),
            ]);
        }
        if let Some(editor) = &self.category_editor {
            handles.extend([
                editor.title.read(cx).focus(),
                editor.description.read(cx).focus(),
                editor.color_input.read(cx).focus(),
            ]);
        }
        if let Some(editor) = &self.admin_editor {
            handles.extend([
                editor.username.read(cx).focus(),
                editor.email.read(cx).focus(),
                editor.flags.read(cx).focus(),
                editor.password.read(cx).focus(),
            ]);
        }
        if let Some(editor) = &self.admin_currency_editor {
            handles.extend([
                editor.code.read(cx).focus(),
                editor.name.read(cx).focus(),
                editor.symbol.read(cx).focus(),
                editor.decimal_digits.read(cx).focus(),
            ]);
        }
        if let Some(editor) = &self.bulk_currency_editor {
            handles.push(editor.search.read(cx).focus());
        }
        handles.contains(&focused)
    }

    pub fn overlay_open(&self) -> bool {
        self.editor.is_some()
            || self.account_editor.is_some()
            || self.category_editor.is_some()
            || self.admin_editor.is_some()
            || self.admin_currency_editor.is_some()
            || self.bulk_currency_editor.is_some()
            || self.confirm.is_some()
            || self.show_shortcuts
            || self.logout_dialog
            || self.calendar.is_some()
    }

    fn current_history_entry(&self) -> HistoryEntry {
        HistoryEntry {
            screen: self.screen,
            detail: self.detail.clone(),
        }
    }

    fn seed_history(&mut self) {
        if self.history.is_empty() {
            self.history.push(self.current_history_entry());
            self.history_index = 0;
        }
    }

    fn record_history(&mut self) {
        self.seed_history();
        let entry = self.current_history_entry();
        if self.history.get(self.history_index) == Some(&entry) {
            return;
        }
        self.history.truncate(self.history_index + 1);
        self.history.push(entry);
        self.history_index = self.history.len() - 1;
    }

    fn apply_history_entry(&mut self, entry: HistoryEntry, cx: &mut Context<Self>) {
        self.screen = entry.screen;
        self.detail = entry.detail;
        self.editor = None;
        self.account_editor = None;
        self.category_editor = None;
        self.admin_editor = None;
        self.admin_currency_editor = None;
        self.bulk_currency_editor = None;
        self.settings_currency_open = false;
        self.confirm = None;
        self.show_shortcuts = false;
        self.logout_dialog = false;
        self.calendar = None;
        self.show_filters = false;
        self.sort_menu_open = false;
        self.search_menu_open = false;
        self.search_menu_all = false;
        self.selected_entries.clear();
        self.selected_accounts.clear();
        self.selected_categories.clear();
        self.page_generation += 1;
        if self.screen == Screen::Admin {
            self.load_admin(cx);
        }
        cx.notify();
    }

    pub fn navigate_history(&mut self, delta: i32, cx: &mut Context<Self>) {
        if self.boot != Boot::Ready {
            return;
        }
        self.seed_history();
        if delta < 0 {
            if self.history_index == 0 {
                return;
            }
            self.history_index -= 1;
        } else {
            if self.history_index + 1 >= self.history.len() {
                return;
            }
            self.history_index += 1;
        }
        let entry = self.history[self.history_index].clone();
        self.apply_history_entry(entry, cx);
    }

    /// Mouse back: close the topmost overlay first, otherwise walk history.
    pub fn mouse_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.boot != Boot::Ready {
            return;
        }
        if self.overlay_open() || self.show_filters {
            self.close_top_overlay(window, cx);
        } else {
            self.navigate_history(-1, cx);
        }
    }

    /// Mouse forward: walk history. Overlays are handled by the back button.
    pub fn mouse_forward(&mut self, cx: &mut Context<Self>) {
        self.navigate_history(1, cx);
    }

    pub fn go(&mut self, screen: Screen, _window: &mut Window, cx: &mut Context<Self>) {
        if self.overlay_open() {
            return;
        }
        self.seed_history();
        if self.screen != screen || self.detail.is_some() {
            self.page_generation += 1;
        }
        if screen == Screen::Categories {
            self.category_count_filter = CategoryCountFilter::Any;
        }
        self.screen = screen;
        self.detail = None;
        self.editor = None;
        self.account_editor = None;
        self.category_editor = None;
        self.admin_currency_editor = None;
        self.bulk_currency_editor = None;
        self.settings_currency_open = false;
        self.confirm = None;
        self.selected_entries.clear();
        self.selected_accounts.clear();
        self.selected_categories.clear();
        if screen == Screen::Entries {
            self.filter = EntryFilter::default();
            self.show_filters = false;
            self.search_tokens.clear();
            self.search_menu_open = false;
            self.search_menu_all = false;
            self.search.update(cx, |input, cx| input.clear(cx));
            self.filter_min.update(cx, |input, cx| input.clear(cx));
            self.filter_max.update(cx, |input, cx| input.clear(cx));
            self.filter_start.update(cx, |input, cx| input.clear(cx));
            self.filter_end.update(cx, |input, cx| input.clear(cx));
        }
        if screen == Screen::Accounts {
            self.account_search.update(cx, |input, cx| input.clear(cx));
        }
        if screen == Screen::Categories {
            self.category_search.update(cx, |input, cx| input.clear(cx));
        }
        if screen == Screen::Admin {
            self.load_admin(cx);
        }
        self.record_history();
        cx.notify();
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.settings.theme = self.settings.theme.toggled();
        set_theme(cx, self.settings.theme);
        let _ = self.settings.save(&self.paths.settings_file);
        cx.notify();
    }

    pub fn set_auto_sync(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.settings.auto_sync = enabled;
        let _ = self.settings.save(&self.paths.settings_file);
        if enabled {
            self.last_auto_sync = Some(std::time::Instant::now());
            self.sync_now(false, cx);
        }
        cx.notify();
    }

    pub fn set_auto_sync_interval(&mut self, minutes: u32, cx: &mut Context<Self>) {
        self.settings.auto_sync_interval_minutes = minutes;
        let _ = self.settings.save(&self.paths.settings_file);
        cx.notify();
    }

    pub fn set_sync_bar_minimized(&mut self, minimized: bool, cx: &mut Context<Self>) {
        self.settings.sync_bar_minimized = minimized;
        if minimized {
            self.sync_details_open = false;
        }
        let _ = self.settings.save(&self.paths.settings_file);
        cx.notify();
    }

    pub fn toggle_sync_details(&mut self, cx: &mut Context<Self>) {
        if self.unsynced_count() == 0 {
            return;
        }
        self.sync_details_open = !self.sync_details_open;
        if self.sync_details_open {
            self.sync_details_seq += 1;
        }
        cx.notify();
    }

    pub fn confirm_clear_local_data(&mut self, cx: &mut Context<Self>) {
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Clear local data?".into(),
            message: "Sunder will remove the local copy on this computer and pull a fresh copy from the server on the next sync. Anything that hasn't synced yet will be lost.".into(),
            confirm_label: "Clear local data".into(),
            action: ConfirmAction::ClearLocalData,
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn default_currency_code(&self) -> String {
        normalize_currency_code(&self.settings.default_currency_code)
    }

    pub fn default_currency(&self) -> Currency {
        self.snapshot.currency(&self.default_currency_code())
    }

    pub fn currency(&self, code: &str) -> Currency {
        self.snapshot.currency(code)
    }

    pub fn set_default_currency(&mut self, code: &str, cx: &mut Context<Self>) {
        let normalized = normalize_currency_code(code);
        self.settings.default_currency_code = normalized.clone();
        if let Some(user) = self.user.as_mut() {
            user.default_currency_code = normalized.clone();
        }
        let meta = self.snapshot.currency(&normalized);
        self.settings.currency_symbol = meta.symbol.clone();
        self.settings.currency_position = if meta.symbol_after {
            crate::format::CurrencyPosition::After
        } else {
            crate::format::CurrencyPosition::Before
        };
        let _ = self.settings.save(&self.paths.settings_file);
        self.settings_currency_open = false;
        self.settings_currency_search
            .update(cx, |input, cx| input.clear(cx));
        cx.notify();

        if self.boot == Boot::Ready
            && (self.access_token.is_some() || self.settings.refresh_token.is_some())
        {
            let api = self.api.clone();
            let access = self.access_token.clone();
            let refresh = self.settings.refresh_token.clone();
            let code_for_req = normalized.clone();
            cx.spawn(async move |this, cx| {
                let result = with_access(api, access, refresh, move |api, token| {
                    let code = code_for_req.clone();
                    Box::pin(async move { api.update_preferences(token, code).await })
                })
                .await;
                this.update(cx, |app, cx| match result {
                    Ok((prefs, token)) => {
                        app.access_token = Some(token);
                        let code = normalize_currency_code(&prefs.default_currency_code);
                        app.settings.default_currency_code = code.clone();
                        if let Some(user) = app.user.as_mut() {
                            user.default_currency_code = code;
                        }
                        if let Some(currency) = prefs.default_currency {
                            if let Some(existing) = app
                                .snapshot
                                .currencies
                                .iter_mut()
                                .find(|c| c.code.eq_ignore_ascii_case(&currency.code))
                            {
                                *existing = currency;
                            } else {
                                app.snapshot.currencies.push(currency);
                            }
                        }
                        app.persist(cx);
                        cx.notify();
                    }
                    Err(error) => {
                        if error.status == Some(401) {
                            app.session_expired(cx);
                        } else {
                            app.toast(
                                ToastKind::Error,
                                format!(
                                    "Saved default currency locally, but couldn't sync preference.\n{}",
                                    error.message
                                ),
                                None,
                                cx,
                            );
                        }
                    }
                })
                .ok();
            })
            .detach();
        }
    }

    pub fn refresh_currencies_and_rates(&mut self, manual: bool, cx: &mut Context<Self>) {
        if self.currencies_loading {
            return;
        }
        self.currencies_loading = true;
        cx.notify();

        let api = self.api.clone();
        let bounds = self.snapshot.rate_date_bounds();
        let generation = self.session_generation;

        cx.spawn(async move |this, cx| {
            let currencies_result = api.list_currencies().await;
            let latest_result = api.rates(Some("USD".into()), None).await;

            let mut history_chunks: Vec<HashMap<String, HashMap<String, f64>>> = Vec::new();
            if let Some((from_str, to_str)) = bounds
                && let (Some(mut cursor), Some(end)) = (
                    crate::format::parse_date(&from_str),
                    crate::format::parse_date(&to_str),
                )
            {
                let today = chrono::Local::now().date_naive();
                let end = end.max(today);
                while cursor <= end {
                    let window_end = (cursor + chrono::Duration::days(364)).min(end);
                    if let Ok(history) = api
                        .rates_history(
                            Some("USD".into()),
                            cursor.format("%Y-%m-%d").to_string(),
                            Some(window_end.format("%Y-%m-%d").to_string()),
                        )
                        .await
                    {
                        history_chunks.push(history.rates);
                    }
                    cursor = window_end + chrono::Duration::days(1);
                }
            }

            this.update(cx, |app, cx| {
                app.currencies_loading = false;
                if generation != app.session_generation {
                    return;
                }
                let mut updated = false;
                let mut error_msg: Option<String> = None;

                match currencies_result {
                    Ok(currencies) if !currencies.is_empty() => {
                        app.snapshot.merge_currencies(currencies);
                        updated = true;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        error_msg = Some(error.message);
                    }
                }

                match latest_result {
                    Ok(latest) => {
                        let date = if !latest.date.is_empty() {
                            latest.date
                        } else {
                            latest.requested_date
                        };
                        if !date.is_empty() && !latest.rates.is_empty() {
                            app.snapshot.merge_rates(date, latest.rates);
                            updated = true;
                        }
                    }
                    Err(error) => {
                        if error_msg.is_none() {
                            error_msg = Some(error.message);
                        }
                    }
                }

                for chunk in history_chunks {
                    if !chunk.is_empty() {
                        app.snapshot.merge_rates_history(chunk);
                        updated = true;
                    }
                }

                if updated {
                    app.persist(cx);
                }
                if manual {
                    if let Some(msg) = error_msg {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't refresh currencies & rates.\n{msg}"),
                            None,
                            cx,
                        );
                    } else {
                        app.toast(
                            ToastKind::Success,
                            format!(
                                "Updated {} currencies and {} rate days.",
                                app.snapshot.active_currencies().len(),
                                app.snapshot.exchange_rates.len()
                            ),
                            None,
                            cx,
                        );
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn set_reduce_motion(&mut self, reduce: bool, cx: &mut Context<Self>) {
        if self.settings.reduce_motion == reduce {
            return;
        }
        self.settings.reduce_motion = reduce;
        let _ = self.settings.save(&self.paths.settings_file);
        cx.set_global(crate::motion::MotionPreference {
            reduce_motion: reduce,
        });
        cx.notify();
    }

    pub fn money(&self, value: f64) -> String {
        format::money_in(value, &self.default_currency())
    }

    pub fn signed_money(&self, value: f64) -> String {
        format::signed_money_in(value, &self.default_currency())
    }

    pub fn compact_money(&self, value: f64) -> String {
        format::compact_money_in(value, &self.default_currency())
    }

    pub fn money_in(&self, value: f64, code: &str) -> String {
        format::money_in(value, &self.currency(code))
    }

    pub fn signed_money_in(&self, value: f64, code: &str) -> String {
        format::signed_money_in(value, &self.currency(code))
    }

    pub fn convert_entry_amount(&self, entry: &Entry) -> f64 {
        self.snapshot
            .convert_entry_amount(entry, &self.default_currency_code())
    }

    pub fn entry_converted_preview(&self, entry: &Entry) -> Option<String> {
        let entry_code = entry.normalized_currency_code();
        let default_code = self.default_currency_code();
        if entry_code == default_code {
            return None;
        }
        let converted = self.snapshot.convert_amount(
            entry.amount,
            &entry_code,
            &default_code,
            &entry.date_ymd(),
        )?;
        Some(format!("≈ {}", self.signed_money(converted)))
    }

    pub fn export_data(&mut self, cx: &mut Context<Self>) {
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let home = directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);
        let file_name = format!(
            "sunder_backup_{}.json",
            chrono::Local::now().format("%Y-%m-%d")
        );
        let receiver = cx.prompt_for_new_path(&home, Some(&file_name));
        let api = self.api.clone();
        cx.spawn(async move |this, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) | Err(_) => return,
                Ok(Err(error)) => {
                    this.update(cx, |app, cx| {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't choose a destination.\n{error}"),
                            None,
                            cx,
                        )
                    })
                    .ok();
                    return;
                }
            };
            let result = with_access(api, access, refresh, |api, access| {
                Box::pin(async move { api.export_data(access).await })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok((bytes, access)) => {
                    app.access_token = Some(access);
                    if let Err(error) = std::fs::write(&path, &bytes) {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't write the backup.\n{error}"),
                            None,
                            cx,
                        );
                    } else {
                        app.toast(
                            ToastKind::Success,
                            format!("Backup saved to {}.", path.display()),
                            None,
                            cx,
                        );
                    }
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Export failed.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn import_data(&mut self, kind: ImportKind, cx: &mut Context<Self>) {
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Choose a data file".into()),
        });
        let api = self.api.clone();
        cx.spawn(async move |this, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(paths))) => match paths.into_iter().next() {
                    Some(path) => path,
                    None => return,
                },
                Ok(Ok(None)) | Err(_) => return,
                Ok(Err(error)) => {
                    this.update(cx, |app, cx| {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't open the file picker.\n{error}"),
                            None,
                            cx,
                        )
                    })
                    .ok();
                    return;
                }
            };
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "import".into());
            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    this.update(cx, |app, cx| {
                        app.toast(
                            ToastKind::Error,
                            format!("Couldn't read that file.\n{error}"),
                            None,
                            cx,
                        )
                    })
                    .ok();
                    return;
                }
            };
            let result = with_access(api, access, refresh, |api, access| {
                let name = name.clone();
                let bytes = bytes.clone();
                Box::pin(async move { api.import_data(access, kind, name, bytes).await })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok((stats, access)) => {
                    app.access_token = Some(access);
                    app.toast(
                        ToastKind::Success,
                        format!(
                            "Imported {} accounts, {} categories, and {} entries.",
                            stats.accounts, stats.categories, stats.entries
                        ),
                        None,
                        cx,
                    );
                    app.sync_now(false, cx);
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else {
                        app.toast(
                            ToastKind::Error,
                            format!("Import failed.\n{}", error.message),
                            None,
                            cx,
                        );
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn open_entry_calendar(&mut self, cx: &mut Context<Self>) {
        let selected = self
            .editor
            .as_ref()
            .and_then(|editor| crate::format::parse_date(editor.date.read(cx).text()))
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        self.modal_seq += 1;
        self.calendar = Some(CalendarState {
            target: CalendarTarget::EntryDate,
            view: CalendarView::Days,
            year: selected.year(),
            month: selected.month(),
            start: Some(selected),
            end: Some(selected),
            hover: None,
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn open_filter_calendar(&mut self, cx: &mut Context<Self>) {
        let start = crate::format::parse_date(self.filter_start.read(cx).text());
        let end = crate::format::parse_date(self.filter_end.read(cx).text());
        let anchor = start.unwrap_or_else(|| chrono::Local::now().date_naive());
        self.modal_seq += 1;
        self.calendar = Some(CalendarState {
            target: CalendarTarget::FilterRange,
            view: CalendarView::Days,
            year: anchor.year(),
            month: anchor.month(),
            start,
            end,
            hover: None,
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn calendar_hover(&mut self, date: Option<NaiveDate>, cx: &mut Context<Self>) {
        if let Some(calendar) = self.calendar.as_mut()
            && calendar.hover != date
        {
            calendar.hover = date;
            cx.notify();
        }
    }

    pub fn calendar_shift(&mut self, direction: i32, cx: &mut Context<Self>) {
        let Some(calendar) = self.calendar.as_mut() else {
            return;
        };
        match calendar.view {
            CalendarView::Days => {
                let mut month = calendar.month as i32 + direction;
                while month < 1 {
                    month += 12;
                    calendar.year -= 1;
                }
                while month > 12 {
                    month -= 12;
                    calendar.year += 1;
                }
                calendar.month = month as u32;
            }
            CalendarView::Months => calendar.year += direction,
            CalendarView::Years => calendar.year += direction * 12,
        }
        cx.notify();
    }

    pub fn calendar_set_view(&mut self, view: CalendarView, cx: &mut Context<Self>) {
        if let Some(calendar) = self.calendar.as_mut() {
            calendar.view = view;
        }
        cx.notify();
    }

    pub fn calendar_select_month(&mut self, month: u32, cx: &mut Context<Self>) {
        if let Some(calendar) = self.calendar.as_mut() {
            calendar.month = month;
            calendar.view = CalendarView::Days;
        }
        cx.notify();
    }

    pub fn calendar_select_year(&mut self, year: i32, cx: &mut Context<Self>) {
        if let Some(calendar) = self.calendar.as_mut() {
            calendar.year = year;
            calendar.view = CalendarView::Months;
        }
        cx.notify();
    }

    pub fn calendar_pick(&mut self, date: NaiveDate, cx: &mut Context<Self>) {
        let Some(calendar) = self.calendar.as_mut() else {
            return;
        };
        match calendar.target {
            CalendarTarget::EntryDate => {
                if let Some(editor) = self.editor.as_ref() {
                    let value = date.format("%Y-%m-%d").to_string();
                    editor
                        .date
                        .update(cx, |input, cx| input.set_text(value, cx));
                }
                self.calendar = None;
            }
            CalendarTarget::FilterRange => {
                let (start, end) = select_calendar_range(calendar.start, calendar.end, date);
                calendar.start = start;
                calendar.end = end;
            }
        }
        cx.notify();
    }

    pub fn calendar_today(&mut self, cx: &mut Context<Self>) {
        let today = chrono::Local::now().date_naive();
        if let Some(calendar) = self.calendar.as_mut() {
            calendar.year = today.year();
            calendar.month = today.month();
            calendar.view = CalendarView::Days;
            // The web picker keeps a completed range pinned to today.
            if calendar.target == CalendarTarget::FilterRange
                && calendar.start.is_some()
                && calendar.end.is_some()
            {
                calendar.start = Some(today);
                calendar.end = Some(today);
                cx.notify();
                return;
            }
        }
        self.calendar_pick(today, cx);
    }

    pub fn calendar_quick_pick(&mut self, months: u32, cx: &mut Context<Self>) {
        let today = chrono::Local::now().date_naive();
        let start = months_before(today, months);
        if let Some(calendar) = self.calendar.as_mut() {
            calendar.start = Some(start);
            calendar.end = Some(today);
            calendar.year = start.year();
            calendar.month = start.month();
            calendar.view = CalendarView::Days;
        }
        cx.notify();
    }

    pub fn calendar_done(&mut self, cx: &mut Context<Self>) {
        let Some(calendar) = self.calendar.as_ref() else {
            return;
        };
        if calendar.target == CalendarTarget::FilterRange {
            if let Some(start) = calendar.start {
                let value = start.format("%Y-%m-%d").to_string();
                self.filter_start
                    .update(cx, |input, cx| input.set_text(value, cx));
            }
            if let Some(end) = calendar.end {
                let value = end.format("%Y-%m-%d").to_string();
                self.filter_end
                    .update(cx, |input, cx| input.set_text(value, cx));
            }
        }
        self.calendar = None;
        cx.notify();
    }

    pub fn calendar_close(&mut self, cx: &mut Context<Self>) {
        self.calendar = None;
        cx.notify();
    }

    pub fn request_discard_unsynced(&mut self, cx: &mut Context<Self>) {
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Discard unsynced changes?".into(),
            message: "Sunder will remove the local copy on this computer and pull a fresh copy from the server. Anything that hasn't synced yet will be lost.".into(),
            confirm_label: "Discard changes".into(),
            action: ConfirmAction::DiscardUnsynced,
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn discard_unsynced_changes(&mut self, cx: &mut Context<Self>) {
        self.sync_details_open = false;
        self.snapshot = Snapshot::default();
        self.settings.last_sync = None;
        let _ = self.snapshot.save(&self.paths.data_file);
        let _ = self.settings.save(&self.paths.settings_file);
        self.selected_entries.clear();
        self.selected_accounts.clear();
        self.selected_categories.clear();
        self.detail = None;
        self.toast(
            ToastKind::Info,
            "Local changes discarded. Pulling a fresh copy from the server.".into(),
            None,
            cx,
        );
        self.sync_now(false, cx);
    }

    pub fn set_category_count_filter(
        &mut self,
        filter: CategoryCountFilter,
        cx: &mut Context<Self>,
    ) {
        if self.category_count_filter == filter {
            return;
        }
        self.category_count_filter = filter;
        cx.notify();
    }

    pub fn clear_category_filters(&mut self, cx: &mut Context<Self>) {
        self.category_count_filter = CategoryCountFilter::Any;
        self.category_search.update(cx, |input, cx| input.clear(cx));
        cx.notify();
    }

    pub fn visible_categories(&self, cx: &App) -> Vec<&Category> {
        let query = self.category_search.read(cx).text().trim().to_lowercase();
        self.snapshot
            .active_categories()
            .into_iter()
            .filter(|category| query.is_empty() || category.title.to_lowercase().contains(&query))
            .filter(|category| {
                self.category_count_filter
                    .matches(self.snapshot.category_entry_count(&category.id))
            })
            .collect()
    }

    pub fn entry_query(&self, cx: &App) -> EntryQuery {
        let mut terms = self.search_tokens.clone();
        terms.extend(EntryQuery::parse(self.search.read(cx).text()).terms);
        EntryQuery { terms }
    }

    /// Remove a committed power-search chip by index.
    pub fn remove_search_term(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.search_tokens.len() {
            self.search_tokens.remove(index);
            cx.notify();
        }
    }

    /// Convert every structured term in the field into a chip (Enter).
    fn commit_all_search_terms(&mut self, cx: &mut Context<Self>) {
        let text = self.search.read(cx).text().to_string();
        let query = EntryQuery::parse(&text);
        let mut terms = query
            .terms
            .into_iter()
            .filter(|term| term.kind != SearchKind::Text)
            .collect::<Vec<_>>();
        if terms.is_empty() {
            return;
        }
        terms.sort_by_key(|term| term.span.start);
        let mut remaining = String::with_capacity(text.len());
        let mut cursor = 0;
        for term in &terms {
            remaining.push_str(&text[cursor..term.span.start]);
            cursor = term.span.end;
        }
        remaining.push_str(&text[cursor..]);
        let remaining = remaining.trim().to_string();
        for term in &mut terms {
            term.span = 0..0;
        }
        self.search_tokens.extend(terms);
        self.search
            .update(cx, |input, cx| input.set_text(remaining, cx));
        self.search_menu_open = false;
        self.search_menu_all = false;
        self.search_suggestion = 0;
        cx.notify();
    }

    pub fn search_suggestions(&self, cx: &App) -> Vec<SearchSuggestion> {
        let text = self.search.read(cx).text().to_string();
        let cursor = self.search.read(cx).cursor_offset();
        let categories = self
            .snapshot
            .active_categories()
            .into_iter()
            .map(|category| category.title.clone())
            .collect::<Vec<_>>();
        let accounts = self
            .snapshot
            .active_accounts()
            .into_iter()
            .map(|account| account.name.clone())
            .collect::<Vec<_>>();
        let mut currencies = Vec::new();
        for entry in self.snapshot.active_entries() {
            let code = entry.normalized_currency_code();
            if !currencies.contains(&code) {
                currencies.push(code);
            }
        }
        for currency in self.snapshot.active_currencies() {
            if !currencies.contains(&currency.code) {
                currencies.push(currency.code.clone());
            }
        }
        let mut titles = Vec::new();
        let mut locations = Vec::new();
        for entry in self.snapshot.active_entries() {
            if titles.len() < 12 && !entry.title.is_empty() && !titles.contains(&entry.title) {
                titles.push(entry.title.clone());
            }
            if locations.len() < 12
                && let Some(location) = entry.location.as_ref()
                && !location.is_empty()
                && !locations.contains(location)
            {
                locations.push(location.clone());
            }
            if titles.len() >= 12 && locations.len() >= 12 {
                break;
            }
        }
        crate::search::suggestions(
            &text,
            cursor,
            &categories,
            &accounts,
            &currencies,
            &titles,
            &locations,
            self.search_menu_all,
        )
    }

    pub fn apply_search_suggestion(&mut self, index: usize, cx: &mut Context<Self>) {
        let suggestions = self.search_suggestions(cx);
        let Some(suggestion) = suggestions.get(index).cloned() else {
            return;
        };
        let text = self.search.read(cx).text().to_string();
        let start = suggestion.start.min(text.len());
        let end = suggestion.end.min(text.len()).max(start);
        let mut next = String::with_capacity(text.len() + suggestion.insert.len());
        next.push_str(&text[..start]);
        next.push_str(&suggestion.insert);
        next.push_str(&text[end..]);
        let commits = suggestion.insert.ends_with(' ');
        self.search.update(cx, |input, cx| input.set_text(next, cx));
        self.search_suggestion = 0;
        self.search_menu_all = false;
        self.search_menu_open = !commits;
        cx.notify();
    }

    pub fn action_search_open(
        &mut self,
        _: &SearchOpenSuggestions,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_menu_open = true;
        self.search_menu_all = true;
        self.search_suggestion = 0;
        cx.notify();
    }

    pub fn action_search_next(&mut self, _: &SearchNext, _: &mut Window, cx: &mut Context<Self>) {
        let suggestions = self.search_suggestions(cx);
        if suggestions.is_empty() {
            return;
        }
        self.search_menu_open = true;
        self.search_suggestion = (self.search_suggestion + 1) % suggestions.len();
        cx.notify();
    }

    pub fn action_search_previous(
        &mut self,
        _: &SearchPrevious,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let suggestions = self.search_suggestions(cx);
        if suggestions.is_empty() {
            return;
        }
        self.search_menu_open = true;
        self.search_suggestion = if self.search_suggestion == 0 {
            suggestions.len() - 1
        } else {
            self.search_suggestion - 1
        };
        cx.notify();
    }

    pub fn action_search_complete(
        &mut self,
        _: &SearchComplete,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let suggestions = self.search_suggestions(cx);
        if self.search_menu_open && !suggestions.is_empty() {
            let index = self.search_suggestion.min(suggestions.len() - 1);
            self.apply_search_suggestion(index, cx);
            return;
        }
        window.focus_next();
    }

    pub fn set_entry_sort(&mut self, sort: SortField, descending: bool, cx: &mut Context<Self>) {
        self.filter.sort = sort;
        self.filter.descending = descending;
        self.sort_menu_open = false;
        cx.notify();
    }

    pub fn visible_entries(&self, cx: &App) -> Vec<&Entry> {
        let query = self.entry_query(cx);
        let min_amount = crate::format::parse_amount(self.filter_min.read(cx).text());
        let max_amount = crate::format::parse_amount(self.filter_max.read(cx).text());
        let start_date = crate::format::parse_date(self.filter_start.read(cx).text());
        let end_date = crate::format::parse_date(self.filter_end.read(cx).text());

        let mut entries: Vec<&Entry> = self
            .snapshot
            .active_entries()
            .into_iter()
            .filter(|entry| {
                let entry_code = entry.normalized_currency_code();
                (self.filter.account_ids.is_empty()
                    || self.filter.account_ids.contains(&entry.account_id))
                    && (self.filter.category_ids.is_empty()
                        || self.filter.category_ids.contains(&entry.category_id))
                    && (self.filter.currency_codes.is_empty()
                        || self
                            .filter
                            .currency_codes
                            .iter()
                            .any(|code| code == &entry_code))
            })
            .filter(|entry| {
                let amount = entry.amount.abs();
                min_amount.is_none_or(|min| amount >= min)
                    && max_amount.is_none_or(|max| amount <= max)
            })
            .filter(|entry| {
                let Some(date) = entry.date_naive() else {
                    return start_date.is_none() && end_date.is_none();
                };
                start_date.is_none_or(|start| date >= start)
                    && end_date.is_none_or(|end| date <= end)
            })
            .filter(|entry| query.matches(entry, &self.snapshot))
            .collect();

        entries.sort_by(|a, b| {
            let ordering = match self.filter.sort {
                SortField::Date => {
                    crate::model::compare_timestamps(a.occurred_at(), b.occurred_at())
                }
                SortField::Amount => self
                    .convert_entry_amount(a)
                    .abs()
                    .partial_cmp(&self.convert_entry_amount(b).abs())
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            };
            let ordering = if self.filter.descending {
                ordering.reverse()
            } else {
                ordering
            };
            ordering.then_with(|| crate::model::compare_timestamps(&b.inserted_at, &a.inserted_at))
        });
        entries
    }

    pub fn active_filter_count(&self, cx: &App) -> usize {
        let mut count = 0;
        if !self.filter.account_ids.is_empty() {
            count += 1;
        }
        if !self.filter.category_ids.is_empty() {
            count += 1;
        }
        if !self.filter.currency_codes.is_empty() {
            count += 1;
        }
        if !self.filter_min.read(cx).text().trim().is_empty() {
            count += 1;
        }
        if !self.filter_max.read(cx).text().trim().is_empty() {
            count += 1;
        }
        if !self.filter_start.read(cx).text().trim().is_empty() {
            count += 1;
        }
        if !self.filter_end.read(cx).text().trim().is_empty() {
            count += 1;
        }
        if self.filter.sort != SortField::Date || !self.filter.descending {
            count += 1;
        }
        count
    }

    pub fn unsynced_count(&self) -> usize {
        self.snapshot
            .dirty_since(self.settings.last_sync.as_deref())
            .len()
    }

    pub fn top_categories(&self, limit: usize) -> Vec<TopCategory> {
        let mut by_category: HashMap<String, f64> = HashMap::new();
        for entry in self
            .snapshot
            .entries
            .iter()
            .filter(|entry| entry.is_active())
        {
            if entry.amount < 0.0 {
                *by_category.entry(entry.category_id.clone()).or_default() +=
                    self.convert_entry_amount(entry).abs();
            }
        }
        let mut categories: Vec<_> = by_category
            .into_iter()
            .map(|(id, spend)| {
                let category = self.snapshot.category(&id);
                TopCategory {
                    title: category
                        .map(|category| category.title.clone())
                        .unwrap_or_else(|| "Uncategorized".into()),
                    color: category
                        .map(|category| category.color.clone())
                        .unwrap_or_else(|| "#8D93A6".into()),
                    spend,
                }
            })
            .collect();
        categories.sort_by(|a, b| {
            b.spend
                .partial_cmp(&a.spend)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        categories.truncate(limit);
        categories
    }

    pub fn current_month_flow(&self) -> (f64, f64) {
        let flows = monthly_flow_converted(&self.snapshot.entries, 1, |entry| {
            self.convert_entry_amount(entry)
        });
        flows
            .last()
            .map(|flow| (flow.income, flow.expenses))
            .unwrap_or((0.0, 0.0))
    }

    pub fn persist(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.snapshot.save(&self.paths.data_file) {
            self.toast(
                ToastKind::Error,
                format!("Couldn't save your data locally.\n{error}"),
                None,
                cx,
            );
        }
        let _ = self.settings.save(&self.paths.settings_file);
    }

    pub fn schedule_sync(&mut self, cx: &mut Context<Self>) {
        if !self.settings.auto_sync {
            return;
        }
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(900))
                .await;
            this.update(cx, |app, cx| app.sync_now(false, cx)).ok();
        })
        .detach();
    }

    pub fn sync_now(&mut self, manual: bool, cx: &mut Context<Self>) {
        if self.boot != Boot::Ready {
            return;
        }
        if self.sync_in_flight {
            self.sync_again = true;
            return;
        }

        self.sync_in_flight = true;
        self.sync_state = SyncState::Syncing;
        self.last_auto_sync = Some(std::time::Instant::now());
        cx.notify();

        let api = self.api.clone();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let last = self.settings.last_sync.clone();
        let payload = self.snapshot.dirty_since(last.as_deref());
        let sync_start = now();
        let fetch_profile = !self.profile_loaded || manual;
        let generation = self.session_generation;

        cx.spawn(async move |this, cx| {
            let result = perform_sync(api, access, refresh, last, payload, fetch_profile).await;
            this.update(cx, |app, cx| {
                app.finish_sync(result, sync_start, manual, generation, cx)
            })
            .ok();
        })
        .detach();
    }

    fn finish_sync(
        &mut self,
        result: Result<(SyncResponse, Option<String>, Option<ProfileResponse>), ApiError>,
        sync_start: String,
        manual: bool,
        generation: u64,
        cx: &mut Context<Self>,
    ) {
        if generation != self.session_generation {
            return;
        }
        self.sync_in_flight = false;
        match result {
            Ok((response, access, profile)) => {
                if let Some(access) = access {
                    self.access_token = Some(access);
                }
                if let Some(profile) = profile {
                    self.settings.username = profile.username.clone();
                    let email = if !profile.email.is_empty() {
                        profile.email.clone()
                    } else {
                        self.user
                            .as_ref()
                            .map(|user| user.email.clone())
                            .unwrap_or_else(|| self.settings.email.clone())
                    };
                    self.settings.email = email.clone();
                    self.settings.user_flags = profile.flags.clone();
                    let default_currency_code =
                        crate::model::normalize_currency_code(&profile.default_currency_code);
                    self.settings.default_currency_code = default_currency_code.clone();
                    self.user = Some(crate::model::User {
                        username: profile.username,
                        email,
                        flags: profile.flags,
                        default_currency_code,
                    });
                    self.profile_loaded = true;
                }
                if let Some(code) = response.default_currency_code.clone() {
                    let normalized = crate::model::normalize_currency_code(&code);
                    self.settings.default_currency_code = normalized.clone();
                    if let Some(user) = self.user.as_mut() {
                        user.default_currency_code = normalized;
                    }
                }
                self.snapshot.apply_sync_response(response);
                self.settings.last_sync = Some(sync_start);
                self.sync_state = SyncState::Success;
                self.sync_details_open = false;
                self.persist(cx);
                if manual {
                    self.toast(
                        ToastKind::Success,
                        "Everything is in sync.".into(),
                        None,
                        cx,
                    );
                }
                if self.logout_after_sync {
                    self.complete_logout(cx);
                    return;
                }
                if manual
                    || self.snapshot.currencies.is_empty()
                    || self.snapshot.exchange_rates.is_empty()
                {
                    self.refresh_currencies_and_rates(false, cx);
                }
            }
            Err(error) => {
                if self.logout_after_sync {
                    self.logout_after_sync = false;
                    self.toast(
                        ToastKind::Error,
                        "Couldn't sync before logging out. You're still signed in.".into(),
                        None,
                        cx,
                    );
                }
                if error.status == Some(401) {
                    self.session_expired(cx);
                } else {
                    self.sync_state = SyncState::Error(error.message.clone());
                    if manual {
                        self.toast(
                            ToastKind::Error,
                            format!("Sync failed.\n{}", error.message),
                            Some(ToastAction::RetrySync),
                            cx,
                        );
                    }
                }
            }
        }

        let again = self.sync_again;
        self.sync_again = false;
        cx.notify();
        if again {
            self.sync_now(false, cx);
        }
    }

    fn start_auto_sync(&mut self, cx: &mut Context<Self>) {
        if self.auto_sync_started {
            return;
        }
        self.auto_sync_started = true;
        self._auto_sync = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(15))
                    .await;
                let alive = this
                    .update(cx, |app, cx| {
                        if app.settings.auto_sync && app.boot == Boot::Ready {
                            let interval = Duration::from_secs(
                                app.settings.auto_sync_interval_minutes.max(1) as u64 * 60,
                            );
                            let due = app
                                .last_auto_sync
                                .is_none_or(|last| last.elapsed() >= interval);
                            if due {
                                app.last_auto_sync = Some(std::time::Instant::now());
                                app.sync_now(false, cx);
                            }
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        }));
    }

    pub fn toast(
        &mut self,
        kind: ToastKind,
        message: String,
        action: Option<ToastAction>,
        cx: &mut Context<Self>,
    ) {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toasts.push(Toast {
            id,
            kind,
            message,
            action,
        });
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(6)).await;
            this.update(cx, |app, cx| {
                app.toasts.retain(|toast| toast.id != id);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn dismiss_toast(&mut self, id: u64, cx: &mut Context<Self>) {
        self.toasts.retain(|toast| toast.id != id);
        cx.notify();
    }

    pub fn run_toast_action(
        &mut self,
        action: ToastAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            ToastAction::UndoEntry { id, previous } => {
                if let Some(previous) = previous {
                    let mut restored = previous;
                    restored.deleted_at = None;
                    restored.updated_at = now();
                    self.snapshot.upsert_entry(restored);
                } else if let Some(entry) = self.snapshot.entry(&id).cloned() {
                    let mut entry = entry;
                    entry.deleted_at = Some(now());
                    entry.updated_at = now();
                    self.snapshot.upsert_entry(entry);
                }
                self.persist(cx);
                self.schedule_sync(cx);
            }
            ToastAction::UndoAccount {
                mut account,
                created,
            } => {
                account.updated_at = now();
                if created {
                    account.deleted_at = Some(now());
                } else {
                    account.deleted_at = None;
                }
                self.snapshot.upsert_account(account);
                self.persist(cx);
                self.schedule_sync(cx);
            }
            ToastAction::UndoCategory {
                mut category,
                created,
            } => {
                category.updated_at = now();
                if created {
                    category.deleted_at = Some(now());
                } else {
                    category.deleted_at = None;
                }
                self.snapshot.upsert_category(category);
                self.persist(cx);
                self.schedule_sync(cx);
            }
            ToastAction::RetrySync => {
                self.sync_now(true, cx);
            }
        }
        cx.notify();
    }

    pub fn clear_local_data(&mut self, cx: &mut Context<Self>) {
        self.snapshot = Snapshot::default();
        self.settings.last_sync = None;
        self.persist(cx);
        self.sync_state = SyncState::Idle;
        self.toast(
            ToastKind::Info,
            "Local data cleared. Pulling a fresh copy from the server.".into(),
            None,
            cx,
        );
        self.sync_now(false, cx);
    }

    pub fn close_top_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut modal_closed = true;
        if self.calendar.is_some() {
            self.calendar = None;
        } else if self.show_shortcuts {
            self.show_shortcuts = false;
        } else if self.logout_dialog {
            self.logout_dialog = false;
        } else if self.confirm.is_some() {
            self.confirm = None;
        } else if self.admin_currency_editor.is_some() {
            self.admin_currency_editor = None;
        } else if self.admin_editor.is_some() {
            self.admin_editor = None;
        } else if self.bulk_currency_editor.is_some() {
            self.bulk_currency_editor = None;
        } else if self.account_editor.is_some() {
            self.account_editor = None;
        } else if self.category_editor.is_some() {
            self.category_editor = None;
        } else if self.editor.is_some() {
            self.editor = None;
        } else if self.settings_currency_open {
            self.settings_currency_open = false;
            modal_closed = false;
        } else if self.show_filters {
            self.show_filters = false;
            modal_closed = false;
        } else {
            modal_closed = false;
        }

        if modal_closed {
            window.blur();
        } else if self.confirm.is_none()
            && self.editor.is_none()
            && self.account_editor.is_none()
            && self.category_editor.is_none()
            && self.bulk_currency_editor.is_none()
            && self.admin_currency_editor.is_none()
            && (!self.search.read(cx).text().is_empty() || !self.search_tokens.is_empty())
        {
            self.search_tokens.clear();
            self.search_menu_open = false;
            self.search_menu_all = false;
            self.search.update(cx, |input, cx| input.clear(cx));
        }
        cx.notify();
    }

    pub fn clear_entry_filters(&mut self, cx: &mut Context<Self>) {
        self.filter = EntryFilter::default();
        self.show_filters = false;
        self.sort_menu_open = false;
        self.search_tokens.clear();
        self.search_menu_open = false;
        self.search_menu_all = false;
        self.search_suggestion = 0;
        self.search.update(cx, |input, cx| input.clear(cx));
        self.filter_min.update(cx, |input, cx| input.clear(cx));
        self.filter_max.update(cx, |input, cx| input.clear(cx));
        self.filter_start.update(cx, |input, cx| input.clear(cx));
        self.filter_end.update(cx, |input, cx| input.clear(cx));
        self.selected_entries.clear();
        cx.notify();
    }

    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.boot != Boot::Ready || self.overlay_open() {
            return;
        }
        self.screen = Screen::Entries;
        self.filter = EntryFilter::default();
        self.show_filters = false;
        let handle = self.search.read(cx).focus();
        window.focus(&handle);
        cx.notify();
    }

    pub fn confirm_action(
        &mut self,
        action: ConfirmAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.confirm = None;
        match action {
            ConfirmAction::ClearLocalData => self.clear_local_data(cx),
            ConfirmAction::DiscardUnsynced => self.discard_unsynced_changes(cx),
            ConfirmAction::DeleteEntry(id) => self.delete_entry_by_id(&id, cx),
            ConfirmAction::DeleteAccount(id) => self.delete_account_by_id(&id, cx),
            ConfirmAction::DeleteCategory(id) => self.delete_category_by_id(&id, cx),
            ConfirmAction::DeleteEntries(ids) => self.delete_entries_by_ids(&ids, cx),
            ConfirmAction::DeleteAccounts(ids) => self.delete_accounts_by_ids(&ids, cx),
            ConfirmAction::DeleteCategories(ids) => self.delete_categories_by_ids(&ids, cx),
            ConfirmAction::AdminDeleteUser(id) => self.admin_delete_user(&id, cx),
            ConfirmAction::AdminDeleteInvite(id) => self.admin_delete_invite(&id, cx),
        }
        cx.notify();
    }

    pub fn soft_delete_entry(&mut self, id: &str) -> Option<Entry> {
        let mut entry = self.snapshot.entry(id)?.clone();
        let previous = entry.clone();
        entry.deleted_at = Some(now());
        entry.updated_at = now();
        self.snapshot.upsert_entry(entry);
        Some(previous)
    }

    pub fn delete_entry_by_id(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(previous) = self.soft_delete_entry(id) else {
            return;
        };
        if let Some(DetailView::Entry(detail_id)) = &self.detail
            && detail_id == id
        {
            // The web client returns to the entry's account after deleting it.
            if previous.account_id.is_empty() {
                self.detail = None;
            } else {
                self.screen = Screen::Accounts;
                self.detail = Some(DetailView::Account(previous.account_id.clone()));
            }
        }
        self.selected_entries.remove(id);
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            "Entry deleted.".into(),
            Some(ToastAction::UndoEntry {
                id: id.to_string(),
                previous: Some(previous),
            }),
            cx,
        );
    }

    pub fn delete_entries_by_ids(&mut self, ids: &[String], cx: &mut Context<Self>) {
        let mut deleted = 0;
        for id in ids {
            if self.soft_delete_entry(id).is_some() {
                deleted += 1;
            }
        }
        if deleted == 0 {
            return;
        }
        self.selected_entries.clear();
        if let Some(DetailView::Entry(id)) = &self.detail
            && ids.contains(id)
        {
            self.detail = None;
        }
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            format!("{} deleted.", format::plural(deleted, "entry", "entries")),
            None,
            cx,
        );
    }

    pub fn soft_delete_account(&mut self, id: &str) -> Option<Account> {
        let mut account = self.snapshot.account(id)?.clone();
        let previous = account.clone();
        account.deleted_at = Some(now());
        account.updated_at = now();
        self.snapshot.upsert_account(account);
        Some(previous)
    }

    pub fn delete_account_by_id(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(previous) = self.soft_delete_account(id) else {
            return;
        };
        if let Some(DetailView::Account(detail_id)) = &self.detail
            && detail_id == id
        {
            self.detail = None;
        }
        self.selected_accounts.remove(id);
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            "Account deleted. Its entries are still in your ledger.".into(),
            Some(ToastAction::UndoAccount {
                account: previous,
                created: false,
            }),
            cx,
        );
    }

    pub fn delete_accounts_by_ids(&mut self, ids: &[String], cx: &mut Context<Self>) {
        let mut deleted = 0;
        for id in ids {
            if self.soft_delete_account(id).is_some() {
                deleted += 1;
            }
        }
        if deleted == 0 {
            return;
        }
        self.selected_accounts.clear();
        if let Some(DetailView::Account(id)) = &self.detail
            && ids.contains(id)
        {
            self.detail = None;
        }
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            format!(
                "{} deleted.",
                format::plural(deleted, "account", "accounts")
            ),
            None,
            cx,
        );
    }

    pub fn soft_delete_category(&mut self, id: &str) -> Option<Category> {
        let mut category = self.snapshot.category(id)?.clone();
        let previous = category.clone();
        category.deleted_at = Some(now());
        category.updated_at = now();
        self.snapshot.upsert_category(category);
        Some(previous)
    }

    pub fn delete_category_by_id(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(previous) = self.soft_delete_category(id) else {
            return;
        };
        if let Some(DetailView::Category(detail_id)) = &self.detail
            && detail_id == id
        {
            self.detail = None;
        }
        self.selected_categories.remove(id);
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            "Category deleted. Its entries keep their history.".into(),
            Some(ToastAction::UndoCategory {
                category: previous,
                created: false,
            }),
            cx,
        );
    }

    pub fn delete_categories_by_ids(&mut self, ids: &[String], cx: &mut Context<Self>) {
        let blocked = ids
            .iter()
            .any(|id| self.snapshot.category_entry_count(id) > 0);
        if blocked {
            self.toast(
                ToastKind::Error,
                "Those categories still have active entries. Move or delete the entries first."
                    .into(),
                None,
                cx,
            );
            return;
        }
        let mut deleted = 0;
        for id in ids {
            if self.soft_delete_category(id).is_some() {
                deleted += 1;
            }
        }
        if deleted == 0 {
            return;
        }
        self.selected_categories.clear();
        if let Some(DetailView::Category(id)) = &self.detail
            && ids.contains(id)
        {
            self.detail = None;
        }
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            format!(
                "{} deleted.",
                format::plural(deleted, "category", "categories")
            ),
            None,
            cx,
        );
    }

    pub fn open_account_detail(&mut self, id: &str, cx: &mut Context<Self>) {
        self.seed_history();
        self.page_generation += 1;
        self.screen = Screen::Accounts;
        self.detail = Some(DetailView::Account(id.to_string()));
        self.selected_accounts.clear();
        self.record_history();
        cx.notify();
    }

    pub fn open_category_detail(&mut self, id: &str, cx: &mut Context<Self>) {
        self.seed_history();
        self.page_generation += 1;
        self.screen = Screen::Categories;
        self.detail = Some(DetailView::Category(id.to_string()));
        self.selected_categories.clear();
        self.record_history();
        cx.notify();
    }

    pub fn open_entry_detail(&mut self, id: &str, cx: &mut Context<Self>) {
        self.seed_history();
        self.page_generation += 1;
        self.screen = Screen::Entries;
        self.detail = Some(DetailView::Entry(id.to_string()));
        self.selected_entries.clear();
        self.record_history();
        cx.notify();
    }

    pub fn close_detail(&mut self, cx: &mut Context<Self>) {
        self.seed_history();
        if self.history_index > 0 {
            self.history_index -= 1;
            let entry = self.history[self.history_index].clone();
            self.apply_history_entry(entry, cx);
            return;
        }
        self.page_generation += 1;
        self.detail = None;
        cx.notify();
    }

    pub fn toggle_selected_entry(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.selected_entries.insert(id.to_string()) {
            self.selected_entries.remove(id);
        }
        cx.notify();
    }

    pub fn toggle_selected_account(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.selected_accounts.insert(id.to_string()) {
            self.selected_accounts.remove(id);
        }
        cx.notify();
    }

    pub fn toggle_selected_category(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.selected_categories.insert(id.to_string()) {
            self.selected_categories.remove(id);
        }
        cx.notify();
    }

    pub fn toggle_all_visible_entries(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<String> = self
            .visible_entries(cx)
            .into_iter()
            .map(|entry| entry.id.clone())
            .collect();
        if ids.iter().all(|id| self.selected_entries.contains(id)) {
            for id in ids {
                self.selected_entries.remove(&id);
            }
        } else {
            self.selected_entries.extend(ids);
        }
        cx.notify();
    }

    pub fn toggle_all_accounts(&mut self, cx: &mut Context<Self>) {
        let query = self.account_search.read(cx).text().trim().to_lowercase();
        let ids: Vec<String> = self
            .snapshot
            .active_accounts()
            .into_iter()
            .filter(|account| query.is_empty() || account.name.to_lowercase().contains(&query))
            .map(|account| account.id.clone())
            .collect();
        if ids.iter().all(|id| self.selected_accounts.contains(id)) {
            self.selected_accounts.clear();
        } else {
            self.selected_accounts.extend(ids);
        }
        cx.notify();
    }

    pub fn toggle_all_categories(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<String> = self
            .visible_categories(cx)
            .into_iter()
            .map(|category| category.id.clone())
            .collect();
        if ids.iter().all(|id| self.selected_categories.contains(id)) {
            self.selected_categories.clear();
        } else {
            self.selected_categories.extend(ids);
        }
        cx.notify();
    }

    pub fn request_delete_entry(&mut self, id: &str, cx: &mut Context<Self>) {
        let title = self
            .snapshot
            .entry(id)
            .map(|entry| entry.title.clone())
            .unwrap_or_else(|| "this entry".into());
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete entry?".into(),
            message: format!("Delete “{title}”? You'll get a moment to undo it afterwards."),
            confirm_label: "Delete entry".into(),
            action: ConfirmAction::DeleteEntry(id.to_string()),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn request_delete_account(&mut self, id: &str, cx: &mut Context<Self>) {
        let name = self.snapshot.account_name(id);
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete account?".into(),
            message: format!(
                "Delete “{name}”? Its entries stay in the ledger and keep contributing to totals."
            ),
            confirm_label: "Delete account".into(),
            action: ConfirmAction::DeleteAccount(id.to_string()),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn request_delete_category(&mut self, id: &str, cx: &mut Context<Self>) {
        let count = self.snapshot.category_entry_count(id);
        if count > 0 {
            self.toast(
                ToastKind::Error,
                format!(
                    "This category still has {}. Move or delete those entries first.",
                    format::plural(count, "active entry", "active entries")
                ),
                None,
                cx,
            );
            return;
        }
        let title = self.snapshot.category_name(id);
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete category?".into(),
            message: format!("Delete “{title}”? Entries keep their history."),
            confirm_label: "Delete category".into(),
            action: ConfirmAction::DeleteCategory(id.to_string()),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn request_delete_selected_entries(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<String> = self.selected_entries.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete entries?".into(),
            message: format!(
                "Delete {}? Sunder keeps them as tombstones so the change syncs everywhere.",
                format::plural(ids.len(), "entry", "entries")
            ),
            confirm_label: "Delete entries".into(),
            action: ConfirmAction::DeleteEntries(ids),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn request_delete_selected_accounts(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<String> = self.selected_accounts.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete accounts?".into(),
            message: format!(
                "Delete {}? Their entries stay in the ledger and keep contributing to totals.",
                format::plural(ids.len(), "account", "accounts")
            ),
            confirm_label: "Delete accounts".into(),
            action: ConfirmAction::DeleteAccounts(ids),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    pub fn request_delete_selected_categories(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<String> = self.selected_categories.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        if ids
            .iter()
            .any(|id| self.snapshot.category_entry_count(id) > 0)
        {
            self.toast(
                ToastKind::Error,
                "Some selected categories still have entries. Move or delete those entries first."
                    .into(),
                None,
                cx,
            );
            return;
        }
        self.modal_seq += 1;
        self.confirm = Some(ConfirmDialog {
            title: "Delete categories?".into(),
            message: format!(
                "Delete {}? Entries keep their history.",
                format::plural(ids.len(), "category", "categories")
            ),
            confirm_label: "Delete categories".into(),
            action: ConfirmAction::DeleteCategories(ids),
            seq: self.modal_seq,
        });
        cx.notify();
    }

    // Action plumbing. Keeping the handlers here means the keymap and the
    // visible controls always go through the same code path.
    pub fn action_new_entry(&mut self, _: &NewEntry, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_input_focused(window, cx) {
            return;
        }
        if self.boot == Boot::Ready && !self.overlay_open() {
            self.open_entry_editor(None, None, None, window, cx);
        }
    }

    pub fn action_new_account(
        &mut self,
        _: &NewAccount,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        if self.boot == Boot::Ready && !self.overlay_open() {
            self.open_account_editor(None, window, cx);
        }
    }

    pub fn action_new_category(
        &mut self,
        _: &NewCategory,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        if self.boot == Boot::Ready && !self.overlay_open() {
            self.open_category_editor(None, window, cx);
        }
    }

    pub fn action_sync(&mut self, _: &SyncNow, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.sync_now(true, cx);
    }

    pub fn action_dashboard(
        &mut self,
        _: &ShowDashboard,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.go(Screen::Dashboard, window, cx);
    }

    pub fn action_entries(&mut self, _: &ShowEntries, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.go(Screen::Entries, window, cx);
    }

    pub fn action_accounts(
        &mut self,
        _: &ShowAccounts,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.go(Screen::Accounts, window, cx);
    }

    pub fn action_categories(
        &mut self,
        _: &ShowCategories,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.go(Screen::Categories, window, cx);
    }

    pub fn action_settings(
        &mut self,
        _: &ShowSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.go(Screen::Settings, window, cx);
    }

    pub fn action_focus_search(
        &mut self,
        _: &FocusSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.focus_search(window, cx);
    }

    pub fn action_save_overlay(
        &mut self,
        _: &SaveOverlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.admin_currency_editor.is_some() {
            self.save_admin_currency(window, cx);
        } else if self.admin_editor.is_some() {
            self.save_admin_user(window, cx);
        } else if self.bulk_currency_editor.is_some() {
            self.apply_bulk_currency_change(window, cx);
        } else if self.account_editor.is_some() {
            self.save_account(window, cx);
        } else if self.category_editor.is_some() {
            self.save_category(window, cx);
        } else if self.editor.is_some() {
            self.save_entry(window, cx);
        }
    }

    pub fn action_close_overlay(
        &mut self,
        _: &CloseOverlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Mirrors the web Modal: pressing Escape inside a dirty form first
        // releases focus instead of throwing the draft away.
        if self.top_overlay_dirty(cx) && self.text_input_focused(window, cx) {
            let root_focus = self.root_focus.clone();
            window.focus(&root_focus);
            return;
        }
        self.close_top_overlay(window, cx);
    }

    /// Whether the topmost create/edit form has unsaved draft content.
    pub fn top_overlay_dirty(&self, cx: &App) -> bool {
        // The web guard applies to the create modals. Edit forms open with
        // content already present, so treat them as non-drafts. When an
        // account/category editor is stacked above the entry editor, it is
        // the one Escape should consider first.
        if let Some(editor) = &self.account_editor {
            if editor.editing.is_some() {
                return false;
            }
            return !editor.name.read(cx).text().trim().is_empty();
        }
        if let Some(editor) = &self.category_editor {
            if editor.editing.is_some() {
                return false;
            }
            return !editor.title.read(cx).text().trim().is_empty()
                || !editor.description.read(cx).text().trim().is_empty()
                || !editor.color.eq_ignore_ascii_case("#6366f1");
        }
        if let Some(editor) = &self.editor {
            if editor.editing.is_some() {
                return false;
            }
            return !editor.amount.read(cx).text().trim().is_empty()
                || !editor.title.read(cx).text().trim().is_empty()
                || !editor.description.read(cx).text().trim().is_empty()
                || !editor.location.read(cx).text().trim().is_empty();
        }
        false
    }

    pub fn action_toggle_theme(
        &mut self,
        _: &ToggleTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.toggle_theme(cx);
    }

    pub fn action_show_shortcuts(
        &mut self,
        _: &ShowShortcuts,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_focused(window, cx) {
            return;
        }
        self.show_shortcuts = !self.show_shortcuts;
        cx.notify();
    }

    fn render_overlays(&self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        crate::ui::overlays::render_overlays(self, window, cx)
    }
}

impl Render for SunderApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(debug_assertions)]
        self.maybe_open_debug_overlay(window, cx);
        let palette: Theme = theme(cx).clone();
        let body = crate::ui::screens::render_body(self, window, cx);

        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(palette.bg)
            .text_color(palette.foreground)
            .font_family(palette.font.clone())
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::action_new_entry))
            .on_action(cx.listener(Self::action_new_account))
            .on_action(cx.listener(Self::action_new_category))
            .on_action(cx.listener(Self::action_show_shortcuts))
            .on_action(cx.listener(Self::action_sync))
            .on_action(cx.listener(Self::action_dashboard))
            .on_action(cx.listener(Self::action_entries))
            .on_action(cx.listener(Self::action_accounts))
            .on_action(cx.listener(Self::action_categories))
            .on_action(cx.listener(Self::action_settings))
            .on_action(cx.listener(Self::action_focus_search))
            .on_action(cx.listener(Self::action_save_overlay))
            .on_action(cx.listener(Self::action_close_overlay))
            .on_action(cx.listener(Self::action_toggle_theme))
            .on_action(cx.listener(Self::action_search_open))
            .on_action(cx.listener(Self::action_search_complete))
            .on_action(cx.listener(Self::action_search_next))
            .on_action(cx.listener(Self::action_search_previous))
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Back),
                cx.listener(|this, _, window, cx| this.mouse_back(window, cx)),
            )
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Forward),
                cx.listener(|this, _, _, cx| this.mouse_forward(cx)),
            )
            .child(body)
            .children(self.render_overlays(window, cx))
    }
}

async fn login_and_load(
    api: ApiClient,
    login: String,
    password: String,
) -> Result<(String, User, String), ApiError> {
    let response: LoginResponse = api.login(login, password).await?;
    if response.refresh_token.is_empty() {
        return Err(ApiError {
            message: "The server didn't return a refresh token. Check the backend configuration."
                .into(),
            status: Some(500),
        });
    }
    let refresh_token = response.refresh_token;
    let refreshed = api.refresh(refresh_token.clone()).await?;
    let access = refreshed.access_token;
    let profile = api.profile(access.clone()).await?;
    Ok((access, profile.into_user(String::new()), refresh_token))
}

pub fn select_calendar_range(
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    clicked: NaiveDate,
) -> (Option<NaiveDate>, Option<NaiveDate>) {
    match (start, end) {
        (Some(start), Some(end)) => {
            if clicked == start {
                (Some(end), None)
            } else if clicked == end {
                (Some(start), None)
            } else {
                (Some(clicked), None)
            }
        }
        (None, _) => (Some(clicked), None),
        (Some(start), None) => {
            if clicked < start {
                (Some(clicked), Some(start))
            } else {
                (Some(start), Some(clicked))
            }
        }
    }
}

/// Mirrors JavaScript's `Date#setMonth` behavior: subtract calendar months and
/// let an overflowing day roll into the following month.
pub fn months_before(date: NaiveDate, months: u32) -> NaiveDate {
    let total = date.year() * 12 + date.month0() as i32 - months as i32;
    let year = total.div_euclid(12);
    let month = total.rem_euclid(12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1)
        .expect("valid month")
        .checked_add_days(chrono::Days::new(date.day0() as u64))
        .expect("valid shifted date")
}

pub fn calendar_grid(year: i32, month: u32) -> Vec<Option<NaiveDate>> {
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return Vec::new();
    };
    let leading = first.weekday().num_days_from_sunday() as usize;
    let days = days_in_month(year, month);
    let mut grid = Vec::with_capacity(leading + days as usize);
    for _ in 0..leading {
        grid.push(None);
    }
    for day in 1..=days {
        grid.push(NaiveDate::from_ymd_opt(year, month, day));
    }
    grid
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|date| date.pred_opt())
        .map(|date| date.day())
        .unwrap_or(30)
}

fn local_owner_mismatch(settings: &Settings, incoming_name: &str, server: &str) -> bool {
    let server_changed = !settings.owner_base_url.is_empty() && settings.owner_base_url != server;
    let user_changed =
        !settings.owner.is_empty() && !incoming_name.is_empty() && settings.owner != incoming_name;
    server_changed || user_changed
}

pub(crate) type ApiFuture<T> = Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send>>;

pub(crate) async fn with_access<T>(
    api: ApiClient,
    access: Option<String>,
    refresh: Option<String>,
    operation: impl Fn(ApiClient, String) -> ApiFuture<T>,
) -> Result<(T, String), ApiError> {
    let mut token = match access {
        Some(access) => access,
        None => {
            let Some(refresh) = refresh.clone() else {
                return Err(ApiError {
                    message: "Sign in to continue.".into(),
                    status: Some(401),
                });
            };
            api.refresh(refresh)
                .await
                .map_err(session_expired)?
                .access_token
        }
    };

    match operation(api.clone(), token.clone()).await {
        Ok(value) => Ok((value, token)),
        Err(error) if error.status == Some(401) => {
            let Some(refresh) = refresh else {
                return Err(error);
            };
            token = api
                .refresh(refresh)
                .await
                .map_err(session_expired)?
                .access_token;
            let value = operation(api, token.clone()).await?;
            Ok((value, token))
        }
        Err(error) => Err(error),
    }
}

fn session_expired(error: ApiError) -> ApiError {
    if matches!(error.status, Some(400 | 401)) {
        ApiError {
            message: "Your session expired. Please sign in again.".into(),
            status: Some(401),
        }
    } else {
        error
    }
}

async fn perform_sync(
    api: ApiClient,
    access: Option<String>,
    refresh: Option<String>,
    last: Option<String>,
    payload: SyncPayload,
    fetch_profile: bool,
) -> Result<(SyncResponse, Option<String>, Option<ProfileResponse>), ApiError> {
    let mut access = access;
    if access.is_none() {
        let Some(refresh) = refresh.clone() else {
            return Err(ApiError {
                message: "Sign in to sync with the server.".into(),
                status: Some(401),
            });
        };
        let refreshed = api.refresh(refresh).await.map_err(session_expired)?;
        access = Some(refreshed.access_token);
    }

    let token = access.clone().unwrap_or_default();
    let mut result = api.sync(token, last.clone(), payload.clone()).await;

    if let (Err(error), Some(refresh)) = (&result, refresh)
        && error.status == Some(401)
    {
        let refreshed = api.refresh(refresh).await.map_err(session_expired)?;
        let token = refreshed.access_token;
        result = api.sync(token.clone(), last, payload).await;
        return finish_sync_result(api, result, Some(token), fetch_profile).await;
    }

    finish_sync_result(api, result, access, fetch_profile).await
}

async fn finish_sync_result(
    api: ApiClient,
    result: Result<SyncResponse, ApiError>,
    access: Option<String>,
    fetch_profile: bool,
) -> Result<(SyncResponse, Option<String>, Option<ProfileResponse>), ApiError> {
    let response = result?;
    let profile = match (&access, fetch_profile) {
        (Some(access), true) => api.profile(access.clone()).await.ok(),
        _ => None,
    };
    Ok((response, access, profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_grid_and_range_selection_match_the_web_picker() {
        let grid = calendar_grid(2026, 11);
        assert_eq!(grid.len(), 30); // Nov 1, 2026 is a Sunday
        assert_eq!(grid[0], Some(NaiveDate::from_ymd_opt(2026, 11, 1).unwrap()));
        assert_eq!(
            grid.last(),
            Some(&Some(NaiveDate::from_ymd_opt(2026, 11, 30).unwrap()))
        );

        let a = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        let b = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap();
        let c = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        assert_eq!(select_calendar_range(None, None, a), (Some(a), None));
        assert_eq!(select_calendar_range(Some(a), None, b), (Some(a), Some(b)));
        assert_eq!(select_calendar_range(Some(b), None, a), (Some(a), Some(b)));
        assert_eq!(select_calendar_range(Some(a), Some(b), c), (Some(c), None));
        assert_eq!(select_calendar_range(Some(a), Some(b), a), (Some(b), None));
        assert_eq!(select_calendar_range(Some(a), Some(b), b), (Some(a), None));

        let jan_31 = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        assert_eq!(
            months_before(jan_31, 1),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
        assert_eq!(
            months_before(jan_31, 12),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()
        );
        // Jan 31 minus three months is Oct 31, not 90 days.
        assert_eq!(
            months_before(jan_31, 3),
            NaiveDate::from_ymd_opt(2025, 10, 31).unwrap()
        );
        // Day 31 rolls into March when February is shorter, like JS setMonth.
        assert_eq!(
            months_before(NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(), 1),
            NaiveDate::from_ymd_opt(2026, 3, 3).unwrap()
        );
    }

    #[test]
    fn entry_filter_chips_toggle_multi_selection() {
        let mut filter = EntryFilter::default();
        assert!(filter.descending);
        filter.toggle_account("a1");
        filter.toggle_account("a2");
        assert_eq!(filter.account_ids, vec!["a1", "a2"]);
        filter.toggle_account("a1");
        assert_eq!(filter.account_ids, vec!["a2"]);
        filter.toggle_category("c1");
        assert_eq!(filter.category_ids, vec!["c1"]);
    }

    #[test]
    fn local_data_is_only_reused_for_the_same_account_and_server() {
        let settings = Settings {
            owner: "alice".into(),
            owner_base_url: "https://sunder.example.com".into(),
            ..Settings::default()
        };

        assert!(!local_owner_mismatch(
            &settings,
            "alice",
            "https://sunder.example.com"
        ));
        assert!(local_owner_mismatch(
            &settings,
            "bob",
            "https://sunder.example.com"
        ));
        assert!(local_owner_mismatch(
            &settings,
            "alice",
            "https://other.example.com"
        ));

        let fresh = Settings::default();
        assert!(!local_owner_mismatch(
            &fresh,
            "alice",
            "http://localhost:4000"
        ));
        let fresh = Settings {
            owner: "alice".into(),
            ..Settings::default()
        };
        assert!(!local_owner_mismatch(&fresh, "", "http://localhost:4000"));
    }

    #[test]
    fn refresh_failures_are_translated_into_session_expiry() {
        let expired = session_expired(ApiError {
            message: "REFRESH_TOKEN_EXPIRED".into(),
            status: Some(400),
        });
        assert_eq!(expired.status, Some(401));

        let offline = session_expired(ApiError {
            message: "network".into(),
            status: None,
        });
        assert_eq!(offline.status, None);
        assert_eq!(offline.message, "network");
    }
}
