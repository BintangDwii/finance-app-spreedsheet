import os
import streamlit as st
from sqlalchemy import create_engine, text

DB_PATH = "data/corporate.db"
DB_URL = f"sqlite:///{DB_PATH}"

def get_engine():
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
    engine = create_engine(DB_URL, connect_args={"check_same_thread": False})
    # WAL mode untuk concurrency
    with engine.connect() as conn:
        conn.execute(text("PRAGMA journal_mode=WAL;"))
        conn.execute(text("PRAGMA foreign_keys=ON;"))
        conn.commit()
    return engine

@st.cache_resource
def get_cached_engine():
    return get_engine()

def get_connection():
    # st.connection untuk yang mau pakai API Streamlit
    try:
        return st.connection("sql", url=DB_URL)
    except Exception:
        # fallback ke engine langsung jika secrets belum load
        return get_cached_engine()

def query_df(sql: str, params: dict | None = None):
    engine = get_cached_engine()
    import pandas as pd
    return pd.read_sql(text(sql), engine, params=params or {})

def execute(sql: str, params: dict | None = None):
    engine = get_cached_engine()
    with engine.begin() as conn:
        conn.execute(text(sql), params or {})
