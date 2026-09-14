use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::{Currency, Jenis, TransaksiId, TransaksiStatus};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Transaksi {
    pub id: TransaksiId,
    pub timestamp: String,
    pub tanggal: NaiveDate,
    pub jenis: Jenis,
    pub divisi: String,
    pub kategori: String,
    pub entitas_terkait: String,
    pub akun_pembayaran: String,
    pub subtotal: f64,
    pub ppn_11: f64,
    pub diskon: f64,
    pub total_akhir: f64,
    pub currency: Currency,
    pub fx_rate: f64,
    pub due_date: Option<NaiveDate>,
    pub reference_no: Option<String>,
    pub status: TransaksiStatus,
    pub file_bukti: Option<String>,
    pub catatan: Option<String>,
    pub created_by: Option<String>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_deleted: bool,
    pub idr_amount: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTransaksiRequest {
    #[validate(length(min = 1))]
    pub tanggal: String, // validated as YYYY-MM-DD in service
    pub jenis: Jenis,
    #[validate(length(min = 3))]
    pub divisi: String,
    #[validate(length(min = 3))]
    pub kategori: String,
    #[validate(length(min = 2))]
    pub entitas_terkait: String,
    #[validate(length(min = 2))]
    pub akun_pembayaran: String,
    pub akun_tujuan: Option<String>,
    #[validate(range(min = 0.01))]
    pub subtotal: f64,
    pub ppn_11: Option<f64>,
    pub diskon: Option<f64>,
    pub currency: Option<Currency>,
    pub fx_rate: Option<f64>,
    pub due_date: Option<String>,
    pub reference_no: Option<String>,
    #[validate(length(max = 500))]
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStatusRequest {
    pub new_status: TransaksiStatus,
    #[validate(length(max = 500))]
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    pub status: Option<TransaksiStatus>,
    pub divisi: Option<String>,
    pub jenis: Option<Jenis>,
    #[validate(length(max = 100))]
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SetBuktiRequest {
    #[validate(length(min = 3, max = 255))]
    pub file_bukti: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Currency, Jenis};

    #[test]
    fn create_request_validation_ok() {
        let req = CreateTransaksiRequest {
            tanggal: "2026-09-12".to_string(),
            jenis: Jenis::Pemasukan,
            divisi: "IT & Engineering".to_string(),
            kategori: "Project/Client".to_string(),
            entitas_terkait: "Klien A".to_string(),
            akun_pembayaran: "BCA Corporate".to_string(),
            akun_tujuan: None,
            subtotal: 1_000_000.0,
            ppn_11: None,
            diskon: None,
            currency: Some(Currency::IDR),
            fx_rate: None,
            due_date: None,
            reference_no: None,
            catatan: Some("test".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_request_validation_fail_short_divisi() {
        let req = CreateTransaksiRequest {
            tanggal: "2026-09-12".to_string(),
            jenis: Jenis::Pengeluaran,
            divisi: "IT".to_string(), // too short
            kategori: "Lainnya".to_string(),
            entitas_terkait: "Klien A".to_string(),
            akun_pembayaran: "BCA".to_string(),
            akun_tujuan: None,
            subtotal: 0.0, // invalid range
            ppn_11: None,
            diskon: None,
            currency: None,
            fx_rate: None,
            due_date: None,
            reference_no: None,
            catatan: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn jenis_serde() {
        let j: Jenis = serde_json::from_str("\"Transfer Internal\"").unwrap();
        assert_eq!(j, Jenis::TransferInternal);
        assert_eq!(
            serde_json::to_string(&Jenis::Pemasukan).unwrap(),
            "\"Pemasukan\""
        );
    }
}
