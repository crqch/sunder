# Web → GPUI feature parity

This map ties the web client (`web/src`) to the desktop implementation.

| Web route / component | GPUI implementation | Status |
| --- | --- | --- |
| `auth/sign-in` | `src/ui/screens.rs::login_screen`, `app.rs::submit_login` | ✅ login is the default route, server URL, Enter submit, token refresh, error states |
| `auth/sign-up` | `login_screen` register mode, `app.rs::submit_register` | ✅ invite, email, username, password, auto sign-in |
| `(app)/+layout` sidebar/nav | `screens.rs::sidebar` | ✅ sections, active state, sync status, theme, user, sign out; the logo returns to the dashboard |
| `(app)/dashboard` | `screens.rs::dashboard` | ✅ balance, total income, total expenses, monthly context, cash-flow chart, top categories, recent activity, Add Entry, Add Account, View All |
| `(app)/entries` | `screens.rs::entries`, `filter_panel`, `search.rs` | ✅ power search with Tab/Ctrl+Space autocomplete (`title:`, `note:`, `location:`, `account:`, `category:`, `amount:>50`, `type:income`, `date:`, `before:`/`after:`), values suggested from real titles, locations, categories and accounts, completed terms as in-field chips with Backspace removal and a focus outline, toolbar sort menu, multi-account/category filters, min/max amount, date range, select-all, bulk delete |
| `(app)/entries/[id]` | `screens.rs::entry_detail` | ✅ read view, account/category links, note, location, edit, delete; deleting returns to the entry's account like the web client |
| `components/EntryForm.svelte` | `overlays.rs::entry_editor_content`, `save_entry` | ✅ expense/income, amount, title, account, category dropdown (color dots, full list, New category), date, wrapping multiline note that scrolls to the cursor, location, validation, edit/create; a created entry opens its detail page |
| `(app)/accounts` | `screens.rs::accounts` | ✅ search, selection, select-all, bulk delete, balance, Add Account |
| `(app)/accounts/[id]` | `screens.rs::account_detail` | ✅ balance, transactions, Add Entry, edit, delete, entry links |
| `(app)/categories` | `screens.rs::categories` | ✅ table view, search, item-count filters (all/empty/1–4/5–9/10+), selection, select-all, bulk delete, edit/delete actions, color, description, Add Category |
| `(app)/categories/[id]` | `screens.rs::category_detail` | ✅ spending, transactions, Add Entry, edit, guarded delete |
| `components/CategoryForm.svelte` | `overlays.rs::category_editor_content` | ✅ name, wrapping multiline description that scrolls to the cursor, 15 preset colors and custom `#rrggbb` color, live preview; a created category opens its detail page |
| `(app)/settings` | `screens.rs::settings` | ✅ account info, logout, auto-sync, intervals, last sync, Sync Now, pending changes, export, import (Sunder + Bagels), custom currency symbol, before/after number formatting, motion preference, version |
| `(app)/admin` | `screens.rs::admin`, `overlays.rs::admin_editor_content` | ✅ users (edit username/email/flags/password with show/hide reveal, delete), invites (list/create/delete), admin-only gate, null-safe list responses |
| `components/SyncIndicator.svelte` | `screens.rs::sync_status_bar`, sidebar status, settings pending list | ✅ syncing/error/unsynced/success, inline expandable pending-changes panel with created/updated/deleted labels and links to each record, Sync Now, Discard/Discard all, persisted minimize/expand (`sync_bar_minimized`); the bottom bar hides while minimized and the sidebar card takes over |
| `components/KeybindsModal.svelte` | `overlays.rs::shortcuts_content` | ✅ grouped shortcuts, chord display, Alt and Cmd/Ctrl bindings |
| `lib/keybinds.svelte.ts` + `KeybindListener.svelte` | `main.rs::bind_keys`, `app.rs::text_input_focused` | ✅ Alt+N A/C/E chords, Alt+Space, Alt+1..5, Alt+E/A/C, Alt+T, Alt+?//, Cmd/Ctrl equivalents; shortcuts ignored while a text field has focus |
| `lib/tooltip.ts` / `use:tooltip` | `components.rs::TooltipView`, `SunderTooltip` trait | ✅ styled tooltip with action label + keycaps on navigation, create, sync, edit, delete, import/export, admin and sync-bar actions |
| `components/Modal.svelte` | `overlays.rs::modal_layer` | ✅ centered modal, scrim, Escape/close, focus first field, rich delete confirmations, dirty-form Escape blurs before closing |
| `components/MultiSelect.svelte` | entries filter chips | ✅ multi-account and multi-category filters |
| `components/DatePicker.svelte` | `overlays.rs::calendar_content`, `app.rs::CalendarState` | ✅ full popover: day/month/year views, month + year navigation, Sunday-first grid, single-date and range modes, hover range preview, 1M/3M/6M/1Y quick picks, Today, Clear, Done, click-outside/Escape close; entry editor and entries filter both have calendar triggers |
| `components/AccountSelect` / `CategorySelect` | account/category chips in entry editor | ✅ visible, direct selection, plus “New account” / “New category” inline creation that selects the new record |
| `lib/auth.ts` | `app.rs` auth/session flow | ✅ bearer auth, refresh, profile, session expiry |
| `lib/db.ts` local-first store | `store.rs` snapshot + `app.rs` | ✅ atomic local persistence, tombstone soft deletes |
| `lib/sync.ts` | `app.rs` sync/merge + `api.rs` | ✅ incremental sync, dirty tracking, last-sync cursor, merge protection |
| `lib/syncState` auto-sync | `app.rs::start_auto_sync`, settings | ✅ toggle, 1/5/15/60 minute interval, retry |
| `lib/themeStore.ts` | `theme.rs`, `app.rs::toggle_theme` | ✅ light/dark, persisted, Alt+T |
| Motion / transitions | `src/motion.rs` | ✅ custom cubic-bezier entrances, 140–180ms UI durations, staggered table rows, modal/calendar/toast/sync-panel entrances, button press feedback, persisted reduced-motion mode |
| Data export/import (`settings`) | `app.rs::export_data` / `import_data`, `api.rs` | ✅ native file picker, Sunder JSON export/import, Bagels import, post-import sync |
| Logout with unsynced changes | `app.rs::begin_logout`, `overlays.rs::logout_content` | ✅ Sync & logout, Discard & logout, Cancel, local cache cleared on logout |

