use crate::error::{AppError, Result};

pub async fn run_blocking<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> Result<R> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Internal(format!("blocking error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn blocking_ok() {
        let v = run_blocking(|| Ok::<i32, AppError>(42)).await.unwrap();
        assert_eq!(v, 42);
    }
}
