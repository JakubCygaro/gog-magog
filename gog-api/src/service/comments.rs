use super::helpers::*;
use super::objects::CommentCreationData;
use super::DbConnection;
use super::TokenSession;
use actix_session::Session;
use actix_web::{web, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use sea_orm::EntityTrait;
use serde::Deserialize;
use std::sync::Mutex;
use uuid::Uuid;
use validator::Validate;

use crate::entity::comments;
use crate::service::{helpers, ServiceResult};
pub fn configure_service(cfg: &mut web::ServiceConfig) {
    let scope = actix_web::web::scope("/comments")
        .service(comments_post)
        .service(comments_get);
    cfg.service(scope);
}
#[actix_web::post("post")]
pub async fn comments_post(
    comment: web::Json<CommentCreationData>,
    db: web::Data<DbConnection>,
    session: Session,
    token_session: web::Data<Mutex<dyn TokenSession>>,
) -> ServiceResult {
    let login = helpers::validate_session(&token_session, &session)?;
    let uid = helpers::get_user_id(&login, &db).await?;
    let comment = comment.into_inner();
    if comment.validate().is_err() {
        return Ok(HttpResponse::BadGateway().finish());
    }
    let comment = comments::ActiveModel {
        content: sea_orm::ActiveValue::Set(comment.content),
        comment_id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
        post_id: sea_orm::ActiveValue::Set(comment.post_id),
        user_id: sea_orm::ActiveValue::Set(uid),
        posted: sea_orm::ActiveValue::Set(Utc::now().naive_utc()),
    };
    comments::Entity::insert(comment)
        .exec(&db.db_connection)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
#[derive(Deserialize)]
pub struct CommentsGetQuery {
    cid: Uuid,
}
#[actix_web::get("")]
pub async fn comments_get(
    query: web::Query<CommentsGetQuery>,
    db: web::Data<DbConnection>,
) -> ServiceResult {
    let cid = query.into_inner().cid;
    let com = comments::Entity::find_by_id(cid)
        .one(&db.db_connection)
        .await?;
    match com {
        None => Ok(HttpResponse::NotFound().finish()),
        Some(c) => Ok(HttpResponse::Found().json(c)),
    }
}
