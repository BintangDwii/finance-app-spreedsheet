import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import streamlit as st
import pandas as pd
import api_client
from utils.auth import current_user, can_approve
from utils.formatters import to_idr
from utils.constants import DIVISI_OPTIONS, STATUS_OPTIONS

st.title("🗃️ Riwayat & Approval")
st.caption("Filter, audit trail, dan alur persetujuan maker-checker — via Axum API.")
st.markdown("---")

user = current_user()

# Load via API (server-side filters)
f_status = st.session_state.get("f_status", "Semua")
f_divisi = st.session_state.get("f_divisi", "Semua")
f_jenis = st.session_state.get("f_jenis", "Semua")
f_search = st.session_state.get("f_search", "")

c1, c2, c3, c4 = st.columns(4)
with c1:
    f_status = st.selectbox("Filter Status", ["Semua"] + STATUS_OPTIONS, key="f_status")
with c2:
    f_divisi = st.selectbox("Filter Divisi", ["Semua"] + DIVISI_OPTIONS, key="f_divisi")
with c3:
    f_jenis = st.selectbox("Filter Jenis", ["Semua", "Pemasukan", "Pengeluaran", "Transfer Internal"], key="f_jenis")
with c4:
    f_search = st.text_input("Cari Entitas / Catatan / Ref", key="f_search")

params = {}
if f_status != "Semua":
    params["status"] = f_status
if f_divisi != "Semua":
    params["divisi"] = f_divisi
if f_jenis != "Semua":
    params["jenis"] = f_jenis
if f_search:
    params["search"] = f_search

try:
    data = api_client.list_transaksi(params)
    df = pd.DataFrame(data) if data else pd.DataFrame()
except Exception as e:
    st.error(f"Gagal load transaksi: {e}")
    df = pd.DataFrame()

if df.empty:
    st.info("Belum ada transaksi sesuai filter.")
else:
    # IDR eq via shared helper to_idr
    df["idr_eq"] = df.apply(lambda r: to_idr(r["total_akhir"], r["currency"], r["fx_rate"]), axis=1)
    st.dataframe(df[["id","tanggal","divisi","jenis","entitas_terkait","total_akhir","currency","fx_rate","idr_eq","status","reference_no"]],
                 width="stretch", hide_index=True, height=380,
                 column_config={
                     "total_akhir": st.column_config.NumberColumn("Total", format="%.2f"),
                     "fx_rate": st.column_config.NumberColumn("Kurs", format="%.2f"),
                     "idr_eq": st.column_config.NumberColumn("IDR Eq", format="%.0f"),
                 })

st.markdown("---")
col_act1, col_act2 = st.columns(2)

with col_act1:
    st.markdown("✅ **Update Status (Approval Matrix)**")
    st.caption("Aturan: Pending→Approved (Checker/Approver, >50jt harus Approver) • Approved→Paid (Approver/Admin, >100jt Admin)")
    id_update = st.number_input("ID Transaksi", min_value=1, step=1, key="upd")
    new_status = st.selectbox("Ubah Menjadi", STATUS_OPTIONS)
    catatan_approval = st.text_input("Catatan Approval (wajib untuk Reject)")
    if st.button("Update Status", type="primary", width="stretch", disabled=not can_approve()):
        if not can_approve():
            st.error("Hanya Checker/Approver/Admin yang bisa approve.")
        else:
            if new_status == "Rejected (Ditolak)" and not catatan_approval.strip():
                st.error("Catatan wajib untuk Reject.")
            else:
                try:
                    res = api_client.update_status(int(id_update), new_status, catatan_approval or "-")
                    st.success(f"#{int(id_update)} {res.get('from')} → {res.get('to')}")
                    st.rerun()
                except Exception as e:
                    st.error(f"Gagal update: {e}")
    if not can_approve():
        st.info("Login sebagai Checker/Approver/Admin untuk approve.")

with col_act2:
    st.markdown("🗑️ **Hapus (Soft Delete)**")
    st.caption("Hanya Admin yang bisa hapus.")
    id_delete = st.number_input("ID Hapus", min_value=1, step=1, key="del")
    if st.button("Hapus (Soft Delete)", width="stretch"):
        if user and user.get("role") != "Admin":
            st.error("Hanya Admin yang bisa hapus.")
        else:
            try:
                api_client.soft_delete(int(id_delete))
                st.success(f"Transaksi #{int(id_delete)} soft-deleted.")
                st.rerun()
            except Exception as e:
                st.error(f"Gagal hapus: {e}")

st.markdown("---")
st.markdown("#### Audit Trail")
try:
    audit_data = api_client.list_audit(100)
    audit = pd.DataFrame(audit_data) if audit_data else pd.DataFrame()
except Exception as e:
    st.error(f"Gagal load audit: {e}")
    audit = pd.DataFrame()

if audit.empty:
    st.info("Belum ada audit log.")
else:
    st.dataframe(audit[["id","transaksi_id","actor","from_status","to_status","catatan","timestamp"]], width="stretch", hide_index=True, height=250)
