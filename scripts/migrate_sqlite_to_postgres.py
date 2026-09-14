"""One-time migration SQLite -> Postgres (scheduled downtime 15-30m).

Idempotent: uses explicit ids + ON CONFLICT DO NOTHING, then fixes sequences.
Run AFTER backend has executed sqlx migrations (it auto-runs on boot).

Usage:
  DATABASE_URL=postgres://finance:finance123@127.0.0.1:5435/corporate python3 scripts/migrate_sqlite_to_postgres.py
"""

import os
import sqlite3
import sys
import urllib.parse
from pathlib import Path

import psycopg2

SQLITE = "data/corporate.db"
PG_URL = os.getenv(
    "DATABASE_URL", "postgres://finance:finance123@localhost:5432/corporate"
)


def pg_connect(url):
    u = urllib.parse.urlparse(url)
    return psycopg2.connect(
        dbname=u.path.lstrip("/"),
        user=u.username,
        password=u.password,
        host=u.hostname,
        port=u.port or 5432,
    )


print(f"Migrating {SQLITE} -> {PG_URL}")
if not Path(SQLITE).exists():
    print("SQLite not found, nothing to migrate")
    sys.exit(0)

lite = sqlite3.connect(SQLITE)
lite.row_factory = sqlite3.Row

pg = pg_connect(PG_URL)
cur = pg.cursor()

moved = {}


def copy(table, cols, pg_cols=None, transform=None):
    rows = lite.execute(f"SELECT {', '.join(cols)} FROM {table}").fetchall()
    targets = pg_cols or cols
    placeholders = ", ".join(["%s"] * len(targets))
    n = 0
    for r in rows:
        vals = [r[c] for c in cols]
        if transform:
            vals = transform(vals)
        cur.execute(
            f"INSERT INTO {table} ({', '.join(targets)}) VALUES ({placeholders}) "
            f"ON CONFLICT DO NOTHING",
            vals,
        )
        n += cur.rowcount
    moved[table] = (len(rows), n)


# Users: only missing usernames (PG argon2 users stay canonical; legacy sha256
# hashes are transparently upgraded to argon2 on next login).
cur.execute("SELECT username FROM users")
existing = {r[0] for r in cur.fetchall()}
lite_users = [
    r for r in lite.execute("SELECT * FROM users").fetchall()
    if r["username"] not in existing
]
for r in lite_users:
    cur.execute(
        "INSERT INTO users (id, username, password_hash, role, divisi) "
        "VALUES (%s,%s,%s,%s,%s) ON CONFLICT DO NOTHING",
        (r["id"], r["username"], r["password_hash"], r["role"], r["divisi"]),
    )
moved["users"] = (5, len(lite_users))

# Accounts: only missing bank_name (003 seed already inserted the same 7).
cur.execute("SELECT bank_name FROM accounts")
existing_acct = {r[0] for r in cur.fetchall()}
lite_accts = [
    r for r in lite.execute("SELECT * FROM accounts").fetchall()
    if r["bank_name"] not in existing_acct
]
for r in lite_accts:
    cur.execute(
        "INSERT INTO accounts (id, bank_name, no_rekening, currency, opening_balance,"
        " limit_overdraft, gl_code, status) VALUES (%s,%s,%s,%s,%s,%s,%s,%s)"
        " ON CONFLICT DO NOTHING",
        (r["id"], r["bank_name"], r["no_rekening"], r["currency"],
         r["opening_balance"], r["limit_overdraft"], r["gl_code"], r["status"]),
    )
moved["accounts"] = (7, len(lite_accts))


def trx_t(vals):
    d = dict(zip(
        ["id", "timestamp", "tanggal", "jenis", "divisi", "kategori", "entitas_terkait",
         "akun_pembayaran", "subtotal", "ppn_11", "diskon", "total_akhir", "currency",
         "fx_rate", "due_date", "reference_no", "status", "file_bukti", "catatan",
         "created_by", "updated_at", "is_deleted"],
        vals,
    ))
    d["is_deleted"] = bool(d["is_deleted"])
    d["total_akhir"] = round(float(d["subtotal"]) + float(d["ppn_11"] or 0) - float(d["diskon"] or 0), 2)
    return [d[c] for c in [
        "id", "timestamp", "tanggal", "jenis", "divisi", "kategori", "entitas_terkait",
        "akun_pembayaran", "subtotal", "ppn_11", "diskon", "total_akhir", "currency",
        "fx_rate", "due_date", "reference_no", "status", "file_bukti", "catatan",
        "created_by", "updated_at", "is_deleted"]]


copy("transaksi",
     ["id", "timestamp", "tanggal", "jenis", "divisi", "kategori", "entitas_terkait",
      "akun_pembayaran", "subtotal", "ppn_11", "diskon", "total_akhir", "currency",
      "fx_rate", "due_date", "reference_no", "status", "file_bukti", "catatan",
      "created_by", "updated_at", "is_deleted"],
     transform=trx_t)

copy("budgets", ["id", "bulan", "divisi", "kategori", "planned", "currency"])
copy("audit_log",
     ["id", "transaksi_id", "actor", "from_status", "to_status", "catatan", "timestamp"])
copy("reconciliation",
     ["id", "statement_date", "description", "amount", "currency",
      "matched_transaksi_id", "status", "uploaded_by"])

# Fix sequences so future inserts don't collide with migrated ids.
for table, seq in [("users", "users_id_seq"), ("accounts", "accounts_id_seq"),
                   ("transaksi", "transaksi_id_seq"), ("budgets", "budgets_id_seq"),
                   ("audit_log", "audit_log_id_seq"),
                   ("reconciliation", "reconciliation_id_seq")]:
    cur.execute(f"SELECT setval('{seq}', COALESCE((SELECT MAX(id) FROM {table}), 1))")

pg.commit()

for table, (seen, inserted) in moved.items():
    print(f"{table}: sqlite={seen} inserted={inserted}")
print("Migration OK")
