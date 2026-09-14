use chrono::NaiveDate;

use crate::domain::Currency;
use crate::error::{AppError, Result};

/// Parsed CSV row: (statement_date, amount, description, currency)
/// Currency is type-safe enum (§1) — raw strings normalized via sanitize.
pub fn parse_csv(body: &str) -> Result<Vec<(NaiveDate, f64, String, Currency)>> {
    let mut rdr = csv::Reader::from_reader(body.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|e| AppError::Validation(format!("CSV header error: {}", e)))?
        .clone();
    let idx_tanggal = headers
        .iter()
        .position(|h| {
            let l = h.to_lowercase();
            l.trim() == "tanggal" || l.trim() == "statement_date"
        })
        .ok_or(AppError::Validation(
            "CSV harus punya kolom tanggal/statement_date".into(),
        ))?;
    let idx_desc = headers
        .iter()
        .position(|h| {
            let l = h.to_lowercase();
            l.trim() == "deskripsi" || l.trim() == "description"
        })
        .unwrap_or(0);
    let idx_amount = headers
        .iter()
        .position(|h| h.to_lowercase().trim() == "amount")
        .ok_or(AppError::Validation("CSV harus punya kolom amount".into()))?;
    let idx_currency = headers
        .iter()
        .position(|h| h.to_lowercase().trim() == "currency");

    let mut out = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| AppError::Validation(e.to_string()))?;
        let tanggal_str = record.get(idx_tanggal).unwrap_or("").trim();
        if tanggal_str.is_empty() {
            continue;
        }
        let date = NaiveDate::parse_from_str(tanggal_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(tanggal_str, "%d/%m/%Y"))
            .or_else(|_| NaiveDate::parse_from_str(tanggal_str, "%Y/%m/%d"))
            .map_err(|_| AppError::Validation(format!("Invalid date {}", tanggal_str)))?;
        let amount_str = record
            .get(idx_amount)
            .unwrap_or("0")
            .trim()
            .replace(',', "");
        let amount: f64 = amount_str
            .parse()
            .map_err(|_| AppError::Validation(format!("Invalid amount {}", amount_str)))?;
        let description_raw = record.get(idx_desc).unwrap_or("").trim().to_string();
        let currency_raw = idx_currency
            .and_then(|i| record.get(i))
            .unwrap_or("IDR")
            .trim()
            .to_string();
        let currency_raw = if currency_raw.is_empty() {
            "IDR".to_string()
        } else {
            currency_raw
        };
        // Single canonical parser (§10) — no duplicated match logic
        let currency = crate::utils::currency::parse_currency(&currency_raw);
        // Sanitize description (§7): strip control chars via shared helper
        let description_clean = crate::utils::sanitize::sanitize_string(&description_raw);
        out.push((date, amount, description_clean, currency));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn parse_csv_ok() {
        let csv = "tanggal,deskripsi,amount,currency\n2026-08-15,Test,1000000,IDR\n";
        let res = parse_csv(csv).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].0, NaiveDate::from_ymd_opt(2026, 8, 15).unwrap());
    }

    #[test]
    fn parse_csv_invalid_date() {
        let csv = "tanggal,deskripsi,amount\ninvalid,Test,1000\n";
        assert!(parse_csv(csv).is_err());
    }
}
