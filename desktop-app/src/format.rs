use crate::model::{Currency, MonthFlow, parse_timestamp};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CurrencyPosition {
    #[default]
    Before,
    After,
}

fn with_symbol(number: String, symbol: &str, position: CurrencyPosition) -> String {
    match position {
        CurrencyPosition::Before => format!("{symbol}{number}"),
        CurrencyPosition::After if symbol.is_empty() => number,
        CurrencyPosition::After => format!("{number} {symbol}"),
    }
}

fn group_whole(whole: u64) -> String {
    let mut grouped = String::new();
    let digits = whole.to_string();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped
}

/// Format a value with grouped thousands and `decimal_digits` decimal places.
pub fn number_with_digits(value: f64, decimal_digits: u8) -> String {
    let digits = (decimal_digits as usize).min(6);
    let negative = value < 0.0;
    let absolute = value.abs();
    let scale = 10_f64.powi(digits as i32);
    let rounded = (absolute * scale).round() as u64;
    let divisor = 10_u64.pow(digits as u32);
    let whole = rounded / divisor;
    let frac = rounded % divisor;

    let grouped = group_whole(whole);
    if digits == 0 {
        format!("{}{}", if negative { "-" } else { "" }, grouped)
    } else {
        format!(
            "{}{}.{:0width$}",
            if negative { "-" } else { "" },
            grouped,
            frac,
            width = digits
        )
    }
}

/// Format a value with grouped thousands and exactly two decimal places.
pub fn number(value: f64) -> String {
    number_with_digits(value, 2)
}

pub fn money(value: f64, symbol: &str, position: CurrencyPosition) -> String {
    with_symbol(number(value.abs()), symbol, position)
}

pub fn signed_money(value: f64, symbol: &str, position: CurrencyPosition) -> String {
    let sign = if value > 0.0 {
        "+"
    } else if value < 0.0 {
        "-"
    } else {
        ""
    };
    let body = with_symbol(number(value.abs()), symbol, position);
    format!("{sign}{body}")
}

pub fn compact_money(value: f64, symbol: &str, position: CurrencyPosition) -> String {
    let absolute = value.abs();
    let sign = if value < 0.0 { "-" } else { "" };
    let body = if absolute >= 1_000_000.0 {
        format!("{:.1}M", absolute / 1_000_000.0)
    } else if absolute >= 10_000.0 {
        format!("{:.0}k", absolute / 1_000.0)
    } else if absolute >= 1_000.0 {
        format!("{:.1}k", absolute / 1_000.0)
    } else {
        format!("{:.0}", absolute)
    };
    format!("{sign}{}", with_symbol(body, symbol, position))
}

fn currency_position(currency: &Currency) -> CurrencyPosition {
    if currency.symbol_after {
        CurrencyPosition::After
    } else {
        CurrencyPosition::Before
    }
}

pub fn money_in(value: f64, currency: &Currency) -> String {
    with_symbol(
        number_with_digits(value.abs(), currency.decimal_digits),
        &currency.symbol,
        currency_position(currency),
    )
}

pub fn signed_money_in(value: f64, currency: &Currency) -> String {
    let sign = if value > 0.0 {
        "+"
    } else if value < 0.0 {
        "-"
    } else {
        ""
    };
    let body = with_symbol(
        number_with_digits(value.abs(), currency.decimal_digits),
        &currency.symbol,
        currency_position(currency),
    );
    format!("{sign}{body}")
}

pub fn compact_money_in(value: f64, currency: &Currency) -> String {
    compact_money(value, &currency.symbol, currency_position(currency))
}

/// Amounts are stored as signed floats. The editor captures a positive number
/// plus an Expense/Income toggle, so we strip anything that is not a digit or a
/// decimal point before parsing.
pub fn parse_amount_with_digits(value: &str, decimal_digits: u8) -> Option<f64> {
    let cleaned: String = value
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    let mut parts = cleaned.split('.');
    let whole = parts.next().unwrap_or("0");
    let fraction = parts.next().unwrap_or("");
    let digits = (decimal_digits as usize).min(6);
    if digits == 0 {
        return if whole.is_empty() { "0" } else { whole }
            .parse::<f64>()
            .ok();
    }
    let normalized = format!(
        "{}.{}",
        if whole.is_empty() { "0" } else { whole },
        fraction.chars().take(digits).collect::<String>()
    );
    normalized.parse::<f64>().ok()
}

pub fn parse_amount(value: &str) -> Option<f64> {
    parse_amount_with_digits(value, 2)
}

pub fn parse_date(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(date);
    }
    parse_timestamp(value).map(|stamp| stamp.date())
}

