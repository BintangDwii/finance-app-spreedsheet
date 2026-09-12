"""
Seed finance_corporate_db.xlsx dengan 1 bulan full transaksi pemasukan & pengeluaran.
- Default: Agustus 2026 (2026-08-01 s/d 2026-08-31)
- Overwrite mode (backup sudah dibuat) + preserve Master_Data
- Rasio: ~70% Pengeluaran, ~30% Pemasukan
- Status: Paid 65%, Approved 15%, Pending 15%, Rejected 5%
- PPN 11% random 60%, Diskon 0-5%
Run: venv/bin/python seed_corporate_db.py [--month 2026-08] [--mode overwrite|append]
"""
import os
import random
import argparse
from datetime import datetime, timedelta, date
import calendar
import pandas as pd

DB_FILE = "finance_corporate_db.xlsx"
UPLOAD_DIR = "attachments"
os.makedirs(UPLOAD_DIR, exist_ok=True)

# Master data pools (sinkron dengan app.py:27-33)
DIVISI = ["IT & Engineering", "Marketing & Sales", "HR & Admin", "Operasional", "Management"]
KAT_MASUK = ["Project/Client", "Retainer Contract", "Pendanaan/Investasi", "Bunga Bank", "Lainnya"]
KAT_KELUAR = ["Software & Cloud (AWS/GCP)", "Gaji & Tunjangan", "Pajak & Legalitas", "Iklan & Ads", "Sewa Kantor", "Lainnya"]
AKUN = ["BCA Corporate", "Mandiri Corporate", "Kas Kecil (Petty Cash)", "Kartu Kredit Perusahaan"]
ENTITAS = ["Klien A", "Klien B", "Vendor AWS", "Vendor Google", "Karyawan Internal", "Lain-lain"]
STATUS = ["Pending (Butuh Approval)", "Approved (Menunggu Bayar)", "Paid (Lunas)", "Rejected (Ditolak)"]

# Template catatan biar kelihatan real
CATATAN_MASUK = {
    "Project/Client": ["Termin 1 Project Klien A", "Pelunasan Project Klien B", "Pembayaran milestone SPRINT-42", "Invoice #INV-2026-08 - Klien A", "Down payment project baru"],
    "Retainer Contract": ["Retainer bulanan Klien A", "Retainer Klien B - Agustus", "Maintenance contract - Klien A", "Support retainer Q3"],
    "Pendanaan/Investasi": ["Pencairan investasi tahap 2", "Pendanaan investor - bridge round", "Injeksi modal operasional"],
    "Bunga Bank": ["Bunga deposito BCA", "Bunga giro Mandiri", "Bunga rekening koran"],
    "Lainnya": ["Refund vendor", "Pendapatan lain-lain", "Koreksi bank masuk"],
}
CATATAN_KELUAR = {
    "Software & Cloud (AWS/GCP)": ["Tagihan AWS Agustus", "GCP billing - BigQuery", "Langganan GitHub Enterprise", "Figma + Notion + Slack", "AWS EC2 & RDS", "Lisensi JetBrains"],
    "Gaji & Tunjangan": ["Gaji karyawan - IT", "Gaji tim Marketing", "THR / Bonus", "Gaji HR & Admin", "Gaji Management", "Tunjangan kesehatan"],
    "Pajak & Legalitas": ["PPh 21 Agustus", "Pajak PPN setoran", "Biaya konsultan legal", "Pengurusan izin usaha", "PPN masukan"],
    "Iklan & Ads": ["Google Ads campaign", "Meta Ads - lead gen", "TikTok Ads", "SEO tools & backlink", "Campaign Q3 launch"],
    "Sewa Kantor": ["Sewa kantor Agustus", "Service charge gedung", "Sewa co-working"],
    "Lainnya": ["ATK kantor", "Konsumsi rapat", "Transport & bensin", "Internet & listrik", "Maintenance AC", "Kurir & logistik"],
}

