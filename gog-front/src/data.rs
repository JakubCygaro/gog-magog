use serde::{Deserialize, Serialize};

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
}


