use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::{AccountId, AccountStatus, Currency};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Account {
    pub id: AccountId,
    pub bank_name: String,
    pub no_rekening: String,
    pub currency: Currency,
    pub opening_balance: f64,
    pub limit_overdraft: f64,
    pub gl_code: Option<String>,
    pub status: AccountStatus,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountRequest {
    #[validate(length(min = 3, max = 100))]
    pub bank_name: String,
    #[validate(length(min = 5, max = 30))]
    pub no_rekening: String,
    pub currency: Currency,
    #[validate(range(min = 0.0))]
    pub opening_balance: f64,
    pub limit_overdraft: Option<f64>,
    #[validate(length(max = 20))]
    pub gl_code: Option<String>,
    pub status: Option<AccountStatus>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAccountStatusRequest {
    pub status: AccountStatus,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SaldoPerRekening {
    pub bank_name: String,
    pub currency: Currency,
    pub opening_balance: f64,
    pub total_masuk: f64,
    pub total_keluar: f64,
    pub saldo: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AccountStatus, Currency};

    #[test]
    fn account_status_serde() {
        assert_eq!(
            serde_json::to_string(&AccountStatus::Aktif).unwrap(),
            "\"Aktif\""
        );
        let parsed: AccountStatus = serde_json::from_str("\"Beku\"").unwrap();
        assert_eq!(parsed, AccountStatus::Beku);
    }

    #[test]
    fn create_account_validate() {
        let req = CreateAccountRequest {
            bank_name: "BCA Corp".to_string(),
            no_rekening: "1234567890".to_string(),
            currency: Currency::IDR,
            opening_balance: 100.0,
            limit_overdraft: None,
            gl_code: Some("1101".to_string()),
            status: Some(AccountStatus::Aktif),
        };
        assert!(req.validate().is_ok());
    }
}
