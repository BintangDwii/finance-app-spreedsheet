import pandas as pd
from utils.formatters import to_idr

def auto_match(transaksi_df: pd.DataFrame, mutation_df: pd.DataFrame, tolerance: float = 1000):
    """
    Return list of (mut_idx, transaksi_id)
    Matching: tanggal selisih <=1 hari & amount selisih < tolerance (dalam IDR)
    Delegates IDR conversion to utils.formatters.to_idr (single source).
    """
    matches = []
    transaksi_df = transaksi_df.copy()
    transaksi_df["tanggal"] = pd.to_datetime(transaksi_df["tanggal"], errors="coerce")
    mutation_df = mutation_df.copy()
    mutation_df["statement_date"] = pd.to_datetime(mutation_df["statement_date"], errors="coerce")
    transaksi_df["idr_amount"] = transaksi_df.apply(lambda r: to_idr(r["total_akhir"], r["currency"], r["fx_rate"]), axis=1)

    used_trans = set()
    for m_idx, m in mutation_df.iterrows():
        if pd.isna(m["statement_date"]) or pd.isna(m["amount"]):
            continue
        candidates = transaksi_df[~transaksi_df["id"].isin(used_trans)]
        for _, t in candidates.iterrows():
            if pd.isna(t["tanggal"]):
                continue
            delta_days = abs((m["statement_date"] - t["tanggal"]).days)
            if delta_days > 1:
                continue
            mut_idr = m["amount"]
            if abs(mut_idr - t["idr_amount"]) < tolerance:
                matches.append((m_idx, int(t["id"])))
                used_trans.add(t["id"])
                break
    return matches
