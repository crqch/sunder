use crate::format::CurrencyPosition;
use crate::model::{
    Account, Category, Currency, DEFAULT_CURRENCY_CODE, Entry, SyncPayload, SyncResponse,
    compare_timestamps, default_currency_code, normalize_currency_code, timestamp_is_newer, today,
};
use crate::theme::ThemeMode;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_BASE_URL: &str = "http://localhost:4000";

/// Everything the app remembers between launches.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub base_url: String,
    pub refresh_token: Option<String>,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub user_flags: Vec<String>,
    /// Which account owns the local snapshot. Signing into a different
    /// account must never upload (or display) someone else's data.
    pub owner: String,
    /// The server the local snapshot belongs to. Switching servers must not
    /// leak or overwrite the previous server's ledger.
    pub owner_base_url: String,
    pub last_sync: Option<String>,
    pub theme: ThemeMode,
    /// User's chosen default display currency code (synced with server preferences).
    #[serde(default = "default_currency_code")]
    pub default_currency_code: String,
    pub currency_symbol: String,
    pub currency_position: CurrencyPosition,
    pub reduce_motion: bool,
    pub auto_sync: bool,
    /// Minutes between background pulls when auto-sync is enabled.
    pub auto_sync_interval_minutes: u32,
    /// Whether the persistent sync-status bar is collapsed.
    pub sync_bar_minimized: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base_url: std::env::var("SUNDER_API_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
            refresh_token: None,
            username: String::new(),
            email: String::new(),
            user_flags: Vec::new(),
            owner: String::new(),
            owner_base_url: String::new(),
            last_sync: None,
            theme: ThemeMode::Light,
            default_currency_code: default_currency_code(),
            currency_symbol: "$".into(),
            currency_position: CurrencyPosition::Before,
            reduce_motion: false,
            auto_sync: true,
            auto_sync_interval_minutes: 5,
            sync_bar_minimized: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        write_private(path, &bytes)
    }

    pub fn normalized_base_url(&self) -> String {
        normalize_base_url(&self.base_url)
    }
}

pub fn normalize_base_url(value: &str) -> String {
    let mut value = value.trim().trim_end_matches('/').to_string();
    if value.is_empty() {
        return DEFAULT_BASE_URL.into();
    }
    if !value.starts_with("http://") && !value.starts_with("https://") {
        value = format!("http://{value}");
    }
    value
}

/// The local-first data store. This is deliberately a compact JSON snapshot:
/// Sunder datasets are small, and a single atomic file is easier to reason
/// about (and to back up) than a database for a v1 desktop client.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub entries: Vec<Entry>,
    /// Server-managed active currency catalog cached for offline use.
    #[serde(default)]
    pub currencies: Vec<Currency>,
    /// Historical USD-based exchange rates keyed by YYYY-MM-DD -> (CODE -> rate per 1 USD).
    #[serde(default)]
    pub exchange_rates: BTreeMap<String, HashMap<String, f64>>,
}

