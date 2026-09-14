//! Canonical domain types live in `shared::domain` (§10 DRY).
//! This module re-exports them so existing `crate::domain::*` paths keep working.
pub use shared::domain::*;
