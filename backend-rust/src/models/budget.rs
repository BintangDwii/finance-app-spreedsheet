use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::{BudgetId, Currency};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Budget {
    pub id: BudgetId,
    pub bulan: String,
    pub divisi: String,
    pub kategori: String,
    pub planned: f64,
    pub currency: Currency,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertBudgetRequest {
    #[validate(length(min = 7, max = 7))]
    #[validate(custom(function = "validate_bulan"))]
    pub bulan: String, // YYYY-MM
    #[validate(length(min = 3))]
    pub divisi: String,
    #[validate(length(min = 3))]
    pub kategori: String,
    #[validate(range(min = 0.01))]
    pub planned: f64,
    pub currency: Option<Currency>,
}

fn validate_bulan(bulan: &str) -> Result<(), validator::ValidationError> {
    if chrono::NaiveDate::parse_from_str(&format!("{}-01", bulan), "%Y-%m-%d").is_err() {
        return Err(validator::ValidationError::new("bulan must be YYYY-MM"));
    }
    Ok(())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VarianceRow {
    pub bulan: String,
    pub divisi: String,
    pub kategori: String,
    pub planned: f64,
    pub actual: f64,
    pub sisa: f64,
    pub burn: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_validate_ok() {
        let req = UpsertBudgetRequest {
            bulan: "2026-08".to_string(),
            divisi: "IT & Engineering".to_string(),
            kategori: "Software & Cloud (AWS/GCP)".to_string(),
            planned: 1_000_000.0,
            currency: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn budget_validate_fail_bulan() {
        let req = UpsertBudgetRequest {
            bulan: "2026/08".to_string(),
            divisi: "IT".to_string(), // too short
            kategori: "Lainnya".to_string(),
            planned: -100.0,
            currency: None,
        };
        assert!(req.validate().is_err());
    }
}
