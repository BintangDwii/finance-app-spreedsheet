//! Shared domain contract — single source of truth for API + Leptos frontend.
//!
//! `db` feature (enabled by backend only) adds `sqlx::Type` derives so the
//! SAME enum maps directly to Postgres custom types (§5). The WASM frontend
//! builds with default features (serde only) — JSON contract stays identical.

pub mod domain;
pub mod format;
