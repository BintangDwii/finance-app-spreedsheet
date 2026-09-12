import streamlit as st
import pandas as pd
from sqlalchemy import text
from database.connection import get_cached_engine, query_df
from utils.auth import current_user, can_approve, guard_transition
from utils.formatters import format_idr

st.title("🗃️ Riwayat & Approval")
st.caption("Filter, audit trail, dan alur persetujuan maker-checker.")
st.markdown("---")

user = current_user()
df = query_df("SELECT * FROM transaksi WHERE is_deleted=0 ORDER BY tanggal DESC, id DESC")
if df.empty:
    st.info("Database masih kosong.")
    st.stop()

# Filters
c1, c2, c3, c4 = st.columns(4)
with c1:
    f_status = st.selectbox("Filter Status", ["Semua", "Pending (Butuh Approval)", "Approved (Menunggu Bayar)", "Paid (Lunas)", "Rejected (Ditolak)", "Reconciled"])
with c2:
    f_divisi = st.selectbox("Filter Divisi", ["Semua", "IT & Engineering", "Marketing & Sales", "HR & Admin", "Operasional", "Management"])
with c3:
    f_jenis = st.selectbox("Filter Jenis", ["Semua", "Pemasukan", "Pengeluaran", "Transfer Internal"])
with c4:
    f_search = st.text_input("Cari Entitas / Catatan / Ref")

df_show = df.copy()
if f_status != "Semua":
    df_show = df_show[df_show["status"] == f_status]
if f_divisi != "Semua":
    df_show = df_show[df_show["divisi"] == f_divisi]
if f_jenis != "Semua":
    df_show = df_show[df_show["jenis"] == f_jenis]
if f_search:
    mask = df_show["entitas_terkait"].str.contains(f_search, case=False, na=False) | df_show["catatan"].str.contains(f_search, case=False, na=False) | df_show["reference_no"].str.contains(f_search, case=False, na=False)
    df_show = df_show[mask]

# tampilkan IDR eq untuk USD
df_show["idr_eq"] = df_show.apply(lambda r: r["total_akhir"]*r["fx_rate"] if r["currency"]=="USD" else r["total_akhir"], axis=1)
st.dataframe(df_show[["id","tanggal","divisi","jenis","entitas_terkait","total_akhir","currency","fx_rate","idr_eq","status","reference_no"]],
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
    st.caption("Aturan: Pending→Approved (Checker/Approver, >50jt harus Approver) • Approved→Paid (Approver/Admin, >100jt Admin) • Paid tidak bisa diubah")
    id_update = st.number_input("ID Transaksi", min_value=1001, step=1, key="upd")
    new_status = st.selectbox("Ubah Menjadi", ["Pending (Butuh Approval)", "Approved (Menunggu Bayar)", "Paid (Lunas)", "Rejected (Ditolak)", "Reconciled"])
    catatan_approval = st.text_input("Catatan Approval (wajib untuk Reject)")
    if st.button("Update Status", type="primary", width="stretch", disabled=not can_approve()):
        if not can_approve():
            st.error("Hanya Checker/Approver/Admin yang bisa approve.")
        else:
            engine = get_cached_engine()
            with engine.begin() as conn:
                row = conn.execute(text("SELECT status, total_akhir, currency, fx_rate FROM transaksi WHERE id=:id AND is_deleted=0"), {"id": int(id_update)}).fetchone()
                if not row:
                    st.error("ID tidak ditemukan.")
                else:
                    cur_status, tot, cur, fx = row[0], float(row[1]), row[2], float(row[3])
                    idr = tot * fx if cur == "USD" else tot
                    allowed, reason = guard_transition(cur_status, new_status, idr, user)
                    if not allowed:
                        st.error(reason)
                    else:
                        if new_status == "Rejected (Ditolak)" and not catatan_approval.strip():
                            st.error("Catatan wajib untuk Reject.")
                        else:
                            conn.execute(text("UPDATE transaksi SET status=:ns, updated_at=datetime('now','localtime') WHERE id=:id"), {"ns": new_status, "id": int(id_update)})
                            conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, :froms, :tos, :cat)"),
                                         {"tid": int(id_update), "actor": user["username"], "froms": cur_status, "tos": new_status, "cat": catatan_approval or "-"})
                            st.success(f"#{int(id_update)} {cur_status} → {new_status}")
                            st.rerun()
    if not can_approve():
        st.info("Login sebagai Checker/Approver/Admin untuk approve.")

with col_act2:
    st.markdown("🗑️ **Hapus (Soft Delete)**")
    st.caption("Hanya Admin yang bisa hapus. Data tidak hilang permanen, masuk audit & flag is_deleted.")
    id_delete = st.number_input("ID Hapus", min_value=1001, step=1, key="del")
    if st.button("Hapus (Soft Delete)", width="stretch"):
        if user["role"] != "Admin":
            st.error("Hanya Admin yang bisa hapus.")
        else:
            engine = get_cached_engine()
            with engine.begin() as conn:
                exists = conn.execute(text("SELECT id FROM transaksi WHERE id=:id AND is_deleted=0"), {"id": int(id_delete)}).fetchone()
                if not exists:
                    st.error("ID tidak ditemukan.")
                else:
                    conn.execute(text("UPDATE transaksi SET is_deleted=1, updated_at=datetime('now','localtime') WHERE id=:id"), {"id": int(id_delete)})
                    conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, 'Active', 'Deleted', 'Soft delete')"),
                                 {"tid": int(id_delete), "actor": user["username"]})
                    st.success(f"Transaksi #{int(id_delete)} soft-deleted.")
                    st.rerun()

st.markdown("---")
st.markdown("#### Audit Trail")
audit = query_df("SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT 100")
if audit.empty:
    st.info("Belum ada audit log.")
else:
    st.dataframe(audit[["id","transaksi_id","actor","from_status","to_status","catatan","timestamp"]], width="stretch", hide_index=True, height=250)
