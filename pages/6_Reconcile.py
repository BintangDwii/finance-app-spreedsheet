import os
import pandas as pd
import streamlit as st
from sqlalchemy import text
from datetime import datetime
from database.connection import get_cached_engine, query_df
from utils.reconciler import auto_match
from utils.auth import current_user

st.title("🔄 Rekonsiliasi Bank")
st.caption("Upload mutasi CSV (format BCA/Mandiri) — auto-match Tanggal ±1 hari & amount.")
st.markdown("---")

user = current_user()

# Template download
tmpl_path = "data/template_mutasi.csv"
if os.path.exists(tmpl_path):
    with open(tmpl_path, "rb") as f:
        st.download_button("📥 Download Template CSV", f.read(), "template_mutasi.csv", "text/csv", width="stretch")
st.info("Format CSV wajib: `tanggal,deskripsi,amount,currency,no_referensi` — contoh: 2026-08-15,Transfer masuk Klien A,73260000,IDR,REF-1004/202608")

# Upload
uploaded = st.file_uploader("Upload Mutasi CSV", type=["csv"])
if uploaded is not None:
    try:
        mut_df = pd.read_csv(uploaded)
        # normalisasi kolom lowercase
        mut_df.columns = [c.strip().lower() for c in mut_df.columns]
        required = {"tanggal","deskripsi","amount"}
        if not required.issubset(set(mut_df.columns)):
            st.error(f"CSV harus punya kolom: {required}. Kolom terdeteksi: {list(mut_df.columns)}")
            st.stop()
        # rename tanggal -> statement_date
        if "statement_date" not in mut_df.columns:
            mut_df["statement_date"] = mut_df["tanggal"]
        if "currency" not in mut_df.columns:
            mut_df["currency"] = "IDR"
        # pastikan amount numeric
        mut_df["amount"] = pd.to_numeric(mut_df["amount"], errors="coerce")
        st.markdown("#### Preview Mutasi")
        st.dataframe(mut_df.head(20), width="stretch", hide_index=True)
        st.caption(f"Total {len(mut_df)} baris, total amount Rp {mut_df['amount'].sum():,.0f}")

        # Load transaksi Paid/Reconciled untuk matching
        trans_df = query_df("SELECT id, tanggal, total_akhir, currency, fx_rate, status FROM transaksi WHERE is_deleted=0 AND status IN ('Paid (Lunas)','Reconciled')")
        if trans_df.empty:
            st.warning("Tidak ada transaksi Paid untuk di-match.")
        else:
            # tolerance IDR 1000
            matches = auto_match(trans_df, mut_df, tolerance=1000)
            st.markdown(f"**Auto-match ditemukan: {len(matches)} dari {len(mut_df)}**")
            if matches:
                # buat tabel hasil
                rows = []
                for m_idx, tid in matches:
                    m = mut_df.iloc[m_idx]
                    t = trans_df[trans_df["id"]==tid].iloc[0]
                    rows.append({"mut_idx": m_idx, "mutasi_tanggal": str(m["statement_date"])[:10], "mutasi_deskripsi": m["deskripsi"], "mutasi_amount": m["amount"], "transaksi_id": tid, "transaksi_tanggal": t["tanggal"], "transaksi_total": t["total_akhir"], "trans_currency": t["currency"]})
                res_df = pd.DataFrame(rows)
                st.dataframe(res_df, width="stretch", hide_index=True)
                # Simpan ke reconciliation table
                if st.button("✅ Konfirmasi Rekonsiliasi (Tandai Reconciled)", type="primary", width="stretch"):
                    engine = get_cached_engine()
                    with engine.begin() as conn:
                        for r in rows:
                            # insert reconciliation
                            conn.execute(text("""
                                INSERT INTO reconciliation (statement_date, description, amount, currency, matched_transaksi_id, status, uploaded_by)
                                VALUES (:sd, :desc, :amt, :cur, :tid, 'Reconciled', :by)
                            """), {"sd": str(r["mutasi_tanggal"]), "desc": r["mutasi_deskripsi"], "amt": float(r["mutasi_amount"]), "cur": "IDR", "tid": int(r["transaksi_id"]), "by": user["username"]})
                            # update transaksi status
                            conn.execute(text("UPDATE transaksi SET status='Reconciled', updated_at=datetime('now','localtime') WHERE id=:id"), {"id": int(r["transaksi_id"])})
                            conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, 'Paid (Lunas)', 'Reconciled', 'Auto-match CSV')"),
                                         {"tid": int(r["transaksi_id"]), "actor": user["username"]})
                        # simpan yang unmatched sebagai Unreconciled
                        matched_idx = {m[0] for m in matches}
                        for idx, m in mut_df.iterrows():
                            if idx not in matched_idx:
                                conn.execute(text("""
                                    INSERT INTO reconciliation (statement_date, description, amount, currency, status, uploaded_by)
                                    VALUES (:sd, :desc, :amt, :cur, 'Unreconciled', :by)
                                """), {"sd": str(m["statement_date"])[:10], "desc": str(m["deskripsi"]), "amt": float(m["amount"]), "cur": str(m.get("currency","IDR")), "by": user["username"]})
                    st.success(f"{len(rows)} transaksi ditandai Reconciled, {len(mut_df)-len(rows)} Unreconciled disimpan.")
                    st.rerun()
            else:
                st.warning("Tidak ada match otomatis. Cek tolerance atau tanggal.")
                # tetap simpan semua sebagai Unreconciled jika user mau
                if st.button("Simpan Semua sebagai Unreconciled"):
                    engine = get_cached_engine()
                    with engine.begin() as conn:
                        for _, m in mut_df.iterrows():
                            conn.execute(text("""
                                INSERT INTO reconciliation (statement_date, description, amount, currency, status, uploaded_by)
                                VALUES (:sd, :desc, :amt, :cur, 'Unreconciled', :by)
                            """), {"sd": str(m["statement_date"])[:10], "desc": str(m["deskripsi"]), "amt": float(m["amount"]), "cur": str(m.get("currency","IDR")), "by": user["username"]})
                    st.success("Disimpan sebagai Unreconciled.")
                    st.rerun()
    except Exception as e:
        st.error(f"Gagal baca CSV: {e}")

