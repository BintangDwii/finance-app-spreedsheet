import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import streamlit as st
import pandas as pd
import api_client
from components.charts import budget_variance_chart
from utils.auth import current_user
from utils.formatters import format_idr
from utils.constants import DIVISI_OPTIONS, KATEGORI_MASUK, KATEGORI_KELUAR

st.title("📈 Anggaran & Variance")
st.caption("Planned vs Actual (Paid/Reconciled) — via Axum API.")
st.markdown("---")

user = current_user()
try:
    budgets_data = api_client.list_budgets()
    budgets = pd.DataFrame(budgets_data) if budgets_data else pd.DataFrame()
    var_data = api_client.variance()
    var_df = pd.DataFrame(var_data) if var_data else pd.DataFrame()
    # For chart actual: need transaksi Paid for chart helper; fetch via API
    trans_data = api_client.list_transaksi({"status": "Paid (Lunas)", "per_page": "200"})
    trans = pd.DataFrame(trans_data) if trans_data else pd.DataFrame()
except Exception as e:
    st.error(f"Gagal load budgets: {e}")
    budgets = pd.DataFrame()
    var_df = pd.DataFrame()
    trans = pd.DataFrame()

if budgets.empty:
    st.info("Belum ada data anggaran.")
    st.stop()

col1, col2 = st.columns(2)
with col1:
    bulan_choice = st.selectbox("Bulan", ["Semua"] + sorted(budgets["bulan"].unique().tolist()))
with col2:
    divisi_choice = st.selectbox("Divisi", ["Semua"] + sorted(budgets["divisi"].unique().tolist()))

b_show = budgets.copy()
if bulan_choice != "Semua":
    b_show = b_show[b_show["bulan"]==bulan_choice]
if divisi_choice != "Semua":
    b_show = b_show[b_show["divisi"]==divisi_choice]

# Use server variance for summary (authoritative)
if not var_df.empty:
    # filter variance to selected bulan/divisi
    v_show = var_df.copy()
    if bulan_choice != "Semua":
        v_show = v_show[v_show["bulan"]==bulan_choice]
    if divisi_choice != "Semua":
        v_show = v_show[v_show["divisi"]==divisi_choice]
    total_planned = v_show["planned"].sum()
    total_actual = v_show["actual"].sum()
else:
    v_show = pd.DataFrame()
    total_planned = b_show["planned"].sum()
    total_actual = 0

burn_total = (total_actual/total_planned*100).round(1) if total_planned else 0
c1, c2, c3, c4 = st.columns(4)
with c1:
    st.metric("Total Planned", format_idr(total_planned))
with c2:
    st.metric("Total Actual", format_idr(total_actual))
with c3:
    st.metric("Burn Rate", f"{burn_total:.1f} %")
with c4:
    sisa = total_planned - total_actual
    st.metric("Sisa Anggaran", format_idr(sisa))

st.progress(min(int(burn_total),100), text=f"{burn_total:.1f}% terpakai")
if burn_total > 80:
    st.warning("⚠️ Burn rate melebihi 80% — perlu kontrol pengeluaran.")

st.markdown("---")
st.markdown("**📊 Variance Chart**")
chart_budgets = b_show.copy()
budget_variance_chart(chart_budgets, trans)

st.markdown("---")
st.markdown("**📋 Detail Anggaran per Kategori**")
# Use v_show for detail table (contains sisa/burn correctly from Rust)
if not v_show.empty:
    v_show["status"] = v_show["burn"].apply(lambda x: "🔴 Over" if x>100 else "🟡 >80%" if x>80 else "🟢 Aman")
    st.dataframe(v_show[["bulan","divisi","kategori","planned","actual","sisa","burn","status"]],
                 width="stretch", hide_index=True, height=400,
                 column_config={
                     "planned": st.column_config.NumberColumn("Planned", format="%.0f"),
                     "actual": st.column_config.NumberColumn("Actual", format="%.0f"),
                     "sisa": st.column_config.NumberColumn("Sisa", format="%.0f"),
                     "burn": st.column_config.NumberColumn("Burn %", format="%.1f"),
                 })
else:
    st.info("Variance detail belum tersedia.")

st.markdown("---")
st.markdown("#### Update / Tambah Anggaran")
st.caption("Hanya Admin/Approver yang bisa ubah anggaran.")
with st.form("edit_budget", clear_on_submit=True):
    c1, c2, c3 = st.columns(3)
    with c1:
        b_bulan = st.selectbox("Bulan (YYYY-MM)", ["2026-08","2026-09","2026-10","2026-11","2026-12"])
        b_divisi = st.selectbox("Divisi", ["IT & Engineering","Marketing & Sales","HR & Admin","Operasional","Management"])
    with c2:
        b_kategori = st.selectbox("Kategori", ["Software & Cloud (AWS/GCP)","Gaji & Tunjangan","Pajak & Legalitas","Iklan & Ads","Sewa Kantor","Lainnya","Project/Client","Retainer Contract","Pendanaan/Investasi","Bunga Bank"])
        b_planned = st.number_input("Planned (IDR)", min_value=0.0, step=500000.0)
    with c3:
        b_currency = st.selectbox("Currency", ["IDR","USD"])
        st.write("")
        st.write("")
        submit = st.form_submit_button("Simpan Anggaran", width="stretch", disabled=(user and user.get("role") not in ("Admin","Approver")))
    if submit:
        if b_planned <= 0:
            st.error("Planned harus >0")
        else:
            try:
                api_client.upsert_budget({"bulan": b_bulan, "divisi": b_divisi, "kategori": b_kategori, "planned": b_planned, "currency": b_currency})
                st.success(f"Anggaran {b_bulan} {b_divisi} - {b_kategori} disimpan: {format_idr(b_planned)}")
                st.rerun()
            except Exception as e:
                st.error(f"Gagal: {e}")

if not v_show.empty:
    st.download_button("Download Anggaran CSV", v_show.to_csv(index=False).encode(), "budgets_variance.csv", "text/csv", width="stretch")
