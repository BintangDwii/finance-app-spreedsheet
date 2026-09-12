import streamlit as st
import pandas as pd
from database.connection import query_df
from components.charts import trend_chart, donut_divisi
from components.cards import metric_card
from utils.formatters import format_idr

st.title("📊 Dashboard & Analytics")
st.caption("Ringkasan keuangan perusahaan — hanya transaksi Paid (Lunas) yang dihitung sebagai arus kas nyata.")
st.markdown("---")

# Load data
df_trans = query_df("SELECT * FROM transaksi WHERE is_deleted=0")
df_budgets = query_df("SELECT * FROM budgets")

if df_trans.empty:
    st.info("Database transaksi masih kosong.")
    st.stop()

# Hitung IDR conversion untuk metrik
def to_idr(r):
    return r["total_akhir"] * r["fx_rate"] if r["currency"] == "USD" else r["total_akhir"]

df_trans["idr"] = df_trans.apply(to_idr, axis=1)
df_paid = df_trans[df_trans["status"] == "Paid (Lunas)"]
df_pending = df_trans[df_trans["status"] == "Pending (Butuh Approval)"]

total_masuk = df_paid[df_paid["jenis"] == "Pemasukan"]["idr"].sum() if not df_paid.empty else 0
total_keluar = df_paid[df_paid["jenis"] == "Pengeluaran"]["idr"].sum() if not df_paid.empty else 0
saldo = total_masuk - total_keluar
pending_total = df_pending["idr"].sum() if not df_pending.empty else 0
budget_planned = df_budgets["planned"].sum() if not df_budgets.empty else 0

# Metrics
c1, c2, c3, c4 = st.columns(4)
with c1:
    metric_card("Pendapatan (Lunas)", total_masuk, delta=format_idr(total_masuk))
with c2:
    metric_card("Pengeluaran (Lunas)", total_keluar, delta=f"-{format_idr(total_keluar)}", delta_color="inverse")
with c3:
    metric_card("Saldo Kas", saldo, delta=format_idr(saldo))
with c4:
    metric_card("Menunggu Approval", pending_total, delta_color="inverse")

# Saldo per rekening
st.markdown("#### Saldo per Rekening (Paid + Opening Balance)")
acc_df = query_df("SELECT * FROM accounts WHERE status='Aktif'")
if not acc_df.empty:
    # hitung mutasi per akun dari transaksi Paid
    paid_by_acc = df_paid.groupby("akun_pembayaran")["idr"].sum().to_dict() if not df_paid.empty else {}
    # untuk akun USD, opening_balance perlu konversi? simpan USD terpisah
    cols = st.columns(len(acc_df))
    for i, (_, acc) in enumerate(acc_df.iterrows()):
        name = acc["bank_name"]
        cur = acc["currency"]
        open_bal = acc["opening_balance"]
        # mutasi: pemasukan +, pengeluaran -, transfer netral (belum ada)
        mutasi = 0
        # pemasukan ke akun ini +
        inc = df_paid[(df_paid["akun_pembayaran"]==name) & (df_paid["jenis"]=="Pemasukan")]["idr"].sum()
        exp = df_paid[(df_paid["akun_pembayaran"]==name) & (df_paid["jenis"]=="Pengeluaran")]["idr"].sum()
        mutasi = inc - exp
        # untuk USD, open_bal dalam USD, mutasi sudah di IDR jika USD? untuk USD akun, transaksi USD idr sudah konversi, tapi saldo USD harus dalam USD
        if cur == "USD":
            # hitung saldo USD murni
            df_usd = df_paid[df_paid["akun_pembayaran"]==name]
            saldo_usd = open_bal + df_usd[df_usd["jenis"]=="Pemasukan"]["total_akhir"].sum() - df_usd[df_usd["jenis"]=="Pengeluaran"]["total_akhir"].sum()
            with cols[i]:
                with st.container(border=True):
                    st.markdown(f"**{name}** · {cur}")
                    st.metric("Saldo", f"$ {saldo_usd:,.2f}", help=f"IDR eq: Rp {saldo_usd*16250:,.0f}")
                    st.caption(f"Rek: {acc['no_rekening']} | GL {acc['gl_code']}")
        else:
            saldo_idr = open_bal + mutasi
            with cols[i]:
                with st.container(border=True):
                    st.markdown(f"**{name}** · {cur}")
                    st.metric("Saldo", format_idr(saldo_idr))
                    st.caption(f"Rek: {acc['no_rekening']} | GL {acc['gl_code']}")

st.markdown("<br>", unsafe_allow_html=True)

# Charts
if not df_paid.empty:
    col1, col2 = st.columns([2,1])
    with col1:
        st.markdown("**📈 Tren Arus Kas (IDR, Paid)**")
        trend_chart(df_paid)
    with col2:
        st.markdown("**🍩 Distribusi Pengeluaran per Divisi**")
        donut_divisi(df_paid)
else:
    st.info("Belum ada data Paid untuk grafik.")

# Budget variance ringkas
st.markdown("---")
st.markdown("#### Variance Anggaran vs Realisasi (Pengeluaran)")
if not df_budgets.empty and not df_paid.empty:
    # agregasi actual per bulan
    df_paid["bulan"] = pd.to_datetime(df_paid["tanggal"]).dt.strftime("%Y-%m")
    actual_month = df_paid[df_paid["jenis"]=="Pengeluaran"].groupby("bulan")["idr"].sum().reset_index()
    budget_month = df_budgets.groupby("bulan")["planned"].sum().reset_index()
    merged = pd.merge(budget_month, actual_month, on="bulan", how="outer").fillna(0)
    merged["burn"] = (merged["idr"] / merged["planned"] * 100).round(1)
    for _, r in merged.iterrows():
        with st.container(border=True):
            c1, c2, c3, c4 = st.columns(4)
            c1.metric(f"Planned {r['bulan']}", format_idr(r["planned"]))
            c2.metric(f"Actual {r['bulan']}", format_idr(r["idr"]))
            c3.metric("Burn Rate", f"{r['burn']:.1f} %")
            c4.progress(min(int(r["burn"]), 100), text=f"{r['burn']:.1f}% terpakai")
            if r["burn"] > 80:
                st.warning(f"⚠️ Burn rate {r['bulan']} >80%")
else:
    st.info("Data budget atau paid belum lengkap untuk variance.")

# Export
with st.expander("📥 Export Data"):
    c1, c2 = st.columns(2)
    with c1:
        st.download_button("Download Transaksi CSV", df_trans.to_csv(index=False).encode(), "transaksi.csv", "text/csv", width="stretch")
    with c2:
        if not df_budgets.empty:
            st.download_button("Download Anggaran CSV", df_budgets.to_csv(index=False).encode(), "budgets.csv", "text/csv", width="stretch")
