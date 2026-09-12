import streamlit as st
import pandas as pd
from sqlalchemy import text
from database.connection import get_cached_engine, query_df
from utils.auth import current_user
from utils.formatters import format_idr

st.title("🏦 Manajemen Rekening")
st.caption("Daftar rekening corporate, saldo real-time, limit, dan transfer antar rekening.")
st.markdown("---")

user = current_user()
acc_df = query_df("SELECT * FROM accounts ORDER BY currency, bank_name")
trans_df = query_df("SELECT * FROM transaksi WHERE is_deleted=0 AND status='Paid (Lunas)'")
# hitung saldo per rekening
def saldo_acc(acc_name, currency, opening):
    df = trans_df[trans_df["akun_pembayaran"]==acc_name] if not trans_df.empty else pd.DataFrame()
    if df.empty:
        return opening, 0, 0
    # semua dalam currency akun? untuk IDR akun, idr; USD akun hitung USD
    if currency == "USD":
        inc = df[df["jenis"]=="Pemasukan"]["total_akhir"].sum()
        exp = df[df["jenis"]=="Pengeluaran"]["total_akhir"].sum()
        # transfer internal: sumber keluar, tujuan masuk — sudah tercatat sebagai Transfer Internal di kedua akun
        trf_in = df[(df["jenis"]=="Transfer Internal") & (df["entitas_terkait"].str.contains("dari", na=False))]["total_akhir"].sum()
        trf_out = df[(df["jenis"]=="Transfer Internal") & (df["entitas_terkait"].str.contains("ke", na=False))]["total_akhir"].sum()
        # sederhanakan: semua transfer dianggap sudah net di df
        saldo = opening + inc - exp  # untuk USD, transfer double count sudah benar karena 2 baris
        return saldo, inc, exp
    else:
        df["idr"] = df.apply(lambda r: r["total_akhir"]*r["fx_rate"] if r["currency"]=="USD" else r["total_akhir"], axis=1)
        inc = df[df["jenis"]=="Pemasukan"]["idr"].sum()
        exp = df[df["jenis"]=="Pengeluaran"]["idr"].sum()
        saldo = opening + inc - exp
        return saldo, inc, exp

# Grid saldo
cols = st.columns(3)
for i, (_, acc) in enumerate(acc_df.iterrows()):
    saldo, inc, exp = saldo_acc(acc["bank_name"], acc["currency"], acc["opening_balance"])
    with cols[i % 3]:
        with st.container(border=True):
            st.markdown(f"**{acc['bank_name']}**")
            st.caption(f"{acc['no_rekening']} • {acc['currency']} • GL {acc['gl_code']} • {acc['status']}")
            if acc["currency"] == "USD":
                st.metric("Saldo", f"$ {saldo:,.2f}", delta=f"Inc $ {inc:,.0f} / Exp $ {exp:,.0f}")
                st.caption(f"≈ Rp {saldo*16250:,.0f} | Limit overdraft Rp {acc['limit_overdraft']:,.0f}")
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
        submitted = st.form_submit_button("Tambah Rekening", width="stretch", disabled=(user["role"]!="Admin"))
        if submitted:
            if not bank_name or not no_rek:
                st.error("Nama dan No. Rekening wajib.")
            else:
                try:
                    engine = get_cached_engine()
                    with engine.begin() as conn:
                        conn.execute(text("""
                            INSERT INTO accounts (bank_name, no_rekening, currency, opening_balance, limit_overdraft, gl_code, status)
                            VALUES (:b, :rek, :cur, :open, :lim, :gl, :st)
                        """), {"b": bank_name, "rek": no_rek, "cur": currency, "open": opening, "lim": limit_od, "gl": gl, "st": status_acc})
                    st.success(f"Rekening {bank_name} ditambahkan.")
                    st.rerun()
                except Exception as e:
                    st.error(f"Gagal: {e}")
    # Edit existing
    st.markdown("#### Update Status Rekening")
    if not acc_df.empty:
        acc_choice = st.selectbox("Pilih Rekening", acc_df["bank_name"].tolist())
        new_status = st.selectbox("Status Baru", ["Aktif", "Beku", "Tutup"], key="upd_status")
        if st.button("Update Status", width="stretch", disabled=(user["role"]!="Admin")):
            engine = get_cached_engine()
            with engine.begin() as conn:
                conn.execute(text("UPDATE accounts SET status=:st WHERE bank_name=:b"), {"st": new_status, "b": acc_choice})
            st.success("Status diperbarui.")
            st.rerun()

with col_table:
    st.markdown("#### Daftar Rekening")
    st.dataframe(acc_df[["id","bank_name","no_rekening","currency","opening_balance","limit_overdraft","gl_code","status"]],
                 width="stretch", hide_index=True,
                 column_config={
                     "opening_balance": st.column_config.NumberColumn("Opening", format="%.0f"),
                     "limit_overdraft": st.column_config.NumberColumn("Limit OD", format="%.0f"),
                 })
    st.markdown("#### Transfer Cepat (Shortcut)")
    st.caption("Gunakan menu Input Transaksi untuk transfer dengan audit penuh. Di sini shortcut Paid langsung.")
    accs = acc_df["bank_name"].tolist()
    if len(accs) >= 2:
        c1, c2, c3 = st.columns(3)
        with c1:
            src = st.selectbox("Dari", accs, key="trf_src")
        with c2:
            dst = st.selectbox("Ke", [a for a in accs if a != src], key="trf_dst")
        with c3:
            amt = st.number_input("Jumlah (IDR)", min_value=0.0, step=100000.0, key="trf_amt")
        if st.button("Transfer Sekarang (Paid)", width="stretch", disabled=(user["role"] not in ("Maker","Admin"))):
            if amt <= 0:
                st.error("Jumlah harus >0")
            else:
                from datetime import datetime
                engine = get_cached_engine()
                with engine.begin() as conn:
                    max_id = conn.execute(text("SELECT COALESCE(MAX(id),1000) FROM transaksi")).scalar()
                    nid = int(max_id)+1
                    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                    tgl = datetime.now().strftime("%Y-%m-%d")
                    ref = f"TRF-{nid:04d}/{tgl[:7].replace('-','')}"
                    for acc, ent, nid2 in [(src, f"Transfer ke {dst}", nid), (dst, f"Transfer dari {src}", nid+1)]:
                        conn.execute(text("""
                            INSERT INTO transaksi (id, timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran,
                              subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, status, catatan, created_by, reference_no)
                            VALUES (:id, :ts, :tgl, 'Transfer Internal', 'Management', 'Transfer Antar Rekening', :ent, :akun, :sub, 0,0,:tot,'IDR',1.0,'Paid (Lunas)', :cat, :by, :ref)
                        """), {"id": nid2, "ts": ts, "tgl": tgl, "ent": ent, "akun": acc, "sub": amt, "tot": amt, "cat": f"Transfer {src}→{dst}", "by": user["username"], "ref": ref})
                st.success(f"Transfer {format_idr(amt)} {src} → {dst} berhasil.")
                st.rerun()
