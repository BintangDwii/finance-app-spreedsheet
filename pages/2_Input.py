import os
import uuid
from datetime import datetime, date
import streamlit as st
from sqlalchemy import text
from database.connection import get_cached_engine, query_df
from utils.auth import current_user, can_create
from utils.formatters import format_idr

st.title("➕ Input Transaksi")
st.caption("Maker membuat transaksi — akan masuk antrean Pending (Butuh Approval).")
st.markdown("---")

user = current_user()
if not can_create():
    st.error("Hanya role Maker/Admin yang bisa input transaksi.")
    st.stop()

# Load master dari DB
acc_df = query_df("SELECT bank_name, currency FROM accounts WHERE status='Aktif'")
accounts = acc_df["bank_name"].tolist() if not acc_df.empty else ["BCA Corporate"]
divisi_list = ["IT & Engineering", "Marketing & Sales", "HR & Admin", "Operasional", "Management"]
kat_masuk = ["Project/Client", "Retainer Contract", "Pendanaan/Investasi", "Bunga Bank", "Lainnya"]
kat_keluar = ["Software & Cloud (AWS/GCP)", "Gaji & Tunjangan", "Pajak & Legalitas", "Iklan & Ads", "Sewa Kantor", "Lainnya"]
entitas_list = ["Klien A", "Klien B", "Vendor AWS", "Vendor Google", "Karyawan Internal", "Lain-lain"]

