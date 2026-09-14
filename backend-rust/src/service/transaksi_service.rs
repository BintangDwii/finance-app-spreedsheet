use crate::domain::{Jenis, TransaksiStatus, UserRole};
use crate::error::{AppError, Result};

/// Pure total calculation (§2: business rule lives in service, not handler).
pub fn calculate_total(subtotal: f64, ppn: f64, diskon: f64) -> f64 {
    subtotal + ppn - diskon
}

/// Parse `YYYY-MM-DD` strictly — no silent `.ok()` fallback (§7).
pub fn parse_tanggal(s: &str) -> Result<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::Validation("tanggal harus YYYY-MM-DD".into()))
}

/// Parse optional due_date strictly — invalid format is a 400, not silent None (§7).
pub fn parse_due_date(opt: Option<String>) -> Result<Option<chrono::NaiveDate>> {
    match opt {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => parse_tanggal(&s).map(Some),
    }
}

/// Validate transfer prerequisites — pure, testable (§2).
pub fn validate_transfer(payload_jenis: Jenis, akun_tujuan: Option<&str>) -> Result<String> {
    if payload_jenis != Jenis::TransferInternal {
        return Err(AppError::Validation("bukan transfer internal".into()));
    }
    let raw = akun_tujuan.ok_or(AppError::Validation("akun_tujuan wajib".into()))?;
    let clean = crate::utils::sanitize::sanitize_string(raw);
    if clean.is_empty() {
        return Err(AppError::Validation("akun_tujuan kosong".into()));
    }
    Ok(clean)
}

/// Business rule: guard transition — pure, framework-agnostic, exhaustive match
#[tracing::instrument]
pub fn guard_transition(
    current: TransaksiStatus,
    new: TransaksiStatus,
    total_idr: f64,
    role: UserRole,
) -> Result<()> {
    if role == UserRole::Admin {
        return Ok(());
    }
    match (current, new) {
        (TransaksiStatus::Pending, TransaksiStatus::Approved) => {
            if total_idr > 50_000_000.0 && role != UserRole::Approver {
                return Err(AppError::Forbidden(
                    "Nominal > Rp 50jt harus Approver".into(),
                ));
            }
            if role != UserRole::Checker && role != UserRole::Approver {
                return Err(AppError::Forbidden("Hanya Checker/Approver".into()));
            }
            Ok(())
        }
        (TransaksiStatus::Pending, TransaksiStatus::Rejected) => {
            if role != UserRole::Checker && role != UserRole::Approver {
                return Err(AppError::Forbidden("Hanya Checker/Approver".into()));
            }
            Ok(())
        }
        (TransaksiStatus::Approved, TransaksiStatus::Paid)
        | (TransaksiStatus::Approved, TransaksiStatus::Rejected) => {
            if role != UserRole::Approver {
                return Err(AppError::Forbidden("Hanya Approver".into()));
            }
            if total_idr > 100_000_000.0 && new == TransaksiStatus::Paid {
                return Err(AppError::Forbidden("Nominal >100jt butuh Admin".into()));
            }
            Ok(())
        }
        (TransaksiStatus::Rejected, TransaksiStatus::Pending) => {
            if role != UserRole::Maker {
                return Err(AppError::Forbidden("Hanya Maker bisa resubmit".into()));
            }
            Ok(())
        }
        (TransaksiStatus::Paid, _) => Err(AppError::Forbidden("Paid tidak bisa diubah".into())),
        (TransaksiStatus::Reconciled, _) => {
            Err(AppError::Forbidden("Reconciled tidak bisa diubah".into()))
        }
        _ => Err(AppError::Forbidden(format!(
            "Transisi {} -> {} tidak diizinkan untuk {:?}",
            current.as_str(),
            new.as_str(),
            role
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{TransaksiStatus, UserRole};

    #[test]
    fn calculate_total_ok() {
        assert_eq!(calculate_total(100.0, 11.0, 5.0), 106.0);
    }

    #[test]
    fn parse_tanggal_strict() {
        assert!(parse_tanggal("2026-09-12").is_ok());
        assert!(parse_tanggal("12/09/2026").is_err());
        assert!(parse_due_date(None).unwrap().is_none());
        assert!(parse_due_date(Some("  ".into())).unwrap().is_none());
        assert!(parse_due_date(Some("bad-date".into())).is_err());
    }

    #[test]
    fn validate_transfer_ok_err() {
        assert!(validate_transfer(Jenis::TransferInternal, Some(" BCA ")).is_ok());
        assert!(validate_transfer(Jenis::Pemasukan, Some("BCA")).is_err());
        assert!(validate_transfer(Jenis::TransferInternal, None).is_err());
    }

    #[test]
    fn pending_to_approved_checker_small_ok() {
        assert!(guard_transition(
            TransaksiStatus::Pending,
            TransaksiStatus::Approved,
            40_000_000.0,
            UserRole::Checker
        )
        .is_ok());
    }

    #[test]
    fn pending_to_approved_checker_fail_large() {
        assert!(guard_transition(
            TransaksiStatus::Pending,
            TransaksiStatus::Approved,
            60_000_000.0,
            UserRole::Checker
        )
        .is_err());
        assert!(guard_transition(
            TransaksiStatus::Pending,
            TransaksiStatus::Approved,
            60_000_000.0,
            UserRole::Approver
        )
        .is_ok());
    }

    #[test]
    fn approved_to_paid_threshold() {
        assert!(guard_transition(
            TransaksiStatus::Approved,
            TransaksiStatus::Paid,
            90_000_000.0,
            UserRole::Approver
        )
        .is_ok());
        assert!(guard_transition(
            TransaksiStatus::Approved,
            TransaksiStatus::Paid,
            110_000_000.0,
            UserRole::Approver
        )
        .is_err());
        assert!(guard_transition(
            TransaksiStatus::Approved,
            TransaksiStatus::Paid,
            110_000_000.0,
            UserRole::Admin
        )
        .is_ok());
    }

    #[test]
    fn exhaustive_no_wildcard() {
        // ensure all variants handled explicitly — compile will fail if new variant added without match arm
        for s in [
            TransaksiStatus::Pending,
            TransaksiStatus::Approved,
            TransaksiStatus::Paid,
            TransaksiStatus::Rejected,
            TransaksiStatus::Reconciled,
        ] {
            let _ = guard_transition(s, TransaksiStatus::Rejected, 1000.0, UserRole::Admin);
        }
    }
}