def random_subtotal(jenis, kategori):
    if jenis == "Pemasukan":
        if kategori == "Project/Client":
            return random.randint(15_000_000, 120_000_000)
        if kategori == "Retainer Contract":
            return random.randint(10_000_000, 50_000_000)
        if kategori == "Pendanaan/Investasi":
            return random.randint(50_000_000, 150_000_000)
        if kategori == "Bunga Bank":
            return random.randint(1_000_000, 5_000_000)
        return random.randint(2_000_000, 10_000_000)
    else:
        if kategori == "Software & Cloud (AWS/GCP)":
            return random.randint(2_000_000, 20_000_000)
        if kategori == "Gaji & Tunjangan":
            return random.randint(5_000_000, 25_000_000)
        if kategori == "Pajak & Legalitas":
            return random.randint(3_000_000, 15_000_000)
        if kategori == "Iklan & Ads":
            return random.randint(1_000_000, 12_000_000)
        if kategori == "Sewa Kantor":
            return random.randint(10_000_000, 20_000_000)
        return random.randint(500_000, 5_000_000)

def pick_divisi(jenis, kategori):
    # bobot divisi biar realistis
    if jenis == "Pemasukan":
        if kategori in ("Project/Client", "Retainer Contract"):
            return random.choices(["IT & Engineering", "Operasional", "Management", "Marketing & Sales"], weights=[35,25,20,20])[0]
        if kategori == "Pendanaan/Investasi":
            return "Management"
        return random.choice(DIVISI)
    else:
        if kategori == "Software & Cloud (AWS/GCP)":
            return "IT & Engineering"
        if kategori == "Gaji & Tunjangan":
            return random.choices(["HR & Admin", "IT & Engineering", "Marketing & Sales", "Operasional"], weights=[40,30,15,15])[0]
        if kategori == "Iklan & Ads":
            return "Marketing & Sales"
        if kategori == "Sewa Kantor":
            return random.choice(["Operasional", "Management", "HR & Admin"])
        if kategori == "Pajak & Legalitas":
            return random.choice(["Management", "HR & Admin", "Operasional"])
        return random.choice(DIVISI)

def pick_entitas(jenis, kategori):
    if jenis == "Pemasukan":
        if kategori in ("Project/Client", "Retainer Contract"):
            return random.choice(["Klien A", "Klien B"])
        if kategori == "Bunga Bank":
            return random.choice(["BCA Corporate", "Mandiri Corporate", "Lain-lain"])
        if kategori == "Pendanaan/Investasi":
            return "Lain-lain"
        return random.choice(ENTITAS)
    else:
        if kategori == "Software & Cloud (AWS/GCP)":
            return random.choice(["Vendor AWS", "Vendor Google"])
        if kategori == "Gaji & Tunjangan":
            return "Karyawan Internal"
        if kategori == "Iklan & Ads":
            return "Vendor Google"
        return random.choice(ENTITAS)

