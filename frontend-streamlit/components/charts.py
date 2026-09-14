import pandas as pd
import plotly.express as px
import streamlit as st
from utils.formatters import to_idr

PALETTE = {"Pemasukan": "#10b981", "Pengeluaran": "#ef4444"}

def trend_chart(df_paid: pd.DataFrame):
    if df_paid.empty:
        st.info("Belum ada data transaksi berstatus 'Paid (Lunas)' untuk menampilkan grafik.")
        return
    df = df_paid.copy()
    df["tanggal"] = pd.to_datetime(df["tanggal"])
    df["idr_total"] = df.apply(lambda r: to_idr(r["total_akhir"], r["currency"], r["fx_rate"]), axis=1)
    grouped = df.groupby(["tanggal", "jenis"])["idr_total"].sum().reset_index()
    fig = px.line(grouped, x="tanggal", y="idr_total", color="jenis", markers=True, line_shape="spline",
                  color_discrete_map=PALETTE)
    fig.update_traces(fill="tozeroy")
    fig.update_layout(hovermode="x unified", margin=dict(l=0,r=0,t=30,b=0), yaxis_title="Total (IDR)", xaxis_title="Tanggal")
    st.plotly_chart(fig, width="stretch")

def donut_divisi(df_paid: pd.DataFrame):
    df_exp = df_paid[df_paid["jenis"]=="Pengeluaran"]
    if df_exp.empty:
        st.info("Belum ada data pengeluaran.")
        return
    df_exp = df_exp.copy()
    df_exp["idr_total"] = df_exp.apply(lambda r: to_idr(r["total_akhir"], r["currency"], r["fx_rate"]), axis=1)
    fig = px.pie(df_exp, names="divisi", values="idr_total", hole=0.4, color_discrete_sequence=px.colors.sequential.Teal)
    fig.update_layout(margin=dict(l=0,r=0,t=30,b=0))
    st.plotly_chart(fig, width="stretch")

def budget_variance_chart(budget_df: pd.DataFrame, actual_df: pd.DataFrame):
    if budget_df.empty:
        st.info("Belum ada data anggaran.")
        return
    if not actual_df.empty:
        actual_df = actual_df.copy()
        actual_df["bulan"] = pd.to_datetime(actual_df["tanggal"]).dt.strftime("%Y-%m")
        actual_df["idr"] = actual_df.apply(lambda r: to_idr(r["total_akhir"], r["currency"], r["fx_rate"]), axis=1)
        actual_agg = actual_df[actual_df["jenis"]=="Pengeluaran"].groupby(["bulan","divisi","kategori"])["idr"].sum().reset_index().rename(columns={"idr":"actual"})
    else:
        actual_agg = pd.DataFrame(columns=["bulan","divisi","kategori","actual"])
    merged = pd.merge(budget_df, actual_agg, on=["bulan","divisi","kategori"], how="left")
    merged["actual"] = merged["actual"].fillna(0)
    merged["variance"] = merged["planned"] - merged["actual"]
    merged["burn"] = (merged["actual"]/merged["planned"]*100).round(1)
    fig = px.bar(merged, x="kategori", y=["planned","actual"], barmode="group", facet_col="bulan", color_discrete_sequence=["#0d9488","#f59e0b"])
    fig.update_layout(margin=dict(l=0,r=0,t=30,b=0), yaxis_title="IDR", xaxis_title="Kategori")
    st.plotly_chart(fig, width="stretch")
    return merged
