import streamlit as st
import api_client

st.set_page_config(page_title="Corporate Finance System", page_icon="🏢", layout="wide")

# Cek backend health di sidebar
with st.sidebar:
    try:
        h = api_client.health()
        st.caption(f"API: {h.get('status','?')} v{h.get('version','')}")
    except Exception as e:
        st.warning(f"Backend belum siap: {e}")

if "token" not in st.session_state:
    st.title("🏢 Corporate Finance Management")
    st.markdown("Sistem manajemen arus kas — login via Axum JWT.")
    st.markdown("---")
    st.markdown("#### Masuk")
    st.caption("Demo: admin/admin123 | maker1/maker123 | checker1/checker123 | approver1/approver123 | viewer1/viewer123")
    with st.form("login"):
        u = st.text_input("Username")
        p = st.text_input("Password", type="password")
        submitted = st.form_submit_button("Masuk", width="stretch")
        if submitted:
            try:
                res = api_client.login(u, p)
                st.session_state["token"] = res["token"]
                st.session_state["user"] = res["user"]
                st.success(f"Welcome {res['user']['username']} ({res['user']['role']})")
                st.rerun()
            except Exception as e:
                st.error(f"Login gagal: {e}")
    st.stop()

# Sidebar user + logout
u = st.session_state.get("user", {})
with st.sidebar:
    st.title("🏢 Corporate Banking")
    st.caption(f"{u.get('username')} • {u.get('role')}" + (f" • {u.get('divisi')}" if u.get('divisi') else ""))
    if st.button("Keluar", width="stretch"):
        st.session_state.pop("token", None)
        st.session_state.pop("user", None)
        st.rerun()

# Navigation — Strangler Fig: frontend murni via API
dashboard = st.Page("pages/1_Dashboard.py", title="Dashboard", icon=":material/dashboard:")
input_page = st.Page("pages/2_Input.py", title="Input Transaksi", icon=":material/add_circle:")
approval = st.Page("pages/3_Approval.py", title="Approval", icon=":material/fact_check:")
accounts = st.Page("pages/4_Accounts.py", title="Rekening", icon=":material/account_balance:")
budgets = st.Page("pages/5_Budgets.py", title="Anggaran", icon=":material/monitoring:")
reconcile = st.Page("pages/6_Reconcile.py", title="Rekonsiliasi", icon=":material/compare_arrows:")

pg = st.navigation([dashboard, input_page, approval, accounts, budgets, reconcile])
pg.run()
