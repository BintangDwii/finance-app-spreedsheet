//! Integration tests — end-to-end business flows without DB (§8).
//! Exercises domain + service + utils layers together as a deployed
//! consumer would (via public lib API, not internal unit paths).

use chrono::NaiveDate;
use corporate_finance_api::{
    domain::{Currency, Jenis, ReconStatus, TransaksiId, TransaksiStatus, UserRole},
    service::{csv_service, reconcile_service, transaksi_service},
    utils::{currency, pagination, reference, sanitize},
};

// ---------- §1: type safety end-to-end ----------

#[test]
fn enums_roundtrip_and_exhaustive() {
    // Serde renames with spaces must survive API boundary
    let s = serde_json::to_string(&TransaksiStatus::Paid).unwrap();
    assert_eq!(s, "\"Paid (Lunas)\"");
    let back: TransaksiStatus = serde_json::from_str(&s).unwrap();
    assert_eq!(back, TransaksiStatus::Paid);

    // Jenis::as_str is single source (§10) — must match serde
    assert_eq!(Jenis::TransferInternal.as_str(), "Transfer Internal");
    let j: Jenis = serde_json::from_str("\"Transfer Internal\"").unwrap();
    assert_eq!(j.as_str(), "Transfer Internal");

    // Newtypes are distinct at compile time; runtime inner preserved.
    // Compile-time proof: function accepting TransaksiId rejects UserId.
    fn accepts_transaksi(_: TransaksiId) {}
    accepts_transaksi(TransaksiId(7));
    assert_eq!(TransaksiId(7).0, 7);

    // ReconStatus serde
    assert_eq!(
        serde_json::to_string(&ReconStatus::Manual).unwrap(),
        "\"Manual\""
    );
}

// ---------- §2 + §3: guard_transition flows ----------

#[test]
fn approval_flow_checker_approver_admin() {
    // Small nominal: Checker can approve
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Pending,
        TransaksiStatus::Approved,
        40_000_000.0,
        UserRole::Checker
    )
    .is_ok());

    // Large nominal: Checker blocked, Approver OK
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Pending,
        TransaksiStatus::Approved,
        60_000_000.0,
        UserRole::Checker
    )
    .is_err());
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Pending,
        TransaksiStatus::Approved,
        60_000_000.0,
        UserRole::Approver
    )
    .is_ok());

    // Paid is immutable except Admin bypass
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Paid,
        TransaksiStatus::Rejected,
        1_000.0,
        UserRole::Approver
    )
    .is_err());
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Paid,
        TransaksiStatus::Rejected,
        1_000.0,
        UserRole::Admin
    )
    .is_ok());

    // Rejected -> Pending only Maker (Admin bypasses)
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Rejected,
        TransaksiStatus::Pending,
        1_000.0,
        UserRole::Maker
    )
    .is_ok());
    assert!(transaksi_service::guard_transition(
        TransaksiStatus::Rejected,
        TransaksiStatus::Pending,
        1_000.0,
        UserRole::Checker
    )
    .is_err());
}

#[test]
fn service_pure_helpers_total_and_dates() {
    assert_eq!(transaksi_service::calculate_total(100.0, 11.0, 5.0), 106.0);
    assert!(transaksi_service::parse_tanggal("2026-09-12").is_ok());
    // Strict: no silent fallback (§7)
    assert!(transaksi_service::parse_tanggal("12/09/2026").is_err());
    assert!(transaksi_service::parse_due_date(Some("bad".into())).is_err());
    assert!(transaksi_service::parse_due_date(None).unwrap().is_none());
    assert!(transaksi_service::validate_transfer(Jenis::TransferInternal, Some(" BCA ")).is_ok());
    assert!(transaksi_service::validate_transfer(Jenis::Pemasukan, Some("BCA")).is_err());
}

// ---------- CSV -> match -> audit flow (reconcile E2E without DB) ----------

