import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import pandas as pd
import streamlit as st
import api_client
from utils.auth import current_user
from utils.formatters import format_idr, status_badge

st.title("🔄 Rekonsiliasi Bank")
st.caption("Upload mutasi CSV — auto-match via Axum API (tanggal ±1 hari & amount).")
st.markdown("---")

user = current_user()

tmpl_path = "data/template_mutasi.csv"
if os.path.exists(tmpl_path):
    with open(tmpl_path, "rb") as f:
        st.download_button("📥 Download Template CSV", f.read(), "template_mutasi.csv", "text/csv", width="stretch")
st.info("Format CSV wajib: `tanggal,deskripsi,amount,currency` — contoh: 2026-08-15,Transfer masuk,73260000,IDR")

uploaded = st.file_uploader("Upload Mutasi CSV", type=["csv"])
if uploaded is not None:
    try:
        mut_df = pd.read_csv(uploaded)
        mut_df.columns = [c.strip().lower() for c in mut_df.columns]
        required = {"tanggal","deskripsi","amount"}
        if not required.issubset(set(mut_df.columns)):
            st.error(f"CSV harus punya kolom: {required}. Kolom terdeteksi: {list(mut_df.columns)}")
            st.stop()
        if "currency" not in mut_df.columns:
            mut_df["currency"] = "IDR"
        mut_df["amount"] = pd.to_numeric(mut_df["amount"], errors="coerce")
        st.markdown("#### Preview Mutasi")
        st.dataframe(mut_df.head(20), width="stretch", hide_index=True)
        st.caption(f"Total {len(mut_df)} baris, total amount {format_idr(mut_df['amount'].sum())}")

        # Build CSV text for API upload (normalize columns to tanggal,deskripsi,amount,currency)
        csv_text = mut_df.to_csv(index=False)
        if st.button("✅ Upload & Auto-Match via API", type="primary", width="stretch"):
            try:
                res = api_client.upload_reconcile(csv_text)
                st.success(f"Matched {res.get('matched')} / Unmatched {res.get('unmatched')} dari {res.get('total')} baris.")
                st.rerun()
            except Exception as e:
                st.error(f"Gagal upload: {e}")
    except Exception as e:
        st.error(f"Gagal baca CSV: {e}")

st.markdown("---")
st.markdown("#### Riwayat Rekonsiliasi (via API)")
try:
    recon_data = api_client.list_reconcile()
    recon = pd.DataFrame(recon_data) if recon_data else pd.DataFrame()
except Exception as e:
    st.error(f"Gagal load reconciliation: {e}")
    recon = pd.DataFrame()

if recon.empty:
    st.info("Belum ada riwayat rekonsiliasi.")
else:
    recon["badge"] = recon["status"].apply(lambda s: status_badge(s))
    st.dataframe(recon[["id","statement_date","description","amount","currency","badge","matched_transaksi_id","uploaded_by","created_at"]],
                 width="stretch", hide_index=True, height=300,
                 column_config={"amount": st.column_config.NumberColumn("Amount", format="%.0f")})
    st.markdown("#### Manual Match")
    col1, col2 = st.columns(2)
    with col1:
        recon_id = st.number_input("ID Rekonsiliasi Unreconciled", min_value=1, step=1)
        trans_id = st.number_input("ID Transaksi untuk di-match", min_value=1, step=1, key="manual_tid")
    with col2:
        st.write("")
        st.write("")
        if st.button("Manual Reconcile", width="stretch"):
            try:
                res = api_client.manual_match(int(recon_id), int(trans_id))
                st.success(f"Manual reconcile berhasil: {res}")
                st.rerun()
            except Exception as e:
                st.error(f"Gagal manual match: {e}")
