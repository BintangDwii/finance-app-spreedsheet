use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::{Currency, ReconStatus, ReconciliationId, TransaksiId};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Reconciliation {
    pub id: ReconciliationId,
    pub statement_date: NaiveDate,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: Currency,
    pub matched_transaksi_id: Option<TransaksiId>,
    pub status: ReconStatus,
    pub uploaded_by: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ManualMatchRequest {
    #[validate(range(min = 1))]
    pub transaksi_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ReconStatus;

    #[test]
    fn recon_status_serde() {
        assert_eq!(
            serde_json::to_string(&ReconStatus::Reconciled).unwrap(),
            "\"Reconciled\""
        );
        let parsed: ReconStatus = serde_json::from_str("\"Unreconciled\"").unwrap();
        assert_eq!(parsed, ReconStatus::Unreconciled);
    }

    #[test]
    fn manual_match_validate() {
        let req = ManualMatchRequest { transaksi_id: 1001 };
        assert!(req.validate().is_ok());
        let bad = ManualMatchRequest { transaksi_id: 0 };
        assert!(bad.validate().is_err());
    }
}
