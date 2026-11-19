use crate::api::AdminUser;
use crate::app::{
    CalendarState, CalendarTarget, CalendarView, SunderApp, Toast, ToastAction, ToastKind,
    calendar_grid,
};
use crate::assets::{Icon, icon};
use crate::format;
use crate::model::{self, Account, Category, Entry, new_id, now};
use crate::theme::{Theme, parse_color, text as type_scale, theme};
use crate::ui::components::{
    BadgeKind, SunderTooltip, chip, danger_button, dot, field_label, ghost_button, icon_badge,
    icon_button, primary_button, secondary_button, shortcut,
};
use crate::ui::input::{InputStyle, TextInput};
use chrono::{Datelike, Duration};
use gpui::{
    AnyElement, BoxShadow, Div, ElementId, IntoElement, MouseButton, NavigationDirection,
    SharedString, Window, div, prelude::*, px, relative,
};

pub struct EntryEditor {
    pub editing: Option<String>,
    pub amount: gpui::Entity<TextInput>,
    pub title: gpui::Entity<TextInput>,
    pub date: gpui::Entity<TextInput>,
    pub description: gpui::Entity<TextInput>,
    pub location: gpui::Entity<TextInput>,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub category_picker_open: bool,
    pub income: bool,
    pub error: Option<String>,
    pub seq: u64,
}

pub struct AccountEditor {
    pub editing: Option<String>,
    pub name: gpui::Entity<TextInput>,
    pub error: Option<String>,
    pub seq: u64,
}

pub struct AdminEditor {
    pub editing: String,
    pub username: gpui::Entity<TextInput>,
    pub email: gpui::Entity<TextInput>,
    pub flags: gpui::Entity<TextInput>,
    pub password: gpui::Entity<TextInput>,
    pub password_masked: bool,
    pub error: Option<String>,
    pub seq: u64,
}

pub struct CategoryEditor {
    pub editing: Option<String>,
    pub title: gpui::Entity<TextInput>,
    pub description: gpui::Entity<TextInput>,
    pub color_input: gpui::Entity<TextInput>,
    pub color: String,
    pub error: Option<String>,
    pub seq: u64,
}

impl SunderApp {
    pub fn open_entry_editor(
        &mut self,
        entry_id: Option<String>,
        account_id: Option<String>,
        category_id: Option<String>,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let entry = entry_id
            .as_deref()
            .and_then(|id| self.snapshot.entry(id))
            .cloned();

        let account_id = entry
            .as_ref()
            .map(|entry| entry.account_id.clone())
            .or(account_id)
            .or_else(|| {
                self.snapshot
                    .active_accounts()
                    .first()
                    .map(|account| account.id.clone())
            });
        let category_id = entry
            .as_ref()
            .map(|entry| entry.category_id.clone())
            .or(category_id)
            .or_else(|| {
                self.snapshot
                    .active_categories()
                    .first()
                    .map(|category| category.id.clone())
            });

        let amount_text = entry
            .as_ref()
            .map(|entry| format::number(entry.amount.abs()))
            .unwrap_or_default();
        let title_text = entry
            .as_ref()
            .map(|entry| entry.title.clone())
            .unwrap_or_default();
        let description_text = entry
            .as_ref()
            .and_then(|entry| entry.description.clone())
            .unwrap_or_default();
        let location_text = entry
            .as_ref()
            .and_then(|entry| entry.location.clone())
            .unwrap_or_default();
        let date_text = entry
            .as_ref()
            .and_then(|entry| entry.date_naive())
            .map(|date| date.format("%Y-%m-%d").to_string())
            .unwrap_or_else(model::today);
        let income = entry
            .as_ref()
            .map(|entry| entry.amount > 0.0)
            .unwrap_or(false);

        let amount_style = InputStyle {
            height: 48.0,
            font_size: 22.0,
            padding_x: 12.0,
            filled: true,
            multiline: false,
            ..InputStyle::default()
        };
        let amount = cx.new(|cx| TextInput::new("0.00", amount_text, amount_style, cx));
        let title = cx
            .new(|cx| TextInput::new("What was this for?", title_text, InputStyle::default(), cx));
        let date = cx.new(|cx| {
            TextInput::new(
                "YYYY-MM-DD",
                date_text,
                InputStyle {
                    height: 34.0,
                    font_size: type_scale::SMALL,
                    padding_x: 10.0,
                    ..InputStyle::default()
                },
                cx,
            )
            .with_prefix(Icon::Calendar)
        });
        let description = cx.new(|cx| {
            TextInput::new(
                "Optional note",
                description_text,
                InputStyle::multiline(),
                cx,
            )
        });
        let location = cx.new(|cx| {
            TextInput::new(
                "Optional location",
                location_text,
                InputStyle::default(),
                cx,
            )
        });

        let handle = amount.read(cx).focus();
        self.modal_seq += 1;
        self.editor = Some(EntryEditor {
            editing: entry_id,
            amount,
            title,
            date,
            description,
            location,
            account_id,
            category_id,
            category_picker_open: false,
            income,
            error: None,
            seq: self.modal_seq,
        });
        window.focus(&handle);
        cx.notify();
    }

    pub(crate) fn set_editor_date(&mut self, offset_days: i64, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.editor.as_ref() else {
            return;
        };
        let date = chrono::Local::now().date_naive() + Duration::days(offset_days);
        editor.date.update(cx, |input, cx| {
            input.set_text(date.format("%Y-%m-%d").to_string(), cx)
        });
    }

