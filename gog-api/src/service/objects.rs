use std::{collections::HashMap, primitive};

use validator::{Validate, ValidationError, ValidationErrorsKind};

use crate::entity::user_data;

#[derive(Clone, serde::Deserialize, Validate)]
pub struct UserUpdateData {
    #[validate(length(max = 250, message = "description was too long"))]
    description: Option<String>,
    #[validate(length(min = 3, max = 15, message = "gender length was inproper"))]
    gender: Option<String>,
}

impl UserUpdateData {
    pub fn update_model(self, model: &mut user_data::ActiveModel) {
        if let Some(desc) = self.description {
            model.description = sea_orm::ActiveValue::Set(Some(desc));
        }
        if let Some(gender) = self.gender {
            model.gender = sea_orm::ActiveValue::Set(Some(gender));
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct ValidationErrorResponse {
    pub(super) reason: String,
    pub(super) errors: validator::ValidationErrors,
}

#[derive(Clone)]
pub struct DbConnection {
    pub(super) db_connection: sea_orm::DatabaseConnection,
}
impl DbConnection {
    pub fn new(db_connection: sea_orm::prelude::DatabaseConnection) -> Self {
        Self {
            db_connection: db_connection,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserLogin {
    pub(super) login: String,
}

#[derive(serde::Deserialize, Debug, Validate)]
pub struct UserCreationData {
    #[validate(length(min = 1), custom(function = "validation::validate_user_login"))]
    pub(super) login: String,
    #[validate(
        length(min = 1),
        custom(function = "validation::validate_user_password")
    )]
    pub(super) password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserDataResponse {
    pub(super) login: String,
    pub(super) id: uuid::Uuid,
    pub(super) description: String,
    pub(super) gender: Option<String>,
    pub(super) created: Option<chrono::DateTime<chrono::Utc>>,
}

mod validation {
    use validator::ValidationError;

    pub fn validate_user_login(login: &str) -> Result<(), ValidationError> {
        if !login.is_ascii() {
            Err(ValidationError::new("2137")
                .with_message("username contains non-ascii characters".into()))
        } else {
            Ok(())
        }
    }

    pub fn validate_user_password(password: &str) -> Result<(), ValidationError> {
        if !password.is_ascii() {
            Err(ValidationError::new("2138")
                .with_message("login contains non-ascii characters".into()))
        } else {
            Ok(())
        }
    }
}
