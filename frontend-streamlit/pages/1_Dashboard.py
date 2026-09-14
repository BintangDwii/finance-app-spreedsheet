import streamlit as st
import pandas as pd
import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import api_client
from components.charts import trend_chart, donut_divisi
from components.cards import metric_card
from utils.formatters import format_idr

st.title("📊 Dashboard & Analytics")
st.caption("Via Axum API — saldo & tren dari Postgres (Rust + sqlx).")
st.markdown("---")

try:
    saldo = api_client.get_saldo()
    per_rek = api_client.per_rekening()
    # transaksi untuk charts — ambil 200 teratas Paid
    trans_raw = api_client.list_transaksi({"status": "Paid (Lunas)", "per_page": "200"})
    df_paid = pd.DataFrame(trans_raw) if trans_raw else pd.DataFrame()
    # budgets variance
    var = api_client.variance()
    df_var = pd.DataFrame(var) if var else pd.DataFrame()
except Exception as e:
    st.error(f"Gagal ambil data API: {e}")
    st.caption("Pastikan backend Rust running di http://localhost:8000 (docker-compose up)")
    st.stop()

# Metrics
c1, c2, c3, c4 = st.columns(4)
with c1:
    metric_card("Pendapatan (Lunas)", saldo.get("masuk",0))
with c2:
    metric_card("Pengeluaran (Lunas)", saldo.get("keluar",0), delta_color="inverse")
with c3:
    metric_card("Saldo Kas", saldo.get("saldo",0))
with c4:
    # pending via filter
    try:
        pending_raw = api_client.list_transaksi({"status": "Pending (Butuh Approval)", "per_page": "200"})
        pending_idr = sum(r.get("idr_amount", r.get("total_akhir",0)) for r in pending_raw)
    except:
        pending_idr = 0
    metric_card("Menunggu Approval", pending_idr, delta_color="inverse")

# Saldo per rekening — dari engine Rust
st.markdown("#### Saldo per Rekening (Rust Materialized)")
if per_rek:
    cols = st.columns(3)
    for i, acc in enumerate(per_rek):
        with cols[i % 3]:
            with st.container(border=True):
                st.markdown(f"**{acc['bank_name']}** · {acc['currency']}")
                st.metric("Saldo", format_idr(acc['saldo'], acc['currency']))
                st.caption(f"Opening {format_idr(acc['opening_balance'], acc['currency'])} | Masuk {format_idr(acc['total_masuk'], acc['currency'])} | Keluar {format_idr(acc['total_keluar'], acc['currency'])}")

st.markdown("---")
if not df_paid.empty:
    # siapkan df_paid agar charts kompatibel: butuh kolom tanggal, jenis, total_akhir, currency, fx_rate, divisi
    # API sudah return total_akhir, currency, fx_rate, tanggal sebagai string
    # perlu normalisasi nama kolom: API returns lowercase? Our Rust returns snake_case lower; convert
    # Pastikan kolom ada
    df_paid["tanggal"] = df_paid["tanggal"] if "tanggal" in df_paid.columns else df_paid.get("tanggal")
    df_paid["jenis"] = df_paid["jenis"] if "jenis" in df_paid.columns else df_paid.get("jenis")
    col1, col2 = st.columns([2,1])
    with col1:
        st.markdown("**📈 Tren Arus Kas (IDR)**")
        # trend_chart expects df_paid with tanggal, jenis, total_akhir, currency, fx_rate
        try:
            trend_chart(df_paid)
        except Exception as e:
            st.error(f"Chart error: {e}")
            st.dataframe(df_paid.head())
    with col2:
        st.markdown("**🍩 Distribusi per Divisi**")
        try:
            donut_divisi(df_paid)
        except Exception as e:
            st.error(str(e))
else:
    st.info("Belum ada data Paid.")

# Variance
if not df_var.empty:
    st.markdown("---")
    st.markdown("#### Variance Anggaran")
    df_var["burn"] = pd.to_numeric(df_var["burn"], errors="coerce")
    total_planned = df_var["planned"].sum()
    total_actual = df_var["actual"].sum()
    burn = (total_actual/total_planned*100).round(1) if total_planned else 0
    c1, c2, c3 = st.columns(3)
    c1.metric("Planned", format_idr(total_planned))
    c2.metric("Actual", format_idr(total_actual))
    c3.metric("Burn", f"{burn:.1f}%")
    st.progress(min(int(burn),100), text=f"{burn:.1f}%")
    st.dataframe(df_var[["bulan","divisi","kategori","planned","actual","sisa","burn"]], width="stretch", hide_index=True)
