# Sunder Desktop

A native desktop client for Sunder, built with [GPUI](https://gpui.rs) (the Rust UI
framework behind Zed). It talks to the same Phoenix API as the web and Android apps,
but keeps a complete local copy of your ledger so the app opens instantly and works
offline.

```
┌───────────────┐        ┌────────────────────┐        ┌──────────────────┐
│  GPUI views   │ ─────▶ │  local JSON ledger │ ─────▶ │  POST /dashboard │
│  (src/ui)     │ ◀───── │  (atomic, 0600)    │ ◀───── │      /eco/sync   │
└───────────────┘        └────────────────────┘        └──────────────────┘
```

A route-by-route parity checklist is in [FEATURE_PARITY.md](FEATURE_PARITY.md).

## What's inside

- **Dashboard** — net balance, total income and expenses with monthly context, a
  six month cash-flow chart, top categories, recent activity, and quick add actions.
- **Entries** — date-grouped history with power search (`title:`, `note:`, `account:`,
  `category:`, `amount:>50`, `type:income`, `date:`, `before:`/`after:`). Typing a
  key opens a Tab-completable autocomplete (or press Ctrl+Space on an empty
  search); `title:`, `location:`, `category:` and `account:` suggest real values,
  and completed terms become removable chips inside the focused search box;
  Backspace on an empty field removes the last chip. Also
  includes an always-visible sort menu (newest/oldest, amount high/low, title A–Z/Z–A),
  multi-account and multi-category filters, amount and date ranges with a full
  calendar popover, select-all, bulk delete, and dedicated entry detail pages.
- **Accounts** — search, balances, selection and bulk delete, plus account detail
  pages with transaction history, add-entry, rename and delete.
- **Categories** — a compact table with search, item-count filters (all/empty/1–4/5–9/10+),
  selection and bulk delete, inline edit/delete actions, custom colors, spending totals,
  and category detail pages with guarded deletion and add-entry.
- **Admin** — admin-only user management (edit username/email/flags/password, delete)
  and invite code management (list, create, delete).
- **Settings** — light/dark theme, custom currency symbol with before/after number
  formatting, full/reduced motion, auto-sync with 1/5/15/60 minute intervals,
  pending-change inspection, export to Sunder JSON, import Sunder JSON or Bagels
  SQLite, unsynced-logout protection, keyboard reference, and sign out.
- **Motion** — a small zeron-inspired catalog of custom cubic-bezier transitions for
  page and tab changes, modal/backdrop entrances, the calendar popover, toasts, the
  sync panel, and table rows, plus press feedback on buttons. Reduced motion is a
  persisted preference.
- **Auth** — login is the default screen; invite registration, automatic token
  refresh, and an offline mode that opens your cached ledger when the server can't be
  reached.
- **Sync** — incremental, tombstone-aware, client-wins conflict resolution (matching
  the existing web and Android clients), automatic retry, a persistent sync-status
  bar with persisted minimize/expand, an inline expandable pending-changes panel,
  links to each pending record, and discard-all support.
- **Tooltips** — key actions carry styled tooltips with labels and keyboard hints,
  matching the web client's tooltip treatment.
- **Keyboard** — text fields support Ctrl/Alt+Backspace and Ctrl/Alt+Delete to remove
  whole words, alongside the usual selection, clipboard and Enter handling.
- **Navigation** — the mouse back/forward buttons walk screen and detail history; back
  closes the topmost dialog or filter panel first.
- **Editors** — the entry form's category control is a dropdown with color dots, the
  full category list and inline “New category”; entry notes and category descriptions
  are true wrapping multiline fields;
  Escape on a dirty form releases focus before it can close; the category color
  picker offers presets plus any custom `#rrggbb` value.

## Running it

Requires a recent Rust toolchain (edition 2024; Rust 1.88 or newer recommended).

```sh
cd desktop-app
cargo run
```

On Linux GPUI needs the usual Wayland/X11 development libraries:

```sh
# Debian/Ubuntu
sudo apt install build-essential clang cmake pkg-config \
  libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
  libfontconfig1-dev libfreetype6-dev libx11-dev libx11-xcb-dev \
  libxcb1-dev libxcb-composite0-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev
```

The app bundles Inter Tight, so typography is consistent across machines. On first
launch, point the **Server** field at your Sunder backend (default
`http://localhost:4000`, or set `SUNDER_API_URL` before the first run).

Run the checks/tests:

```sh
cargo check
cargo test
cargo clippy --all-targets
```

`cargo test` includes interaction tests that click through the real widgets with
GPUI's test platform (sync-bar minimize/expand, password reveal, calendar pick plus
month/year drill-down and quick-pick ranges, multiline note, tooltip hover, shortcut
suppression, dirty-form Escape, inline account/category creation, entry save plus
detail navigation, account-create and entry-delete navigation, dashboard quick add,
pending-change navigation, theme persistence, category creation and item-count
filtering, entries search, bulk delete, unsynced-logout confirmation, account/category
edit-delete, amount/date filters and sorting, calendar navigation/Today, entry
validation and income toggle, currency formatting, motion preference, soft-wrap
layout, and persisted line breaks).