## Verification

- `cargo test` — 90 tests: API contracts, sync merge rules, power-search parsing and suggestions, filtering primitives, persistence, calendar grid/range semantics, login boot mode, sync-bar visibility, soft-wrap and word-deletion layout, motion, and interaction tests.
- Interaction tests (`src/ui/interaction_tests.rs`) drive the real widgets through GPUI's test platform:
  - sync-bar minimize → persisted preference → sidebar expand click round-trip;
  - the sync bar expands its inline pending-changes panel, account/category/entry rows open their details, and Discard all opens the confirmation;
  - admin password reveal click toggles both the flag and the input mask;
  - clicking a calendar day writes the selected date into the entry editor;
  - month/year calendar views drill down (Days → Months → Years → Months), prev/next navigates across year boundaries, Today pins the editor date and a completed filter range, Clear empties the range, Cancel leaves the entry date untouched, and a 3M quick pick subtracts calendar months before writing the filter range on Done;
  - the entry editor creates and selects a brand-new account and category inline (matching AccountSelect/CategorySelect), then saves a new expense and navigates to its detail page; the income toggle saves a positive amount and an empty draft reports validation instead of saving;
  - creating an account opens its detail page, and deleting an entry from its detail page returns to that entry's account;
  - the dashboard “Add account” action opens the account editor;
  - clicking a pending unsynced row opens the matching detail page;
  - the sidebar theme toggle flips the global theme and persists the choice;
  - creating a category opens its detail page, the categories table filters by item count (empty/1–4/5–9/10+), and typing in the entries search narrows the visible list;
  - bulk delete removes the selected entries, accounts, and categories and leaves tombstones;
  - signing out with unsynced changes opens the confirmation dialog and Cancel keeps the session;
  - account and category detail edit/delete round trips update or tombstone the record and clear the detail view;
  - amount/date filters narrow the entry list and amount sorting reorders it;
  - currency position (before/after) and a custom symbol format amounts and persist;
  - the motion preference toggles between full and reduced motion and persists;
  - a long logical note line produces soft-wrap boundaries at the field width, and offsets inside it map to lower wrapped rows;
  - entry notes and category descriptions keep line breaks through the save path;
  - a multiline note keeps `\n` when Enter is pressed (typing `First line`, Enter, `Second line`);
  - Alt shortcuts are ignored while the search input is focused and work again from the root focus;
  - hovering a `sunder_tooltip` target renders the tooltip view after the show delay;
  - Escape on a dirty editor blurs first and only closes on the second press.
- `cargo clippy --all-targets` — no warnings (except upstream `proc-macro-error2` future-incompat note).
- `cargo build` — succeeds.
- Mock-server runs — boot, refresh, sync, profile/flags, and persistence verified; expired-session path clears the token and returns to sign-in.
- Offline smoke — every top-level screen, every detail view, and the entry/calendar/admin overlays rendered from the cached snapshot with the server stopped; local-first data stayed usable.
- Dirty-sync smoke — with pending local changes and an unreachable server, the app rendered cached entries and the offline sync bar without panicking.
- First launch opens the login screen directly; there is no marketing/landing page in the desktop app.
- Smoke-rendered `SUNDER_START_OVERLAY=calendar`, `filter-calendar`, `entry`, and `admin`, plus the categories table and settings currency/motion controls, against populated mock data; no panics or render errors.
- Smoke-rendered every top-level screen (`entries`, `accounts`, `categories`, `settings`, `admin`), every detail view (`entry`, `account`, `category`), and every overlay (`entry`, `account`, `category`, `admin`, `shortcuts`, `logout`, `confirm`) against populated mock data; no panics or render errors.
- Multipart import, byte export, admin list/update, and login/sync API contracts are covered by the mock-server tests.
