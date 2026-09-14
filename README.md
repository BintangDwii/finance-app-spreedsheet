# Corporate Finance — Full Rust (Axum + Leptos + Postgres)

Sistem keuangan korporat: Axum REST API + Leptos WASM SPA (same-origin) + Postgres.
Standar kode: `AGENTS.MD`.

## Struktur

```text
backend-rust/   Axum API (handlers / service / db / repository / middleware)
shared/         Kontrak domain + format (dipakai API & frontend, feature `db` utk sqlx)
frontend/       Leptos 0.8 CSR SPA (login, dashboard, input, approval, accounts, budgets, reconcile)
backend-rust/migrations/  001 schema · 002 users · 003 accounts
scripts/        migrate_sqlite_to_postgres.py (one-shot legacy) + SQLITE_BASELINE.txt
data/corporate.db  SQLite legacy — HAPUS setelah migrasi Postgres terverifikasi
```

## Dev

```sh
# DB (butuh docker): docker compose up -d postgres
# API (auto-migrate + seed saat boot):
DATABASE_URL=postgres://finance:finance123@127.0.0.1:5435/corporate \
JWT_SECRET='<min-32-char>' cargo run -p corporate-finance-api
# Frontend:
cd frontend && trunk serve   # dev, proxy /api ke :8000
trunk build --release        # dist/ diserve Axum via FRONTEND_DIR
```

Login default: `admin/admin123`, `maker1/maker123`, `checker1/checker123`,
`approver1/approver123`, `viewer1/viewer123`.

## Gates

```sh
cargo fmt --all --check && cargo clippy --offline --all-targets && cargo test --offline --workspace
```
