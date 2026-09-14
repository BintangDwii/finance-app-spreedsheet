-- 001_init.sql — corporate finance Postgres schema (dari database/migrations.py)
CREATE TYPE role_t AS ENUM ('Maker','Checker','Approver','Admin','Viewer');
CREATE TYPE status_t AS ENUM ('Pending (Butuh Approval)','Approved (Menunggu Bayar)','Paid (Lunas)','Rejected (Ditolak)','Reconciled');
CREATE TYPE jenis_t AS ENUM ('Pemasukan','Pengeluaran','Transfer Internal');
CREATE TYPE currency_t AS ENUM ('IDR','USD');
CREATE TYPE account_status_t AS ENUM ('Aktif','Beku','Tutup');
CREATE TYPE recon_status_t AS ENUM ('Unreconciled','Reconciled','Manual');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role role_t NOT NULL,
    divisi TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    bank_name TEXT UNIQUE NOT NULL,
    no_rekening TEXT UNIQUE NOT NULL,
    currency currency_t NOT NULL DEFAULT 'IDR',
    opening_balance NUMERIC(19,2) NOT NULL DEFAULT 0,
    limit_overdraft NUMERIC(19,2) NOT NULL DEFAULT 0,
    gl_code TEXT,
    status account_status_t NOT NULL DEFAULT 'Aktif',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE transaksi (
    id BIGSERIAL PRIMARY KEY,
    timestamp TEXT NOT NULL,
    tanggal DATE NOT NULL,
    jenis jenis_t NOT NULL,
    divisi TEXT NOT NULL,
    kategori TEXT NOT NULL,
    entitas_terkait TEXT NOT NULL,
    akun_pembayaran TEXT NOT NULL REFERENCES accounts(bank_name) ON UPDATE CASCADE,
    subtotal NUMERIC(19,2) NOT NULL,
    ppn_11 NUMERIC(19,2) NOT NULL DEFAULT 0,
    diskon NUMERIC(19,2) NOT NULL DEFAULT 0,
    total_akhir NUMERIC(19,2) NOT NULL,
    currency currency_t NOT NULL DEFAULT 'IDR',
    fx_rate NUMERIC(12,4) NOT NULL DEFAULT 1.0,
    due_date DATE,
    reference_no TEXT,
    status status_t NOT NULL,
    file_bukti TEXT DEFAULT '-',
    catatan TEXT DEFAULT '-',
    created_by TEXT,
    updated_at TIMESTAMPTZ DEFAULT now(),
    is_deleted BOOLEAN DEFAULT false,
    idr_amount NUMERIC(19,2) GENERATED ALWAYS AS (total_akhir * fx_rate) STORED,
    CONSTRAINT total_check CHECK (total_akhir = subtotal + ppn_11 - diskon)
);

CREATE TABLE budgets (
    id SERIAL PRIMARY KEY,
    bulan TEXT NOT NULL CHECK (bulan ~ '^[0-9]{4}-[0-9]{2}$'),
    divisi TEXT NOT NULL,
    kategori TEXT NOT NULL,
    planned NUMERIC(19,2) NOT NULL,
    currency currency_t NOT NULL DEFAULT 'IDR',
    UNIQUE(bulan, divisi, kategori)
);

CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    transaksi_id BIGINT NOT NULL REFERENCES transaksi(id) ON DELETE CASCADE,
    actor TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT,
    catatan TEXT,
    timestamp TIMESTAMPTZ DEFAULT now(),
    diff JSONB
);

CREATE TABLE reconciliation (
    id SERIAL PRIMARY KEY,
    statement_date DATE NOT NULL,
    description TEXT,
    amount NUMERIC(19,2) NOT NULL,
    currency currency_t NOT NULL DEFAULT 'IDR',
    matched_transaksi_id BIGINT REFERENCES transaksi(id) ON DELETE SET NULL,
    status recon_status_t NOT NULL DEFAULT 'Unreconciled',
    uploaded_by TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_transaksi_tanggal ON transaksi(tanggal);
CREATE INDEX idx_transaksi_status ON transaksi(status) WHERE is_deleted=false;
CREATE INDEX idx_transaksi_jenis ON transaksi(jenis);
CREATE INDEX idx_transaksi_is_deleted ON transaksi(is_deleted);
CREATE INDEX idx_transaksi_akun ON transaksi(akun_pembayaran);
CREATE INDEX idx_transaksi_tanggal_status ON transaksi(tanggal, status);
CREATE INDEX idx_transaksi_fts ON transaksi USING GIN (to_tsvector('simple', entitas_terkait || ' ' || COALESCE(catatan,'') || ' ' || COALESCE(reference_no,'')));
CREATE INDEX idx_budgets_bulan ON budgets(bulan);
CREATE INDEX idx_audit_transaksi ON audit_log(transaksi_id, timestamp DESC);
CREATE INDEX idx_recon_status ON reconciliation(status);
