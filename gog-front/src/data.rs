use leptos_router::Params;

use crate::loader::HasKey;
pub use gog_commons::data_structures::*;
//#[derive(Clone, Serialize)]
//pub struct LoginData {
//    pub login: String,
//    pub password: String
//}
//
//#[derive(Clone, serde::Serialize, Debug)]
//pub struct CommentCreationData {
//    pub content: String,
//    pub post_id: String,
//}

//#[derive(serde::Deserialize, serde::Serialize, Clone)]
//pub struct UserCreationData {
//    pub login: String,
//    pub password: String,
//}

//#[derive(Clone, serde::Deserialize, Serialize, Debug, Default)]
//pub struct UserData {
//    pub login: String,
//    pub id: String,
//    pub description: String,
//    pub gender: String,
//    pub created: Option<chrono::DateTime<chrono::Utc>>
//}

//#[derive(Clone, serde::Deserialize, Serialize, Debug, Default)]
//pub struct  PostData {
//    pub login: String,
//    pub post_id: String,
//    pub user_id: String,
//    pub posted: chrono::naive::NaiveDateTime,
//    pub content: String,
//}

//#[derive(Clone, serde::Deserialize, Serialize, Debug)]
//pub struct PostCreationData {
//    pub content: String
//}

#[derive(leptos::Params, PartialEq, Clone)]
pub struct UserProfileQuery {
    pub name: Option<String>,
    pub id: Option<uuid::Uuid>
}
#[derive(leptos::Params, PartialEq, Clone)]
pub struct PostQuery {
    pub id: Option<uuid::Uuid>
}
//#[derive(Clone, serde::Deserialize, Serialize, Debug)]
//pub struct CommentData {
//    pub comment_id: String,
//    pub post_id: String,
//    pub user_id: String,
//    pub posted: chrono::naive::NaiveDateTime,
//    pub content: String,
//}
impl HasKey for CommentData {
    fn key(&self) -> uuid::Uuid {
        self.comment_id
    }
}
