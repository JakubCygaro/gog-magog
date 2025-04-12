use crate::entity::user_data;

//#[derive(Clone, serde::Deserialize, Validate)]
//pub struct UserUpdateData {
//    #[validate(length(max = 250, message = "description was too long"))]
//    description: Option<String>,
//    #[validate(length(min = 3, max = 15, message = "gender length was inproper"))]
//    gender: Option<String>,
//}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct UserProfileQuery {
    pub username: Option<String>,
    pub user_id: Option<uuid::Uuid>,
}

pub trait UserUpdateDataExt {
    fn update_model(self, model: &mut user_data::ActiveModel);
}

impl UserUpdateDataExt for gog_commons::data_structures::UserUpdateData {
    fn update_model(self, model: &mut user_data::ActiveModel) {
        if let Some(desc) = self.description {
            model.description = sea_orm::ActiveValue::Set(Some(desc));
        }
        if let Some(gender) = self.gender {
            model.gender = sea_orm::ActiveValue::Set(Some(gender));
        }
    }
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

//#[derive(serde::Deserialize, Debug, Validate)]
//pub struct UserCreationData {
//    #[validate(length(min = 1), custom(function = "validation::validate_user_login"))]
//    pub(super) login: String,
//    #[validate(
//        length(min = 1),
//        custom(function = "validation::validate_user_password")
//    )]
//    pub(super) password: String,
//}

//mod validation {
//    use validator::ValidationError;
//
//    pub fn validate_user_login(login: &str) -> Result<(), ValidationError> {
//        if !login.is_ascii() || login.contains(char::is_whitespace) {
//            Err(ValidationError::new("2137")
//                .with_message("username contains whitespace or non-ascii characters".into()))
//        } else {
//            Ok(())
//        }
//    }
//
//    pub fn validate_user_password(password: &str) -> Result<(), ValidationError> {
//        if !password.is_ascii() {
//            Err(ValidationError::new("2138")
//                .with_message("login contains non-ascii characters".into()))
//        } else {
//            Ok(())
//        }
//    }
//}