    pub fn save_entry(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.editor.as_ref() else {
            return;
        };

        let amount_value = format::parse_amount(editor.amount.read(cx).text());
        let title_value = editor.title.read(cx).text().trim().to_string();
        let date_value = editor.date.read(cx).text().trim().to_string();
        let description_value = editor.description.read(cx).text().trim().to_string();
        let location_value = editor.location.read(cx).text().trim().to_string();
        let account_id = editor.account_id.clone();
        let category_id = editor.category_id.clone();
        let raw_amount = editor.amount.read(cx).text().trim().to_string();
        let income = if raw_amount.starts_with('+') || raw_amount.starts_with('=') {
            true
        } else if raw_amount.starts_with('-') {
            false
        } else {
            editor.income
        };

        let error = if amount_value.is_none_or(|amount| amount <= 0.0) {
            Some("Enter an amount greater than zero.".to_string())
        } else if title_value.is_empty() {
            Some("Give this entry a title.".to_string())
        } else if account_id.is_none() {
            Some("Create an account first, then add this entry.".to_string())
        } else if category_id.is_none() {
            Some("Create a category first, then add this entry.".to_string())
        } else if format::parse_date(&date_value).is_none() {
            Some("Use a date like 2026-11-12.".to_string())
        } else {
            None
        };

        if let Some(error) = error {
            if let Some(editor) = self.editor.as_mut() {
                editor.error = Some(error);
            }
            cx.notify();
            return;
        }

        let amount = amount_value.unwrap_or(0.0) * if income { 1.0 } else { -1.0 };
        let date_stamp = format::parse_date(&date_value)
            .and_then(|date| date.and_hms_opt(12, 0, 0))
            .map(|stamp| stamp.format("%Y-%m-%dT%H:%M:%S%.6f").to_string())
            .unwrap_or_else(now);
        let editing = self
            .editor
            .as_ref()
            .and_then(|editor| editor.editing.clone());
        let previous = editing
            .as_deref()
            .and_then(|id| self.snapshot.entry(id))
            .cloned();

        let entry = if let Some(previous) = previous.clone() {
            Entry {
                account_id: account_id.unwrap_or_default(),
                category_id: category_id.unwrap_or_default(),
                title: title_value,
                description: (!description_value.is_empty()).then_some(description_value),
                location: (!location_value.is_empty()).then_some(location_value),
                amount,
                date: date_stamp,
                updated_at: now(),
                ..previous
            }
        } else {
            Entry {
                id: new_id(),
                account_id: account_id.unwrap_or_default(),
                category_id: category_id.unwrap_or_default(),
                title: title_value,
                description: (!description_value.is_empty()).then_some(description_value),
                location: (!location_value.is_empty()).then_some(location_value),
                amount,
                currency_code: self.settings.default_currency_code.clone(),
                date: date_stamp,
                deleted_at: None,
                inserted_at: now(),
                updated_at: now(),
            }
        };

        let id = entry.id.clone();
        let is_edit = editing.is_some();
        self.snapshot.upsert_entry(entry);
        if !is_edit {
            self.screen = crate::app::Screen::Entries;
            self.detail = Some(crate::app::DetailView::Entry(id.clone()));
        }
        self.editor = None;
        window.blur();
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Success,
            if is_edit {
                "Entry updated.".into()
            } else {
                "Entry added.".into()
            },
            Some(ToastAction::UndoEntry {
                id,
                previous: previous.clone(),
            }),
            cx,
        );
        cx.notify();
    }

    pub fn delete_entry(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.editor.as_ref() else {
            return;
        };
        let Some(id) = editor.editing.clone() else {
            return;
        };
        let Some(mut entry) = self.snapshot.entry(&id).cloned() else {
            return;
        };
        let previous = entry.clone();
        let account_id = entry.account_id.clone();
        entry.deleted_at = Some(now());
        entry.updated_at = now();
        self.snapshot.upsert_entry(entry);
        if let Some(crate::app::DetailView::Entry(detail_id)) = &self.detail
            && detail_id == &id
        {
            // The web sends you back to the entry's account after a delete.
            if account_id.is_empty() {
                self.detail = None;
            } else {
                self.screen = crate::app::Screen::Accounts;
                self.detail = Some(crate::app::DetailView::Account(account_id));
            }
        }
        self.editor = None;
        window.blur();
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Info,
            "Entry deleted.".into(),
            Some(ToastAction::UndoEntry {
                id,
                previous: Some(previous),
            }),
            cx,
        );
        cx.notify();
    }

    pub fn open_account_editor(
        &mut self,
        account_id: Option<String>,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let account = account_id
            .as_deref()
            .and_then(|id| self.snapshot.account(id))
            .cloned();
        let name_text = account
            .as_ref()
            .map(|account| account.name.clone())
            .unwrap_or_default();
        let name =
            cx.new(|cx| TextInput::new("Checking account", name_text, InputStyle::large(), cx));
        let handle = name.read(cx).focus();
        self.modal_seq += 1;
        self.account_editor = Some(AccountEditor {
            editing: account_id,
            name,
            error: None,
            seq: self.modal_seq,
        });
        window.focus(&handle);
        cx.notify();
    }

    pub fn save_account(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.account_editor.as_ref() else {
            return;
        };
        let name = editor.name.read(cx).text().trim().to_string();
        let editing = editor.editing.clone();
        if name.is_empty() {
            if let Some(editor) = self.account_editor.as_mut() {
                editor.error = Some("Give this account a name.".into());
            }
            cx.notify();
            return;
        }
        let duplicate = self.snapshot.active_accounts().iter().any(|account| {
            account.name.eq_ignore_ascii_case(&name)
                && editing.as_deref() != Some(account.id.as_str())
        });
        if duplicate {
            if let Some(editor) = self.account_editor.as_mut() {
                editor.error = Some("An account with that name already exists.".into());
            }
            cx.notify();
            return;
        }

        let previous = editing
            .as_deref()
            .and_then(|id| self.snapshot.account(id))
            .cloned();
        let account = if let Some(previous) = previous.clone() {
            Account {
                name,
                updated_at: now(),
                ..previous
            }
        } else {
            Account::new(name)
        };
        let created = previous.is_none();
        let undo = account.clone();
        let account_id = account.id.clone();
        self.snapshot.upsert_account(account);
        if let Some(editor) = self.editor.as_mut() {
            editor.account_id = Some(account_id.clone());
        } else if created {
            self.screen = crate::app::Screen::Accounts;
            self.detail = Some(crate::app::DetailView::Account(account_id.clone()));
        }
        self.account_editor = None;
        window.blur();
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Success,
            if created {
                "Account created.".into()
            } else {
                "Account updated.".into()
            },
            Some(ToastAction::UndoAccount {
                account: undo,
                created,
            }),
            cx,
        );
        cx.notify();
    }

    pub fn delete_account(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.account_editor.as_ref() else {
            return;
        };
        let Some(id) = editor.editing.clone() else {
            return;
        };
        let Some(mut account) = self.snapshot.account(&id).cloned() else {
            return;
        };
        let previous = account.clone();
        account.deleted_at = Some(now());
        account.updated_at = now();
        self.snapshot.upsert_account(account);
        if let Some(crate::app::DetailView::Account(detail_id)) = &self.detail
            && detail_id == &id
        {
            self.detail = None;
        }
        self.account_editor = None;
        window.blur();
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
        cx.notify();
    }

    pub fn open_category_editor(
        &mut self,
        category_id: Option<String>,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let category = category_id
            .as_deref()
            .and_then(|id| self.snapshot.category(id))
            .cloned();
        let title = category
            .as_ref()
            .map(|category| category.title.clone())
            .unwrap_or_default();
        let description = category
            .as_ref()
            .and_then(|category| category.description.clone())
            .unwrap_or_default();
        let color = category
            .as_ref()
            .map(|category| category.color.clone())
            .unwrap_or_else(|| "#6366f1".into());

        let title_input = cx.new(|cx| TextInput::new("Groceries", title, InputStyle::large(), cx));
        cx.subscribe(&title_input, |_, _, event, cx| {
            if matches!(event, crate::ui::input::TextInputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        let description_input = cx.new(|cx| {
            TextInput::new(
                "What belongs in this category?",
                description,
                InputStyle::multiline(),
                cx,
            )
        });
        let color_input =
            cx.new(|cx| TextInput::new("#6366f1", color.clone(), InputStyle::compact(), cx));
        cx.subscribe(&color_input, |this, _, event, cx| {
            if matches!(event, crate::ui::input::TextInputEvent::Change)
                && let Some(editor) = this.category_editor.as_mut()
            {
                let value = editor.color_input.read(cx).text().trim().to_string();
                if is_hex_color(&value) {
                    editor.color = value;
                }
                cx.notify();
            }
        })
        .detach();
        let handle = title_input.read(cx).focus();
        self.modal_seq += 1;
        self.category_editor = Some(CategoryEditor {
            editing: category_id,
            title: title_input,
            description: description_input,
            color_input,
            color,
            error: None,
            seq: self.modal_seq,
        });
        window.focus(&handle);
        cx.notify();
    }

    pub fn set_category_color(&mut self, color: String, cx: &mut gpui::Context<Self>) {
        if let Some(editor) = self.category_editor.as_mut() {
            editor.color = color.clone();
            editor
                .color_input
                .update(cx, |input, cx| input.set_text(color, cx));
        }
        cx.notify();
    }

    pub fn save_category(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.category_editor.as_ref() else {
            return;
        };
        let title = editor.title.read(cx).text().trim().to_string();
        let description = editor.description.read(cx).text().trim().to_string();
        let color = editor.color_input.read(cx).text().trim().to_string();
        let color = if is_hex_color(&color) {
            color
        } else {
            editor.color.clone()
        };
        let editing = editor.editing.clone();
        if title.is_empty() {
            if let Some(editor) = self.category_editor.as_mut() {
                editor.error = Some("Give this category a name.".into());
            }
            cx.notify();
            return;
        }
        let duplicate = self.snapshot.active_categories().iter().any(|category| {
            category.title.eq_ignore_ascii_case(&title)
                && editing.as_deref() != Some(category.id.as_str())
        });
        if duplicate {
            if let Some(editor) = self.category_editor.as_mut() {
                editor.error = Some("A category with that name already exists.".into());
            }
            cx.notify();
            return;
        }

        let previous = editing
            .as_deref()
            .and_then(|id| self.snapshot.category(id))
            .cloned();
        let category = if let Some(previous) = previous.clone() {
            Category {
                title,
                description: (!description.is_empty()).then_some(description),
                color,
                updated_at: now(),
                ..previous
            }
        } else {
            let mut category = Category::new(title, color);
            if !description.is_empty() {
                category.description = Some(description);
            }
            category
        };
        let created = previous.is_none();
        let undo = category.clone();
        let category_id = category.id.clone();
        self.snapshot.upsert_category(category);
        if let Some(editor) = self.editor.as_mut() {
            editor.category_id = Some(category_id.clone());
        } else if created {
            self.screen = crate::app::Screen::Categories;
            self.detail = Some(crate::app::DetailView::Category(category_id.clone()));
        }
        self.category_editor = None;
        window.blur();
        self.persist(cx);
        self.schedule_sync(cx);
        self.toast(
            ToastKind::Success,
            if created {
                "Category created.".into()
            } else {
                "Category updated.".into()
            },
            Some(ToastAction::UndoCategory {
                category: undo,
                created,
            }),
            cx,
        );
        cx.notify();
    }

    pub fn delete_category(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.category_editor.as_ref() else {
            return;
        };
        let Some(id) = editor.editing.clone() else {
            return;
        };
        let entries = self.snapshot.category_entry_count(&id);
        if entries > 0 {
            if let Some(editor) = self.category_editor.as_mut() {
                editor.error = Some(format!(
                    "This category still has {}. Move or delete those entries first.",
                    format::plural(entries, "active entry", "active entries")
                ));
            }
            cx.notify();
            return;
        }
        let Some(mut category) = self.snapshot.category(&id).cloned() else {
            return;
        };
        let previous = category.clone();
        category.deleted_at = Some(now());
        category.updated_at = now();
        self.snapshot.upsert_category(category);
        if let Some(crate::app::DetailView::Category(detail_id)) = &self.detail
            && detail_id == &id
        {
            self.detail = None;
        }
        self.category_editor = None;
        window.blur();
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
        cx.notify();
    }
    pub fn open_admin_editor(
        &mut self,
        user: AdminUser,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let username =
            cx.new(|cx| TextInput::new("Username", user.username.clone(), InputStyle::large(), cx));
        let email = cx.new(|cx| {
            TextInput::new("Email", user.email.clone(), InputStyle::default(), cx)
                .with_prefix(Icon::Globe)
        });
        let flags = cx.new(|cx| {
            TextInput::new(
                "Flags, comma separated (e.g. is_admin)",
                user.flags.join(", "),
                InputStyle::default(),
                cx,
            )
        });
        let password = cx.new(|cx| {
            TextInput::new("New password (optional)", "", InputStyle::default(), cx)
                .masked(true)
                .with_prefix(Icon::Key)
        });
        let handle = username.read(cx).focus();
        self.modal_seq += 1;
        self.admin_editor = Some(AdminEditor {
            editing: user.id,
            username,
            email,
            flags,
            password,
            password_masked: true,
            error: None,
            seq: self.modal_seq,
        });
        window.focus(&handle);
        cx.notify();
    }

    pub fn save_admin_user(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(editor) = self.admin_editor.as_ref() else {
            return;
        };
        let username = editor.username.read(cx).text().trim().to_string();
        let email = editor.email.read(cx).text().trim().to_string();
        let flags_text = editor.flags.read(cx).text().to_string();
        let password = editor.password.read(cx).text().to_string();
        let id = editor.editing.clone();
        if username.is_empty() || email.is_empty() {
            if let Some(editor) = self.admin_editor.as_mut() {
                editor.error = Some("Username and email are required.".into());
            }
            cx.notify();
            return;
        }
        let flags = flags_text
            .split(',')
            .map(str::trim)
            .filter(|flag| !flag.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let access = self.access_token.clone();
        let refresh = self.settings.refresh_token.clone();
        let api = self.api.clone();
        let update_id = id.clone();
        window.blur();
        cx.spawn(async move |this, cx| {
            let result = crate::app::with_access(api, access, refresh, move |api, access| {
                let id = id.clone();
                let username = username.clone();
                let email = email.clone();
                let flags = flags.clone();
                let password = (!password.is_empty()).then_some(password.clone());
                Box::pin(async move {
                    api.admin_update_user(access, id, username, email, flags, password)
                        .await
                })
            })
            .await;
            this.update(cx, |app, cx| match result {
                Ok((user, access)) => {
                    app.access_token = Some(access);
                    if let Some(existing) =
                        app.admin.users.iter_mut().find(|item| item.id == update_id)
                    {
                        *existing = user;
                    }
                    app.admin_editor = None;
                    app.toast(ToastKind::Success, "User updated.".into(), None, cx);
                }
                Err(error) => {
                    if error.status == Some(401) {
                        app.session_expired(cx);
                    } else if let Some(editor) = app.admin_editor.as_mut() {
                        editor.error = Some(error.message);
                        cx.notify();
                    }
                }
            })
            .ok();
        })
        .detach();
    }
}

pub fn render_overlays(
    app: &SunderApp,
    window: &mut Window,
    cx: &mut gpui::Context<SunderApp>,
) -> Vec<AnyElement> {
    let palette = theme(cx).clone();
    let mut layers = Vec::new();

    if let Some(editor) = app.editor.as_ref() {
        let content = entry_editor_content(app, &palette, editor, cx);
        layers.push(modal_layer(
            &palette,
            editor.seq,
            560.0,
            content.into_any_element(),
            true,
            cx,
        ));
    }

    if let Some(editor) = app.account_editor.as_ref() {
        let content = account_editor_content(app, &palette, editor, cx);
        layers.push(modal_layer(
            &palette,
            editor.seq,
            430.0,
            content.into_any_element(),
            false,
            cx,
        ));
    }

    if let Some(editor) = app.category_editor.as_ref() {
        let content = category_editor_content(app, &palette, editor, cx);
        layers.push(modal_layer(
            &palette,
            editor.seq,
            480.0,
            content.into_any_element(),
            false,
            cx,
        ));
    }

    if let Some(editor) = app.admin_editor.as_ref() {
        let content = admin_editor_content(&palette, editor, cx);
        layers.push(modal_layer(
            &palette,
            editor.seq,
            500.0,
            content.into_any_element(),
            false,
            cx,
        ));
    }

    if let Some(confirm) = app.confirm.as_ref() {
        let content = confirm_content(&palette, confirm, cx);
        layers.push(modal_layer(
            &palette,
            confirm.seq,
            430.0,
            content.into_any_element(),
            false,
            cx,
        ));
    }

    if app.logout_dialog {
        let content = logout_content(app, &palette, cx);
        layers.push(modal_layer(
            &palette,
            app.modal_seq,
            440.0,
            content.into_any_element(),
            false,
            cx,
        ));
    }

    if app.show_shortcuts {
        let content = shortcuts_content(&palette, cx);
        layers.push(modal_layer(
            &palette,
            app.modal_seq,
            560.0,
            content.into_any_element(),
            false,
            cx,
        ));
    }

    if let Some(calendar) = app.calendar.as_ref() {
        let content = calendar_content(app, &palette, calendar, cx);
        layers.push(calendar_layer(
            &palette,
            calendar,
            content.into_any_element(),
            cx,
        ));
    }

    if !app.toasts.is_empty() {
        layers.push(toast_layer(app, &palette, window, cx));
    }

    layers
}

fn modal_layer(
    palette: &Theme,
    seq: u64,
    width: f32,
    content: AnyElement,
    fill_height: bool,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    let card = div()
        .id(ElementId::named_usize("modal-card", seq as usize))
        .w(px(width))
        .when(fill_height, |element| element.h(relative(0.9)))
        .max_h(relative(0.95))
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(palette.surface)
        .rounded_xl()
        .border_1()
        .border_color(palette.border)
        .shadow(vec![BoxShadow {
            color: palette.shadow.opacity(1.35),
            offset: gpui::point(px(0.0), px(18.0)),
            blur_radius: px(44.0),
            spread_radius: px(0.0),
        }])
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(content);

    let reduce_motion = crate::motion::reduce_motion(cx);
    let card = crate::motion::dialog_in(
        ElementId::named_usize("modal-in", seq as usize),
        card,
        reduce_motion,
    );
    let scrim = div().absolute().inset_0().bg(palette.overlay);
    let scrim = crate::motion::backdrop_in(
        ElementId::named_usize("modal-scrim", seq as usize),
        scrim,
        reduce_motion,
    );

    div()
        .absolute()
        .inset_0()
        .occlude()
        .flex()
        .items_center()
        .justify_center()
        .p(px(24.0))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, window, cx| this.close_top_overlay(window, cx)),
        )
        .on_mouse_down(
            MouseButton::Navigate(NavigationDirection::Back),
            cx.listener(|this, _, window, cx| this.mouse_back(window, cx)),
        )
        .child(scrim)
        .child(card)
        .into_any_element()
}

fn modal_header(
    palette: &Theme,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    div()
        .flex()
        .items_start()
        .justify_between()
        .gap(px(16.0))
        .px(px(20.0))
        .pt(px(18.0))
        .pb(px(14.0))
        .border_b_1()
        .border_color(palette.border)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.0))
                .child(
                    div()
                        .text_size(px(18.0))
                        .line_height(px(22.0))
                        .font_weight(type_scale::SEMIBOLD)
                        .text_color(palette.foreground)
                        .child(title.into()),
                )
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.muted)
                        .child(subtitle.into()),
                ),
        )
        .child(
            icon_button(palette, "modal-close", Icon::X, true)
                .on_click(cx.listener(|this, _, window, cx| this.close_top_overlay(window, cx)))
                .sunder_tooltip("Close", &["Esc"]),
        )
}

