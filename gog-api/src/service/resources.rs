use super::entity;
use super::entity::prelude::*;
use super::errors;
use super::helpers;
use super::objects::{UserCreationData, UserDataResponse, UserLogin};
use super::DbConnection;
use crate::session::TokenSession;
use crate::{
    entity::{login_data, user_pfp},
    errors::{ServiceError, SessionValidationError},
};
use actix_session::Session;
use actix_web::{
    self,
    web::{self, Bytes},
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use log::{debug, error, info, log, Level};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter,
};
use std::{borrow::BorrowMut, collections::HashMap, ops::Deref, str::FromStr, sync::Mutex};
use uuid::Uuid;
use validator::Validate;

static PFP_BYTES_MAX: usize = 25_000;
pub async fn user_upload_pfp(
    payload: Bytes,
    db: web::Data<DbConnection>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> Result<HttpResponse, ServiceError> {
    if payload.len() > PFP_BYTES_MAX {
        return Ok(HttpResponse::BadRequest()
            .reason("uploaded file exceeded allowed size")
            .finish());
    }

    let login = helpers::validate_session(&token_session, &session)?;
    let id = helpers::get_user_id(&login, &db).await?;

    let db = &db.db_connection;
    let mut model = match UserPfp::find_by_id(id).one(db).await? {
        Some(m) => m.into_active_model(),
        None => {
            let m = user_pfp::ActiveModel {
                user_id: ActiveValue::Set(id),
                data: ActiveValue::Set(None),
            };
            let res = UserPfp::insert(m.clone()).exec(db).await?;
            m
        }
    };

    model.data = ActiveValue::Set(Some(payload.to_vec()));
    UserPfp::update(model).exec(db).await?;

    Ok(HttpResponse::Ok().finish())
}
