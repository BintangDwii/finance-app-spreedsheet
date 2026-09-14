use validator::Validate;

use crate::error::{AppError, Result};

pub fn validate_or_400<T: Validate>(payload: &T) -> Result<()> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))
}
