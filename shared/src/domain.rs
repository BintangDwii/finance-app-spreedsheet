use serde::{Deserialize, Serialize};

// ---------- Enums (serde contract is canonical; sqlx mapping behind `db`) ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "role_t", rename_all = "PascalCase"))]
#[serde(rename_all = "PascalCase")]
pub enum UserRole {
    Maker,
    Checker,
    Approver,
    Admin,
    Viewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(
    feature = "db",
    sqlx(type_name = "status_t", rename_all = "PascalCase")
)]
pub enum TransaksiStatus {
    #[cfg_attr(feature = "db", sqlx(rename = "Pending (Butuh Approval)"))]
    #[serde(rename = "Pending (Butuh Approval)")]
    Pending,
    #[cfg_attr(feature = "db", sqlx(rename = "Approved (Menunggu Bayar)"))]
    #[serde(rename = "Approved (Menunggu Bayar)")]
    Approved,
    #[cfg_attr(feature = "db", sqlx(rename = "Paid (Lunas)"))]
    #[serde(rename = "Paid (Lunas)")]
    Paid,
    #[cfg_attr(feature = "db", sqlx(rename = "Rejected (Ditolak)"))]
    #[serde(rename = "Rejected (Ditolak)")]
    Rejected,
    #[cfg_attr(feature = "db", sqlx(rename = "Reconciled"))]
    #[serde(rename = "Reconciled")]
    Reconciled,
}

impl TransaksiStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending (Butuh Approval)",
            Self::Approved => "Approved (Menunggu Bayar)",
            Self::Paid => "Paid (Lunas)",
            Self::Rejected => "Rejected (Ditolak)",
            Self::Reconciled => "Reconciled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "jenis_t", rename_all = "PascalCase"))]
pub enum Jenis {
    Pemasukan,
    Pengeluaran,
    #[cfg_attr(feature = "db", sqlx(rename = "Transfer Internal"))]
    #[serde(rename = "Transfer Internal")]
    TransferInternal,
}

impl Jenis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pemasukan => "Pemasukan",
            Self::Pengeluaran => "Pengeluaran",
            Self::TransferInternal => "Transfer Internal",
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(
    feature = "db",
    sqlx(type_name = "currency_t", rename_all = "PascalCase")
)]
pub enum Currency {
    // DB + JSON labels are upper-case acronyms, NOT PascalCase ("Idr").
    #[cfg_attr(feature = "db", sqlx(rename = "IDR"))]
    #[serde(rename = "IDR")]
    IDR,
    #[cfg_attr(feature = "db", sqlx(rename = "USD"))]
    #[serde(rename = "USD")]
    USD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(
    feature = "db",
    sqlx(type_name = "account_status_t", rename_all = "PascalCase")
)]
pub enum AccountStatus {
    Aktif,
    Beku,
    Tutup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(
    feature = "db",
    sqlx(type_name = "recon_status_t", rename_all = "PascalCase")
)]
pub enum ReconStatus {
    Unreconciled,
    Reconciled,
    Manual,
}

// ---------- Newtype IDs (§1: prevent primitive obsession) ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct UserId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct TransaksiId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct AccountId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct BudgetId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct ReconciliationId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(transparent))]
pub struct AuditId(pub i32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_role_serde_pascal() {
        let json = serde_json::to_string(&UserRole::Admin).unwrap();
        assert_eq!(json, "\"Admin\"");
        let parsed: UserRole = serde_json::from_str("\"Maker\"").unwrap();
        assert_eq!(parsed, UserRole::Maker);
    }

    #[test]
    fn transaksi_status_serde_with_spaces() {
        let json = serde_json::to_string(&TransaksiStatus::Pending).unwrap();
        assert_eq!(json, "\"Pending (Butuh Approval)\"");
        let parsed: TransaksiStatus = serde_json::from_str("\"Paid (Lunas)\"").unwrap();
        assert_eq!(parsed, TransaksiStatus::Paid);
        assert_eq!(TransaksiStatus::Reconciled.as_str(), "Reconciled");
    }

    #[test]
    fn currency_serde() {
        assert_eq!(serde_json::to_string(&Currency::USD).unwrap(), "\"USD\"");
        assert_eq!(
            serde_json::from_str::<Currency>("\"IDR\"").unwrap(),
            Currency::IDR
        );
    }

    #[test]
    fn newtype_ids_distinct() {
        let uid = UserId(1);
        let tid = TransaksiId(1);
        assert_eq!(uid.0, 1);
        assert_eq!(tid.0, 1);
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(uid);
        assert!(set.contains(&UserId(1)));
        assert!(!set.contains(&UserId(2)));
    }

    #[test]
    fn exhaustive_match_no_wildcard() {
        fn handle(s: TransaksiStatus) -> &'static str {
            match s {
                TransaksiStatus::Pending => "pending",
                TransaksiStatus::Approved => "approved",
                TransaksiStatus::Paid => "paid",
                TransaksiStatus::Rejected => "rejected",
                TransaksiStatus::Reconciled => "reconciled",
            }
        }
        assert_eq!(handle(TransaksiStatus::Paid), "paid");
    }
}
