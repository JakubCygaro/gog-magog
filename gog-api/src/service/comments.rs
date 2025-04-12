use super::helpers::*;
use super::DbConnection;
use super::TokenSession;
use actix_session::Session;
use actix_web::{web, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use gog_commons::data_structures::CommentCreationData;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
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
    cid: Option<Uuid>,
    pid: Option<Uuid>,
    limit: Option<u64>,
}
#[actix_web::get("")]
pub async fn comments_get(
    query: web::Query<CommentsGetQuery>,
    db: web::Data<DbConnection>,
) -> ServiceResult {
    let query = query.into_inner();
    let pid = query.pid;
    let cid = query.cid;
    let limit = query.limit;
    match (pid, cid) {
        (None, None) | (Some(_), Some(_)) => Ok(HttpResponse::BadRequest().finish()),
        (None, Some(cid)) => {
            let com = comments::Entity::find_by_id(cid)
                .one(&db.db_connection)
                .await?;
            match com {
                None => Ok(HttpResponse::NotFound().finish()),
                Some(c) => Ok(HttpResponse::Found().json(c)),
            }
        }
        (Some(pid), None) => {
            let com = comments::Entity::find()
                .filter(comments::Column::PostId.eq(pid))
                .limit(limit)
                .all(&db.db_connection)
                .await?;
            Ok(HttpResponse::Found().json(&com))
        }
    }
}
