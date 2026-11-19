//! Power search for the entry list.
//!
//! The search box accepts a small, forgiving query language in addition to
//! plain text:
//!
//! ```text
//! title:coffee amount:>25 type:income category:"Eating out" after:2026-01-01
//! ```
//!
//! Every word is a term and terms are ANDed. Unknown keys stay plain text, so
//! a search for `12:30` still works.

use crate::format;
use crate::model::Entry;
use crate::store::Snapshot;
use chrono::{Datelike, NaiveDate};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchKind {
    Title,
    Note,
    Location,
    Category,
    Account,
    Currency,
    Type,
    Amount,
    Date,
    Text,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchValue {
    Text(String),
    Amount {
        min: Option<f64>,
        max: Option<f64>,
    },
    Date {
        year: i32,
        month: Option<u32>,
        day: Option<u32>,
    },
    Before(NaiveDate),
    After(NaiveDate),
    Income(bool),
}

/// One parsed filter token, including the byte span it came from so the UI can
/// remove it without touching the rest of the query.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchTerm {
    pub kind: SearchKind,
    pub value: SearchValue,
    /// Prefix shown in the chip, e.g. `Title: contains`.
    pub label: &'static str,
    /// Human-readable value shown after the label.
    pub display: String,
    pub span: Range<usize>,
}

