//! SPA static serving — Axum serves the Trunk-built Leptos `dist/` same-origin
//! so the HttpOnly session cookie just works (no CORS needed).
//! No extra dependency: `tokio::fs` only (§10: avoid premature abstraction).

use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

fn dist_dir() -> String {
    if let Ok(d) = std::env::var("FRONTEND_DIR") {
        return d;
    }
    for cand in ["../frontend/dist", "./frontend/dist", "./dist"] {
        if std::path::Path::new(&format!("{cand}/index.html")).exists() {
            return cand.to_string();
        }
    }
    "../frontend/dist".to_string()
}

fn content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}

async fn serve_file(full: String) -> Option<Response> {
    let bytes = tokio::fs::read(&full).await.ok()?;
    let ct = content_type(&full);
    Some(
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, ct)],
            Body::from(bytes),
        )
            .into_response(),
    )
}

/// Axum fallback: API/health unknowns → JSON 404; everything else → dist file
/// or `index.html` (Leptos router owns client routes).
#[tracing::instrument]
pub async fn serve(uri: axum::http::Uri) -> Response {
    let path = uri.path();
    if path == "/health" || path.starts_with("/api/") {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": "Not found"})),
        )
            .into_response();
    }
    // Reject traversal (§7) — serve only inside dist_dir.
    if path.contains("..") {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let dir = dist_dir();
    let trimmed = path.trim_start_matches('/');
    if !trimmed.is_empty() {
        let candidate = format!("{dir}/{trimmed}");
        if std::path::Path::new(&candidate).is_file() {
            if let Some(res) = serve_file(candidate).await {
                return res;
            }
        }
    }
    match serve_file(format!("{dir}/index.html")).await {
        Some(res) => res,
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(
                serde_json::json!({"error": "Frontend not built — run `trunk build` in frontend/"}),
            ),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_known() {
        assert_eq!(content_type("a.wasm"), "application/wasm");
        assert_eq!(content_type("a.js"), "text/javascript; charset=utf-8");
        assert_eq!(content_type("a.bin"), "application/octet-stream");
    }
}
