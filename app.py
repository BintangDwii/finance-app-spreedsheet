import os
import pandas as pd
import streamlit as st
import plotly.express as px
from datetime import datetime

# --- KONFIGURASI HALAMAN ---
st.set_page_config(page_title="Corporate Finance System", page_icon="🏢", layout="wide")
DB_FILE = "finance_corporate_db.xlsx"
UPLOAD_DIR = "attachments"

# Buat folder untuk menyimpan bukti transaksi jika belum ada
os.makedirs(UPLOAD_DIR, exist_ok=True)

# --- INISIALISASI DATABASE EXCEL ---
def init_corporate_excel():
    if not os.path.exists(DB_FILE):
        with pd.ExcelWriter(DB_FILE, engine="openpyxl") as writer:
            # Sheet 1: Transaksi Utama
            pd.DataFrame(columns=[
                "ID", "Timestamp", "Tanggal", "Jenis", "Divisi", "Kategori", 
                "Entitas_Terkait", "Akun_Pembayaran", "Subtotal", "PPN_11", 
                "Diskon", "Total_Akhir", "Status", "File_Bukti", "Catatan"
            ]).to_excel(writer, sheet_name="Transaksi", index=False)
            
            # Sheet 2: Master Data (Untuk Dropdown yang sangat detail)
            pd.DataFrame({
                "Divisi": ["IT & Engineering", "Marketing & Sales", "HR & Admin", "Operasional", "Management", None],
                "Kategori_Pemasukan": ["Project/Client", "Retainer Contract", "Pendanaan/Investasi", "Bunga Bank", "Lainnya", None],
                "Kategori_Pengeluaran": ["Software & Cloud (AWS/GCP)", "Gaji & Tunjangan", "Pajak & Legalitas", "Iklan & Ads", "Sewa Kantor", "Lainnya"],
                "Akun_Pembayaran": ["BCA Corporate", "Mandiri Corporate", "Kas Kecil (Petty Cash)", "Kartu Kredit Perusahaan", None, None],
                "Entitas": ["Klien A", "Klien B", "Vendor AWS", "Vendor Google", "Karyawan Internal", "Lain-lain"],
                "Status_Transaksi": ["Pending (Butuh Approval)", "Approved (Menunggu Bayar)", "Paid (Lunas)", "Rejected (Ditolak)", None, None]
            }).to_excel(writer, sheet_name="Master_Data", index=False)

init_corporate_excel()

# --- LOAD DATA ---
def load_data():
    df_trans = pd.read_excel(DB_FILE, sheet_name="Transaksi")
    df_master = pd.read_excel(DB_FILE, sheet_name="Master_Data")
    return df_trans, df_master

df_trans, df_master = load_data()

# --- HEADER APLIKASI ---
st.title("🏢 Corporate Finance Management")
st.markdown("Sistem manajemen arus kas, persetujuan (approval), dan pelacakan anggaran perusahaan.")
st.markdown("---")

# --- NAVIGASI TAB ---
tab_dash, tab_input, tab_data = st.tabs([
    "📊 Dashboard & Analytics", 
    "➕ Input Transaksi", 
    "🗃️ Riwayat & Approval"
])

