//! Corporate Finance API — library root (§2 Clean Architecture, §8 testability).
//! Binary (`main.rs`) re-uses these modules so `tests/` can import the crate.

pub mod db;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod service;
pub mod utils;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}
