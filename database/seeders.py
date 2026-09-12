import os
import hashlib
import random
import calendar
from datetime import datetime, date
import pandas as pd
from sqlalchemy import text
from database.connection import get_cached_engine
from database.migrations import run_migrations

EXCEL_FILE = "finance_corporate_db.xlsx"
DATA_DIR = "data"

def hash_pw(pw: str) -> str:
    return hashlib.sha256(pw.encode()).hexdigest()

def seed_accounts(conn):
    # 7 akun: 4 existing + 3 baru (BRI, BNI, USD)
    accounts = [
        ("BCA Corporate", "8001234567", "IDR", 500_000_000, 0, "1101", "Aktif"),
        ("Mandiri Corporate", "1400012345678", "IDR", 300_000_000, 0, "1102", "Aktif"),
        ("Kas Kecil (Petty Cash)", "CASH-001", "IDR", 25_000_000, 0, "1103", "Aktif"),
        ("Kartu Kredit Perusahaan", "CC-5200-001", "IDR", 0, 100_000_000, "2101", "Aktif"),
        ("BRI Corporate", "1200123456789", "IDR", 250_000_000, 0, "1104", "Aktif"),
        ("BNI Corporate", "150012345678", "IDR", 200_000_000, 0, "1105", "Aktif"),
        ("BCA USD", "8009876543", "USD", 25000, 0, "1106", "Aktif"),
    ]
    for b, rek, cur, open_bal, limit, gl, st in accounts:
        conn.execute(text("""
            INSERT OR IGNORE INTO accounts (bank_name, no_rekening, currency, opening_balance, limit_overdraft, gl_code, status)
            VALUES (:b, :rek, :cur, :open_bal, :limit, :gl, :st)
        """), {"b": b, "rek": rek, "cur": cur, "open_bal": open_bal, "limit": limit, "gl": gl, "st": st})

def seed_users(conn):
    users = [
        ("admin", hash_pw("admin123"), "Admin", None),
        ("maker1", hash_pw("maker123"), "Maker", "IT & Engineering"),
        ("checker1", hash_pw("checker123"), "Checker", "Management"),
        ("approver1", hash_pw("approver123"), "Approver", "Management"),
        ("viewer1", hash_pw("viewer123"), "Viewer", None),
    ]
    for u, h, r, d in users:
        conn.execute(text("""
            INSERT OR IGNORE INTO users (username, password_hash, role, divisi)
            VALUES (:u, :h, :r, :d)
        """), {"u": u, "h": h, "r": r, "d": d})

def seed_transaksi(conn):
    # cek sudah ada?
    cnt = conn.execute(text("SELECT COUNT(*) FROM transaksi")).scalar()
    if cnt and cnt > 0:
        print(f"Transaksi sudah ada {cnt} rows, skip migrasi Excel")
        return cnt
    if not os.path.exists(EXCEL_FILE):
        print(f"{EXCEL_FILE} tidak ditemukan, skip")
        return 0
    df = pd.read_excel(EXCEL_FILE, sheet_name="Transaksi")
    # tambah kolom multi-currency jika belum ada
    # 5% transaksi jadi USD (pilih random Pemasukan Project/Investasi)
    random.seed(42)
    usd_idx = set(random.sample(list(df.index), k=max(3, int(len(df)*0.05))))
    inserted = 0
    for _, r in df.iterrows():
        is_usd = r.name in usd_idx and r["Akun_Pembayaran"] in ["BCA Corporate", "Mandiri Corporate"]
        currency = "USD" if is_usd else "IDR"
        fx = round(random.uniform(16200, 16500), 2) if is_usd else 1.0
        akun = "BCA USD" if is_usd else r["Akun_Pembayaran"]
        # USD: subtotal disimpan dalam USD (konversi dari IDR / fx)
        if is_usd:
            subtotal_usd = round(float(r["Subtotal"]) / fx, 2)
            ppn_usd = round(float(r["PPN_11"]) / fx, 2)
            diskon_usd = round(float(r["Diskon"]) / fx, 2)
            total_usd = round(float(r["Total_Akhir"]) / fx, 2)
        else:
            subtotal_usd = float(r["Subtotal"])
            ppn_usd = float(r["PPN_11"])
            diskon_usd = float(r["Diskon"])
            total_usd = float(r["Total_Akhir"])
        # due_date = tanggal + 30 hari untuk Pengeluaran
        try:
            tgl = pd.to_datetime(r["Tanggal"])
            due = (tgl + pd.Timedelta(days=30)).strftime("%Y-%m-%d") if r["Jenis"] == "Pengeluaran" else None
        except:
            due = None
        ref = f"REF-{int(r['ID']):04d}/{pd.to_datetime(r['Tanggal']).strftime('%Y%m')}" if pd.notna(r["ID"]) else None
        conn.execute(text("""
            INSERT OR IGNORE INTO transaksi
            (id, timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran,
             subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, due_date, reference_no,
             status, file_bukti, catatan, created_by)
            VALUES (:id, :ts, :tgl, :jenis, :divisi, :kat, :ent, :akun,
                    :sub, :ppn, :disk, :tot, :cur, :fx, :due, :ref,
                    :st, :bukti, :cat, :by)
        """), {
            "id": int(r["ID"]),
            "ts": str(r["Timestamp"]),
            "tgl": str(r["Tanggal"])[:10],
            "jenis": str(r["Jenis"]),
            "divisi": str(r["Divisi"]),
            "kat": str(r["Kategori"]),
            "ent": str(r["Entitas_Terkait"]),
            "akun": str(akun),
            "sub": subtotal_usd,
            "ppn": ppn_usd,
            "disk": diskon_usd,
            "tot": total_usd,
            "cur": currency,
            "fx": fx,
            "due": due,
            "ref": ref,
            "st": str(r["Status"]),
            "bukti": str(r["File_Bukti"]),
            "cat": str(r["Catatan"]),
            "by": "maker1",
        })
        inserted += 1
    print(f"Migrasi transaksi: {inserted} rows dari Excel ({len(usd_idx)} jadi USD)")
    return inserted