impl SearchTerm {
    pub fn matches(&self, entry: &Entry, snapshot: &Snapshot) -> bool {
        let category = snapshot.category_name(&entry.category_id).to_lowercase();
        let account = snapshot.account_name(&entry.account_id).to_lowercase();
        let currency = entry.normalized_currency_code().to_lowercase();
        match (&self.kind, &self.value) {
            (SearchKind::Title, SearchValue::Text(value)) => {
                entry.title.to_lowercase().contains(value)
            }
            (SearchKind::Note, SearchValue::Text(value)) => entry
                .description
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(value),
            (SearchKind::Location, SearchValue::Text(value)) => entry
                .location
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(value),
            (SearchKind::Category, SearchValue::Text(value)) => category.contains(value),
            (SearchKind::Account, SearchValue::Text(value)) => account.contains(value),
            (SearchKind::Currency, SearchValue::Text(value)) => currency.contains(value),
            (SearchKind::Type, SearchValue::Income(income)) => {
                if *income {
                    entry.amount > 0.0
                } else {
                    entry.amount < 0.0
                }
            }
            (SearchKind::Amount, SearchValue::Amount { min, max }) => {
                let amount = entry.amount.abs();
                min.is_none_or(|min| amount >= min) && max.is_none_or(|max| amount <= max)
            }
            (SearchKind::Date, SearchValue::Date { year, month, day }) => {
                let Some(date) = entry.date_naive() else {
                    return false;
                };
                date.year() == *year
                    && month.is_none_or(|month| date.month() == month)
                    && day.is_none_or(|day| date.day() == day)
            }
            (SearchKind::Date, SearchValue::Before(date)) => {
                entry.date_naive().is_some_and(|entry| entry < *date)
            }
            (SearchKind::Date, SearchValue::After(date)) => {
                entry.date_naive().is_some_and(|entry| entry > *date)
            }
            (SearchKind::Text, SearchValue::Text(value)) => {
                entry.title.to_lowercase().contains(value)
                    || entry
                        .description
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(value)
                    || entry
                        .location
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(value)
                    || category.contains(value)
                    || account.contains(value)
                    || currency.contains(value)
            }
            _ => true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntryQuery {
    pub terms: Vec<SearchTerm>,
}

impl EntryQuery {
    pub fn parse(input: &str) -> Self {
        let terms = tokenize(input)
            .into_iter()
            .filter_map(|token| parse_token(&token))
            .collect();
        Self { terms }
    }

    pub fn matches(&self, entry: &Entry, snapshot: &Snapshot) -> bool {
        self.terms.iter().all(|term| term.matches(entry, snapshot))
    }
}

struct Token {
    text: String,
    span: Range<usize>,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();
    let mut start: Option<usize> = None;
    let mut quoted = false;

    for (index, ch) in input.char_indices() {
        if ch == '"' {
            if start.is_none() {
                start = Some(index);
            }
            quoted = !quoted;
            continue;
        }
        if ch.is_whitespace() && !quoted {
            if let Some(start) = start.take() {
                tokens.push(Token {
                    text: std::mem::take(&mut buffer),
                    span: start..index,
                });
            }
            continue;
        }
        if start.is_none() {
            start = Some(index);
        }
        buffer.push(ch);
    }

    if let Some(start) = start {
        tokens.push(Token {
            text: buffer,
            span: start..input.len(),
        });
    }

    tokens
}

fn parse_token(token: &Token) -> Option<SearchTerm> {
    let raw = token.text.trim();
    if raw.is_empty() {
        return None;
    }
    let Some((key, rest)) = raw.split_once(':') else {
        return Some(text_term(
            SearchKind::Text,
            "Search:",
            raw,
            token.span.clone(),
        ));
    };
    let key = key.trim().to_lowercase();
    let rest = rest.trim();
    let value = strip_operator(rest);

    let term = match key.as_str() {
        "title" | "name" => text_value(SearchKind::Title, "Title: contains", value),
        "note" | "description" | "desc" => text_value(SearchKind::Note, "Note: contains", value),
        "location" | "place" | "where" => {
            text_value(SearchKind::Location, "Location: contains", value)
        }
        "category" | "cat" => text_value(SearchKind::Category, "Category: is", value),
        "account" | "acc" => text_value(SearchKind::Account, "Account: is", value),
        "currency" | "cur" => text_value(SearchKind::Currency, "Currency: is", value),
        "type" | "status" | "kind" | "is" => match value.to_lowercase().as_str() {
            "income" | "in" | "incoming" | "credit" | "deposit" => Some(income_term(true)),
            "expense" | "out" | "outgoing" | "debit" | "spend" => Some(income_term(false)),
            _ => None,
        },
        "amount" | "amt" | "value" => parse_amount_filter(value),
        "min" => format::parse_amount(value).map(|min| amount_term(Some(min), None)),
        "max" => format::parse_amount(value).map(|max| amount_term(None, Some(max))),
        "date" | "on" | "month" => parse_date_filter(rest),
        "before" => format::parse_date(value).map(before_term),
        "after" => format::parse_date(value).map(after_term),
        _ => None,
    };

    Some(
        term.unwrap_or_else(|| text_term(SearchKind::Text, "Search:", raw, token.span.clone()))
            .with_span(token.span.clone()),
    )
}

impl SearchTerm {
    fn with_span(mut self, span: Range<usize>) -> Self {
        self.span = span;
        self
    }
}

fn strip_operator(value: &str) -> &str {
    let value = value.trim();
    for prefix in ["contains ", "is ", "equals ", "= "] {
        if let Some(stripped) = value.strip_prefix(prefix) {
            return stripped.trim();
        }
    }
    value
}

fn text_value(kind: SearchKind, label: &'static str, value: &str) -> Option<SearchTerm> {
    if value.is_empty() {
        return None;
    }
    Some(SearchTerm {
        kind,
        value: SearchValue::Text(value.to_lowercase()),
        label,
        display: value.to_string(),
        span: 0..0,
    })
}

fn text_term(kind: SearchKind, label: &'static str, value: &str, span: Range<usize>) -> SearchTerm {
    SearchTerm {
        kind,
        value: SearchValue::Text(value.to_lowercase()),
        label,
        display: value.to_string(),
        span,
    }
}

fn income_term(income: bool) -> SearchTerm {
    SearchTerm {
        kind: SearchKind::Type,
        value: SearchValue::Income(income),
        label: "Type: is",
        display: if income { "Income" } else { "Expense" }.into(),
        span: 0..0,
    }
}

fn amount_term(min: Option<f64>, max: Option<f64>) -> SearchTerm {
    let display = match (min, max) {
        (Some(min), Some(max)) if (min - max).abs() < f64::EPSILON => format!("= {}", trim(min)),
        (Some(min), Some(max)) => format!("{} - {}", trim(min), trim(max)),
        (Some(min), None) => format!(">= {}", trim(min)),
        (None, Some(max)) => format!("<= {}", trim(max)),
        (None, None) => String::new(),
    };
    SearchTerm {
        kind: SearchKind::Amount,
        value: SearchValue::Amount { min, max },
        label: "Amount:",
        display,
        span: 0..0,
    }
}

fn before_term(date: NaiveDate) -> SearchTerm {
    SearchTerm {
        kind: SearchKind::Date,
        value: SearchValue::Before(date),
        label: "Date:",
        display: format!("before {}", date.format("%Y-%m-%d")),
        span: 0..0,
    }
}

fn after_term(date: NaiveDate) -> SearchTerm {
    SearchTerm {
        kind: SearchKind::Date,
        value: SearchValue::After(date),
        label: "Date:",
        display: format!("after {}", date.format("%Y-%m-%d")),
        span: 0..0,
    }
}

fn parse_amount_filter(value: &str) -> Option<SearchTerm> {
    let value = value.trim().replace([',', ' '], "");
    if value.is_empty() {
        return None;
    }
    if let Some((min, max)) = value.split_once("..") {
        let min = format::parse_amount(min)?;
        let max = format::parse_amount(max)?;
        return Some(amount_term(Some(min), Some(max)));
    }
    if let Some((min, max)) = value.split_once('-') {
        let min = format::parse_amount(min)?;
        let max = format::parse_amount(max)?;
        return Some(amount_term(Some(min), Some(max)));
    }
    for (operator, take_min) in [(">=", true), ("<=", false), (">", true), ("<", false)] {
        if let Some(rest) = value.strip_prefix(operator) {
            let amount = format::parse_amount(rest)?;
            return Some(if take_min {
                amount_term(Some(amount), None)
            } else {
                amount_term(None, Some(amount))
            });
        }
    }
    let amount = format::parse_amount(value.strip_prefix('=').unwrap_or(&value))?;
    Some(amount_term(Some(amount), Some(amount)))
}

fn parse_date_filter(value: &str) -> Option<SearchTerm> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix(">=").or_else(|| value.strip_prefix('>')) {
        return format::parse_date(rest.trim()).map(after_term);
    }
    if let Some(rest) = value.strip_prefix("<=").or_else(|| value.strip_prefix('<')) {
        return format::parse_date(rest.trim()).map(before_term);
    }

    let parts = value.split('-').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    let year = parts.first()?.parse::<i32>().ok()?;
    let month = match parts.get(1) {
        Some(month) => Some(month.parse::<u32>().ok()?),
        None => None,
    };
    let day = match parts.get(2) {
        Some(day) => Some(day.parse::<u32>().ok()?),
        None => None,
    };
    if !(1..=9999).contains(&year) {
        return None;
    }
    if let Some(month) = month
        && !(1..=12).contains(&month)
    {
        return None;
    }
    if let (Some(month), Some(day)) = (month, day) {
        NaiveDate::from_ymd_opt(year, month, day)?;
    }

    Some(SearchTerm {
        kind: SearchKind::Date,
        value: SearchValue::Date { year, month, day },
        label: "Date:",
        display: value.to_string(),
        span: 0..0,
    })
}

fn trim(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_entry(title: &str, amount: f64, date: &str) -> Entry {
        Entry {
            id: title.into(),
            account_id: "a1".into(),
            category_id: "c1".into(),
            title: title.into(),
            description: Some("Morning coffee run".into()),
            location: Some("Corner Cafe".into()),
            amount,
            currency_code: crate::model::default_currency_code(),
            date: date.into(),
            deleted_at: None,
            inserted_at: date.into(),
            updated_at: date.into(),
        }
    }

    fn snapshot() -> Snapshot {
        let mut snapshot = Snapshot::default();
        snapshot.accounts.push(crate::model::Account {
            id: "a1".into(),
            name: "Checking".into(),
            ..Default::default()
        });
        snapshot.categories.push(crate::model::Category {
            id: "c1".into(),
            title: "Eating out".into(),
            color: "#22c55e".into(),
            ..Default::default()
        });
        snapshot
    }

    #[test]
    fn parses_plain_text_and_structured_terms() {
        let query =
            EntryQuery::parse("coffee title:latte amount:>25 type:income category:\"Eating out\"");
        assert_eq!(query.terms.len(), 5);
        assert!(matches!(query.terms[0].value, SearchValue::Text(_)));
        assert_eq!(query.terms[0].display, "coffee");
        assert_eq!(query.terms[1].kind, SearchKind::Title);
        assert_eq!(query.terms[2].display, ">= 25");
        assert_eq!(query.terms[3].display, "Income");
        assert_eq!(query.terms[4].display, "Eating out");
    }

    #[test]
    fn unknown_keys_stay_plain_text() {
        let query = EntryQuery::parse("12:30 status:done");
        assert_eq!(query.terms.len(), 2);
        assert_eq!(query.terms[0].label, "Search:");
        assert_eq!(query.terms[0].display, "12:30");
        assert_eq!(query.terms[1].display, "status:done");
    }

    #[test]
    fn queries_filter_entries() {
        let snapshot = snapshot();
        let query = EntryQuery::parse("title:coffee amount:>3 type:expense category:eating");
        let entry = fixture_entry("Coffee", -4.5, "2026-01-15");
        assert!(query.matches(&entry, &snapshot));

        let income = fixture_entry("Salary", 900.0, "2026-01-15");
        assert!(!query.matches(&income, &snapshot));
        let cheap = fixture_entry("Coffee", -2.0, "2026-01-15");
        assert!(!query.matches(&cheap, &snapshot));
    }

    #[test]
    fn date_terms_match_prefixes_and_ranges() {
        let snapshot = snapshot();
        let entry = fixture_entry("Coffee", -4.5, "2026-01-15");
        assert!(EntryQuery::parse("date:2026-01").matches(&entry, &snapshot));
        assert!(EntryQuery::parse("date:2026-01-15").matches(&entry, &snapshot));
        assert!(!EntryQuery::parse("date:2026-02").matches(&entry, &snapshot));
        assert!(EntryQuery::parse("after:2026-01-14").matches(&entry, &snapshot));
        assert!(EntryQuery::parse("before:2026-01-16").matches(&entry, &snapshot));
        assert!(!EntryQuery::parse("before:2026-01-15").matches(&entry, &snapshot));
    }

    #[test]
    fn spans_cover_the_typed_token() {
        let query = EntryQuery::parse("title:coffee amount:>3");
        assert_eq!(
            &"title:coffee amount:>3"[query.terms[0].span.clone()],
            "title:coffee"
        );
        assert_eq!(
            &"title:coffee amount:>3"[query.terms[1].span.clone()],
            "amount:>3"
        );
    }
}

/// One row in the power-search autocomplete menu.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchSuggestion {
    /// Key badge shown at the start of the row (`title`, `category`, …).
    pub badge: String,
    /// Human-readable value or description.
    pub label: String,
    /// Text that replaces the current word when the row is accepted.
    pub insert: String,
    pub start: usize,
    pub end: usize,
}

const KEY_SUGGESTIONS: &[(&str, &str)] = &[
    ("title", "Match the entry title"),
    ("note", "Match the note"),
    ("location", "Match the location"),
    ("category", "Filter by category"),
    ("account", "Filter by account"),
    ("currency", "Filter by currency code"),
    ("amount", "Filter by amount, e.g. amount:>50"),
    ("type", "Income or expense"),
    ("date", "A year, month or day"),
    ("before", "Entries before a date"),
    ("after", "Entries after a date"),
];

fn canonical_key(key: &str) -> &str {
    match key {
        "name" => "title",
        "description" | "desc" => "note",
        "place" | "where" => "location",
        "cat" => "category",
        "acc" => "account",
        "cur" => "currency",
        "amt" | "value" => "amount",
        "status" | "kind" | "is" => "type",
        "on" | "month" => "date",
        other => other,
    }
}

fn word_bounds(input: &str, cursor: usize) -> (usize, usize) {
    let cursor = cursor.min(input.len());
    let mut start = cursor;
    let mut quoted = false;
    for (index, ch) in input[..cursor].char_indices().rev() {
        if ch == '"' {
            quoted = !quoted;
        }
        if ch.is_whitespace() && !quoted {
            start = index + ch.len_utf8();
            break;
        }
        start = index;
    }

    let mut end = cursor;
    let mut quoted = false;
    for (offset, ch) in input[cursor..].char_indices() {
        if ch == '"' {
            quoted = !quoted;
        }
        if ch.is_whitespace() && !quoted {
            break;
        }
        end = cursor + offset + ch.len_utf8();
    }
    (start, end)
}

fn quoted(value: &str) -> bool {
    value.contains(' ')
}

pub fn suggestions(
    input: &str,
    cursor: usize,
    categories: &[String],
    accounts: &[String],
    currencies: &[String],
    titles: &[String],
    locations: &[String],
    show_all_keys: bool,
) -> Vec<SearchSuggestion> {
    let (start, end) = word_bounds(input, cursor);
    let word = &input[start..end];

    let Some((raw_key, raw_value)) = word.split_once(':') else {
        let prefix = word.trim().to_lowercase();
        if !show_all_keys && prefix.len() < 2 {
            return Vec::new();
        }
        return KEY_SUGGESTIONS
            .iter()
            .filter(|(key, _)| show_all_keys || key.starts_with(&prefix))
            .map(|(key, detail)| SearchSuggestion {
                badge: (*key).to_string(),
                label: (*detail).to_string(),
                insert: format!("{key}:"),
                start,
                end,
            })
            .collect();
    };

    let key_lower = raw_key.trim().to_lowercase();
    let key = canonical_key(&key_lower);
    let value = raw_value.trim().trim_matches('"');
    let lower = value.to_lowercase();

    let values: Vec<String> = match key {
        "title" => titles.to_vec(),
        "location" => locations.to_vec(),
        "category" => categories.to_vec(),
        "account" => accounts.to_vec(),
        "currency" => currencies.to_vec(),
        "type" => vec!["Income".to_string(), "Expense".to_string()],
        "amount" => vec![
            ">10".to_string(),
            ">50".to_string(),
            ">100".to_string(),
            "<50".to_string(),
            "<100".to_string(),
        ],
        _ => return Vec::new(),
    };

    let mut matches = values
        .into_iter()
        .filter(|candidate| candidate.to_lowercase().starts_with(&lower))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        matches = match key {
            "title" => titles
                .iter()
                .filter(|candidate| candidate.to_lowercase().contains(&lower))
                .cloned()
                .collect(),
            "location" => locations
                .iter()
                .filter(|candidate| candidate.to_lowercase().contains(&lower))
                .cloned()
                .collect(),
            "category" => categories
                .iter()
                .filter(|candidate| candidate.to_lowercase().contains(&lower))
                .cloned()
                .collect(),
            "account" => accounts
                .iter()
                .filter(|candidate| candidate.to_lowercase().contains(&lower))
                .cloned()
                .collect(),
            "currency" => currencies
                .iter()
                .filter(|candidate| candidate.to_lowercase().contains(&lower))
                .cloned()
                .collect(),
            _ => Vec::new(),
        };
    }
    matches.truncate(10);

