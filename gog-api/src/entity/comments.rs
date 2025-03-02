use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, serde::Serialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub posted: DateTime,
    pub content: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::login_data::Entity",
        from = "Column::UserId",
        to = "super::login_data::Column::UserId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    LoginData,
    #[sea_orm(
        belongs_to = "super::posts::Entity",
        from = "Column::PostId",
        to = "super::posts::Column::PostId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Posts,
}

impl Related<super::login_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LoginData.def()
    }
}
impl Related<super::posts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
