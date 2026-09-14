import streamlit as st
import pandas as pd

def transaksi_table(df: pd.DataFrame, height: int = 400):
    if df.empty:
        st.info("Tidak ada data.")
        return
    cols = ["id","tanggal","divisi","jenis","entitas_terkait","total_akhir","currency","fx_rate","status","reference_no"]
    cols = [c for c in cols if c in df.columns]
    st.dataframe(df[cols], width="stretch", hide_index=True, height=height,
                 column_config={
                     "total_akhir": st.column_config.NumberColumn("Total", format="%.0f"),
                     "fx_rate": st.column_config.NumberColumn("Kurs", format="%.2f"),
                 })

def budgets_table(df: pd.DataFrame):
    if df.empty:
        st.info("Belum ada anggaran.")
        return
    st.dataframe(df, width="stretch", hide_index=True, column_config={
        "planned": st.column_config.NumberColumn("Planned (IDR)", format="%.0f"),
    })