fn entry_editor_content(
    app: &SunderApp,
    palette: &Theme,
    editor: &EntryEditor,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let title = if editor.editing.is_some() {
        "Edit entry"
    } else {
        "New entry"
    };
    let subtitle = if editor.editing.is_some() {
        "Update the details and save."
    } else {
        "Expense by default. Switch to income when money comes in."
    };

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

    let expense_selected = !editor.income;
    let income_selected = editor.income;
    let expense_bg = palette.expense_tint;
    let expense_fg = palette.expense;
    let income_bg = palette.income_tint;
    let income_fg = palette.income;

    let segmented = div()
        .flex()
        .gap(px(8.0))
        .child(
            div()
                .id("entry-expense")
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .h(px(38.0))
                .rounded_lg()
                .border_1()
                .cursor_pointer()
                .text_size(px(type_scale::SMALL))
                .font_weight(type_scale::MEDIUM)
                .when(expense_selected, |style| {
                    style
                        .bg(expense_bg)
                        .border_color(palette.expense.opacity(0.25))
                        .text_color(expense_fg)
                })
                .when(!expense_selected, |style| {
                    style
                        .bg(palette.surface)
                        .border_color(palette.border)
                        .text_color(palette.muted)
                })
                .debug_selector(|| "entry-expense".to_string())
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(editor) = this.editor.as_mut() {
                        editor.income = false;
                    }
                    cx.notify();
                }))
                .child(icon(
                    Icon::Minus,
                    14.0,
                    if expense_selected {
                        expense_fg
                    } else {
                        palette.muted
                    },
                ))
                .child("Expense"),
        )
        .child(
            div()
                .id("entry-income")
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .h(px(38.0))
                .rounded_lg()
                .border_1()
                .cursor_pointer()
                .text_size(px(type_scale::SMALL))
                .font_weight(type_scale::MEDIUM)
                .when(income_selected, |style| {
                    style
                        .bg(income_bg)
                        .border_color(palette.income.opacity(0.25))
                        .text_color(income_fg)
                })
                .when(!income_selected, |style| {
                    style
                        .bg(palette.surface)
                        .border_color(palette.border)
                        .text_color(palette.muted)
                })
                .debug_selector(|| "entry-income".to_string())
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(editor) = this.editor.as_mut() {
                        editor.income = true;
                    }
                    cx.notify();
                }))
                .child(icon(
                    Icon::Plus,
                    14.0,
                    if income_selected {
                        income_fg
                    } else {
                        palette.muted
                    },
                ))
                .child("Income"),
        );

    let symbol_el = || {
        div()
            .text_size(px(26.0))
            .font_weight(type_scale::SEMIBOLD)
            .text_color(if editor.income {
                palette.income
            } else {
                palette.foreground
            })
            .child(app.symbol().to_string())
    };
    let symbol_before = app.currency_position() == crate::format::CurrencyPosition::Before;
    let mut amount_row = div().flex().items_center().gap(px(8.0));
    if symbol_before {
        amount_row = amount_row.child(symbol_el());
    }
    amount_row = amount_row.child(div().flex_1().child(editor.amount.clone()));
    if !symbol_before {
        amount_row = amount_row.child(symbol_el());
    }
    let amount_row = crate::motion::tab_in(
        ElementId::Name(format!("entry-amount-{}-{}", editor.seq, editor.income).into()),
        amount_row,
        crate::motion::reduce_motion(cx),
    );

    let mut account_chips = div().flex().flex_wrap().gap(px(6.0));
    if accounts.is_empty() {
        account_chips = account_chips.child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child("No accounts yet. Add one from the Accounts screen."),
        );
    }
    for (id, name) in accounts {
        let selected = editor.account_id.as_deref() == Some(id.as_str());
        account_chips = account_chips.child(
            chip(
                palette,
                ElementId::Name(format!("entry-account-{id}").into()),
                name,
                selected,
                None,
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(editor) = this.editor.as_mut() {
                    editor.account_id = Some(id.clone());
                }
                cx.notify();
            })),
        );
    }
    account_chips = account_chips.child(
        ghost_button(
            palette,
            "entry-new-account",
            "New account",
            Some(Icon::Plus),
        )
        .debug_selector(|| "entry-new-account".to_string())
        .on_click(cx.listener(|this, _, window, cx| {
            this.open_account_editor(None, window, cx);
        })),
    );

    let selected_category = editor
        .category_id
        .as_deref()
        .and_then(|id| {
            categories
                .iter()
                .find(|(category_id, ..)| category_id == id)
        })
        .cloned();

    let category_select = div()
        .id("entry-category-select")
        .debug_selector(|| "entry-category-select".to_string())
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .h(px(38.0))
        .px(px(10.0))
        .rounded_lg()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_muted)
        .cursor_pointer()
        .hover({
            let hover = palette.surface_hover;
            move |style| style.bg(hover)
        })
        .on_click(cx.listener(|this, _, _, cx| {
            if let Some(editor) = this.editor.as_mut() {
                editor.category_picker_open = !editor.category_picker_open;
            }
            cx.notify();
        }))
        .child(match &selected_category {
            Some((_, name, color)) => div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .min_w(px(0.0))
                .child(dot(parse_color(color), 9.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .text_color(palette.foreground)
                        .truncate()
                        .child(name.clone()),
                ),
            None => div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child("Choose a category"),
        })
        .child(icon(Icon::ChevronDown, 14.0, palette.muted));

    let mut category_options = div()
        .id("entry-category-panel")
        .debug_selector(|| "entry-category-panel".to_string())
        .flex()
        .flex_wrap()
        .gap(px(6.0))
        .w_full()
        .p(px(8.0))
        .rounded_lg()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface)
        .shadow(vec![BoxShadow {
            color: palette.shadow.opacity(0.6),
            offset: gpui::point(px(0.0), px(8.0)),
            blur_radius: px(24.0),
            spread_radius: px(0.0),
        }]);
    if categories.is_empty() {
        category_options = category_options.child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child("No categories yet. Add one from the Categories screen."),
        );
    }
    for (id, name, color) in categories {
        let selected = editor.category_id.as_deref() == Some(id.as_str());
        let debug_id = format!("entry-category-{id}");
        category_options = category_options.child(
            chip(
                palette,
                ElementId::Name(format!("entry-category-{id}").into()),
                name,
                selected,
                Some(parse_color(&color)),
            )
            .debug_selector(move || debug_id.clone())
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(editor) = this.editor.as_mut() {
                    editor.category_id = Some(id.clone());
                    editor.category_picker_open = false;
                }
                cx.notify();
            })),
        );
    }
    category_options = category_options.child(
        ghost_button(
            palette,
            "entry-new-category",
            "New category",
            Some(Icon::Plus),
        )
        .debug_selector(|| "entry-new-category".to_string())
        .on_click(cx.listener(|this, _, window, cx| {
            if let Some(editor) = this.editor.as_mut() {
                editor.category_picker_open = false;
            }
            this.open_category_editor(None, window, cx);
        })),
    );

    let error = editor.error.as_ref().map(|error| {
        div()
            .flex()
            .items_start()
            .gap(px(8.0))
            .p(px(9.0))
            .rounded_lg()
            .bg(palette.expense_tint)
            .border_1()
            .border_color(palette.expense.opacity(0.25))
            .child(icon(Icon::Alert, 14.0, palette.expense))
            .child(
                div()
                    .text_size(px(type_scale::SMALL))
                    .text_color(palette.expense)
                    .child(error.clone()),
            )
    });

    let mut footer = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(20.0))
        .py(px(14.0))
        .border_t_1()
        .border_color(palette.border);
    if editor.editing.is_some() {
        footer = footer.child(
            crate::ui::components::danger_ghost_button(
                palette,
                "entry-delete",
                "Delete",
                Some(Icon::Trash),
            )
            .on_click(cx.listener(|this, _, window, cx| this.delete_entry(window, cx))),
        );
    }
    footer = footer
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(palette.subtle)
                        .child("Save"),
                )
                .child(crate::ui::components::keycap(
                    palette,
                    format::modifier_label(),
                ))
                .child(crate::ui::components::keycap(palette, "↵")),
        )
        .child(
            secondary_button(palette, "entry-cancel", "Cancel", None)
                .debug_selector(|| "entry-cancel".to_string())
                .on_click(cx.listener(|this, _, window, cx| this.close_top_overlay(window, cx))),
        )
        .child(
            primary_button(palette, "entry-save", "Save entry", Some(Icon::Check))
                .debug_selector(|| "entry-save".to_string())
                .on_click(cx.listener(|this, _, window, cx| this.save_entry(window, cx)))
                .sunder_tooltip("Save entry", &[format::modifier_label(), "Enter"]),
        );

    let description_row = div()
        .flex()
        .gap(px(10.0))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(field_label(palette, "Note"))
                .child(editor.description.clone()),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(field_label(palette, "Location"))
                .child(editor.location.clone()),
        );

    div()
        .flex()
        .flex_col()
        .h_full()
        .child(modal_header(palette, title, subtitle, cx))
        .child(
            div()
                .id(ElementId::named_usize("entry-form", editor.seq as usize))
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .p(px(20.0))
                .child(segmented)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Amount"))
                        .child(amount_row),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Title"))
                        .child(editor.title.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Account"))
                        .child(account_chips),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Category"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(category_select)
                                .when(editor.category_picker_open, |element| {
                                    element.child(category_options)
                                }),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Date"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(div().w(px(158.0)).child(editor.date.clone()))
                                .child(
                                    ghost_button(
                                        palette,
                                        "date-calendar",
                                        "Calendar",
                                        Some(Icon::Calendar),
                                    )
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.open_entry_calendar(cx)),
                                    ),
                                )
                                .child(ghost_button(palette, "date-today", "Today", None).on_click(
                                    cx.listener(|this, _, _, cx| this.set_editor_date(0, cx)),
                                ))
                                .child(
                                    ghost_button(palette, "date-yesterday", "Yesterday", None)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_editor_date(-1, cx)
                                        })),
                                ),
                        ),
                )
                .child(description_row),
        )
        .when_some(error, |element, error| {
            element.child(div().px(px(20.0)).pb(px(12.0)).child(error))
        })
        .child(footer)
}

