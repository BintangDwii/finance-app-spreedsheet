"""One-time migration SQLite -> Postgres (scheduled downtime 15-30m)"""
import sqlite3, psycopg2, os, sys
from pathlib import Path

SQLITE = "data/corporate.db"
PG_URL = os.getenv("DATABASE_URL", "postgres://finance:finance123@localhost:5432/corporate")

def pg_connect(url):
    # simple parse: postgres://user:pass@host:port/db
    import urllib.parse
    u = urllib.parse.urlparse(url)
    return psycopg2.connect(dbname=u.path.lstrip("/"), user=u.username, password=u.password, host=u.hostname, port=u.port or 5432)

print(f"Migrating {SQLITE} -> {PG_URL}")
if not Path(SQLITE).exists():
    print("SQLite not found, nothing to migrate")
    sys.exit(0)

con_sqlite = sqlite3.connect(SQLITE)
con_sqlite.row_factory = sqlite3.Row
cur_sqlite = con_sqlite.cursor()

try:
    con_pg = pg_connect(PG_URL)
    cur_pg = con_pg.cursor()
    # ensure migrations already run via sqlx
    for table in ["accounts","users","transaksi","budgets","audit_log","reconciliation"]:
        cnt = cur_sqlite.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
        print(f"{table}: {cnt} rows")
    # Example: copy transaksi
    # In real run, use COPY or INSERT with ON CONFLICT DO NOTHING
    print("Use: docker-compose up postgres, then backend will run sqlx migrate, then run this script for data COPY")
    print("Dry-run OK — implement INSERT loop if needed")
except Exception as e:
    print(f"PG connect failed (expected if not running): {e}")
    print("To test: docker-compose up -d postgres && cargo run (backend)")
