use crate::{entity::login_data, errors::SessionValidationError};

use super::entity::prelude::*;
use super::errors;
pub use super::objects::DbConnection;
use super::SESSION_ID;
use crate::session::TokenSession;
use actix_session::Session;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{str::FromStr, sync::Mutex};
use uuid::Uuid;

pub fn validate_session(
    token_session: &actix_web::web::Data<Mutex<dyn TokenSession>>,
    session: &Session,
) -> Result<String, errors::SessionValidationError> {
    let Ok(Some(uuid_string)) = session.get::<String>(SESSION_ID) else {
        return Err(errors::SessionValidationError::NoCookie);
    };
    let uuid = uuid::Uuid::from_str(&uuid_string).or_else(|e| {
        Err(SessionValidationError::Other {
            source: Box::new(e),
        })
    })?;

    let mut lock = token_session.lock();
    let sess = lock.as_mut().unwrap();
    let Some(usr_login) = sess.get_user(&uuid) else {
        return Err(SessionValidationError::NoCookie);
    };
    Ok(usr_login)
}

pub async fn get_user_id(login: &str, db: &DbConnection) -> Result<Uuid, errors::UserIdError> {
    let usr = LoginData::find()
        .filter(login_data::Column::Login.eq(login))
        .one(&db.db_connection)
        .await?;
    match usr {
        Some(u) => Ok(u.user_id),
        None => Err(errors::UserIdError::NoUser),
    }
}
