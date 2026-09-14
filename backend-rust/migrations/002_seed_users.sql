-- Seed users with Argon2id hashes (passwords: admin123, maker123, checker123, approver123, viewer123)
-- Generated via argon2-cffi PasswordHasher (m=65536,t=3,p=4)
INSERT INTO users (username, password_hash, role, divisi) VALUES
 ('admin', '$argon2id$v=19$m=65536,t=3,p=4$Oq8A2esGTErBJh4mRa/BwQ$3EYWwXcKBj/dh2s+k/mK0AHDhutM25QgcrLuqN7dRQc', 'Admin', NULL),
 ('maker1', '$argon2id$v=19$m=65536,t=3,p=4$XJxWa9fttXlU/HvMM1CbEA$scw5G7pzWrb1LAkHDTm1JEndJZMoBd2wufAcFLRuhrg', 'Maker', 'IT & Engineering'),
 ('checker1', '$argon2id$v=19$m=65536,t=3,p=4$ILR1xK1Z7ZC8MLeH1eBorg$i9OlRsegioTWmI2dWBEoWs9GHIV9rUeZ+t7wSvJMwdE', 'Checker', 'Management'),
 ('approver1', '$argon2id$v=19$m=65536,t=3,p=4$0whhWxnTDeIAebtD+mn1bg$WSfuyax7pweY7NgNKJbjtpUQwMTvv0bC31uOrbj9KfA', 'Approver', 'Management'),
 ('viewer1', '$argon2id$v=19$m=65536,t=3,p=4$3r8N5RBmjfYQB4laXfshNA$VQ5wmjTuA+X/GvIPR0ZBCr0o23As0ZvuXIr4oybvZiQ', 'Viewer', NULL)
ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash, role = EXCLUDED.role, divisi = EXCLUDED.divisi;