fn account_editor_content(
    app: &SunderApp,
    palette: &Theme,
    editor: &AccountEditor,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let title = if editor.editing.is_some() {
        "Edit account"
    } else {
        "New account"
    };
    let error = editor.error.as_ref().map(|error| {
        div()
            .flex()
            .items_start()
            .gap(px(8.0))
            .p(px(9.0))
            .rounded_lg()
            .bg(palette.expense_tint)
            .border_1()
            .border_color(palette.expense.opacity(0.25))
            .child(icon(Icon::Alert, 14.0, palette.expense))
            .child(
                div()
                    .text_size(px(type_scale::SMALL))
                    .text_color(palette.expense)
                    .child(error.clone()),
            )
    });

    let mut footer = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(20.0))
        .py(px(14.0))
        .border_t_1()
        .border_color(palette.border);
    if editor.editing.is_some() {
        footer = footer.child(
            crate::ui::components::danger_ghost_button(
                palette,
                "account-delete",
                "Delete",
                Some(Icon::Trash),
            )
            .on_click(cx.listener(|this, _, window, cx| this.delete_account(window, cx))),
        );
    }
    footer = footer
        .child(div().flex_1())
        .child(
            secondary_button(palette, "account-cancel", "Cancel", None)
                .on_click(cx.listener(|this, _, window, cx| this.close_top_overlay(window, cx)))
                .sunder_tooltip("Cancel", &["Esc"]),
        )
        .child(
            primary_button(palette, "account-save", "Save account", Some(Icon::Check))
                .debug_selector(|| "account-save".to_string())
                .on_click(cx.listener(|this, _, window, cx| this.save_account(window, cx)))
                .sunder_tooltip("Save account", &[format::modifier_label(), "Enter"]),
        );

    let entry_count = editor
        .editing
        .as_deref()
        .map(|id| app.snapshot.account_entry_count(id))
        .unwrap_or(0);
    let hint = if editor.editing.is_some() && entry_count > 0 {
        format!(
            "Deleting this account leaves its {} in your ledger, where they'll still be included in totals.",
            format::plural(entry_count, "entry", "entries")
        )
    } else {
        "Accounts group entries. Name it after the place your money lives.".into()
    };

    div()
        .flex()
        .flex_col()
        .child(modal_header(
            palette,
            title,
            "A checking account, card, or savings pot.",
            cx,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .p(px(20.0))
                .when_some(error, |element, error| element.child(error))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Name"))
                        .child(editor.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .line_height(px(16.0))
                        .text_color(palette.subtle)
                        .child(hint),
                ),
        )
        .child(footer)
}

