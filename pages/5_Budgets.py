import streamlit as st
import pandas as pd
from sqlalchemy import text
from database.connection import get_cached_engine, query_df
from components.charts import budget_variance_chart
from utils.auth import current_user
from utils.formatters import format_idr

st.title("📈 Anggaran & Variance")
st.caption("Planned vs Actual (Paid, Pengeluaran) — burn-rate dan sisa anggaran.")
st.markdown("---")

user = current_user()
budgets = query_df("SELECT * FROM budgets ORDER BY bulan, divisi, kategori")
trans = query_df("SELECT * FROM transaksi WHERE is_deleted=0 AND status='Paid (Lunas)'")

if budgets.empty:
    st.info("Belum ada data anggaran.")
    st.stop()

# Filter
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

# Hitung actual per budget cell
trans["bulan"] = pd.to_datetime(trans["tanggal"]).dt.strftime("%Y-%m")
trans["idr"] = trans.apply(lambda r: r["total_akhir"]*r["fx_rate"] if r["currency"]=="USD" else r["total_akhir"], axis=1)
actual = trans[trans["jenis"]=="Pengeluaran"].groupby(["bulan","divisi","kategori"])["idr"].sum().reset_index().rename(columns={"idr":"actual"})

merged = pd.merge(b_show, actual, on=["bulan","divisi","kategori"], how="left")
merged["actual"] = merged["actual"].fillna(0)
merged["sisa"] = merged["planned"] - merged["actual"]
merged["burn"] = (merged["actual"]/merged["planned"]*100).round(1)
merged["status"] = merged["burn"].apply(lambda x: "🔴 Over" if x>100 else "🟡 >80%" if x>80 else "🟢 Aman")

# Summary cards
total_planned = b_show["planned"].sum()
total_actual = merged["actual"].sum()
burn_total = (total_actual/total_planned*100).round(1) if total_planned else 0
c1, c2, c3, c4 = st.columns(4)
with c1:
    st.metric("Total Planned", format_idr(total_planned))
with c2:
    st.metric("Total Actual", format_idr(total_actual))
with c3:
    st.metric("Burn Rate", f"{burn_total:.1f} %")
with c4:
    st.metric("Sisa Anggaran", format_idr(total_planned - total_actual))

st.progress(min(int(burn_total),100), text=f"{burn_total:.1f}% terpakai")
if burn_total > 80:
    st.warning("⚠️ Burn rate melebihi 80% — perlu kontrol pengeluaran.")

st.markdown("---")
st.markdown("**📊 Variance Chart**")
chart_budgets = b_show.copy()
# chart butuh DataFrame budgets asli
budget_variance_chart(chart_budgets, trans)

st.markdown("---")
st.markdown("**📋 Detail Anggaran per Kategori**")
st.dataframe(merged[["bulan","divisi","kategori","planned","actual","sisa","burn","status"]],
             width="stretch", hide_index=True, height=400,
             column_config={
                 "planned": st.column_config.NumberColumn("Planned", format="%.0f"),
                 "actual": st.column_config.NumberColumn("Actual", format="%.0f"),
                 "sisa": st.column_config.NumberColumn("Sisa", format="%.0f"),
                 "burn": st.column_config.NumberColumn("Burn %", format="%.1f"),
             })

# Edit budget
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
        submit = st.form_submit_button("Simpan Anggaran", width="stretch", disabled=(user["role"] not in ("Admin","Approver")))
    if submit:
        if b_planned <= 0:
            st.error("Planned harus >0")
        else:
            engine = get_cached_engine()
            with engine.begin() as conn:
                conn.execute(text("""
                    INSERT INTO budgets (bulan, divisi, kategori, planned, currency)
                    VALUES (:b, :d, :k, :p, :cur)
                    ON CONFLICT(bulan, divisi, kategori) DO UPDATE SET planned=:p, currency=:cur
                """), {"b": b_bulan, "d": b_divisi, "k": b_kategori, "p": b_planned, "cur": b_currency})
            st.success(f"Anggaran {b_bulan} {b_divisi} - {b_kategori} disimpan: {format_idr(b_planned)}")
            st.rerun()

# Export
st.download_button("Download Anggaran CSV", merged.to_csv(index=False).encode(), "budgets_variance.csv", "text/csv", width="stretch")
