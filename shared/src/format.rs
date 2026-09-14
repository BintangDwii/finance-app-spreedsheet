//! Pure display/math helpers shared by API and Leptos frontend (§10).
//! No I/O, no framework dependency — safe for WASM.

use crate::domain::Currency;

/// Convert any amount to IDR.
pub fn to_idr(amount: f64, currency: Currency, fx_rate: f64) -> f64 {
    match currency {
        Currency::USD => amount * fx_rate,
        Currency::IDR => amount,
    }
}

/// `calculate_total = subtotal + ppn - diskon` (single source with backend service).
pub fn calculate_total(subtotal: f64, ppn: f64, diskon: f64) -> f64 {
    subtotal + ppn - diskon
}

/// Format as `Rp1.234.567` (thousands separated by `.`, no decimals).
#[allow(clippy::manual_is_multiple_of)]
pub fn format_idr(amount: f64) -> String {
    let n = amount.round() as i64;
    let neg = n < 0;
    let digits: Vec<char> = n.abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(*c);
    }
    format!("{}Rp{}", if neg { "-" } else { "" }, out)
}

/// Burn rate percent `actual/planned*100`, `None` when planned <= 0.
pub fn burn_rate(actual: f64, planned: f64) -> Option<f64> {
    if planned <= 0.0 {
        None
    } else {
        Some(actual / planned * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idr_math() {
        assert_eq!(to_idr(100.0, Currency::USD, 16250.0), 1_625_000.0);
        assert_eq!(to_idr(100.0, Currency::IDR, 999.0), 100.0);
        assert_eq!(calculate_total(100.0, 11.0, 5.0), 106.0);
    }

    #[test]
    fn idr_format() {
        assert_eq!(format_idr(1_625_000.0), "Rp1.625.000");
        assert_eq!(format_idr(0.0), "Rp0");
        assert_eq!(format_idr(-5000.0), "-Rp5.000");
    }

    #[test]
    fn burn() {
        assert_eq!(burn_rate(50.0, 100.0), Some(50.0));
        assert_eq!(burn_rate(10.0, 0.0), None);
    }
}
