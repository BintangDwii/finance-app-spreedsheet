-- Seed 7 default accounts (ported from legacy database/seeders.py).
-- Idempotent: safe to re-run.
INSERT INTO accounts (bank_name, no_rekening, currency, opening_balance, limit_overdraft, gl_code, status) VALUES
 ('BCA Corporate', '8001234567', 'IDR', 500000000, 0, '1101', 'Aktif'),
 ('Mandiri Corporate', '1400012345678', 'IDR', 300000000, 0, '1102', 'Aktif'),
 ('Kas Kecil (Petty Cash)', 'CASH-001', 'IDR', 25000000, 0, '1103', 'Aktif'),
 ('Kartu Kredit Perusahaan', 'CC-5200-001', 'IDR', 0, 100000000, '2101', 'Aktif'),
 ('BRI Corporate', '1200123456789', 'IDR', 250000000, 0, '1104', 'Aktif'),
 ('BNI Corporate', '150012345678', 'IDR', 200000000, 0, '1105', 'Aktif'),
 ('BCA USD', '8009876543', 'USD', 25000, 0, '1106', 'Aktif')
ON CONFLICT (bank_name) DO UPDATE SET
  no_rekening = EXCLUDED.no_rekening,
  currency = EXCLUDED.currency,
  gl_code = EXCLUDED.gl_code,
  status = EXCLUDED.status;
