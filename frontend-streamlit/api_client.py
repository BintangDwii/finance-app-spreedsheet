"""Wrapper httpx untuk Axum backend — semua request pakai Bearer JWT dari st.session_state['token']"""
import os
import httpx
import streamlit as st

API_BASE = os.getenv("API_BASE", "http://localhost:8000")

def _headers():
    token = st.session_state.get("token")
    if token:
        return {"Authorization": f"Bearer {token}"}
    return {}

def _client():
    return httpx.Client(base_url=API_BASE, timeout=15.0, headers=_headers())

def login(username: str, password: str):
    with httpx.Client(base_url=API_BASE, timeout=10) as c:
        r = c.post("/api/auth/login", json={"username": username, "password": password})
        r.raise_for_status()
        return r.json()

def get_me():
    with _client() as c:
        r = c.get("/api/auth/me")
        r.raise_for_status()
        return r.json()

def list_transaksi(params: dict | None = None):
    with _client() as c:
        r = c.get("/api/transaksi", params=params or {})
        r.raise_for_status()
        return r.json()["data"]

def create_transaksi(payload: dict):
    with _client() as c:
        r = c.post("/api/transaksi", json=payload)
        r.raise_for_status()
        return r.json()

def update_status(tid: int, new_status: str, catatan: str | None = None):
    with _client() as c:
        r = c.post(f"/api/transaksi/{tid}/status", json={"new_status": new_status, "catatan": catatan})
        r.raise_for_status()
        return r.json()

def soft_delete(tid: int):
    with _client() as c:
        r = c.delete(f"/api/transaksi/{tid}")
        r.raise_for_status()
        return r.json()

def get_saldo():
    with _client() as c:
        r = c.get("/api/saldo")
        r.raise_for_status()
        return r.json()

def per_rekening():
    with _client() as c:
        r = c.get("/api/saldo/per-rekening")
        r.raise_for_status()
        return r.json()["data"]

def list_budgets():
    with _client() as c:
        r = c.get("/api/budgets")
        r.raise_for_status()
        return r.json()["data"]

def variance():
    with _client() as c:
        r = c.get("/api/budgets/variance")
        r.raise_for_status()
        return r.json()["data"]

def upsert_budget(payload: dict):
    with _client() as c:
        r = c.post("/api/budgets", json=payload)
        r.raise_for_status()
        return r.json()

def list_accounts():
    with _client() as c:
        r = c.get("/api/accounts")
        r.raise_for_status()
        return r.json()["data"]

def create_account(payload: dict):
    with _client() as c:
        r = c.post("/api/accounts", json=payload)
        r.raise_for_status()
        return r.json()

def update_account_status(account_id: int, status: str):
    with _client() as c:
        r = c.post(f"/api/accounts/{account_id}/status", json={"status": status})
        r.raise_for_status()
        return r.json()

def list_reconcile():
    with _client() as c:
        r = c.get("/api/reconcile")
        r.raise_for_status()
        return r.json()["data"]

def upload_reconcile(csv_text: str):
    # Axum expects raw CSV body
    with _client() as c:
        r = c.post("/api/reconcile/upload", content=csv_text.encode(), headers={**_headers(), "Content-Type": "text/csv"})
        r.raise_for_status()
        return r.json()

def manual_match(recon_id: int, transaksi_id: int):
    with _client() as c:
        r = c.post(f"/api/reconcile/{recon_id}/match", json={"transaksi_id": transaksi_id})
        r.raise_for_status()
        return r.json()

def list_audit(limit: int = 100):
    with _client() as c:
        r = c.get("/api/audit", params={"limit": limit})
        r.raise_for_status()
        return r.json()["data"]

def health():
    with httpx.Client(base_url=API_BASE) as c:
        r = c.get("/health")
        r.raise_for_status()
        return r.json()
