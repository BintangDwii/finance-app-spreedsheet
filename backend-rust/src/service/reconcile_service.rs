use chrono::NaiveDate;

use crate::domain::TransaksiId;

/// High-performance matching — pure logic, no framework coupling
/// Returns Vec<(mut_idx, transaksi_id)> where amount diff < 1000 and date ±1
#[tracing::instrument(skip(trans, mutations))]
pub fn match_candidates(
    trans: &[(TransaksiId, NaiveDate, f64)],
    mutations: &[(NaiveDate, f64)],
) -> Vec<(usize, TransaksiId)> {
    let mut matches = Vec::new();
    let mut used = std::collections::HashSet::new();
    for (m_idx, (m_date, m_amt)) in mutations.iter().enumerate() {
        let mut best: Option<TransaksiId> = None;
        for (tid, t_date, t_amt) in trans.iter() {
            if used.contains(tid) {
                continue;
            }
            let delta = (*m_date - *t_date).num_days().abs();
            if delta > 1 {
                continue;
            }
            if (*m_amt - *t_amt).abs() < 1000.0 {
                best = Some(*tid);
                break;
            }
        }
        if let Some(tid) = best {
            matches.push((m_idx, tid));
            used.insert(tid);
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn match_within_tolerance() {
        let trans = vec![(
            TransaksiId(1001),
            NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
            1_000_000.0,
        )];
        let muts = vec![(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(), 1_000_500.0)];
        let m = match_candidates(&trans, &muts);
        assert_eq!(m, vec![(0, TransaksiId(1001))]);
    }

    #[test]
    fn no_match_outside_tolerance() {
        let trans = vec![(
            TransaksiId(1001),
            NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
            1_000_000.0,
        )];
        let muts = vec![(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(), 2_000_000.0)];
        assert!(match_candidates(&trans, &muts).is_empty());
    }

    #[test]
    fn no_match_date_far() {
        let trans = vec![(
            TransaksiId(1001),
            NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
            1_000_000.0,
        )];
        let muts = vec![(NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(), 1_000_000.0)];
        assert!(match_candidates(&trans, &muts).is_empty());
    }
}