Debug builds accept smoke-test overrides for the starting view:
`SUNDER_START_SCREEN=entries|accounts|categories|settings|admin`,
`SUNDER_START_DETAIL=account:<id>|category:<id>|entry:<id>`, and
`SUNDER_START_OVERLAY=entry|account|category|admin|shortcuts|logout|confirm|calendar|filter-calendar`.
They make it easy to exercise every screen and overlay against a development or
mock backend without clicking through. A fresh data directory opens the login screen.

## Local data

| What | Where |
| --- | --- |
| Ledger snapshot | `~/.local/share/sunder/data.json` |
| Settings and refresh token | `~/.config/sunder/settings.json` |
| Portable/test override | `SUNDER_DATA_DIR=/some/dir` |

Both files are written atomically and permissions are restricted to the current user
on Unix (`0600` files, `0700` directories). The refresh token never leaves the
settings file except as a request body to `/auth/refresh_token`.

The snapshot is keyed to the server and account that created it. Signing into a
different account, or switching servers, clears the cached ledger before the first
sync so one account's data can never be uploaded to another.

## Keyboard shortcuts

`⌘` means `Cmd` on macOS and `Ctrl` on Linux/Windows.

| Shortcut | Action |
| --- | --- |
| `⌘N` | New entry |
| `⌘R` | Sync now |
| `⌘F` | Search entries |
| `⌘1` … `⌘5` | Dashboard / Entries / Accounts / Categories / Settings |
| `⌘↵` | Save the open editor |
| `Esc` | Close the open editor, filter panel, or search |
| `⌘⇧T` | Toggle theme |

## Code map

```
src/
  main.rs       window setup, bundled fonts, global keybindings
  app.rs        application state, auth, sync orchestration, actions, root view
  api.rs        typed HTTP client (login, register, refresh, profile, sync)
  model.rs      Account / Category / Entry / User, timestamps, monthly buckets
  store.rs      settings + snapshot persistence, dirty tracking, merge rules
  theme.rs      light/dark palette and type scale
  format.rs     money/date formatting and parsing
  assets.rs     embedded SVG icon set + logo
  ui/
    input.rs    full text input (selection, clipboard, IME, masking)
    components.rs  buttons, cards, chips, badges, empty states, spinner
    screens.rs  login, shell, dashboard, entries, accounts, categories, settings
    overlays.rs entry/account/category editors, confirmations, toasts
```

### Sync notes

The client mirrors the wire format used by the other Sunder clients:

1. Send every record whose `updated_at` is newer than the last successful sync.
   Deletions are tombstones (`deleted_at`) and are sent too.
2. The server resolves conflicts client-wins, writes the batch, and returns all
   records changed since the timestamp that were not part of the request.
3. Merge without clobbering a local edit newer than the server row.
4. Store the sync *start* time as the next cursor, so edits that race the request
   are re-fetched rather than skipped.

The backend uses naive UTC timestamps on the wire, so the desktop client does too.
No timezone conversions happen for ledger dates; a date picked as `2026-11-12` is
stored at noon UTC to stay on the same calendar day.

## Design notes

The UI follows the existing Sunder visual language (cool indigo, mint income, coral
expenses, tinted neutrals) with native desktop density.

- Press feedback is immediate (`:active`-style background changes, no animation on
  keyboard actions); pages rise 8px over 220ms, and modals rise 14px over 230ms from
  a fading backdrop. Tab, calendar-view and auth-mode changes fade through with a
  small offset.
- Modals are true modal surfaces: the backdrop fades in and occludes everything
  beneath it, so clicks, hover cursors and scroll wheels cannot reach the page while
  a dialog is open. Clicking the backdrop closes only the dialog.
- Popovers and menus are not used for selection — account and category choices are
  visible chips, which keeps the form direct and avoids hidden state.
- Modals are centered (they are not anchored to a trigger); text fields keep a
  visible focus border, and destructive actions are forgiving: entry/account/category
  deletes and edits can be undone from the toast.
- The layout is intentionally scroll-first rather than accordion-heavy; the sidebar
  status is always visible and clicking it syncs.

GPUI 0.2 does not expose `prefers-reduced-motion`; the only non-essential motion is
the 160–180 ms entrance fade, so the reduced-motion experience remains effectively
the same. Inputs support selection, clipboard, and IME composition.

## Notes and limitations

- The local store is a JSON snapshot. That is deliberate for a v1: it is easy to back
  up and inspect, and Sunder datasets are small. A SQLite store can be introduced
  behind `store.rs` without touching the views.
- Dates are entered as `YYYY-MM-DD` with Today/Yesterday shortcuts rather than a
  calendar popover; all date filtering and storage is fully implemented.
- Sync conflicts are resolved by the server's client-wins policy; there is no
  per-field merge UI.
- `vendor/xattr` is a one-line patch to xattr 0.2.3, whose `libc::ENOATTR` reference
  no longer compiles on Linux. It is pulled in by GPUI's GitHub download helper and is
  otherwise unused.

## License

MIT. Inter Tight is licensed under the SIL Open Font License; see
`assets/fonts/OFL.txt`.