fn category_editor_content(
    _app: &SunderApp,
    palette: &Theme,
    editor: &CategoryEditor,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    const COLORS: [&str; 15] = [
        "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#22c55e", "#10b981", "#06b6d4", "#3b82f6",
        "#6366f1", "#8b5cf6", "#d946ef", "#f43f5e", "#000000", "#52525b", "#78716c",
    ];

    let title = if editor.editing.is_some() {
        "Edit category"
    } else {
        "New category"
    };
    let error = editor.error.as_ref().map(|error| {
        div()
            .flex()
            .items_start()
            .gap(px(8.0))
            .p(px(9.0))
            .rounded_lg()
            .bg(palette.expense_tint)
            .border_1()
            .border_color(palette.expense.opacity(0.25))
            .child(icon(Icon::Alert, 14.0, palette.expense))
            .child(
                div()
                    .text_size(px(type_scale::SMALL))
                    .text_color(palette.expense)
                    .child(error.clone()),
            )
    });

    let mut swatches = div().flex().flex_wrap().gap(px(7.0));
    for color in COLORS {
        let selected = editor.color.eq_ignore_ascii_case(color);
        let id = ElementId::Name(format!("category-color-{color}").into());
        let value = color.to_string();
        swatches = swatches.child(
            div()
                .id(id)
                .flex()
                .items_center()
                .justify_center()
                .size(px(30.0))
                .rounded_full()
                .cursor_pointer()
                .border_2()
                .border_color(if selected {
                    palette.primary
                } else {
                    gpui::transparent_black()
                })
                .hover({
                    let border = palette.border_strong;
                    move |style| style.border_color(border)
                })
                .on_click(
                    cx.listener(move |this, _, _, cx| this.set_category_color(value.clone(), cx)),
                )
                .child(
                    div()
                        .size(px(22.0))
                        .rounded_full()
                        .bg(parse_color(color))
                        .border_1()
                        .border_color(palette.overlay.opacity(0.3)),
                ),
        );
    }

    let mut footer = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(20.0))
        .py(px(14.0))
        .border_t_1()
        .border_color(palette.border);
    if editor.editing.is_some() {
        footer = footer.child(
            crate::ui::components::danger_ghost_button(
                palette,
                "category-delete",
                "Delete",
                Some(Icon::Trash),
            )
            .on_click(cx.listener(|this, _, window, cx| this.delete_category(window, cx))),
        );
    }
    footer = footer
        .child(div().flex_1())
        .child(
            secondary_button(palette, "category-cancel", "Cancel", None)
                .on_click(cx.listener(|this, _, window, cx| this.close_top_overlay(window, cx)))
                .sunder_tooltip("Cancel", &["Esc"]),
        )
        .child(
            primary_button(palette, "category-save", "Save category", Some(Icon::Check))
                .debug_selector(|| "category-save".to_string())
                .on_click(cx.listener(|this, _, window, cx| this.save_category(window, cx)))
                .sunder_tooltip("Save category", &[format::modifier_label(), "Enter"]),
        );

    div()
        .flex()
        .flex_col()
        .child(modal_header(
            palette,
            title,
            "A label and a color. Keep it short and recognizable.",
            cx,
        ))
        .child(
            div()
                .id(ElementId::named_usize("category-form", editor.seq as usize))
                .flex()
                .flex_col()
                .gap(px(14.0))
                .p(px(20.0))
                .when_some(error, |element, error| element.child(error))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Name"))
                        .child(editor.title.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(field_label(palette, "Description"))
                        .child(editor.description.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(field_label(palette, "Color"))
                        .child(swatches)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(div().w(px(120.0)).child(editor.color_input.clone()))
                                .child(
                                    div()
                                        .text_size(px(type_scale::MICRO))
                                        .text_color(palette.subtle)
                                        .child("Or enter any #rrggbb value"),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .p(px(10.0))
                        .rounded_lg()
                        .bg(palette.surface_muted)
                        .child(dot(parse_color(&editor.color), 10.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::MEDIUM)
                                .text_color(palette.foreground)
                                .child({
                                    let title = editor.title.read(cx).text().to_string();
                                    if title.is_empty() {
                                        "Category preview".to_string()
                                    } else {
                                        title
                                    }
                                }),
                        ),
                ),
        )
        .child(footer)
}

fn confirm_content(
    palette: &Theme,
    confirm: &crate::app::ConfirmDialog,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let action = confirm.action.clone();
    div()
        .flex()
        .flex_col()
        .child(modal_header(
            palette,
            confirm.title.clone(),
            "This cannot be undone.",
            cx,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .p(px(20.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .line_height(px(19.0))
                        .text_color(palette.muted)
                        .child(confirm.message.clone()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(8.0))
                        .child(
                            secondary_button(palette, "confirm-cancel", "Cancel", None).on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.confirm = None;
                                    cx.notify();
                                }),
                            ),
                        )
                        .child(
                            danger_button(
                                palette,
                                "confirm-run",
                                confirm.confirm_label.clone(),
                                Some(Icon::Trash),
                            )
                            .debug_selector(|| "confirm-run".to_string())
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.confirm_action(action.clone(), window, cx)
                                },
                            )),
                        ),
                ),
        )
}

