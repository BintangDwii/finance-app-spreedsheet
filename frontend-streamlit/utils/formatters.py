import pandas as pd

def format_idr(x: float, currency: str = "IDR") -> str:
    if pd.isna(x):
        return "-"
    if currency == "USD":
        return f"$ {x:,.2f}"
    return f"Rp {x:,.0f}"

def format_idr_fx(amount: float, currency: str, fx_rate: float) -> str:
    if currency == "USD":
        return f"$ {amount:,.2f} (Rp {amount*fx_rate:,.0f})"
    return format_idr(amount)

def calc_total(subtotal: float, ppn: float, diskon: float) -> float:
    return (subtotal + ppn) - diskon

def to_idr(amount: float, currency: str, fx_rate: float) -> float:
    if currency == "USD":
        return amount * fx_rate
    return amount

def status_badge(status: str) -> str:
    mapping = {
        "Paid (Lunas)": "🟢",
        "Pending (Butuh Approval)": "🟡",
        "Approved (Menunggu Bayar)": "🔵",
        "Rejected (Ditolak)": "🔴",
        "Reconciled": "✅",
    }
    return mapping.get(status, "⚪") + " " + status

DEFAULT_FX_RATE = 16250.0