impl Snapshot {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        write_private(path, &bytes)
    }

    pub fn active_accounts(&self) -> Vec<&Account> {
        let mut accounts: Vec<_> = self.accounts.iter().filter(|a| a.is_active()).collect();
        accounts.sort_by_key(|a| a.name.to_lowercase());
        accounts
    }

    pub fn active_categories(&self) -> Vec<&Category> {
        let mut categories: Vec<_> = self
            .categories
            .iter()
            .filter(|category| category.is_active())
            .collect();
        categories.sort_by_key(|a| a.title.to_lowercase());
        categories
    }

    pub fn active_entries(&self) -> Vec<&Entry> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry.is_active())
            .collect();
        entries.sort_by(|a, b| {
            compare_timestamps(b.occurred_at(), a.occurred_at())
                .then_with(|| compare_timestamps(&b.inserted_at, &a.inserted_at))
        });
        entries
    }

    pub fn account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|account| account.id == id)
    }

    pub fn category(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|category| category.id == id)
    }

    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn currency(&self, code: &str) -> Currency {
        let code = normalize_currency_code(code);
        self.currencies
            .iter()
            .find(|currency| currency.code.eq_ignore_ascii_case(&code))
            .cloned()
            .unwrap_or_else(|| Currency::fallback(&code))
    }

    pub fn active_currencies(&self) -> Vec<&Currency> {
        let mut list: Vec<_> = self.currencies.iter().filter(|c| c.is_active).collect();
        list.sort_by(|a, b| a.code.cmp(&b.code));
        list
    }

    /// Merge freshly fetched active currencies into the local cache while keeping
    /// previously cached currencies that are no longer returned marked inactive so
    /// historical entries retain their symbol, name, and decimal digits.
    pub fn merge_currencies(&mut self, incoming: Vec<Currency>) {
        if incoming.is_empty() {
            return;
        }
        let mut merged: HashMap<String, Currency> = HashMap::new();
        for mut existing in std::mem::take(&mut self.currencies) {
            let code = normalize_currency_code(&existing.code);
            existing.code = code.clone();
            existing.is_active = false;
            merged.insert(code, existing);
        }
        for mut currency in incoming {
            let code = normalize_currency_code(&currency.code);
            currency.code = code.clone();
            merged.insert(code, currency);
        }
        let mut list: Vec<Currency> = merged.into_values().collect();
        list.sort_by(|a, b| a.code.cmp(&b.code));
        self.currencies = list;
    }

    pub fn merge_rates(&mut self, date: String, rates: HashMap<String, f64>) {
        if date.trim().is_empty() || rates.is_empty() {
            return;
        }
        let bucket = self.exchange_rates.entry(date).or_default();
        for (code, rate) in rates {
            if rate.is_finite() && rate > 0.0 {
                bucket.insert(normalize_currency_code(&code), rate);
            }
        }
        bucket.insert(DEFAULT_CURRENCY_CODE.to_string(), 1.0);
    }

    pub fn merge_rates_history(&mut self, history: HashMap<String, HashMap<String, f64>>) {
        for (date, rates) in history {
            self.merge_rates(date, rates);
        }
    }

    /// Find the USD -> `code` rate for `date_ymd`, searching nearest earlier date first,
    /// then falling back to nearest later date, independently per currency so a sparse
    /// rate day (such as a single manual admin override) cannot shadow earlier full days.
    fn find_usd_rate_for_currency(&self, code: &str, date_ymd: &str) -> Option<f64> {
        if code == DEFAULT_CURRENCY_CODE {
            return Some(1.0);
        }
        let key = date_ymd.to_string();
        for (_, rates) in self.exchange_rates.range(..=key.clone()).rev() {
            if let Some(&rate) = rates.get(code)
                && rate.is_finite()
                && rate > 0.0
            {
                return Some(rate);
            }
        }
        for (_, rates) in self.exchange_rates.range(key..) {
            if let Some(&rate) = rates.get(code)
                && rate.is_finite()
                && rate > 0.0
            {
                return Some(rate);
            }
        }
        None
    }

    /// Look up the conversion rate from `from` to `to` on `date_ymd` (YYYY-MM-DD),
    /// resolving each currency independently via exact date, nearest earlier cached date,
    /// or nearest later cached date as fallback.
    pub fn rate_for(&self, from: &str, to: &str, date_ymd: &str) -> Option<f64> {
        let from = normalize_currency_code(from);
        let to = normalize_currency_code(to);
        if from == to {
            return Some(1.0);
        }
        let rate_from = self.find_usd_rate_for_currency(&from, date_ymd)?;
        let rate_to = self.find_usd_rate_for_currency(&to, date_ymd)?;
        if rate_from <= 0.0 || rate_to <= 0.0 {
            return None;
        }
        Some(rate_to / rate_from)
    }

    pub fn convert_amount(&self, amount: f64, from: &str, to: &str, date_ymd: &str) -> Option<f64> {
        self.rate_for(from, to, date_ymd).map(|rate| amount * rate)
    }

    pub fn convert_entry_amount(&self, entry: &Entry, target_currency: &str) -> f64 {
        let from = entry.normalized_currency_code();
        let date = entry.date_ymd();
        self.convert_amount(entry.amount, &from, target_currency, &date)
            .unwrap_or(entry.amount)
    }

    /// Date bounds covering all active entries (capped to today) for fetching historical rates.
    pub fn rate_date_bounds(&self) -> Option<(String, String)> {
        let today_ymd = today();
        let mut earliest: Option<String> = None;
        for entry in self.entries.iter().filter(|entry| entry.is_active()) {
            let ymd = entry.date_ymd();
            if earliest.as_ref().is_none_or(|cur| ymd < *cur) {
                earliest = Some(ymd);
            }
        }
        let from = earliest.unwrap_or_else(|| today_ymd.clone());
        let from = if from > today_ymd {
            today_ymd.clone()
        } else {
            from
        };
        Some((from, today_ymd))
    }

    pub fn account_name(&self, id: &str) -> String {
        self.account(id)
            .map(|account| account.name.clone())
            .unwrap_or_else(|| "Unknown account".into())
    }

    pub fn category_name(&self, id: &str) -> String {
        self.category(id)
            .map(|category| category.title.clone())
            .unwrap_or_else(|| "Uncategorized".into())
    }

    pub fn category_color(&self, id: &str) -> String {
        self.category(id)
            .map(|category| category.color.clone())
            .unwrap_or_else(|| "#8D93A6".into())
    }

    pub fn balance(&self) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active())
            .map(|entry| entry.amount)
            .sum()
    }

    pub fn converted_balance(&self, target_currency: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active())
            .map(|entry| self.convert_entry_amount(entry, target_currency))
            .sum()
    }

    pub fn income(&self) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.amount > 0.0)
            .map(|entry| entry.amount)
            .sum()
    }

    pub fn converted_income(&self, target_currency: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.amount > 0.0)
            .map(|entry| self.convert_entry_amount(entry, target_currency))
            .sum()
    }

    pub fn expenses(&self) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.amount < 0.0)
            .map(|entry| entry.amount.abs())
            .sum()
    }

    pub fn converted_expenses(&self, target_currency: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.amount < 0.0)
            .map(|entry| self.convert_entry_amount(entry, target_currency).abs())
            .sum()
    }

    pub fn account_balance(&self, account_id: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.account_id == account_id)
            .map(|entry| entry.amount)
            .sum()
    }

    pub fn converted_account_balance(&self, account_id: &str, target_currency: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.account_id == account_id)
            .map(|entry| self.convert_entry_amount(entry, target_currency))
            .sum()
    }

    pub fn account_entry_count(&self, account_id: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.account_id == account_id)
            .count()
    }

    pub fn category_spend(&self, category_id: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| {
                entry.is_active() && entry.category_id == category_id && entry.amount < 0.0
            })
            .map(|entry| entry.amount.abs())
            .sum()
    }

    pub fn converted_category_spend(&self, category_id: &str, target_currency: &str) -> f64 {
        self.entries
            .iter()
            .filter(|entry| {
                entry.is_active() && entry.category_id == category_id && entry.amount < 0.0
            })
            .map(|entry| self.convert_entry_amount(entry, target_currency).abs())
            .sum()
    }

    pub fn category_entry_count(&self, category_id: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_active() && entry.category_id == category_id)
            .count()
    }

    pub fn upsert_account(&mut self, account: Account) {
        upsert_by_id(&mut self.accounts, account, |a| &a.id);
    }

    pub fn upsert_category(&mut self, category: Category) {
        upsert_by_id(&mut self.categories, category, |c| &c.id);
    }

    pub fn upsert_entry(&mut self, entry: Entry) {
        upsert_by_id(&mut self.entries, entry, |e| &e.id);
    }

    /// Apply a sync response, never letting a stale server row clobber a newer
    /// local edit. The server resolves conflicts client-wins, but a request can
    /// race with a fresh local edit — this keeps us honest.
    pub fn apply_sync_response(&mut self, response: SyncResponse) {
        for account in response.accounts {
            let local_updated = self
                .accounts
                .iter()
                .find(|local| local.id == account.id)
                .map(|local| local.updated_at.as_str());
            if should_replace(local_updated, &account.updated_at) {
                self.upsert_account(account);
            }
        }
        for category in response.categories {
            let local_updated = self
                .categories
                .iter()
                .find(|local| local.id == category.id)
                .map(|local| local.updated_at.as_str());
            if should_replace(local_updated, &category.updated_at) {
                self.upsert_category(category);
            }
        }
        for entry in response.entries {
            let local_updated = self
                .entries
                .iter()
                .find(|local| local.id == entry.id)
                .map(|local| local.updated_at.as_str());
            if should_replace(local_updated, &entry.updated_at) {
                self.upsert_entry(entry);
            }
        }
    }

    /// Changes since the last successful sync, including tombstones.
    pub fn dirty_since(&self, last_sync: Option<&str>) -> SyncPayload {
        let is_dirty = |updated_at: &str| match last_sync {
            None => true,
            Some(last) => timestamp_is_newer(updated_at, last) || updated_at.is_empty(),
        };

        SyncPayload {
            accounts: self
                .accounts
                .iter()
                .filter(|account| is_dirty(&account.updated_at))
                .cloned()
                .collect(),
            categories: self
                .categories
                .iter()
                .filter(|category| is_dirty(&category.updated_at))
                .cloned()
                .collect(),
            entries: self
                .entries
                .iter()
                .filter(|entry| is_dirty(&entry.updated_at))
                .cloned()
                .collect(),
        }
    }
}