UPLOAD_DIR = "attachments"
os.makedirs(UPLOAD_DIR, exist_ok=True)

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
            fx_rate = st.number_input("Kurs (IDR per USD)", min_value=1.0, value=16250.0, step=100.0)
        due_date = None
        if jenis == "Pengeluaran":
            due_date = st.date_input("Jatuh Tempo (Opsional)", value=None)
        reference_no = st.text_input("No. Referensi (Opsional)", placeholder="INV-001 / REF-xxxx")
        file_bukti = st.file_uploader("Upload Bukti (PDF/JPG/PNG)", type=["pdf","png","jpg","jpeg"])
        catatan = st.text_input("Keterangan Tambahan")

    st.markdown("---")
    st.markdown("**Rincian Nominal**")
    col_nom1, col_nom2, col_nom3 = st.columns(3)
    with col_nom1:
        subtotal = st.number_input(f"Subtotal ({currency})", min_value=0.0, step=50000.0 if currency=="IDR" else 100.0)
    with col_nom2:
        is_ppn = st.checkbox("Tambah PPN (11%)", disabled=(jenis=="Transfer Internal"))
        ppn_val = (subtotal * 0.11) if is_ppn and jenis != "Transfer Internal" else 0.0
        ppn_str = format_idr(ppn_val, currency) if currency == "IDR" else f"$ {ppn_val:,.2f}"
        st.write(f"*PPN Terhitung: {ppn_str}*")
    with col_nom3:
        diskon = st.number_input(f"Diskon / Potongan ({currency})", min_value=0.0, step=10000.0 if currency=="IDR" else 10.0, disabled=(jenis=="Transfer Internal"))
    total_akhir = (subtotal + ppn_val) - diskon
    if currency == "USD":
        st.info(f"**GRAND TOTAL: $ {total_akhir:,.2f}  ≈ Rp {total_akhir*fx_rate:,.0f}**")
    else:
        st.info(f"**GRAND TOTAL: Rp {total_akhir:,.0f}**")

    submitted = st.form_submit_button("Simpan Transaksi (Pending)", width="stretch")
    if submitted:
        if subtotal <= 0:
            st.error("Subtotal harus lebih besar dari 0!")
        elif jenis == "Transfer Internal" and not akun_tujuan:
            st.error("Pilih akun tujuan transfer.")
        else:
            file_path = "-"
            if file_bukti is not None:
                ext = os.path.splitext(file_bukti.name)[1]
                fname = f"{uuid.uuid4().hex}{ext}"
                file_path = os.path.join(UPLOAD_DIR, fname)
                with open(file_path, "wb") as f:
                    f.write(file_bukti.getbuffer())
                # simpan nama asli di catatan jika perlu
            engine = get_cached_engine()
            with engine.begin() as conn:
                max_id = conn.execute(text("SELECT COALESCE(MAX(id),1000) FROM transaksi")).scalar()
                new_id = int(max_id) + 1
                ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                # Untuk transfer internal: buat 2 baris? Sederhanakan: 1 baris jenis Transfer dengan akun sumber, catatan sebut tujuan
                # Alternatif: buat 2 baris atomik — buat pengeluaran dari sumber + pemasukan ke tujuan dengan reference sama
                if jenis == "Transfer Internal":
                    ref = reference_no if reference_no else f"TRF-{new_id:04d}/{tgl.strftime('%Y%m')}"
                    # Baris 1: keluar dari sumber
                    conn.execute(text("""
                        INSERT INTO transaksi (id, timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran,
                          subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, due_date, reference_no, status, file_bukti, catatan, created_by)
                        VALUES (:id, :ts, :tgl, 'Transfer Internal', :divisi, :kat, :ent, :akun, :sub, 0, 0, :tot, :cur, :fx, :due, :ref, 'Paid (Lunas)', :bukti, :cat, :by)
                    """), {
                        "id": new_id, "ts": ts, "tgl": str(tgl), "divisi": divisi, "kat": "Transfer Antar Rekening",
                        "ent": f"Transfer ke {akun_tujuan}", "akun": akun, "sub": subtotal, "tot": total_akhir,
                        "cur": currency, "fx": fx_rate, "due": str(due_date) if due_date else None, "ref": ref,
                        "bukti": file_path, "cat": f"Transfer {akun} → {akun_tujuan}" + (f" | {catatan}" if catatan else ""), "by": user["username"]
                    })
                    # Baris 2: masuk ke tujuan (mirror, Paid juga)
                    conn.execute(text("""
                        INSERT INTO transaksi (id, timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran,
                          subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, due_date, reference_no, status, file_bukti, catatan, created_by)
                        VALUES (:id, :ts, :tgl, 'Transfer Internal', :divisi, :kat, :ent, :akun, :sub, 0, 0, :tot, :cur, :fx, :due, :ref, 'Paid (Lunas)', :bukti, :cat, :by)
                    """), {
                        "id": new_id+1, "ts": ts, "tgl": str(tgl), "divisi": divisi, "kat": "Transfer Antar Rekening",
                        "ent": f"Transfer dari {akun}", "akun": akun_tujuan, "sub": subtotal, "tot": total_akhir,
                        "cur": currency, "fx": fx_rate, "due": str(due_date) if due_date else None, "ref": ref,
                        "bukti": file_path, "cat": f"Transfer {akun} → {akun_tujuan}" + (f" | {catatan}" if catatan else ""), "by": user["username"]
                    })
                    # audit
                    conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, '-', 'Paid (Lunas)', 'Transfer Internal')"), {"tid": new_id, "actor": user["username"]})
                    conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, '-', 'Paid (Lunas)', 'Transfer Internal')"), {"tid": new_id+1, "actor": user["username"]})
                    st.success(f"Transfer #{new_id}/{new_id+1} {akun} → {akun_tujuan} sebesar {format_idr(total_akhir, currency)} berhasil (langsung Paid).")
                else:
                    ref = reference_no if reference_no else f"REF-{new_id:04d}/{tgl.strftime('%Y%m')}"
                    conn.execute(text("""
                        INSERT INTO transaksi (id, timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran,
                          subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, due_date, reference_no, status, file_bukti, catatan, created_by)
                        VALUES (:id, :ts, :tgl, :jenis, :divisi, :kat, :ent, :akun, :sub, :ppn, :disk, :tot, :cur, :fx, :due, :ref, 'Pending (Butuh Approval)', :bukti, :cat, :by)
                    """), {
                        "id": new_id, "ts": ts, "tgl": str(tgl), "jenis": jenis, "divisi": divisi, "kat": kategori,
                        "ent": entitas, "akun": akun, "sub": subtotal, "ppn": ppn_val, "disk": diskon, "tot": total_akhir,
                        "cur": currency, "fx": fx_rate, "due": str(due_date) if due_date else None, "ref": ref,
                        "bukti": file_path, "cat": catatan if catatan else "-", "by": user["username"]
                    })
                    conn.execute(text("INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES (:tid, :actor, '-', 'Pending (Butuh Approval)', :cat)"), {"tid": new_id, "actor": user["username"], "cat": catatan or "-"})
                    st.success(f"Transaksi #{new_id} berhasil disimpan sebagai Pending.")
            st.rerun()
