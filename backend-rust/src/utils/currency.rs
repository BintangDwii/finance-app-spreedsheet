use crate::domain::{Currency, TransaksiStatus};

/// SQL fragment for paid-status filters — single source for db layers (§10).
/// (Parenthesized `PAID_STATUSES` const removed as dead duplication.)
pub const PAID_STATUSES_SQL: &str = "'Paid (Lunas)','Reconciled'";

/// Type-safe access to paid statuses as enums (§1).
pub fn paid_statuses() -> [TransaksiStatus; 2] {
    [TransaksiStatus::Paid, TransaksiStatus::Reconciled]
}

/// Canonical currency parser (§10): single place for "USD else IDR" rule.
/// Returns type-safe enum; String version below delegates to this.
pub fn parse_currency(s: &str) -> Currency {
    match s.trim().to_uppercase().as_str() {
        "USD" => Currency::USD,
        _ => Currency::IDR,
    }
}

pub fn to_idr(total: f64, currency: Currency, fx_rate: f64) -> f64 {
    // Single implementation lives in shared::format (§10 DRY) — delegate.
    shared::format::to_idr(total, currency, fx_rate)
}

pub fn fx_or_one(fx: Option<f64>) -> f64 {
    fx.unwrap_or(1.0)
}

pub fn currency_or_idr(c: Option<Currency>) -> Currency {
    c.unwrap_or(Currency::IDR)
}

pub fn sanitize_currency(s: &str) -> String {
    match parse_currency(s) {
        Currency::USD => "USD".to_string(),
        Currency::IDR => "IDR".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Currency;

    #[test]
    fn to_idr_usd() {
        assert_eq!(to_idr(100.0, Currency::USD, 16250.0), 1_625_000.0);
        assert_eq!(to_idr(100.0, Currency::IDR, 999.0), 100.0);
    }
}