fn should_replace(local_updated: Option<&str>, remote_updated: &str) -> bool {
    match local_updated {
        None => true,
        Some(local) => compare_timestamps(local, remote_updated) != Ordering::Greater,
    }
}

fn upsert_by_id<T>(items: &mut Vec<T>, value: T, id: impl Fn(&T) -> &str) {
    let value_id = id(&value).to_string();
    if let Some(existing) = items.iter_mut().find(|item| id(item) == value_id) {
        *existing = value;
    } else {
        items.push(value);
    }
}

#[derive(Clone)]
pub struct Paths {
    pub data_file: PathBuf,
    pub settings_file: PathBuf,
    pub root: PathBuf,
}

impl Paths {
    pub fn discover() -> Result<Self> {
        let (root, config) = if let Ok(dir) = std::env::var("SUNDER_DATA_DIR") {
            let root = PathBuf::from(dir);
            (root.clone(), root)
        } else {
            let dirs = directories::ProjectDirs::from("dev", "crqch", "sunder")
                .context("could not determine a data directory for Sunder")?;
            (
                dirs.data_local_dir().to_path_buf(),
                dirs.config_dir().to_path_buf(),
            )
        };

        fs::create_dir_all(&root)?;
        fs::create_dir_all(&config)?;
        restrict_dir(&root);
        restrict_dir(&config);

        Ok(Self {
            data_file: root.join("data.json"),
            settings_file: config.join("settings.json"),
            root,
        })
    }

