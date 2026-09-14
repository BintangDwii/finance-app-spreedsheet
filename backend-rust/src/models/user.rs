use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::{UserId, UserRole};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub divisi: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6, max = 100))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserPublic {
    pub username: String,
    pub role: UserRole,
    pub divisi: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UserRole;

    #[test]
    fn login_validation_ok() {
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn login_validation_fail_short() {
        let req = LoginRequest {
            username: "ab".to_string(),
            password: "123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn user_role_newtype_distinct() {
        let uid = UserId(1);
        assert_eq!(uid.0, 1);
        let role = UserRole::Admin;
        assert_eq!(serde_json::to_string(&role).unwrap(), "\"Admin\"");
    }
}