fn admin_editor_content(
    palette: &Theme,
    editor: &AdminEditor,
    cx: &mut gpui::Context<SunderApp>,
) -> Div {
    let error = editor.error.as_ref().map(|error| {
        div()
            .flex()
            .items_start()
            .gap(px(8.0))
            .p(px(9.0))
            .rounded_lg()
            .bg(palette.expense_tint)
            .border_1()
            .border_color(palette.expense.opacity(0.25))
            .child(icon(Icon::Alert, 14.0, palette.expense))
            .child(
                div()
                    .text_size(px(type_scale::SMALL))
                    .text_color(palette.expense)
                    .child(error.clone()),
            )
    });
    div()
        .flex()
        .flex_col()
        .child(modal_header(
            palette,
            "Edit user",
            "Update the account or reset its password.",
            cx,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .when_some(error, |element, error| element.child(error))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(field_label(palette, "Username"))
                        .child(editor.username.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(field_label(palette, "Email"))
                        .child(editor.email.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(field_label(palette, "Flags"))
                        .child(editor.flags.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(field_label(palette, "Password"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().flex_1().child(editor.password.clone()))
                                .child(
                                    icon_button(
                                        palette,
                                        "admin-password-reveal",
                                        if editor.password.read(cx).is_masked() {
                                            Icon::Eye
                                        } else {
                                            Icon::EyeOff
                                        },
                                        true,
                                    )
                                    .debug_selector(|| "admin-password-reveal".to_string())
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            if let Some(editor) = this.admin_editor.as_mut() {
                                                editor.password_masked = !editor.password_masked;
                                                let masked = editor.password_masked;
                                                editor.password.update(cx, |input, cx| {
                                                    input.set_masked(masked, cx)
                                                });
                                            }
                                            cx.notify();
                                        },
                                    )),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_size(px(type_scale::MICRO))
                        .text_color(palette.subtle)
                        .child("Leave the password empty unless you want to reset it."),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(8.0))
                .px(px(20.0))
                .py(px(14.0))
                .border_t_1()
                .border_color(palette.border)
                .child(
                    secondary_button(palette, "admin-user-cancel", "Cancel", None).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.admin_editor = None;
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    primary_button(palette, "admin-user-save", "Save user", Some(Icon::Check))
                        .on_click(
                            cx.listener(|this, _, window, cx| this.save_admin_user(window, cx)),
                        ),
                ),
        )
}

fn logout_content(app: &SunderApp, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    div()
        .flex()
        .flex_col()
        .child(modal_header(
            palette,
            "Unsynced changes",
            "You're about to leave with work that hasn't reached the server.",
            cx,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .p(px(20.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .line_height(px(19.0))
                        .text_color(palette.muted)
                        .child(format!(
                            "You have {}. Logging out now will also remove the local copy on this computer.",
                            format::plural(app.logout_unsynced, "unsynced change", "unsynced changes")
                        )),
                )
                .child(
                    primary_button(
                        palette,
                        "logout-sync",
                        "Sync & log out",
                        Some(Icon::Refresh),
                    )
                    .w_full()
                    .on_click(cx.listener(|this, _, _, cx| this.sync_and_logout(cx))),
                )
                .child(
                    danger_button(
                        palette,
                        "logout-discard",
                        "Discard & log out",
                        Some(Icon::Trash),
                    )
                    .w_full()
                    .on_click(cx.listener(|this, _, _, cx| this.discard_and_logout(cx))),
                )
                .child(
                    secondary_button(palette, "logout-cancel", "Cancel", None)
                        .debug_selector(|| "logout-cancel".to_string())
                        .w_full()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.logout_dialog = false;
                            cx.notify();
                        })),
                ),
        )
}

fn shortcut_entry(palette: &Theme, description: &str, groups: &[&[&str]]) -> Div {
    let mut keys = div().flex().items_center().gap(px(4.0));
    for (index, group) in groups.iter().enumerate() {
        if index > 0 {
            keys = keys.child(
                div()
                    .text_size(px(9.0))
                    .text_color(palette.subtle)
                    .child("or"),
            );
        }
        keys = keys.child(shortcut(palette, group));
    }
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child(description.to_string()),
        )
        .child(keys)
}

fn shortcut_entry_chord(
    palette: &Theme,
    description: &str,
    first: &[&str],
    second: &[&str],
) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .text_size(px(type_scale::SMALL))
                .text_color(palette.muted)
                .child(description.to_string()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(shortcut(palette, first))
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(palette.subtle)
                        .child("then"),
                )
                .child(shortcut(palette, second)),
        )
}