#[test]
fn csv_parse_then_match_candidates_e2e() {
    let csv = "tanggal,deskripsi,amount,currency\n2026-08-15,Transfer masuk,1000000,IDR\n2026-08-20,Jauh,500,IDR\n";
    let rows = csv_service::parse_csv(csv).unwrap();
    assert_eq!(rows.len(), 2);
    // Currency is type-safe enum (§1)
    assert_eq!(rows[0].3, Currency::IDR);
    // Description sanitized (§7)
    assert!(!rows[0].2.contains('<'));

    let trans = vec![(
        TransaksiId(1001),
        NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        1_000_500.0, // within 1000 tolerance
    )];
    let muts: Vec<(NaiveDate, f64)> = rows.iter().map(|(d, a, _, _)| (*d, *a)).collect();
    let m = reconcile_service::match_candidates(&trans, &muts);
    // First row matches (±1 day, <1000 diff), second does not (date far)
    assert_eq!(m, vec![(0, TransaksiId(1001))]);
}

#[test]
fn csv_rejects_bad_header_and_date() {
    let bad_header = "foo,bar\n1,2\n";
    assert!(csv_service::parse_csv(bad_header).is_err());
    let bad_date = "tanggal,deskripsi,amount\nnot-a-date,X,100\n";
    assert!(csv_service::parse_csv(bad_date).is_err());
}

// ---------- §7 + §10: shared utils ----------

#[test]
fn sanitize_and_pagination_contract() {
    assert_eq!(sanitize::escape_like("100%_\\"), "100\\%\\_\\\\");
    assert_eq!(
        sanitize::sanitize_gl_code(Some("GL-1<script>".into())),
        Some("GL-1script".into())
    );
    assert_eq!(sanitize::sanitize_gl_code(Some("   ".into())), None);

    let (page, per_page, offset) = pagination::normalize_page(Some(0), Some(999));
    assert_eq!((page, per_page, offset), (1, 200, 0));

    // Currency helpers
    assert_eq!(currency::to_idr(100.0, Currency::USD, 16000.0), 1_600_000.0);
    assert_eq!(currency::to_idr(100.0, Currency::IDR, 999.0), 100.0);
    assert_eq!(
        currency::paid_statuses(),
        [TransaksiStatus::Paid, TransaksiStatus::Reconciled]
    );
    // Canonical parser (§10): single rule, type-safe
    assert_eq!(currency::parse_currency("usd"), Currency::USD);
    assert_eq!(currency::parse_currency(" USD "), Currency::USD);
    assert_eq!(currency::parse_currency("idr"), Currency::IDR);
    assert_eq!(currency::parse_currency("anything-else"), Currency::IDR);
    assert_eq!(currency::sanitize_currency("USD"), "USD");
    assert_eq!(currency::sanitize_currency("eur"), "IDR");

    // Reference generator is unique per call prefix
    let r1 = reference::sanitize_reference(None, "REF");
    let r2 = reference::sanitize_reference(Some("  abc  ".into()), "REF");
    assert!(r1.starts_with("REF-"));
    assert_eq!(r2, "abc");
}

// ---------- DTO validation (§7) ----------

#[test]
fn dto_validation_rejects_bad_payload() {
    use corporate_finance_api::models::transaksi::CreateTransaksiRequest;
    use validator::Validate;
    let bad = CreateTransaksiRequest {
        tanggal: "2026-09-12".into(),
        jenis: Jenis::Pengeluaran,
        divisi: "IT".into(), // too short
        kategori: "Lainnya".into(),
        entitas_terkait: "Klien".into(),
        akun_pembayaran: "BCA".into(),
        akun_tujuan: None,
        subtotal: 0.0, // invalid
        ppn_11: None,
        diskon: None,
        currency: None,
        fx_rate: None,
        due_date: None,
        reference_no: None,
        catatan: None,
    };
    assert!(bad.validate().is_err());
}
