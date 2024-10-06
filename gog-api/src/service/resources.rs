use super::entity::prelude::*;
use super::helpers;
use super::DbConnection;
use crate::session::TokenSession;
use crate::{entity::user_pfp, errors::ServiceError};
use actix_session::Session;
use actix_web::{
    self,
    web::{self, Bytes},
    HttpResponse,
};
use infer;
use log::debug;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel};
use std::sync::Mutex;

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

    let Some(file_type) = infer::get(&payload) else {
        return Ok(HttpResponse::BadRequest()
            .reason("unknown file type")
            .finish());
    };

    let mime = file_type.mime_type();
    if mime != "image/jpg" && mime != "image/jpeg" {
        return Ok(HttpResponse::BadRequest()
            .reason("uploaded file was not a valid jpg/jpeg file")
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