fn shortcuts_content(palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> Div {
    let m = format::modifier_label();
    div()
        .flex()
        .flex_col()
        .child(modal_header(
            palette,
            "Keyboard shortcuts",
            "Everything Sunder can do without the mouse.",
            cx,
        ))
        .child(
            div()
                .id("shortcuts-scroll")
                .max_h(px(500.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .p(px(20.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child("Navigate"),
                        )
                        .child(shortcut_entry(palette, "Dashboard", &[&[m, "1"], &["Alt", "1"]]))
                        .child(shortcut_entry(palette, "Entries", &[&[m, "2"], &["Alt", "E"]]))
                        .child(shortcut_entry(palette, "Accounts", &[&[m, "3"], &["Alt", "A"]]))
                        .child(shortcut_entry(palette, "Categories", &[&[m, "4"], &["Alt", "C"]]))
                        .child(shortcut_entry(palette, "Settings", &[&[m, "5"]]))
                        .child(shortcut_entry(palette, "Search entries", &[&[m, "F"]])),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child("Create"),
                        )
                        .child(shortcut_entry(
                            palette,
                            "New entry",
                            &[&[m, "N"], &["Alt", "Space"]],
                        ))
                        .child(shortcut_entry_chord(
                            palette,
                            "New entry (two-key)",
                            &["Alt", "N"],
                            &["E"],
                        ))
                        .child(shortcut_entry_chord(
                            palette,
                            "New account",
                            &["Alt", "N"],
                            &["A"],
                        ))
                        .child(shortcut_entry_chord(
                            palette,
                            "New category",
                            &["Alt", "N"],
                            &["C"],
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child("Data"),
                        )
                        .child(shortcut_entry(palette, "Sync now", &[&[m, "R"]]))
                        .child(shortcut_entry(palette, "Toggle theme", &[&["Alt", "T"], &[m, "Shift", "T"]]))
                        .child(shortcut_entry(palette, "This help", &[&["Alt", "/"], &[m, "/"]])),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(type_scale::SMALL))
                                .font_weight(type_scale::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child("Editing"),
                        )
                        .child(shortcut_entry(palette, "Save an editor", &[&[m, "Enter"]]))
                        .child(shortcut_entry(palette, "Close a dialog", &[&["Esc"]])),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .child(icon(Icon::Sparkles, 13.0, palette.subtle))
                        .child(
                            div()
                                .text_size(px(type_scale::MICRO))
                                .text_color(palette.subtle)
                                .child("Two-key shortcuts are chords: press the first, then the second."),
                        ),
                ),
        )
}

fn toast_layer(
    app: &SunderApp,
    palette: &Theme,
    _window: &mut Window,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    let mut toasts = div()
        .absolute()
        .right(px(18.0))
        .bottom(px(18.0))
        .w(px(350.0))
        .flex()
        .flex_col()
        .gap(px(8.0));

    for toast in &app.toasts {
        toasts = toasts.child(toast_card(toast, palette, cx));
    }

    toasts.into_any_element()
}

fn toast_card(toast: &Toast, palette: &Theme, cx: &mut gpui::Context<SunderApp>) -> AnyElement {
    let (tone, icon_name) = match toast.kind {
        ToastKind::Success => (palette.income, Icon::Check),
        ToastKind::Error => (palette.expense, Icon::Alert),
        ToastKind::Info => (palette.primary, Icon::Sparkles),
    };

    let action = toast.action.clone();
    let mut action_button = None;
    if let Some(action) = action.clone() {
        let (label, icon_name) = match &action {
            ToastAction::RetrySync => ("Retry", Icon::Refresh),
            _ => ("Undo", Icon::Undo),
        };
        let id = toast.id;
        action_button = Some(
            crate::ui::components::link_button(
                palette,
                ("toast-action", id as usize),
                label,
                Some(icon_name),
            )
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| {
                this.dismiss_toast(id, cx);
                this.run_toast_action(action.clone(), window, cx);
            })),
        );
    }

    let id = toast.id;
    let card = div()
        .id(ElementId::named_usize("toast", id as usize))
        .flex()
        .items_start()
        .gap(px(10.0))
        .p(px(12.0))
        .bg(palette.surface)
        .rounded_xl()
        .border_1()
        .border_color(palette.border)
        .shadow(vec![BoxShadow {
            color: palette.shadow,
            offset: gpui::point(px(0.0), px(10.0)),
            blur_radius: px(26.0),
            spread_radius: px(0.0),
        }])
        .child(icon_badge(
            palette,
            icon_name,
            match toast.kind {
                ToastKind::Success => BadgeKind::Success,
                ToastKind::Error => BadgeKind::Danger,
                ToastKind::Info => BadgeKind::Primary,
            },
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(5.0))
                .flex_1()
                .min_w(px(0.0))
                .child(
                    div()
                        .text_size(px(type_scale::SMALL))
                        .line_height(px(17.0))
                        .text_color(palette.foreground)
                        .child(toast.message.clone()),
                )
                .when_some(action_button, |element, button| element.child(button)),
        )
        .child(
            icon_button(
                palette,
                ElementId::named_usize("toast-close", id as usize),
                Icon::X,
                true,
            )
            .on_click(cx.listener(move |this, _, _, cx| this.dismiss_toast(id, cx))),
        );
    let _ = tone;
    crate::motion::toast_in(
        ElementId::named_usize("toast-in", id as usize),
        card,
        crate::motion::reduce_motion(cx),
    )
    .into_any_element()
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn calendar_layer(
    palette: &Theme,
    calendar: &CalendarState,
    content: AnyElement,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    let card = div()
        .id(ElementId::named_usize(
            "calendar-popover",
            calendar.seq as usize,
        ))
        .w(px(330.0))
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(palette.surface)
        .rounded_xl()
        .border_1()
        .border_color(palette.border)
        .shadow(vec![BoxShadow {
            color: palette.shadow.opacity(1.35),
            offset: gpui::point(px(0.0), px(16.0)),
            blur_radius: px(40.0),
            spread_radius: px(0.0),
        }])
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(content);

    let card = crate::motion::menu_in(
        ElementId::named_usize("calendar-in", calendar.seq as usize),
        card,
        crate::motion::reduce_motion(cx),
    );

    div()
        .absolute()
        .inset_0()
        .occlude()
        .flex()
        .items_center()
        .justify_center()
        .p(px(24.0))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| this.calendar_close(cx)),
        )
        .on_mouse_down(
            MouseButton::Navigate(NavigationDirection::Back),
            cx.listener(|this, _, window, cx| this.mouse_back(window, cx)),
        )
        .child(card)
        .into_any_element()
}

