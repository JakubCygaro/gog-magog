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

#[derive(serde::Deserialize, Debug)]
pub struct UserCreationData {
    pub(super) login: String,
    pub(super) password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserDataResponse {
    pub(super) login: String,
    pub(super) id: uuid::Uuid,
    pub(super) description: String,
}
