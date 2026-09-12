import streamlit as st
from utils.formatters import format_idr

def metric_card(label: str, value: float, currency: str = "IDR", delta: str | None = None, delta_color: str = "normal", help_text: str | None = None):
    st.metric(label=label, value=format_idr(value, currency), delta=delta, delta_color=delta_color, help=help_text)

def kpi_row(total_masuk: float, total_keluar: float, saldo: float, pending: float, budget_planned: float | None = None, cols: int = 4):
    cols_obj = st.columns(cols)
    with cols_obj[0]:
        metric_card("Pendapatan (Lunas)", total_masuk, delta=format_idr(total_masuk))
    with cols_obj[1]:
        metric_card("Pengeluaran (Lunas)", total_keluar, delta=f"-{format_idr(total_keluar)}", delta_color="inverse")
    with cols_obj[2]:
        metric_card("Saldo Kas", saldo, delta=format_idr(saldo))
    with cols_obj[3]:
        metric_card("Menunggu Approval", pending, delta_color="inverse")
    if budget_planned is not None and cols > 4:
        with cols_obj[4]:
            metric_card("Anggaran Terencana", budget_planned)
