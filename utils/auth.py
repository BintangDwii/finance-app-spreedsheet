import hashlib
import streamlit as st
from sqlalchemy import text
from database.connection import get_cached_engine

def hash_pw(pw: str) -> str:
    return hashlib.sha256(pw.encode()).hexdigest()

def verify_user(username: str, password: str):
    engine = get_cached_engine()
    with engine.connect() as conn:
        row = conn.execute(text("SELECT username, password_hash, role, divisi FROM users WHERE username=:u"), {"u": username}).fetchone()
        if row and row[1] == hash_pw(password):
            return {"username": row[0], "role": row[2], "divisi": row[3]}
    return None

def login_form():
    st.markdown("#### Masuk ke Sistem")
    st.caption("Demo akun: admin/admin123 | maker1/maker123 | checker1/checker123 | approver1/approver123 | viewer1/viewer123")
    with st.form("login"):
        u = st.text_input("Username")
        p = st.text_input("Password", type="password")
        submitted = st.form_submit_button("Masuk", width="stretch")
        if submitted:
            user = verify_user(u, p)
            if user:
                st.session_state["user"] = user
                st.success(f"Selamat datang, {user['username']} ({user['role']})")
                st.rerun()
            else:
                st.error("Username atau password salah.")

def require_login():
    if "user" not in st.session_state:
        st.warning("Silakan login terlebih dahulu.")
        login_form()
        st.stop()

def current_user():
    return st.session_state.get("user")

def has_role(*roles):
    u = current_user()
    if not u:
        return False
    if u["role"] == "Admin":
        return True
    return u["role"] in roles

def can_create():
    return has_role("Maker", "Admin")

def can_approve():
    return has_role("Checker", "Approver", "Admin")

def logout_button():
    u = current_user()
    if u:
        c1, c2 = st.columns([3,1])
        with c1:
            st.caption(f"Login sebagai **{u['username']}** — {u['role']}" + (f" / {u['divisi']}" if u['divisi'] else ""))
        with c2:
            if st.button("Keluar", width="stretch"):
                del st.session_state["user"]
                st.rerun()

def guard_transition(current_status: str, new_status: str, total_idr: float, user: dict):
    """Return (allowed:bool, reason:str)"""
    role = user["role"] if user else "Viewer"
    if role == "Admin":
        return True, ""
    # Aturan guard nominal
    # Pending -> Approved : Checker atau Approver, tapi >50jt harus Approver/Admin
    # Approved -> Paid/Rejected : hanya Approver/Admin
    # Paid -> Rejected : hanya Admin
    # Rejected -> Pending (re-submit) : Maker/Admin
    if current_status == "Pending (Butuh Approval)":
        if new_status == "Approved (Menunggu Bayar)":
            if total_idr > 50_000_000 and role not in ("Approver",):
                return False, "Nominal > Rp 50jt harus disetujui Approver/Admin."
            if role not in ("Checker", "Approver"):
                return False, "Hanya Checker/Approver yang bisa approve Pending."
            return True, ""
        if new_status == "Rejected (Ditolak)":
            if role not in ("Checker", "Approver"):
                return False, "Hanya Checker/Approver yang bisa reject."
            return True, ""
    if current_status == "Approved (Menunggu Bayar)":
        if new_status in ("Paid (Lunas)", "Rejected (Ditolak)"):
            if role not in ("Approver",):
                return False, "Hanya Approver/Admin yang bisa ubah Approved → Paid/Rejected."
            if total_idr > 100_000_000 and role != "Admin" and new_status == "Paid (Lunas)":
                return False, "Nominal > Rp 100jt butuh Admin."
            return True, ""
    if current_status == "Rejected (Ditolak)":
        if new_status == "Pending (Butuh Approval)" and role in ("Maker",):
            return True, ""
    if current_status == "Paid (Lunas)":
        return False, "Transaksi Lunas tidak bisa diubah (hubungi Admin)."
    return False, f"Transisi {current_status} → {new_status} tidak diizinkan untuk role {role}."