fn calendar_content(
    app: &SunderApp,
    palette: &Theme,
    calendar: &CalendarState,
    cx: &mut gpui::Context<SunderApp>,
) -> AnyElement {
    let mut content = div().flex().flex_col().gap(px(10.0)).p(px(12.0));

    if calendar.target == CalendarTarget::FilterRange {
        let mut quick = div().flex().gap(px(6.0));
        for (label, months) in [("1M", 1u32), ("3M", 3), ("6M", 6), ("1Y", 12)] {
            quick = quick.child(
                div()
                    .id(ElementId::Name(format!("calendar-quick-{label}").into()))
                    .debug_selector(|| format!("calendar-quick-{label}"))
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(26.0))
                    .rounded_md()
                    .bg(palette.surface_muted)
                    .text_size(px(type_scale::MICRO))
                    .text_color(palette.muted)
                    .cursor_pointer()
                    .hover(|style| style.bg(palette.primary_tint).text_color(palette.primary))
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.calendar_quick_pick(months, cx)),
                    )
                    .child(label),
            );
        }
        content = content.child(quick);
    }

    let mut header = div().flex().items_center().gap(px(4.0));
    header = header.child(
        icon_button(
            palette,
            ElementId::named_usize("calendar-prev", calendar.seq as usize),
            Icon::ArrowLeft,
            true,
        )
        .debug_selector(|| "calendar-prev".to_string())
        .on_click(cx.listener(|this, _, _, cx| this.calendar_shift(-1, cx))),
    );
    header = header.child(div().flex_1());
    let center = match calendar.view {
        CalendarView::Days => {
            div()
                .flex()
                .items_center()
                .gap(px(2.0))
                .child(
                    div()
                        .id(ElementId::named_usize(
                            "calendar-month-label",
                            calendar.seq as usize,
                        ))
                        .debug_selector(|| "calendar-month-label".to_string())
                        .px(px(8.0))
                        .py(px(4.0))
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.foreground)
                        .hover(|style| style.bg(palette.surface_muted))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.calendar_set_view(CalendarView::Months, cx)
                        }))
                        .child(MONTH_NAMES[(calendar.month - 1) as usize]),
                )
                .child(
                    div()
                        .id(ElementId::named_usize(
                            "calendar-year-label",
                            calendar.seq as usize,
                        ))
                        .debug_selector(|| "calendar-year-label".to_string())
                        .px(px(8.0))
                        .py(px(4.0))
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(type_scale::SMALL))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.foreground)
                        .hover(|style| style.bg(palette.surface_muted))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.calendar_set_view(CalendarView::Years, cx)
                        }))
                        .child(calendar.year.to_string()),
                )
        }
        CalendarView::Months => div().flex().items_center().child(
            div()
                .id(ElementId::named_usize(
                    "calendar-year-label",
                    calendar.seq as usize,
                ))
                .debug_selector(|| "calendar-year-label".to_string())
                .px(px(8.0))
                .py(px(4.0))
                .rounded_md()
                .cursor_pointer()
                .text_size(px(type_scale::SMALL))
                .font_weight(type_scale::MEDIUM)
                .text_color(palette.foreground)
                .hover(|style| style.bg(palette.surface_muted))
                .on_click(
                    cx.listener(|this, _, _, cx| this.calendar_set_view(CalendarView::Years, cx)),
                )
                .child(calendar.year.to_string()),
        ),
        CalendarView::Years => {
            let page_start = calendar.year.div_euclid(12) * 12;
            div()
                .px(px(8.0))
                .py(px(4.0))
                .text_size(px(type_scale::SMALL))
                .font_weight(type_scale::MEDIUM)
                .text_color(palette.foreground)
                .child(format!("{page_start} - {}", page_start + 11))
        }
    };
    header = header.child(center);
    header = header.child(div().flex_1()).child(
        icon_button(
            palette,
            ElementId::named_usize("calendar-next", calendar.seq as usize),
            Icon::ArrowRight,
            true,
        )
        .debug_selector(|| "calendar-next".to_string())
        .on_click(cx.listener(|this, _, _, cx| this.calendar_shift(1, cx))),
    );
    content = content.child(header);

    match calendar.view {
        CalendarView::Days => {
            let mut weekdays = div().grid().grid_cols(7).gap(px(2.0));
            for label in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                weekdays = weekdays.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(22.0))
                        .text_size(px(type_scale::MICRO))
                        .font_weight(type_scale::MEDIUM)
                        .text_color(palette.muted)
                        .child(label),
                );
            }
            content = content.child(weekdays);

            let mut grid = div().grid().grid_cols(7).gap(px(2.0));
            let days = calendar_grid(calendar.year, calendar.month);
            let mut cells = days;
            while !cells.len().is_multiple_of(7) {
                cells.push(None);
            }
            let preview_end = calendar.end.or(calendar.hover);
            for cell in cells {
                let Some(date) = cell else {
                    grid = grid.child(div().h(px(34.0)));
                    continue;
                };
                let is_start = calendar.start == Some(date);
                let is_end = calendar.end == Some(date)
                    || (calendar.end.is_none() && calendar.hover == Some(date));
                let in_range = match (calendar.start, preview_end) {
                    (Some(start), Some(end)) => {
                        let (low, high) = if start <= end {
                            (start, end)
                        } else {
                            (end, start)
                        };
                        date > low && date < high
                    }
                    _ => false,
                };
                let is_today = date == chrono::Local::now().date_naive();
                let selected = is_start || is_end;
                let mut day = div()
                    .id(ElementId::Name(
                        format!("calendar-day-{}-{}", calendar.seq, date.day()).into(),
                    ))
                    .debug_selector(move || format!("calendar-day-{}", date.day()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(34.0))
                    .rounded_md()
                    .cursor_pointer()
                    .text_size(px(type_scale::SMALL))
                    .when(selected, |style| {
                        style
                            .bg(palette.primary)
                            .text_color(palette.on_primary)
                            .font_weight(type_scale::SEMIBOLD)
                    })
                    .when(!selected && in_range, |style| {
                        style.bg(palette.primary_tint).text_color(palette.primary)
                    })
                    .when(!selected && !in_range, |style| {
                        style
                            .text_color(palette.foreground)
                            .hover(|style| style.bg(palette.surface_muted))
                    })
                    .when(is_today && !selected, |style| {
                        style
                            .text_color(palette.primary)
                            .font_weight(type_scale::SEMIBOLD)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| this.calendar_pick(date, cx)))
                    .on_mouse_move(
                        cx.listener(move |this, _, _, cx| this.calendar_hover(Some(date), cx)),
                    )
                    .child(date.day().to_string());
                if in_range {
                    day = day.rounded_none();
                }
                grid = grid.child(day);
            }
            content = content.child(grid);
        }
        CalendarView::Months => {
            let mut grid = div().grid().grid_cols(3).gap(px(4.0)).py(px(4.0));
            for (index, name) in MONTH_NAMES.iter().enumerate() {
                let month = (index + 1) as u32;
                let selected = month == calendar.month;
                grid =
                    grid.child(
                        div()
                            .id(ElementId::Name(format!("calendar-month-{month}").into()))
                            .debug_selector(move || format!("calendar-month-{month}"))
                            .flex()
                            .items_center()
                            .justify_center()
                            .h(px(32.0))
                            .rounded_md()
                            .cursor_pointer()
                            .text_size(px(type_scale::SMALL))
                            .when(selected, |style| {
                                style
                                    .bg(palette.primary)
                                    .text_color(palette.on_primary)
                                    .font_weight(type_scale::SEMIBOLD)
                            })
                            .when(!selected, |style| {
                                style
                                    .text_color(palette.foreground)
                                    .hover(|style| style.bg(palette.surface_muted))
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.calendar_select_month(month, cx)
                            }))
                            .child(&name[..3]),
                    );
            }
            content = content.child(grid);
        }
        CalendarView::Years => {
            let page_start = calendar.year.div_euclid(12) * 12;
            let mut grid = div().grid().grid_cols(3).gap(px(4.0)).py(px(4.0));
            for offset in 0..12 {
                let year = page_start + offset;
                let selected = year == calendar.year;
                grid = grid.child(
                    div()
                        .id(ElementId::Name(format!("calendar-year-{year}").into()))
                        .debug_selector(move || format!("calendar-year-{year}"))
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(32.0))
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(type_scale::SMALL))
                        .when(selected, |style| {
                            style
                                .bg(palette.primary)
                                .text_color(palette.on_primary)
                                .font_weight(type_scale::SEMIBOLD)
                        })
                        .when(!selected, |style| {
                            style
                                .text_color(palette.foreground)
                                .hover(|style| style.bg(palette.surface_muted))
                        })
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.calendar_select_year(year, cx)),
                        )
                        .child(year.to_string()),
                );
            }
            content = content.child(grid);
        }
    }

    let mut footer = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .pt(px(8.0))
        .border_t_1()
        .border_color(palette.border);
    footer = footer.child(
        ghost_button(
            palette,
            ElementId::named_usize("calendar-today", calendar.seq as usize),
            "Today",
            None,
        )
        .debug_selector(|| "calendar-today".to_string())
        .on_click(cx.listener(|this, _, _, cx| this.calendar_today(cx))),
    );
    footer = footer.child(div().flex_1());
    if calendar.target == CalendarTarget::FilterRange {
        footer = footer.child(
            primary_button(
                palette,
                ElementId::named_usize("calendar-done", calendar.seq as usize),
                "Done",
                Some(Icon::Check),
            )
            .debug_selector(|| "calendar-done".to_string())
            .on_click(cx.listener(|this, _, _, cx| this.calendar_done(cx))),
        );
        if calendar.start.is_some() || calendar.end.is_some() {
            footer = footer.child(
                ghost_button(
                    palette,
                    ElementId::named_usize("calendar-clear", calendar.seq as usize),
                    "Clear",
                    None,
                )
                .debug_selector(|| "calendar-clear".to_string())
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(calendar) = this.calendar.as_mut() {
                        calendar.start = None;
                        calendar.end = None;
                        calendar.hover = None;
                    }
                    cx.notify();
                })),
            );
        }
    } else {
        footer = footer.child(
            secondary_button(
                palette,
                ElementId::named_usize("calendar-cancel", calendar.seq as usize),
                "Cancel",
                None,
            )
            .debug_selector(|| "calendar-cancel".to_string())
            .on_click(cx.listener(|this, _, _, cx| this.calendar_close(cx))),
        );
    }
    let _ = app;
    let content = content.child(footer);
    let view = match calendar.view {
        CalendarView::Days => "days",
        CalendarView::Months => "months",
        CalendarView::Years => "years",
    };
    crate::motion::tab_in(
        ElementId::Name(
            format!(
                "calendar-view-{}-{}-{}-{}",
                calendar.seq, view, calendar.year, calendar.month
            )
            .into(),
        ),
        content,
        crate::motion::reduce_motion(cx),
    )
    .into_any_element()
}
