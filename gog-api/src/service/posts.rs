use actix_session::Session;
use actix_web::{
    dev::Path,
    web::{self, Data, Json, Query},
    Either, HttpResponse,
};
use sea_orm::{
    entity, ActiveModelBehavior, ActiveValue, ColumnTrait, EntityOrSelect, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    entity::{login_data, posts},
    errors::ServiceError,
    session::TokenSession,
};

use super::{helpers, objects::PostCreationData, DbConnection, ServiceResult};
use std::sync::Mutex;

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

#[actix_web::get("newest/{amount}")]
async fn posts_newest(
    amount: actix_web::web::Path<u64>,
    db: Data<DbConnection>,
) -> super::ServiceResult {
    let posts = super::entity::posts::Entity::find()
        .order_by_desc(posts::Column::Posted)
        .limit(Some(amount.into_inner()))
        .all(&db.db_connection)
        .await?;

    // let body = serde_json::to_string(&posts)
    //     .or_else(|e| Err(super::ServiceError::ServerError{source:Box::new(e)}))?;

    Ok(HttpResponse::Found().json(posts))
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

#[actix_web::get("id/{post_id}")]
async fn posts_id(post_id: web::Path<Uuid>, db: Data<DbConnection>) -> super::ServiceResult {
    let post = posts::Entity::find_by_id(post_id.into_inner())
        .one(&db.db_connection)
        .await?;
    match post {
        Some(p) => Ok(HttpResponse::Found().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}
