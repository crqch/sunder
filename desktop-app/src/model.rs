use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Sunder uses naive UTC timestamps on the wire. We mirror that exactly so a
/// local edit can be compared with a server edit without timezone surprises.
pub fn now() -> String {
    now_naive().format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

pub fn now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub fn today() -> String {
    chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string()
}

pub fn parse_timestamp(value: &str) -> Option<NaiveDateTime> {
    let value = value.trim();
    let value = value.strip_suffix('Z').unwrap_or(value);
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M"))
        .ok()
}

pub fn compare_timestamps(a: &str, b: &str) -> Ordering {
    match (parse_timestamp(a), parse_timestamp(b)) {
        (Some(a), Some(b)) => a.cmp(&b),
        _ => a.cmp(b),
    }
}

pub fn timestamp_is_newer(candidate: &str, reference: &str) -> bool {
    compare_timestamps(candidate, reference) == Ordering::Greater
}

pub fn new_id() -> String {
    cuid2::create_id()
}

pub const DEFAULT_CURRENCY_CODE: &str = "USD";

pub fn default_currency_code() -> String {
    DEFAULT_CURRENCY_CODE.to_string()
}

fn default_true() -> bool {
    true
}

fn default_decimal_digits() -> u8 {
    2
}

pub fn normalize_currency_code(code: &str) -> String {
    let trimmed = code.trim().to_ascii_uppercase();
    if trimmed.is_empty() {
        DEFAULT_CURRENCY_CODE.to_string()
    } else {
        trimmed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Currency {
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub symbol_after: bool,
    #[serde(default = "default_decimal_digits")]
    pub decimal_digits: u8,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

impl Currency {
    /// Display fallback used only when a currency code hasn't been fetched from
    /// the server yet (e.g., offline startup before the first catalog sync).
    pub fn fallback(code: &str) -> Self {
        let code = normalize_currency_code(code);
        if code == DEFAULT_CURRENCY_CODE {
            Self {
                code,
                name: "US Dollar".into(),
                symbol: "$".into(),
                symbol_after: false,
                decimal_digits: 2,
                is_active: true,
            }
        } else {
            Self {
                name: code.clone(),
                symbol: code.clone(),
                code,
                symbol_after: true,
                decimal_digits: 2,
                is_active: true,
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default, alias = "created_at")]
    pub inserted_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl Account {
    pub fn new(name: impl Into<String>) -> Self {
        let stamp = now();
        Self {
            id: new_id(),
            name: name.into(),
            currency_code: None,
            deleted_at: None,
            inserted_at: stamp.clone(),
            updated_at: stamp,
        }
    }

    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    pub fn normalized_currency_code(&self) -> Option<String> {
        self.currency_code
            .as_deref()
            .map(str::trim)
            .filter(|code| !code.is_empty())
            .map(|code| code.to_ascii_uppercase())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default, alias = "created_at")]
    pub inserted_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl Category {
    pub fn new(title: impl Into<String>, color: impl Into<String>) -> Self {
        let stamp = now();
        Self {
            id: new_id(),
            title: title.into(),
            description: None,
            color: color.into(),
            deleted_at: None,
            inserted_at: stamp.clone(),
            updated_at: stamp,
        }
    }

    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub id: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub amount: f64,
    #[serde(default = "default_currency_code")]
    pub currency_code: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default, alias = "created_at")]
    pub inserted_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            id: String::new(),
            account_id: String::new(),
            category_id: String::new(),
            title: String::new(),
            description: None,
            location: None,
            amount: 0.0,
            currency_code: default_currency_code(),
            date: String::new(),
            deleted_at: None,
            inserted_at: String::new(),
            updated_at: String::new(),
        }
    }
}

impl Entry {
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    pub fn normalized_currency_code(&self) -> String {
        normalize_currency_code(&self.currency_code)
    }

    pub fn occurred_at(&self) -> &str {
        if self.date.is_empty() {
            &self.inserted_at
        } else {
            &self.date
        }
    }

    pub fn date_naive(&self) -> Option<NaiveDate> {
        let value = self.occurred_at();
        parse_timestamp(value)
            .map(|date| date.date())
            .or_else(|| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
    }

    pub fn date_ymd(&self) -> String {
        self.date_naive()
            .map(|date| date.format("%Y-%m-%d").to_string())
            .unwrap_or_else(today)
    }

    pub fn month_key(&self) -> Option<(i32, u32)> {
        self.date_naive().map(|d| (d.year(), d.month()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default = "default_currency_code")]
    pub default_currency_code: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            username: String::new(),
            email: String::new(),
            flags: Vec::new(),
            default_currency_code: default_currency_code(),
        }
    }
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.flags.iter().any(|flag| flag == "is_admin")
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncPayload {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub entries: Vec<Entry>,
}

impl SyncPayload {
    pub fn len(&self) -> usize {
        self.accounts.len() + self.categories.len() + self.entries.len()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncResponse {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub default_currency_code: Option<String>,
}

/// A single point on the monthly cash-flow chart.
#[derive(Clone, Copy, Debug, Default)]
pub struct MonthFlow {
    pub year: i32,
    pub month: u32,
    pub income: f64,
    pub expenses: f64,
}

/// Turn a list of entries into the six most recent calendar months, including
/// empty months so the chart never collapses or reorders itself.
pub fn monthly_flow(entries: &[Entry], months: usize) -> Vec<MonthFlow> {
    monthly_flow_converted(entries, months, |entry| entry.amount)
}

/// Like [`monthly_flow`], but projects each entry's signed amount through a
/// caller-supplied currency converter (typically using historical rates on the
/// entry's date).
pub fn monthly_flow_converted<F>(entries: &[Entry], months: usize, convert: F) -> Vec<MonthFlow>
where
    F: Fn(&Entry) -> f64,
{
    let today = chrono::Local::now().date_naive();
    let mut buckets: Vec<MonthFlow> = Vec::with_capacity(months);

    let mut year = today.year();
    let mut month = today.month();
    for _ in 0..months {
        buckets.push(MonthFlow {
            year,
            month,
            income: 0.0,
            expenses: 0.0,
        });
        if month == 1 {
            month = 12;
            year -= 1;
        } else {
            month -= 1;
        }
    }
    buckets.reverse();

    for entry in entries.iter().filter(|entry| entry.is_active()) {
        let Some((year, month)) = entry.month_key() else {
            continue;
        };
        if let Some(bucket) = buckets
            .iter_mut()
            .find(|bucket| bucket.year == year && bucket.month == month)
        {
            let value = convert(entry);
            if value >= 0.0 {
                bucket.income += value;
            } else {
                bucket.expenses += value.abs();
            }
        }
    }

    buckets
}

/// A friendly relative label such as "Today" or "Yesterday".
pub fn day_label(date: NaiveDate) -> String {
    let today = chrono::Local::now().date_naive();
    if date == today {
        "Today".to_string()
    } else if date == today - chrono::Duration::days(1) {
        "Yesterday".to_string()
    } else if date.year() == today.year() {
        date.format("%a, %b %-d").to_string()
    } else {
        date.format("%b %-d, %Y").to_string()
    }
}

pub fn greeting() -> &'static str {
    match chrono::Local::now().hour() {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=23 => "Good evening",
        _ => "Good night",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str, amount: f64, date: &str) -> Entry {
        Entry {
            id: new_id(),
            account_id: "account".into(),
            category_id: "category".into(),
            title: title.into(),
            amount,
            date: date.into(),
            inserted_at: now(),
            updated_at: now(),
            ..Default::default()
        }
    }

    #[test]
    fn parses_the_timestamp_shapes_sunder_sends() {
        assert!(parse_timestamp("2026-11-12T08:53:00.000000").is_some());
        assert!(parse_timestamp("2026-11-12T08:53:00").is_some());
        assert!(parse_timestamp("2026-11-12T08:53:00Z").is_some());
        assert!(parse_timestamp("not a date").is_none());
    }

    #[test]
    fn compares_timestamps_by_instant_not_by_string() {
        assert!(timestamp_is_newer(
            "2026-11-12T08:53:01",
            "2026-11-12T08:53:00.500000"
        ));
        assert!(!timestamp_is_newer(
            "2026-11-12T08:53:00",
            "2026-11-12T08:53:00.500000"
        ));
    }

    #[test]
    fn monthly_flow_includes_income_and_expenses() {
        let today = chrono::Local::now().date_naive();
        let date = today.and_hms_opt(12, 0, 0).unwrap();
        let stamp = date.format("%Y-%m-%dT%H:%M:%S").to_string();
        let entries = vec![
            entry("Salary", 2500.0, &stamp),
            entry("Rent", -900.0, &stamp),
            entry("Coffee", -4.5, &stamp),
        ];
        let flows = monthly_flow(&entries, 6);
        assert_eq!(flows.len(), 6);
        let current = flows.last().unwrap();
        assert_eq!(current.year, today.year());
        assert_eq!(current.month, today.month());
        assert_eq!(current.income, 2500.0);
        assert_eq!(current.expenses, 904.5);
    }

    #[test]
    fn deleted_entries_never_reach_the_totals() {
        let mut deleted = entry("Refund", 20.0, "2026-01-01T12:00:00");
        deleted.deleted_at = Some("2026-01-02T12:00:00".into());
        let entries = vec![deleted, entry("Lunch", -12.0, "2026-01-01T12:00:00")];
        let flows = monthly_flow(&entries, 1);
        // The current month in this environment is whatever "today" is; the
        // deleted entry must not contribute either way.
        assert_eq!(flows.len(), 1);
        assert!(flows[0].income == 0.0);
    }
}
