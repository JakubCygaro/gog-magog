use actix_session::Session;
use actix_web::{
    dev::Path,
    web::{self, Data, Json, Query},
    Either, HttpResponse,
};
use sea_orm::{
    entity, ActiveModelBehavior, ActiveValue, ColumnTrait, EntityOrSelect, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Related,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    entity::{comments, login_data, posts},
    errors::ServiceError,
    session::TokenSession,
};

use super::{helpers, DbConnection, ServiceResult};
use gog_commons::data_structures::{PostCreationData, PostsFilter};
use std::sync::Mutex;
pub fn configure_service(cfg: &mut web::ServiceConfig) {
    let posts_scope = web::scope("/posts")
        .service(posts_create)
        .service(posts_newest)
        .service(posts_user)
        .service(posts_filter)
        .service(posts_id)
        .service(posts_comments);
    cfg.service(posts_scope);
}
#[actix_web::post("create")]
async fn posts_create(
    post_data: Json<PostCreationData>,
    session: Session,
    token_session: Data<Mutex<dyn TokenSession>>,
    db: Data<DbConnection>,
) -> super::ServiceResult {
    if let Err(errors) = post_data.validate() {
        return Ok(HttpResponse::BadRequest()
            .reason("post creation data validation failed")
            .json(errors));
    };

    let login = helpers::validate_session(&token_session, &session)?;
    let id = helpers::get_user_id(&login, &db).await?;

    let model = posts::ActiveModel {
        post_id: ActiveValue::Set(Uuid::new_v4()),
        user_id: ActiveValue::Set(id),
        posted: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        content: ActiveValue::Set(post_data.content.clone()),
    };

    posts::Entity::insert(model).exec(&db.db_connection).await?;
    Ok(HttpResponse::Created().finish())
}

#[actix_web::post("filter")]
async fn posts_filter(db: Data<DbConnection>, filter: Json<PostsFilter>) -> super::ServiceResult {
    let filter = filter.into_inner();
    let mut posts = super::entity::posts::Entity::find().find_also_related(login_data::Entity);

    if let Some(un) = filter.username {
        posts = posts.filter(login_data::Column::Login.eq(un));
    }
    posts = posts.limit(filter.limit.unwrap_or(20));
    let posts = posts
        .order_by_desc(posts::Column::Posted)
        .limit(filter.limit)
        .all(&db.db_connection)
        .await?;

    // let body = serde_json::to_string(&posts)
    //     .or_else(|e| Err(super::ServiceError::ServerError{source:Box::new(e)}))?;

    Ok(HttpResponse::Found().json(
        posts
            .into_iter()
            .map(|e| PostResponse {
                login: match e.1 {
                    Some(m) => m.login,
                    None => String::new(),
                },
                content: e.0.content,
                post_id: e.0.post_id,
                user_id: e.0.post_id,
                posted: e.0.posted,
            })
            .collect::<Vec<_>>(),
    ))
}

#[actix_web::get("newest/{amount}")]
async fn posts_newest(
    amount: actix_web::web::Path<u64>,
    db: Data<DbConnection>,
) -> super::ServiceResult {
    let posts = super::entity::posts::Entity::find()
        .find_also_related(login_data::Entity)
        .order_by_desc(posts::Column::Posted)
        .limit(Some(amount.into_inner()))
        .all(&db.db_connection)
        .await?;

    // let body = serde_json::to_string(&posts)
    //     .or_else(|e| Err(super::ServiceError::ServerError{source:Box::new(e)}))?;

    Ok(HttpResponse::Found().json(
        posts
            .into_iter()
            .map(|e| PostResponse {
                login: match e.1 {
                    Some(m) => m.login,
                    None => String::new(),
                },
                content: e.0.content,
                post_id: e.0.post_id,
                user_id: e.0.post_id,
                posted: e.0.posted,
            })
            .collect::<Vec<_>>(),
    ))
}

#[derive(Serialize)]
struct PostResponse {
    login: String,
    user_id: Uuid,
    post_id: Uuid,
    posted: chrono::naive::NaiveDateTime,
    content: String,
}

#[derive(serde::Deserialize)]
struct PostLoginQuery {
    login: String,
    amount: u64,
}

#[derive(serde::Deserialize)]
struct PostIdQuery {
    user_id: Uuid,
    amount: u64,
}

#[actix_web::get("user")]
async fn posts_user(
    query: Either<Query<PostLoginQuery>, Query<PostIdQuery>>,
    db: Data<DbConnection>,
) -> super::ServiceResult {
    let (id, amount) = match query {
        Either::Right(idq) => (idq.user_id, idq.amount),
        Either::Left(loginq) => {
            let model = login_data::Entity::find_by_id(&loginq.login)
                .one(&db.db_connection)
                .await?;
            let id = model.ok_or_else(|| ServiceError::UserNotFound {})?.user_id;
            (id, loginq.amount)
        }
    };

    let posts = super::entity::posts::Entity::find()
        .filter(posts::Column::UserId.eq(id))
        .order_by_desc(posts::Column::Posted)
        .limit(Some(amount))
        .all(&db.db_connection)
        .await?;

    Ok(HttpResponse::Found().json(posts))
}

#[derive(Clone, serde::Deserialize, Serialize, Debug, Default)]
pub struct PostData {
    pub login: String,
    pub post_id: String,
    pub user_id: String,
    pub posted: chrono::naive::NaiveDateTime,
    pub content: String,
}
#[actix_web::get("id/{post_id}")]
async fn posts_id(post_id: web::Path<Uuid>, db: Data<DbConnection>) -> super::ServiceResult {
    let post = posts::Entity::find_by_id(post_id.into_inner())
        .find_also_related(login_data::Entity)
        .one(&db.db_connection)
        .await?
        .map_or(None, |(p, l)| {
            Some(PostData {
                login: l.map_or("".to_owned(), |m| m.login),
                post_id: p.post_id.to_string(),
                user_id: p.user_id.to_string(),
                posted: p.posted,
                content: p.content,
            })
        });
    match post {
        Some(p) => Ok(HttpResponse::Found().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}
#[derive(Deserialize)]
pub struct PostCommentsQuery {
    pid: Uuid,
    amount: u64,
}
#[actix_web::get("comments")]
pub async fn posts_comments(
    query: web::Query<PostCommentsQuery>,
    db: web::Data<DbConnection>,
) -> ServiceResult {
    let query = query.into_inner();
    let pid = query.pid;
    let comments = comments::Entity::find()
        .filter(comments::Column::PostId.eq(pid))
        .order_by_desc(comments::Column::Posted)
        .limit(query.amount)
        .all(&db.db_connection)
        .await?;
    //.iter()
    //.map(|comment_model| comment_model.comment_id)
    //.collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(comments))
}
