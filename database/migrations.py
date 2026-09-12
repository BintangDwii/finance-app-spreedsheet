from sqlalchemy import text
from database.connection import get_cached_engine

DDL = """
-- Users untuk RBAC
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('Maker','Checker','Approver','Admin','Viewer')),
    divisi TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bank_name TEXT NOT NULL,
    no_rekening TEXT UNIQUE NOT NULL,
    currency TEXT NOT NULL DEFAULT 'IDR',
    opening_balance REAL NOT NULL DEFAULT 0,
    limit_overdraft REAL NOT NULL DEFAULT 0,
    gl_code TEXT,
    status TEXT NOT NULL DEFAULT 'Aktif' CHECK(status IN ('Aktif','Beku','Tutup')),
    created_at TEXT DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS transaksi (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    tanggal TEXT NOT NULL,
    jenis TEXT NOT NULL CHECK(jenis IN ('Pemasukan','Pengeluaran','Transfer Internal')),
    divisi TEXT NOT NULL,
    kategori TEXT NOT NULL,
    entitas_terkait TEXT NOT NULL,
    akun_pembayaran TEXT NOT NULL,
    subtotal REAL NOT NULL,
    ppn_11 REAL NOT NULL DEFAULT 0,
    diskon REAL NOT NULL DEFAULT 0,
    total_akhir REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'IDR',
    fx_rate REAL NOT NULL DEFAULT 1.0,
    due_date TEXT,
    reference_no TEXT,
    status TEXT NOT NULL CHECK(status IN ('Pending (Butuh Approval)','Approved (Menunggu Bayar)','Paid (Lunas)','Rejected (Ditolak)','Reconciled')),
    file_bukti TEXT DEFAULT '-',
    catatan TEXT DEFAULT '-',
    created_by TEXT,
    updated_at TEXT DEFAULT (datetime('now','localtime')),
    is_deleted INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS budgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bulan TEXT NOT NULL,
    divisi TEXT NOT NULL,
    kategori TEXT NOT NULL,
    planned REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'IDR',
    UNIQUE(bulan, divisi, kategori)
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaksi_id INTEGER NOT NULL,
    actor TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT,
    catatan TEXT,
    timestamp TEXT DEFAULT (datetime('now','localtime')),
    diff TEXT,
    FOREIGN KEY (transaksi_id) REFERENCES transaksi(id)
);

CREATE TABLE IF NOT EXISTS reconciliation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    statement_date TEXT NOT NULL,
    description TEXT,
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'IDR',
    matched_transaksi_id INTEGER,
    status TEXT NOT NULL DEFAULT 'Unreconciled' CHECK(status IN ('Unreconciled','Reconciled','Manual')),
    uploaded_by TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (matched_transaksi_id) REFERENCES transaksi(id)
);

CREATE INDEX IF NOT EXISTS idx_transaksi_tanggal ON transaksi(tanggal);
CREATE INDEX IF NOT EXISTS idx_transaksi_status ON transaksi(status);
CREATE INDEX IF NOT EXISTS idx_transaksi_jenis ON transaksi(jenis);
CREATE INDEX IF NOT EXISTS idx_budgets_bulan ON budgets(bulan);
"""

def run_migrations():
    engine = get_cached_engine()
    with engine.begin() as conn:
        for stmt in DDL.strip().split(";"):
            s = stmt.strip()
            if s:
                conn.execute(text(s))
    return True

if __name__ == "__main__":
    run_migrations()
    print("Migrations OK")
