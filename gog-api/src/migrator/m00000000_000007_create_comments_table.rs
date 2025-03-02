use super::m00000000_000001_create_login_table::LoginData;
use super::m00000000_000006_create_posts_table::Posts;
use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00000000_000007_create_comments_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .col(ColumnDef::new(Comments::CommentId).uuid().primary_key())
                    .col(ColumnDef::new(Comments::PostId).uuid().not_null())
                    .col(ColumnDef::new(Comments::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_id")
                            .from(Comments::Table, Comments::UserId)
                            .to(LoginData::Table, LoginData::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-post_id")
                            .from(Comments::Table, Comments::PostId)
                            .to(Posts::Table, Posts::PostId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Comments::Posted).date_time().not_null())
                    .col(ColumnDef::new(Comments::Content).text().not_null())
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comments::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Comments {
    Table,
    CommentId,
    PostId,
    UserId,
    Posted,
    Content,
}