def pick_akun(jenis):
    if jenis == "Pemasukan":
        return random.choices(["BCA Corporate", "Mandiri Corporate"], weights=[60,40])[0]
    else:
        return random.choices(["BCA Corporate", "Mandiri Corporate", "Kartu Kredit Perusahaan", "Kas Kecil (Petty Cash)"], weights=[35,25,25,15])[0]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--month", default="2026-08", help="YYYY-MM format, e.g. 2026-08")
    parser.add_argument("--mode", default="overwrite", choices=["overwrite","append"], help="overwrite atau append")
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    random.seed(args.seed)

    try:
        year, month = map(int, args.month.split("-"))
    except:
        raise ValueError("Format --month harus YYYY-MM, contoh 2026-08")

    _, days_in_month = calendar.monthrange(year, month)
    start_date = date(year, month, 1)
    end_date = date(year, month, days_in_month)

    # Load existing atau init
    if os.path.exists(DB_FILE):
        df_trans_existing = pd.read_excel(DB_FILE, sheet_name="Transaksi")
        df_master = pd.read_excel(DB_FILE, sheet_name="Master_Data")
    else:
        df_trans_existing = pd.DataFrame(columns=["ID","Timestamp","Tanggal","Jenis","Divisi","Kategori","Entitas_Terkait","Akun_Pembayaran","Subtotal","PPN_11","Diskon","Total_Akhir","Status","File_Bukti","Catatan"])
        df_master = pd.DataFrame({
            "Divisi": ["IT & Engineering", "Marketing & Sales", "HR & Admin", "Operasional", "Management", None],
            "Kategori_Pemasukan": ["Project/Client", "Retainer Contract", "Pendanaan/Investasi", "Bunga Bank", "Lainnya", None],
            "Kategori_Pengeluaran": ["Software & Cloud (AWS/GCP)", "Gaji & Tunjangan", "Pajak & Legalitas", "Iklan & Ads", "Sewa Kantor", "Lainnya"],
            "Akun_Pembayaran": ["BCA Corporate", "Mandiri Corporate", "Kas Kecil (Petty Cash)", "Kartu Kredit Perusahaan", None, None],
            "Entitas": ["Klien A", "Klien B", "Vendor AWS", "Vendor Google", "Karyawan Internal", "Lain-lain"],
            "Status_Transaksi": ["Pending (Butuh Approval)", "Approved (Menunggu Bayar)", "Paid (Lunas)", "Rejected (Ditolak)", None, None]
        })

    # Tentukan ID awal
    if args.mode == "append" and not df_trans_existing.empty and pd.notna(df_trans_existing["ID"].max()):
        next_id = int(df_trans_existing["ID"].max()) + 1
        base_df = df_trans_existing.copy()
    else:
        next_id = 1001
        base_df = pd.DataFrame(columns=df_trans_existing.columns) if args.mode=="overwrite" else df_trans_existing.copy()
        if args.mode == "overwrite":
            # kosongkan untuk overwrite
            base_df = base_df.iloc[0:0]

    rows = []
    total_days = days_in_month
    sewa_count = 0

    for day in range(1, days_in_month+1):
        cur = date(year, month, day)
        weekday = cur.weekday()  # 0=Mon
        is_weekend = weekday >= 5
        # jumlah transaksi per hari - denser untuk 65+ per bulan
        if is_weekend:
            n = random.choices([0,1,2], weights=[35,45,20])[0]
        else:
            # weekday: 2-4 transaksi, skew ke 3
            n = random.choices([2,3,4], weights=[25,45,30])[0]

        # special: tanggal awal bulan ada pemasukan retainer + sewa
        # tanggal gajian: 25-28 sering ada gaji
        for _ in range(n):
            jenis = random.choices(["Pemasukan", "Pengeluaran"], weights=[30,70])[0]
            if jenis == "Pemasukan":
                kategori = random.choices(KAT_MASUK, weights=[40,30,10,10,10])[0]
            else:
                # batasi Sewa Kantor hanya 1-2x sebulan
                if sewa_count >= 2:
                    avail_keluar = [k for k in KAT_KELUAR if k != "Sewa Kantor"]
                    kategori = random.choices(avail_keluar, weights=[25,25,15,25,10])[0]
                else:
                    kategori = random.choices(KAT_KELUAR, weights=[25,25,15,25,5,5])[0]
                    if kategori == "Sewa Kantor":
                        sewa_count += 1

            # paksa pemasukan di tgl 1-5 dan 15-20 biar cashflow sehat
            if day in [1,2,3,15,16,20] and random.random() < 0.6:
                jenis = "Pemasukan"
                kategori = random.choice(["Project/Client","Retainer Contract"])
            # paksa pengeluaran gaji di tgl 25-28
            if day in [25,26,27,28] and random.random() < 0.5:
                jenis = "Pengeluaran"
                kategori = "Gaji & Tunjangan"

            divisi = pick_divisi(jenis, kategori)
            entitas = pick_entitas(jenis, kategori)
            akun = pick_akun(jenis)
            subtotal = random_subtotal(jenis, kategori)
            # bulatkan ke 100rb biar rapi
            subtotal = round(subtotal / 100000) * 100000

            is_ppn = random.random() < 0.60
            ppn = round(subtotal * 0.11) if is_ppn else 0.0
            # diskon 30% chance, 2-10%
            if random.random() < 0.30:
                diskon = round(subtotal * random.uniform(0.02, 0.10) / 10000) * 10000
            else:
                diskon = 0.0
            total_akhir = (subtotal + ppn) - diskon

            status = random.choices(STATUS, weights=[15,15,65,5])[0]

            # timestamp jam kerja 08:00-17:00
            hour = random.randint(8,17)
            minute = random.randint(0,59)
            second = random.randint(0,59)
            ts = datetime(year, month, day, hour, minute, second).strftime("%Y-%m-%d %H:%M:%S")

            catatan_pool = CATATAN_MASUK[kategori] if jenis=="Pemasukan" else CATATAN_KELUAR[kategori]
            catatan = random.choice(catatan_pool)

            rows.append({
                "ID": next_id,
                "Timestamp": ts,
                "Tanggal": str(cur),
                "Jenis": jenis,
                "Divisi": divisi,
                "Kategori": kategori,
                "Entitas_Terkait": entitas,
                "Akun_Pembayaran": akun,
                "Subtotal": float(subtotal),
                "PPN_11": float(ppn),
                "Diskon": float(diskon),
                "Total_Akhir": float(total_akhir),
                "Status": status,
                "File_Bukti": "-",
                "Catatan": catatan
            })
            next_id += 1

    df_new = pd.DataFrame(rows, columns=["ID","Timestamp","Tanggal","Jenis","Divisi","Kategori","Entitas_Terkait","Akun_Pembayaran","Subtotal","PPN_11","Diskon","Total_Akhir","Status","File_Bukti","Catatan"])
    # sort by Tanggal + ID
    df_new = df_new.sort_values(["Tanggal","ID"]).reset_index(drop=True)
    # re-assign ID sequential kalau overwrite
    if args.mode == "overwrite":
        df_new["ID"] = range(1001, 1001+len(df_new))
        df_final = df_new
    else:
        df_final = pd.concat([base_df, df_new], ignore_index=True)
        df_final = df_final.sort_values(["Tanggal","ID"]).reset_index(drop=True)

    # Tulis ke Excel
    with pd.ExcelWriter(DB_FILE, engine="openpyxl", mode="w") as writer:
        df_final.to_excel(writer, sheet_name="Transaksi", index=False)
        df_master.to_excel(writer, sheet_name="Master_Data", index=False)

    # Summary
    print(f"Seed selesai: {args.month} ({start_date} s/d {end_date})")
    print(f"Mode: {args.mode}, Total transaksi baru: {len(df_new)}, Total di DB: {len(df_final)}")
    print(f"ID range: {df_final['ID'].min()} - {df_final['ID'].max()}")
    print(f"Pemasukan: {(df_final['Jenis']=='Pemasukan').sum()} rows, Rp {df_final[df_final['Jenis']=='Pemasukan']['Total_Akhir'].sum():,.0f}")
    print(f"Pengeluaran: {(df_final['Jenis']=='Pengeluaran').sum()} rows, Rp {df_final[df_final['Jenis']=='Pengeluaran']['Total_Akhir'].sum():,.0f}")
    # breakdown status
    print("Per Status:")
    print(df_final["Status"].value_counts().to_string())
    print("Per Divisi:")
    print(df_final["Divisi"].value_counts().to_string())
    # cek formula
    mismatch = df_final[abs((df_final["Subtotal"]+df_final["PPN_11"]-df_final["Diskon"])-df_final["Total_Akhir"]) > 1]
    print(f"Formula check mismatch: {len(mismatch)} rows" + (" - OK" if len(mismatch)==0 else " - ERROR!"))
    print(f"File: {os.path.abspath(DB_FILE)}")

if __name__ == "__main__":
    main()
