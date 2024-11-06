use chrono::{DateTime, NaiveTime};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct LoginData {
    pub login: String,
    pub password: String
}



#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct UserCreationData {
    pub login: String,
    pub password: String,
}

#[derive(Clone, serde::Deserialize, Serialize, Debug, Default)]
pub struct UserData {
    pub login: String,
    pub id: String,
    pub description: String,
    pub gender: String,
    pub created: Option<chrono::DateTime<chrono::Utc>>
}

#[derive(Clone, serde::Deserialize, Serialize, Debug, Default)]
pub struct  PostData {
    pub login: String,
    pub post_id: String,
    pub user_id: String,
    pub posted: chrono::naive::NaiveDateTime,
    pub content: String,
}

#[derive(Clone, serde::Deserialize, Serialize, Debug)]
pub struct PostCreationData {
    pub content: String
}