pub fn month_day(value: &str) -> String {
    parse_date(value)
        .map(|date| date.format("%b %-d").to_string())
        .unwrap_or_else(|| "—".to_string())
}

pub fn month_short(flow: &MonthFlow) -> String {
    NaiveDate::from_ymd_opt(flow.year, flow.month, 1)
        .map(|date| date.format("%b").to_string())
        .unwrap_or_else(|| "—".to_string())
}

pub fn initials(value: &str) -> String {
    let mut letters = value
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>();
    if letters.is_empty() {
        letters = value.chars().take(2).collect();
    }
    letters.to_uppercase()
}

/// The platform's primary shortcut modifier, for display in the UI.
pub fn modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}

pub fn plural(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}

pub fn relative_from_now(value: &str) -> String {
    let Some(timestamp) = parse_timestamp(value) else {
        return "never".into();
    };
    let now = chrono::Utc::now().naive_utc();
    let seconds = (now - timestamp).num_seconds();
    if seconds < 0 {
        return "just now".into();
    }
    if seconds < 60 {
        "just now".into()
    } else if seconds < 3_600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 86_400 {
        format!("{}h ago", seconds / 3_600)
    } else if seconds < 604_800 {
        format!("{}d ago", seconds / 86_400)
    } else {
        timestamp.format("%b %-d").to_string()
    }
}

pub fn today_label() -> String {
    chrono::Local::now().format("%A, %B %-d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands_and_rounds_cents() {
        assert_eq!(number(0.0), "0.00");
        assert_eq!(number(12.3), "12.30");
        assert_eq!(number(1234.567), "1,234.57");
        assert_eq!(number(-1234.5), "-1,234.50");
        assert_eq!(number(1_000_000.0), "1,000,000.00");
    }

    #[test]
    fn formats_signed_money_for_the_ledger() {
        assert_eq!(signed_money(12.5, "$", CurrencyPosition::Before), "+$12.50");
        assert_eq!(
            signed_money(-12.5, "$", CurrencyPosition::Before),
            "-$12.50"
        );
        assert_eq!(signed_money(0.0, "€", CurrencyPosition::Before), "€0.00");
        assert_eq!(money(12.5, "$", CurrencyPosition::Before), "$12.50");
        assert_eq!(money(12.5, "€", CurrencyPosition::After), "12.50 €");
        assert_eq!(
            signed_money(12.5, "kr", CurrencyPosition::After),
            "+12.50 kr"
        );
        assert_eq!(
            signed_money(-12.5, "kr", CurrencyPosition::After),
            "-12.50 kr"
        );
        assert_eq!(
            compact_money(12_000.0, "$", CurrencyPosition::Before),
            "$12k"
        );
        assert_eq!(
            compact_money(1_500.0, "€", CurrencyPosition::After),
            "1.5k €"
        );
    }

    #[test]
    fn accepts_friendly_amount_input() {
        assert_eq!(parse_amount("1,234.56"), Some(1234.56));
        assert_eq!(parse_amount(" $12.50 "), Some(12.5));
        assert_eq!(parse_amount("12.345"), Some(12.34));
        assert_eq!(parse_amount(""), None);
        assert_eq!(parse_amount("not money"), None);
    }

    #[test]
    fn parses_dates_and_timestamps() {
        assert_eq!(
            parse_date("2026-11-12"),
            NaiveDate::from_ymd_opt(2026, 11, 12)
        );
        assert_eq!(
            parse_date("2026-11-12T18:30:00"),
            NaiveDate::from_ymd_opt(2026, 11, 12)
        );
        assert_eq!(parse_date("nope"), None);
    }

    #[test]
    fn formats_currency_metadata_with_digits_and_symbol_position() {
        let jpy = Currency {
            code: "JPY".into(),
            name: "Japanese Yen".into(),
            symbol: "¥".into(),
            symbol_after: false,
            decimal_digits: 0,
            is_active: true,
        };
        let pln = Currency {
            code: "PLN".into(),
            name: "Polish Zloty".into(),
            symbol: "zł".into(),
            symbol_after: true,
            decimal_digits: 2,
            is_active: true,
        };
        let kwd = Currency {
            code: "KWD".into(),
            name: "Kuwaiti Dinar".into(),
            symbol: "KD".into(),
            symbol_after: true,
            decimal_digits: 3,
            is_active: true,
        };
        assert_eq!(money_in(1234.6, &jpy), "¥1,235");
        assert_eq!(signed_money_in(-1234.6, &jpy), "-¥1,235");
        assert_eq!(money_in(42.5, &pln), "42.50 zł");
        assert_eq!(signed_money_in(42.5, &pln), "+42.50 zł");
        assert_eq!(money_in(1.2345, &kwd), "1.235 KD");
    }
}
