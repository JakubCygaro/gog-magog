use actix_session::Session;
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use sea_orm::{ActiveModelBehavior, ActiveValue, EntityTrait};
use uuid::Uuid;
use validator::Validate;

use crate::{entity::posts, errors::ServiceError, session::TokenSession};

use super::{helpers, objects::PostCreationData, DbConnection};
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