# ==========================================
# TAB 1: DASHBOARD & ANALYTICS
# ==========================================
with tab_dash:
    st.subheader("Ringkasan Keuangan Perusahaan")
    
    # Metrik hanya menghitung yang berstatus "Paid (Lunas)" untuk arus kas nyata
    df_paid = df_trans[df_trans['Status'] == 'Paid (Lunas)'] if not df_trans.empty else pd.DataFrame()
    
    total_masuk = df_paid[df_paid['Jenis'] == 'Pemasukan']['Total_Akhir'].sum() if not df_paid.empty else 0
    total_keluar = df_paid[df_paid['Jenis'] == 'Pengeluaran']['Total_Akhir'].sum() if not df_paid.empty else 0
    saldo_bersih = total_masuk - total_keluar
    
    # Hitung yang masih Pending
    total_pending = df_trans[df_trans['Status'] == 'Pending (Butuh Approval)']['Total_Akhir'].sum() if not df_trans.empty else 0

    # Tampilan Kartu Metrik
    c1, c2, c3, c4 = st.columns(4)
    c1.metric("Pendapatan Bersih (Lunas)", f"Rp {total_masuk:,.0f}")
    c2.metric("Pengeluaran (Lunas)", f"Rp {total_keluar:,.0f}")
    c3.metric("Saldo Kas Saat Ini", f"Rp {saldo_bersih:,.0f}", delta=f"Rp {saldo_bersih:,.0f}")
    c4.metric("Menunggu Approval (Pending)", f"Rp {total_pending:,.0f}", delta_color="inverse")

    st.markdown("<br>", unsafe_allow_html=True)

    if not df_paid.empty:
        col_chart1, col_chart2 = st.columns([2, 1])
        
        with col_chart1:
            st.markdown("**📈 Tren Arus Kas (Berdasarkan Transaksi Lunas)**")
            df_trend = df_paid.copy()
            df_trend['Tanggal'] = pd.to_datetime(df_trend['Tanggal'])
            df_grouped = df_trend.groupby(['Tanggal', 'Jenis'])['Total_Akhir'].sum().reset_index()
            
            fig_trend = px.line(
                df_grouped, x='Tanggal', y='Total_Akhir', color='Jenis',
                markers=True, line_shape='spline',
                color_discrete_map={"Pemasukan": "#10b981", "Pengeluaran": "#ef4444"}
            )
            fig_trend.update_traces(fill='tozeroy')
            fig_trend.update_layout(hovermode="x unified", margin=dict(l=0, r=0, t=30, b=0))
            st.plotly_chart(fig_trend, use_container_width=True)

        with col_chart2:
            st.markdown("**🍩 Distribusi Pengeluaran per Divisi**")
            df_expense = df_paid[df_paid['Jenis'] == 'Pengeluaran']
            if not df_expense.empty:
                fig_donut = px.pie(
                    df_expense, names='Divisi', values='Total_Akhir', hole=0.4,
                    color_discrete_sequence=px.colors.sequential.Teal
                )
                fig_donut.update_layout(margin=dict(l=0, r=0, t=30, b=0))
                st.plotly_chart(fig_donut, use_container_width=True)
            else:
                st.info("Belum ada data pengeluaran.")
    else:
        st.info("Belum ada data transaksi berstatus 'Paid (Lunas)' untuk menampilkan grafik.")


# ==========================================
# TAB 2: INPUT TRANSAKSI (FORM UI MODERN)
# ==========================================
with tab_input:
    st.subheader("Form Pencatatan Transaksi Perusahaan")
    
    with st.form("form_corporate", clear_on_submit=True):
        col1, col2 = st.columns(2)
        
        with col1:
            tgl = st.date_input("Tanggal Transaksi")
            jenis = st.selectbox("Jenis Arus Kas", ["Pemasukan", "Pengeluaran"])
            divisi = st.selectbox("Divisi / Departemen", df_master['Divisi'].dropna().tolist())
            
            kategori_list = df_master['Kategori_Pemasukan'].dropna().tolist() if jenis == "Pemasukan" else df_master['Kategori_Pengeluaran'].dropna().tolist()
            kategori = st.selectbox("Kategori Transaksi", kategori_list)
            
        with col2:
            entitas = st.selectbox("Entitas Terkait (Klien/Vendor/Karyawan)", df_master['Entitas'].dropna().tolist())
            akun = st.selectbox("Akun Pembayaran / Penerimaan", df_master['Akun_Pembayaran'].dropna().tolist())
            file_bukti = st.file_uploader("Upload Bukti (Invoice/Struk) - PDF/JPG/PNG", type=["pdf", "png", "jpg", "jpeg"])
            catatan = st.text_input("Keterangan Tambahan")

        st.markdown("---")
        st.markdown("**Rincian Nominal (IDR)**")
        
        col_nom1, col_nom2, col_nom3 = st.columns(3)
        with col_nom1:
            subtotal = st.number_input("Subtotal (Sebelum Pajak & Diskon)", min_value=0.0, step=50000.0)
        with col_nom2:
            is_ppn = st.checkbox("Tambah PPN (11%)")
            ppn_val = (subtotal * 0.11) if is_ppn else 0.0
            st.write(f"*PPN Terhitung: Rp {ppn_val:,.0f}*")
        with col_nom3:
            diskon = st.number_input("Diskon / Potongan (Jika ada)", min_value=0.0, step=10000.0)
            
        total_akhir = (subtotal + ppn_val) - diskon
        st.info(f"**GRAND TOTAL: Rp {total_akhir:,.0f}**")
        
        submit_btn = st.form_submit_button("Simpan Transaksi (Masuk antrean Pending)", use_container_width=True)
        
        if submit_btn:
            if subtotal <= 0:
                st.error("Subtotal harus lebih besar dari 0!")
            else:
                # Handle File Upload
                file_path = "-"
                if file_bukti is not None:
                    file_path = os.path.join(UPLOAD_DIR, file_bukti.name)
                    with open(file_path, "wb") as f:
                        f.write(file_bukti.getbuffer())

                new_id = int(df_trans['ID'].max() + 1) if not df_trans.empty and pd.notna(df_trans['ID'].max()) else 1001
                timestamp_now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                
                new_row = pd.DataFrame([{
                    "ID": new_id, "Timestamp": timestamp_now, "Tanggal": str(tgl),
                    "Jenis": jenis, "Divisi": divisi, "Kategori": kategori,
                    "Entitas_Terkait": entitas, "Akun_Pembayaran": akun,
                    "Subtotal": subtotal, "PPN_11": ppn_val, "Diskon": diskon,
                    "Total_Akhir": total_akhir, "Status": "Pending (Butuh Approval)",
                    "File_Bukti": file_path, "Catatan": catatan if catatan else "-"
                }])
                
                updated_df = pd.concat([df_trans, new_row], ignore_index=True)
                with pd.ExcelWriter(DB_FILE, engine="openpyxl", mode="w") as writer:
                    updated_df.to_excel(writer, sheet_name="Transaksi", index=False)
                    df_master.to_excel(writer, sheet_name="Master_Data", index=False)
                    
                st.success(f"Transaksi #{new_id} berhasil disimpan sebagai 'Pending'.")
                st.rerun()