    matches
        .into_iter()
        .map(|candidate| {
            let inserted = if quoted(&candidate) {
                format!("{key}:\"{candidate}\" ")
            } else {
                format!("{key}:{candidate} ")
            };
            SearchSuggestion {
                badge: key.to_string(),
                label: candidate,
                insert: inserted,
                start,
                end,
            }
        })
        .collect()
}

#[cfg(test)]
mod suggestion_tests {
    use super::*;

    #[test]
    fn suggests_keys_after_a_prefix() {
        let list = suggestions("tit", 3, &[], &[], &[], &[], &[], false);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].badge, "title");
        assert_eq!(list[0].insert, "title:");
        assert_eq!(&"tit"[list[0].start..list[0].end], "tit");
    }

    #[test]
    fn suggests_category_values_and_quotes_spaces() {
        let categories = vec!["Eating out".to_string(), "Groceries".to_string()];
        let list = suggestions("category:Ea", 11, &categories, &[], &[], &[], &[], false);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].label, "Eating out");
        assert_eq!(list[0].insert, "category:\"Eating out\" ");
    }

    #[test]
    fn ctrl_space_can_show_all_keys_for_an_empty_word() {
        let list = suggestions("", 0, &[], &[], &[], &[], &[], true);
        assert_eq!(list.len(), KEY_SUGGESTIONS.len());
        assert_eq!(list[0].badge, "title");
        assert!(suggestions("", 0, &[], &[], &[], &[], &[], false).is_empty());
    }

    #[test]
    fn suggests_existing_titles() {
        let titles = vec!["Coffee".to_string(), "Coffee beans".to_string()];
        let list = suggestions("title:co", 8, &[], &[], &[], &titles, &[], false);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].insert, "title:Coffee ");
        assert_eq!(list[1].insert, "title:\"Coffee beans\" ");
    }

    #[test]
    fn suggests_types_and_amounts() {
        let list = suggestions("type:in", 7, &[], &[], &[], &[], &[], false);
        assert_eq!(list[0].label, "Income");
        assert_eq!(list[0].insert, "type:Income ");

        let list = suggestions("amount:>", 9, &[], &[], &[], &[], &[], false);
        assert!(list.iter().any(|item| item.label == ">50"));
    }

    #[test]
    fn suggests_currency_key_and_values() {
        let currencies = vec!["EUR".to_string(), "PLN".to_string(), "USD".to_string()];
        let key_list = suggestions("cur", 3, &[], &[], &currencies, &[], &[], false);
        assert_eq!(key_list.len(), 1);
        assert_eq!(key_list[0].badge, "currency");
        assert_eq!(key_list[0].insert, "currency:");

        let value_list = suggestions("cur:eu", 6, &[], &[], &currencies, &[], &[], false);
        assert_eq!(value_list.len(), 1);
        assert_eq!(value_list[0].badge, "currency");
        assert_eq!(value_list[0].label, "EUR");
        assert_eq!(value_list[0].insert, "currency:EUR ");
    }
}