st.markdown("---")
st.markdown("#### Riwayat Rekonsiliasi")
recon = query_df("SELECT * FROM reconciliation ORDER BY created_at DESC LIMIT 100")
if recon.empty:
    st.info("Belum ada riwayat rekonsiliasi.")
else:
    # badge status
    def badge(s):
        return {"Reconciled":"✅","Unreconciled":"🟡","Manual":"🔵"}.get(s,"⚪") + " " + s
    recon["badge"] = recon["status"].apply(badge)
    st.dataframe(recon[["id","statement_date","description","amount","currency","badge","matched_transaksi_id","uploaded_by","created_at"]],
                 width="stretch", hide_index=True, height=300,
                 column_config={"amount": st.column_config.NumberColumn("Amount", format="%.0f")})
    # manual reconcile
    st.markdown("#### Manual Match")
    col1, col2 = st.columns(2)
    with col1:
        recon_id = st.number_input("ID Rekonsiliasi Unreconciled", min_value=1, step=1)
        trans_id = st.number_input("ID Transaksi untuk di-match", min_value=1001, step=1, key="manual_tid")
    with col2:
        st.write("")
        st.write("")
        if st.button("Manual Reconcile", width="stretch"):
            engine = get_cached_engine()
            with engine.begin() as conn:
                r = conn.execute(text("SELECT status FROM reconciliation WHERE id=:id"), {"id": int(recon_id)}).fetchone()
                t = conn.execute(text("SELECT status FROM transaksi WHERE id=:id AND is_deleted=0"), {"id": int(trans_id)}).fetchone()
                if not r:
                    st.error("ID rekonsiliasi tidak ditemukan.")
                elif r[0] == "Reconciled":
                    st.error("Sudah Reconciled.")
                elif not t:
                    st.error("Transaksi tidak ditemukan.")
                else:
                    conn.execute(text("UPDATE reconciliation SET matched_transaksi_id=:tid, status='Manual' WHERE id=:rid"), {"tid": int(trans_id), "rid": int(recon_id)})
                    conn.execute(text("UPDATE transaksi SET status='Reconciled' WHERE id=:id"), {"id": int(trans_id)})
                    conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, :froms, 'Reconciled', 'Manual reconcile')"),
                                 {"tid": int(trans_id), "actor": user["username"], "froms": t[0]})
                    st.success("Manual reconcile berhasil.")
                    st.rerun()
