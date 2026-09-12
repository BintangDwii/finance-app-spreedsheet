import streamlit as st
from database.migrations import run_migrations
from database.seeders import run_seeders
from utils.auth import login_form, logout_button, current_user
from utils.i18n import T

st.set_page_config(page_title="Corporate Finance System", page_icon="🏢", layout="wide")

# init DB sekali
try:
    run_migrations()
    run_seeders()
except Exception as e:
    st.error(f"DB init error: {e}")

# --- Auth gate ---
if "user" not in st.session_state:
    st.title(f"🏢 {T['app_title']}")
    st.markdown(T["app_subtitle"])
    st.markdown("---")
    login_form()
    st.stop()

# Header + logout
u = current_user()
st.sidebar.title("🏢 Corporate Banking")
st.sidebar.caption(f"{u['username']} • {u['role']}" + (f" • {u['divisi']}" if u['divisi'] else ""))
if st.sidebar.button(T["logout"], width="stretch"):
    del st.session_state["user"]
    st.rerun()

# Navigation — modular
dashboard = st.Page("pages/1_Dashboard.py", title=T["nav_dashboard"], icon=":material/dashboard:")
input_page = st.Page("pages/2_Input.py", title=T["nav_input"], icon=":material/add_circle:")
approval = st.Page("pages/3_Approval.py", title=T["nav_approval"], icon=":material/fact_check:")
accounts = st.Page("pages/4_Accounts.py", title=T["nav_accounts"], icon=":material/account_balance:")
budgets = st.Page("pages/5_Budgets.py", title=T["nav_budgets"], icon=":material/monitoring:")
reconcile = st.Page("pages/6_Reconcile.py", title=T["nav_reconcile"], icon=":material/compare_arrows:")

pg = st.navigation([dashboard, input_page, approval, accounts, budgets, reconcile])
pg.run()
