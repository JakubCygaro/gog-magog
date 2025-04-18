#[cfg(feature = "backend")]
pub mod validation;

pub mod data_structures {
    use serde;
    #[cfg(feature = "backend")]
    use validator::Validate;
    use uuid::Uuid;
    #[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
    #[cfg_attr(feature = "backend", derive(Validate))]
    pub struct CommentCreationData {
        #[cfg_attr(feature = "backend",
            validate(length(min = 1, max = 300, message = "comment content of disallowed size")))]
        pub content: String,
        pub post_id: Uuid,
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct PostsFilter {
        pub username: Option<String>,
        pub user_id: Option<Uuid>,
        pub limit: Option<u64>,
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    #[cfg_attr(feature = "backend", derive(Validate))]
    pub struct PostCreationData {
        #[cfg_attr(feature = "backend",
            validate(length(min = 1, max = 300, message = "post content of disallowed size")))]
        pub content: String,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct UserDataResponse {
        pub login: String,
        pub id: uuid::Uuid,
        pub description: String,
        pub gender: Option<String>,
        pub created: Option<chrono::DateTime<chrono::Utc>>,
    }
    #[derive(Clone, serde::Deserialize, serde::Serialize, Debug, Default)]
    pub struct UserData {
        pub login: String,
        pub id: String,
        pub description: String,
        pub gender: String,
        pub created: Option<chrono::DateTime<chrono::Utc>>
    }
    #[derive(Clone, serde::Deserialize, serde::Serialize, Debug, Default)]
    pub struct  PostData {
        pub login: String,
        pub post_id: String,
        pub user_id: String,
        pub posted: chrono::DateTime<chrono::Utc>,
        pub content: String,
    }
    #[derive(Clone, serde::Serialize, Debug)]
    pub struct ValidationErrorResponse {
        pub reason: String,
        pub errors: validator::ValidationErrors,
    }
    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct UserLogin {
        pub login: String,
    }
    #[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
    #[cfg_attr(feature = "backend", derive(Validate))]
    pub struct UserCreationData {
        #[cfg_attr(feature = "backend",
            validate(length(min = 1), custom(function = "crate::validation::validate_user_login")))]
        pub login: String,
        #[cfg_attr(feature = "backend",
            validate(
            length(min = 1),
            custom(function = "crate::validation::validate_user_password")
        ))]
        pub password: String,
    }
    #[derive(Clone, serde::Deserialize)]
    #[cfg_attr(feature = "backend", derive(Validate))]
    pub struct UserUpdateData {
        #[cfg_attr(feature = "backend",
            validate(length(max = 250, message = "description was too long")))]
        pub description: Option<String>,
        #[cfg_attr(feature = "backend",
            validate(length(min = 3, max = 15, message = "gender length was inproper")))]
        pub gender: Option<String>,
    }
    #[derive(Clone, serde::Serialize)]
    pub struct LoginData {
        pub login: String,
        pub password: String
    }
    #[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
    pub struct CommentData {
        pub comment_id: Uuid,
        pub post_id: Uuid,
        pub user_id: Uuid,
        pub user_name: String,
        pub posted: chrono::DateTime<chrono::Utc>,
        pub content: String,
    }
}
