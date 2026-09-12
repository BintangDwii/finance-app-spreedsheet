import pandas as pd
from datetime import datetime

def auto_match(transaksi_df: pd.DataFrame, mutation_df: pd.DataFrame, tolerance: float = 1000):
    """
    transaksi_df: harus ada kolom id, tanggal, total_akhir, currency, fx_rate, status
    mutation_df: kolom statement_date, amount, currency
    Return list of (mut_idx, transaksi_id)
    Matching: tanggal selisih <=1 hari & amount selisih < tolerance (dalam IDR)
    """
    matches = []
    # normalisasi
    transaksi_df = transaksi_df.copy()
    transaksi_df["tanggal"] = pd.to_datetime(transaksi_df["tanggal"], errors="coerce")
    mutation_df = mutation_df.copy()
    mutation_df["statement_date"] = pd.to_datetime(mutation_df["statement_date"], errors="coerce")
    # hitung IDR amount transaksi
    def to_idr_row(r):
        if r["currency"] == "USD":
            return r["total_akhir"] * r["fx_rate"]
        return r["total_akhir"]
    transaksi_df["idr_amount"] = transaksi_df.apply(to_idr_row, axis=1)

    used_trans = set()
    for m_idx, m in mutation_df.iterrows():
        if pd.isna(m["statement_date"]) or pd.isna(m["amount"]):
            continue
        # filter transaksi Paid/Reconciled yang belum terpakai
        candidates = transaksi_df[~transaksi_df["id"].isin(used_trans)]
        # juga filter amount mirip
        for _, t in candidates.iterrows():
            if pd.isna(t["tanggal"]):
                continue
            delta_days = abs((m["statement_date"] - t["tanggal"]).days)
            if delta_days > 1:
                continue
            # amount: mutasi bisa IDR, transaksi IDR amount
            mut_idr = m["amount"]  # diasumsikan CSV dalam IDR (jika USD, perlu fx — disederhanakan)
            if abs(mut_idr - t["idr_amount"]) < tolerance:
                matches.append((m_idx, int(t["id"])))
                used_trans.add(t["id"])
                break
    return matches
