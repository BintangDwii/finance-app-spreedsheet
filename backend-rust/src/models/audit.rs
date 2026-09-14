use chrono::NaiveDateTime;
use serde::Serialize;

use crate::domain::{AuditId, TransaksiId};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: AuditId,
    pub transaksi_id: TransaksiId,
    pub actor: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub catatan: Option<String>,
    pub timestamp: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_id_newtype() {
        let id = AuditId(42);
        assert_eq!(id.0, 42);
    }
}
