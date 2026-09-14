use axum::Json;

#[tracing::instrument]
pub async fn health_check() -> Json<serde_json::Value> {
    tracing::info!("health check");
    Json(
        serde_json::json!({"status":"ok","service":"corporate-finance-api","version": env!("CARGO_PKG_VERSION")}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_ok() {
        let v = health_check().await;
        assert_eq!(v.0["status"], "ok");
    }
}