def seed_budgets(conn):
    cnt = conn.execute(text("SELECT COUNT(*) FROM budgets")).scalar()
    if cnt and cnt >= 60:
        print(f"Budgets sudah ada {cnt}, skip")
        return cnt
    # hapus dulu jika parsial
    conn.execute(text("DELETE FROM budgets"))
    # Hitung rata-rata actual per kategori per divisi dari Paid
    df = pd.read_sql(text("SELECT divisi, kategori, AVG(total_akhir*fx_rate) as avg_idr FROM transaksi WHERE status='Paid (Lunas)' AND jenis='Pengeluaran' GROUP BY divisi, kategori"), conn)
    # fallback avg
    fallback = {
        "Software & Cloud (AWS/GCP)": 12_000_000,
        "Gaji & Tunjangan": 15_000_000,
        "Pajak & Legalitas": 8_000_000,
        "Iklan & Ads": 6_000_000,
        "Sewa Kantor": 15_000_000,
        "Lainnya": 3_000_000,
        "Project/Client": 60_000_000,
        "Retainer Contract": 30_000_000,
        "Pendanaan/Investasi": 80_000_000,
        "Bunga Bank": 3_000_000,
    }
    divisi = ["IT & Engineering", "Marketing & Sales", "HR & Admin", "Operasional", "Management"]
    kategori_all = ["Software & Cloud (AWS/GCP)", "Gaji & Tunjangan", "Pajak & Legalitas", "Iklan & Ads", "Sewa Kantor", "Lainnya"]
    random.seed(123)
    rows = 0
    for bulan in ["2026-08", "2026-09"]:
        for d in divisi:
            for k in kategori_all:
                # cari avg
                m = df[(df["divisi"]==d) & (df["kategori"]==k)]
                if not m.empty:
                    avg = float(m.iloc[0]["avg_idr"])
                    planned = avg * random.uniform(1.1, 1.35) * random.randint(1,2)  # 1-2 transaksi per budget cell
                else:
                    planned = fallback.get(k, 5_000_000) * random.uniform(1.0, 1.5)
                planned = round(planned / 100000) * 100000
                conn.execute(text("""
                    INSERT OR IGNORE INTO budgets (bulan, divisi, kategori, planned, currency)
                    VALUES (:b, :d, :k, :p, 'IDR')
                """), {"b": bulan, "d": d, "k": k, "p": planned})
                rows += 1
    # tambah budget pemasukan (supaya variance lengkap) - 10 rows tambahan
    kat_masuk = ["Project/Client", "Retainer Contract", "Pendanaan/Investasi", "Bunga Bank", "Lainnya"]
    for bulan in ["2026-08", "2026-09"]:
        for k in kat_masuk:
            # global pemasukan planned
            planned = fallback[k] * random.uniform(2.0, 3.0)
            planned = round(planned / 100000) * 100000
            # simpan di divisi Management sebagai aggregate
            conn.execute(text("""
                INSERT OR IGNORE INTO budgets (bulan, divisi, kategori, planned, currency)
                VALUES (:b, 'Management', :k, :p, 'IDR')
            """), {"b": bulan, "k": k, "p": planned})
            rows += 1
            if rows >= 60:
                break
        if rows >= 60:
            break
    print(f"Seed budgets: {rows} rows")
    return rows

def run_seeders():
    run_migrations()
    engine = get_cached_engine()
    with engine.begin() as conn:
        seed_accounts(conn)
        seed_users(conn)
        seed_transaksi(conn)
        seed_budgets(conn)
    # buat template CSV rekonsiliasi
    os.makedirs(DATA_DIR, exist_ok=True)
    tmpl = os.path.join(DATA_DIR, "template_mutasi.csv")
    if not os.path.exists(tmpl):
        with open(tmpl, "w") as f:
            f.write("tanggal,deskripsi,amount,currency,no_referensi\n")
            f.write("2026-08-15,Transfer masuk Klien A,73260000,IDR,REF-1004/202608\n")
            f.write("2026-08-03,Tagihan AWS,11322000,IDR,REF-1005/202608\n")
            f.write("2026-09-10,Payment Vendor Google,5000000,IDR,INV-001\n")
        print(f"Template CSV dibuat: {tmpl}")
    print("Seeders OK")

if __name__ == "__main__":
    run_seeders()
