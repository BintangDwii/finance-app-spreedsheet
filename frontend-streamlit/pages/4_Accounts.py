import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import streamlit as st
import pandas as pd
import api_client
from utils.auth import current_user
from utils.formatters import format_idr, format_idr_fx, DEFAULT_FX_RATE

st.title("🏦 Manajemen Rekening")
st.caption("Daftar rekening corporate, saldo via Axum API.")
st.markdown("---")

user = current_user()

try:
    accounts_data = api_client.list_accounts()
    acc_df = pd.DataFrame(accounts_data) if accounts_data else pd.DataFrame()
    per_rek = api_client.per_rekening()
    # build map for saldo display from per_rekening (authoritative)
    saldo_map = {r["bank_name"]: r for r in (per_rek or [])}
except Exception as e:
    st.error(f"Gagal load akun: {e}")
    acc_df = pd.DataFrame()
    per_rek = []
    saldo_map = {}

if acc_df.empty:
    st.info("Belum ada rekening.")
else:
    cols = st.columns(3)
    for i, (_, acc) in enumerate(acc_df.iterrows()):
        rec = saldo_map.get(acc["bank_name"], {})
        saldo = rec.get("saldo", acc.get("opening_balance", 0))
        inc = rec.get("total_masuk", 0)
        exp = rec.get("total_keluar", 0)
        with cols[i % 3]:
            with st.container(border=True):
                st.markdown(f"**{acc['bank_name']}**")
                st.caption(f"{acc['no_rekening']} • {acc['currency']} • GL {acc.get('gl_code','-')} • {acc['status']}")
                if acc["currency"] == "USD":
                    st.metric("Saldo", format_idr(saldo, "USD"), delta=f"Inc {format_idr(inc, 'USD')} / Exp {format_idr(exp, 'USD')}")
                    st.caption(f"{format_idr_fx(saldo, 'USD', DEFAULT_FX_RATE)} | Limit overdraft {format_idr(acc['limit_overdraft'])}")
                else:
                    st.metric("Saldo", format_idr(saldo), delta=f"Masuk {format_idr(inc)} / Keluar {format_idr(exp)}")
                    st.caption(f"Opening {format_idr(acc['opening_balance'])} | Limit {format_idr(acc['limit_overdraft'])}")

st.markdown("---")
col_form, col_table = st.columns([1,2])
with col_form:
    st.markdown("#### Tambah Rekening Baru")
    st.caption("Hanya Admin yang bisa menambah rekening.")
    with st.form("add_acc", clear_on_submit=True):
        bank_name = st.text_input("Nama Bank/Rekening")
        no_rek = st.text_input("No. Rekening")
        currency = st.selectbox("Currency", ["IDR", "USD"])
        opening = st.number_input("Opening Balance", min_value=0.0, step=100000.0)
        limit_od = st.number_input("Limit Overdraft", min_value=0.0, step=100000.0)
        gl = st.text_input("GL Code", placeholder="1107")
        status_acc = st.selectbox("Status", ["Aktif", "Beku", "Tutup"])
        submitted = st.form_submit_button("Tambah Rekening", width="stretch", disabled=(user and user.get("role")!="Admin"))
        if submitted:
            if not bank_name or not no_rek:
                st.error("Nama dan No. Rekening wajib.")
            else:
                try:
                    api_client.create_account({
                        "bank_name": bank_name.strip(),
                        "no_rekening": no_rek.strip(),
                        "currency": currency,
                        "opening_balance": opening,
                        "limit_overdraft": limit_od,
                        "gl_code": gl.strip() or None,
                        "status": status_acc
                    })
                    st.success(f"Rekening {bank_name} ditambahkan.")
                    st.rerun()
                except Exception as e:
                    st.error(f"Gagal: {e}")
    st.markdown("#### Update Status Rekening")
    if not acc_df.empty:
        acc_choice = st.selectbox("Pilih Rekening", acc_df["bank_name"].tolist())
        new_status = st.selectbox("Status Baru", ["Aktif", "Beku", "Tutup"], key="upd_status")
        if st.button("Update Status", width="stretch", disabled=(user and user.get("role")!="Admin")):
            row = acc_df[acc_df["bank_name"]==acc_choice].iloc[0]
            try:
                api_client.update_account_status(int(row["id"]), new_status)
                st.success("Status diperbarui.")
                st.rerun()
            except Exception as e:
                st.error(f"Gagal: {e}")

with col_table:
    st.markdown("#### Daftar Rekening")
    if not acc_df.empty:
        st.dataframe(acc_df[["id","bank_name","no_rekening","currency","opening_balance","limit_overdraft","gl_code","status"]],
                     width="stretch", hide_index=True,
                     column_config={
                         "opening_balance": st.column_config.NumberColumn("Opening", format="%.0f"),
                         "limit_overdraft": st.column_config.NumberColumn("Limit OD", format="%.0f"),
                     })
    st.markdown("#### Transfer Cepat (Shortcut)")
    st.caption("Via API Transfer Internal (langsung Paid).")
    if not acc_df.empty and len(acc_df) >= 2:
        accs = acc_df["bank_name"].tolist()
        c1, c2, c3 = st.columns(3)
        with c1:
            src = st.selectbox("Dari", accs, key="trf_src")
        with c2:
            dst = st.selectbox("Ke", [a for a in accs if a != src], key="trf_dst")
        with c3:
            amt = st.number_input("Jumlah (IDR)", min_value=0.0, step=100000.0, key="trf_amt")
        if st.button("Transfer Sekarang (Paid)", width="stretch", disabled=(user and user.get("role") not in ("Maker","Admin"))):
            if amt <= 0:
                st.error("Jumlah harus >0")
            else:
                try:
                    res = api_client.create_transaksi({
                        "tanggal": pd.Timestamp.now().strftime("%Y-%m-%d"),
                        "jenis": "Transfer Internal",
                        "divisi": "Management",
                        "kategori": "Transfer Antar Rekening",
                        "entitas_terkait": f"Transfer {src}→{dst}",
                        "akun_pembayaran": src,
                        "akun_tujuan": dst,
                        "subtotal": amt,
                        "currency": "IDR",
                        "fx_rate": 1.0,
                        "catatan": f"Transfer {src}→{dst}"
                    })
                    st.success(f"Transfer {format_idr(amt)} {src} → {dst} berhasil (ref={res.get('reference')}).")
                    st.rerun()
                except Exception as e:
                    st.error(f"Gagal transfer: {e}")
