import os
import sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import streamlit as st
from datetime import date
import api_client
from utils.auth import current_user, can_create
from utils.formatters import format_idr, format_idr_fx, calc_total, DEFAULT_FX_RATE
from utils.constants import DIVISI_OPTIONS, KATEGORI_MASUK, KATEGORI_KELUAR, ENTITAS_OPTIONS

st.title("➕ Input Transaksi")
st.caption("Maker membuat transaksi — via Axum API (Rust).")
st.markdown("---")

user = current_user()
if not can_create():
    st.error("Hanya role Maker/Admin yang bisa input transaksi.")
    st.stop()

# Load master via API (single source of truth)
try:
    accounts_data = api_client.list_accounts()
    accounts = [a["bank_name"] for a in accounts_data] if accounts_data else ["BCA Corporate"]
    currency_map = {a["bank_name"]: a.get("currency", "IDR") for a in accounts_data}
except Exception as e:
    st.error(f"Gagal load akun via API: {e}")
    accounts = ["BCA Corporate"]
    accounts_data = []

divisi_list = DIVISI_OPTIONS
kat_masuk = KATEGORI_MASUK
kat_keluar = KATEGORI_KELUAR
entitas_list = ENTITAS_OPTIONS

with st.form("form_corporate", clear_on_submit=True):
    col1, col2 = st.columns(2)
    with col1:
        tgl = st.date_input("Tanggal Transaksi", value=date.today())
        jenis = st.selectbox("Jenis Arus Kas", ["Pemasukan", "Pengeluaran", "Transfer Internal"])
        divisi = st.selectbox("Divisi / Departemen", divisi_list)
        if jenis == "Pemasukan":
            kategori = st.selectbox("Kategori Transaksi", kat_masuk)
        elif jenis == "Transfer Internal":
            kategori = st.selectbox("Kategori Transaksi", ["Transfer Antar Rekening"])
        else:
            kategori = st.selectbox("Kategori Transaksi", kat_keluar)
    with col2:
        entitas = st.selectbox("Entitas Terkait (Klien/Vendor/Karyawan)", entitas_list)
        akun = st.selectbox("Akun Sumber / Penerimaan", accounts)
        akun_tujuan = None
        if jenis == "Transfer Internal":
            akun_tujuan = st.selectbox("Akun Tujuan Transfer", [a for a in accounts if a != akun])
        currency = st.selectbox("Mata Uang", ["IDR", "USD"])
        fx_rate = 1.0
        if currency == "USD":
            fx_rate = st.number_input("Kurs (IDR per USD)", min_value=1.0, value=DEFAULT_FX_RATE, step=100.0)
        due_date = None
        if jenis == "Pengeluaran":
            due_date = st.date_input("Jatuh Tempo (Opsional)", value=None)
        reference_no = st.text_input("No. Referensi (Opsional)", placeholder="INV-001 / REF-xxxx")
        catatan = st.text_input("Keterangan Tambahan")

    st.markdown("---")
    st.markdown("**Rincian Nominal**")
    col_nom1, col_nom2, col_nom3 = st.columns(3)
    with col_nom1:
        subtotal = st.number_input(f"Subtotal ({currency})", min_value=0.0, step=50000.0 if currency == "IDR" else 100.0)
    with col_nom2:
        is_ppn = st.checkbox("Tambah PPN (11%)", disabled=(jenis == "Transfer Internal"))
        ppn_val = (subtotal * 0.11) if is_ppn and jenis != "Transfer Internal" else 0.0
        st.write(f"*PPN Terhitung: {format_idr(ppn_val, currency)}*")
    with col_nom3:
        diskon = st.number_input(f"Diskon / Potongan ({currency})", min_value=0.0, step=10000.0 if currency == "IDR" else 10.0, disabled=(jenis == "Transfer Internal"))
    total_akhir = calc_total(subtotal, ppn_val, diskon)
    st.info(f"**GRAND TOTAL: {format_idr_fx(total_akhir, currency, fx_rate)}**")

    submitted = st.form_submit_button("Simpan Transaksi (Pending)", width="stretch")
    if submitted:
        if subtotal <= 0:
            st.error("Subtotal harus lebih besar dari 0!")
        elif jenis == "Transfer Internal" and not akun_tujuan:
            st.error("Pilih akun tujuan transfer.")
        else:
            payload = {
                "tanggal": str(tgl),
                "jenis": jenis,
                "divisi": divisi,
                "kategori": kategori,
                "entitas_terkait": entitas,
                "akun_pembayaran": akun,
                "subtotal": subtotal,
                "ppn_11": ppn_val if jenis != "Transfer Internal" else 0,
                "diskon": diskon if jenis != "Transfer Internal" else 0,
                "currency": currency,
                "fx_rate": fx_rate,
                "reference_no": reference_no or None,
                "catatan": catatan or None,
            }
            if jenis == "Transfer Internal":
                payload["akun_tujuan"] = akun_tujuan
            if due_date:
                payload["due_date"] = str(due_date)
            try:
                res = api_client.create_transaksi(payload)
                if jenis == "Transfer Internal":
                    st.success(f"Transfer {akun} → {akun_tujuan} sebesar {format_idr(total_akhir, currency)} berhasil (ids={res.get('ids')}, ref={res.get('reference')}).")
                else:
                    st.success(f"Transaksi #{res.get('id')} berhasil disimpan sebagai {res.get('status')}.")
                st.rerun()
            except Exception as e:
                st.error(f"Gagal simpan via API: {e}")