# ==========================================
# TAB 3: RIWAYAT & MANAJEMEN APPROVAL
# ==========================================
with tab_data:
    st.subheader("Manajemen Data & Alur Persetujuan (Approval)")
    
    if not df_trans.empty:
        # Filter UI
        c_filt1, c_filt2, c_filt3 = st.columns(3)
        with c_filt1:
            f_status = st.selectbox("Filter Status", ["Semua"] + df_master['Status_Transaksi'].dropna().tolist())
        with c_filt2:
            f_divisi = st.selectbox("Filter Divisi", ["Semua"] + df_master['Divisi'].dropna().tolist())
        with c_filt3:
            f_search = st.text_input("Cari Entitas / Catatan")

        # Apply Filters
        df_show = df_trans.copy()
        if f_status != "Semua": df_show = df_show[df_show['Status'] == f_status]
        if f_divisi != "Semua": df_show = df_show[df_show['Divisi'] == f_divisi]
        if f_search:
            df_show = df_show[df_show['Entitas_Terkait'].str.contains(f_search, case=False) | df_show['Catatan'].str.contains(f_search, case=False)]

        # Tampilkan Tabel (Hanya kolom penting agar tidak kepanjangan)
        st.dataframe(
            df_show[["ID", "Tanggal", "Divisi", "Jenis", "Entitas_Terkait", "Total_Akhir", "Status"]],
            use_container_width=True,
            hide_index=True
        )
        
        st.markdown("---")
        
        # Fitur Update Status & Hapus (Untuk Manager/Admin)
        col_act1, col_act2 = st.columns(2)
        
        with col_act1:
            st.markdown("✅ **Update Status Transaksi (Approval)**")
            id_update = st.number_input("ID Transaksi untuk diupdate", min_value=1001, step=1)
            new_status = st.selectbox("Ubah Status Menjadi", df_master['Status_Transaksi'].dropna().tolist())
            if st.button("Update Status", type="primary"):
                if id_update in df_trans['ID'].values:
                    df_trans.loc[df_trans['ID'] == id_update, 'Status'] = new_status
                    with pd.ExcelWriter(DB_FILE, engine="openpyxl", mode="w") as writer:
                        df_trans.to_excel(writer, sheet_name="Transaksi", index=False)
                        df_master.to_excel(writer, sheet_name="Master_Data", index=False)
                    st.success(f"Status Transaksi #{id_update} diperbarui menjadi {new_status}!")
                    st.rerun()
                else:
                    st.error("ID tidak ditemukan.")

        with col_act2:
            st.markdown("🗑️ **Hapus Data Transaksi**")
            id_delete = st.number_input("ID Transaksi untuk dihapus", min_value=1001, step=1, key="del")
            if st.button("Hapus Data Permanen"):
                if id_delete in df_trans['ID'].values:
                    df_trans = df_trans[df_trans['ID'] != id_delete]
                    with pd.ExcelWriter(DB_FILE, engine="openpyxl", mode="w") as writer:
                        df_trans.to_excel(writer, sheet_name="Transaksi", index=False)
                        df_master.to_excel(writer, sheet_name="Master_Data", index=False)
                    st.success(f"Transaksi #{id_delete} berhasil dihapus.")
                    st.rerun()
                else:
                    st.error("ID tidak ditemukan.")
    else:
        st.info("Database transaksi masih kosong. Silakan input data di Tab 'Input Transaksi'.")
