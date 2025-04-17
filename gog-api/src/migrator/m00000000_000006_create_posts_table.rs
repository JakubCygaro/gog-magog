use super::m00000000_000001_create_login_table::LoginData;
use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00000000_000006_create_posts_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Posts::Table)
                    .col(ColumnDef::new(Posts::PostId).uuid().primary_key())
                    .col(ColumnDef::new(Posts::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_id")
                            .from(Posts::Table, Posts::UserId)
                            .to(LoginData::Table, LoginData::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Posts::Posted).timestamp().not_null())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Posts {
    Table,
    PostId,
    UserId,
    Posted,
    Content,
}
