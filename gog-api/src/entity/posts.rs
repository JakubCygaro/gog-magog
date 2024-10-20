use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, serde::Serialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
}

impl Related<super::login_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LoginData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