    pub fn location_label(&self) -> String {
        self.root.display().to_string()
    }
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    restrict_file(&tmp);
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Windows cannot rename over an existing file.
            let _ = fs::remove_file(path);
            fs::rename(&tmp, path)?;
            Ok(())
        }
    }
}

#[cfg(unix)]
fn restrict_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file(_path: &Path) {}

#[cfg(unix)]
fn restrict_dir(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
fn restrict_dir(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Account, Category, Entry};

    fn entry(id: &str, updated_at: &str, amount: f64) -> Entry {
        Entry {
            id: id.into(),
            account_id: "a".into(),
            category_id: "c".into(),
            title: id.into(),
            amount,
            date: updated_at.into(),
            inserted_at: updated_at.into(),
            updated_at: updated_at.into(),
            ..Default::default()
        }
    }

    #[test]
    fn normalizes_friendly_server_urls() {
        assert_eq!(
            normalize_base_url("localhost:4000"),
            "http://localhost:4000"
        );
        assert_eq!(
            normalize_base_url("https://sunder.example.com/"),
            "https://sunder.example.com"
        );
        assert_eq!(normalize_base_url("  "), DEFAULT_BASE_URL);
    }

    #[test]
    fn dirty_records_are_the_ones_after_the_last_sync() {
        let mut snapshot = Snapshot::default();
        snapshot
            .entries
            .push(entry("old", "2026-01-01T10:00:00", 1.0));
        snapshot
            .entries
            .push(entry("new", "2026-01-01T12:00:00", 2.0));

        let payload = snapshot.dirty_since(Some("2026-01-01T11:00:00"));
        assert_eq!(payload.entries.len(), 1);
        assert_eq!(payload.entries[0].id, "new");

        let first_sync = snapshot.dirty_since(None);
        assert_eq!(first_sync.entries.len(), 2);
    }

    #[test]
    fn sync_never_clobbers_a_newer_local_edit() {
        let mut snapshot = Snapshot::default();
        snapshot
            .entries
            .push(entry("one", "2026-01-02T10:00:00", 1.0));

        snapshot.apply_sync_response(SyncResponse {
            entries: vec![entry("one", "2026-01-01T10:00:00", -50.0)],
            ..Default::default()
        });
        assert_eq!(snapshot.entries[0].amount, 1.0);

        snapshot.apply_sync_response(SyncResponse {
            entries: vec![entry("one", "2026-01-03T10:00:00", 9.0)],
            ..Default::default()
        });
        assert_eq!(snapshot.entries[0].amount, 9.0);
    }

    #[test]
    fn balances_and_category_spend_ignore_tombstones() {
        let mut snapshot = Snapshot::default();
        snapshot.accounts.push(Account::new("Checking"));
        snapshot.categories.push(Category::new("Food", "#ef4444"));
        let account = snapshot.accounts[0].id.clone();
        let category = snapshot.categories[0].id.clone();

        let mut lunch = entry("lunch", "2026-01-01T10:00:00", -12.0);
        lunch.account_id = account.clone();
        lunch.category_id = category.clone();
        let mut salary = entry("salary", "2026-01-01T10:00:00", 500.0);
        salary.account_id = account.clone();
        salary.category_id = category.clone();
        let mut gone = entry("gone", "2026-01-01T10:00:00", -100.0);
        gone.deleted_at = Some("2026-01-02T10:00:00".into());

        snapshot.entries.extend([lunch, salary, gone]);

        assert_eq!(snapshot.balance(), 488.0);
        assert_eq!(snapshot.account_balance(&account), 488.0);
        assert_eq!(snapshot.category_spend(&category), 12.0);
        assert_eq!(snapshot.account_entry_count(&account), 2);
    }

    #[test]
    fn snapshots_round_trip_privately() {
        let dir = std::env::temp_dir().join(format!("sunder-store-{}", crate::model::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("data.json");
        let mut snapshot = Snapshot::default();
        snapshot
            .entries
            .push(entry("one", "2026-01-01T10:00:00", 3.5));
        snapshot.save(&path).unwrap();

        let loaded = Snapshot::load(&path);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].id, "one");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn converts_multi_currency_entries_using_historical_rates() {
        let mut snapshot = Snapshot::default();
        snapshot.merge_rates(
            "2026-01-01".into(),
            HashMap::from([("EUR".into(), 0.8), ("PLN".into(), 4.0)]),
        );
        snapshot.merge_rates(
            "2026-01-10".into(),
            HashMap::from([("EUR".into(), 0.9), ("PLN".into(), 4.5)]),
        );

        let mut e1 = entry("e1", "2026-01-05T12:00:00", -80.0);
        e1.currency_code = "EUR".into();
        let mut e2 = entry("e2", "2026-01-12T12:00:00", 100.0);
        e2.currency_code = "USD".into();
        snapshot.entries.extend([e1, e2]);

        // On 2026-01-05, nearest earlier rate is 2026-01-01 (EUR=0.8 -> -80 EUR = -100 USD).
        // On 2026-01-12, +100 USD = +100 USD. Net in USD = 0.0.
        assert!((snapshot.converted_balance("USD") - 0.0).abs() < 1e-6);

        // In PLN: -80 EUR on Jan 5 -> -80 * (4.0/0.8) = -400 PLN; +100 USD on Jan 12 -> +450 PLN; net = +50 PLN.
        assert!((snapshot.converted_balance("PLN") - 50.0).abs() < 1e-6);
    }

    #[test]
    fn legacy_snapshot_defaults_entry_currency_to_usd() {
        let raw = r#"{"accounts":[{"id":"a1","name":"Cash"}],"categories":[],"entries":[{"id":"e1","account_id":"a1","category_id":"c1","title":"Old","amount":-5.0,"date":"2026-01-01T12:00:00"}]}"#;
        let snapshot: Snapshot = serde_json::from_str(raw).unwrap();
        assert_eq!(snapshot.entries[0].currency_code, "USD");
        assert_eq!(snapshot.accounts[0].currency_code, None);
    }

    #[test]
    fn sparse_rate_day_does_not_shadow_earlier_rates_and_missing_returns_none() {
        let mut snapshot = Snapshot::default();
        // Full rate day on Jan 1
        snapshot.merge_rates(
            "2026-01-01".into(),
            HashMap::from([("EUR".into(), 0.8), ("PLN".into(), 4.0)]),
        );
        // Sparse manual override day on Jan 5 (only EUR)
        snapshot.merge_rates("2026-01-05".into(), HashMap::from([("EUR".into(), 0.9)]));

        // On Jan 6, EUR comes from Jan 5 (0.9) and PLN falls back to Jan 1 (4.0)
        let eur_to_pln = snapshot.rate_for("EUR", "PLN", "2026-01-06").unwrap();
        assert!((eur_to_pln - (4.0 / 0.9)).abs() < 1e-6);

        // Earlier than any cached date falls back to nearest later date per currency
        let earlier = snapshot.rate_for("EUR", "USD", "2025-12-20").unwrap();
        assert!((earlier - (1.0 / 0.8)).abs() < 1e-6);

        // Currency absent everywhere returns None
        assert_eq!(snapshot.rate_for("JPY", "USD", "2026-01-06"), None);
        assert_eq!(snapshot.convert_amount(100.0, "JPY", "USD", "2026-01-06"), None);
    }

    #[test]
    fn merge_currencies_keeps_deactivated_currency_metadata() {
        let mut snapshot = Snapshot::default();
        snapshot.currencies = vec![
            Currency {
                code: "USD".into(),
                name: "US Dollar".into(),
                symbol: "$".into(),
                symbol_after: false,
                decimal_digits: 2,
                is_active: true,
            },
            Currency {
                code: "PLN".into(),
                name: "Polish Zloty".into(),
                symbol: "zł".into(),
                symbol_after: true,
                decimal_digits: 2,
                is_active: true,
            },
        ];

        // Server refresh returns only active USD
        snapshot.merge_currencies(vec![Currency {
            code: "USD".into(),
            name: "US Dollar".into(),
            symbol: "$".into(),
            symbol_after: false,
            decimal_digits: 2,
            is_active: true,
        }]);

        assert_eq!(snapshot.active_currencies().len(), 1);
        let pln = snapshot.currency("PLN");
        assert_eq!(pln.symbol, "zł");
        assert!(!pln.is_active);
    }
}

#[cfg(test)]
mod settings_tests {
    use super::*;

    #[test]
    fn settings_round_trip_sync_bar_and_intervals() {
        let dir = std::env::temp_dir().join(format!("sunder-settings-{}", crate::model::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        let settings = Settings {
            auto_sync_interval_minutes: 15,
            sync_bar_minimized: true,
            currency_symbol: "€".into(),
            currency_position: crate::format::CurrencyPosition::After,
            reduce_motion: true,
            ..Settings::default()
        };
        settings.save(&path).unwrap();

        let loaded = Settings::load(&path);
        assert_eq!(loaded.auto_sync_interval_minutes, 15);
        assert!(loaded.sync_bar_minimized);
        assert_eq!(loaded.currency_symbol, "€");
        assert_eq!(
            loaded.currency_position,
            crate::format::CurrencyPosition::After
        );
        assert!(loaded.reduce_motion);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
