"""Central enums & constants — single source for AGENTS.MD Rule 1 & 10"""
from enum import StrEnum

DEFAULT_FX_RATE = 16250.0

class UserRole(StrEnum):
    Maker = "Maker"
    Checker = "Checker"
    Approver = "Approver"
    Admin = "Admin"
    Viewer = "Viewer"

class TransaksiStatus(StrEnum):
    Pending = "Pending (Butuh Approval)"
    Approved = "Approved (Menunggu Bayar)"
    Paid = "Paid (Lunas)"
    Rejected = "Rejected (Ditolak)"
    Reconciled = "Reconciled"

class Divisi(StrEnum):
    IT = "IT & Engineering"
    Marketing = "Marketing & Sales"
    HR = "HR & Admin"
    Operasional = "Operasional"
    Management = "Management"

DIVISI_OPTIONS = [d.value for d in Divisi]
STATUS_OPTIONS = [s.value for s in TransaksiStatus]

KATEGORI_MASUK = ["Project/Client", "Retainer Contract", "Pendanaan/Investasi", "Bunga Bank", "Lainnya"]
KATEGORI_KELUAR = ["Software & Cloud (AWS/GCP)", "Gaji & Tunjangan", "Pajak & Legalitas", "Iklan & Ads", "Sewa Kantor", "Lainnya"]
ENTITAS_OPTIONS = ["Klien A", "Klien B", "Vendor AWS", "Vendor Google", "Karyawan Internal", "Lain-lain"]
