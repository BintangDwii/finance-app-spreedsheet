use corporate_finance_api::{
    error::{AppError, Result},
    handlers, middleware, AppState,
};

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::Internal("DATABASE_URL not set".into()))?;
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .map_err(AppError::Database)?;
    tracing::info!("Connected to Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Migration failed: {}", e)))?;
    tracing::info!("Migrations OK");

    let app_state = AppState { pool: pool.clone() };

    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/me", get(handlers::auth::me));

    let protected_routes = Router::new()
        .route(
            "/api/transaksi",
            get(handlers::transactions::list).post(handlers::transactions::create),
        )
        .route(
            "/api/transaksi/:id/status",
            post(handlers::transactions::update_status),
        )
        .route(
            "/api/transaksi/:id",
            axum::routing::delete(handlers::transactions::soft_delete),
        )
        .route("/api/saldo", get(handlers::saldo::get_saldo))
        .route(
            "/api/saldo/per-rekening",
            get(handlers::saldo::per_rekening),
        )
        .route(
            "/api/budgets",
            get(handlers::budgets::list).post(handlers::budgets::upsert),
        )
        .route("/api/budgets/variance", get(handlers::budgets::variance))
        .route(
            "/api/reconcile/upload",
            post(handlers::reconcile::upload_csv),
        )
        .route("/api/reconcile", get(handlers::reconcile::list))
        .route(
            "/api/reconcile/:id/match",
            post(handlers::reconcile::manual_match),
        )
        .route(
            "/api/accounts",
            get(handlers::accounts::list).post(handlers::accounts::create),
        )
        .route(
            "/api/accounts/:id/status",
            post(handlers::accounts::update_status),
        )
        .route("/api/audit", get(handlers::audit::list))
        .route(
            "/api/export/transaksi",
            get(handlers::export::export_transaksi),
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth::jwt_auth,
        ));

    // CORS whitelist via env — AGENTS.MD 7
    let cors_origin = std::env::var("CORS_ALLOWED_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    tracing::info!(origin=%cors_origin, "CORS allowed origin");
    let cors = CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000")),
        )
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
