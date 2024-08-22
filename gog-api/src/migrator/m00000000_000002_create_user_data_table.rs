use sea_orm_migration::prelude::*;

use super::m00000000_000001_create_login_table;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00000000_000002_create_user_data_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use m00000000_000001_create_login_table::LoginData;
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(UserData::Table)
                    .col(
                        ColumnDef::new(UserData::UserId)
                            .text()
                            .not_null()
                            .primary_key(), //.unique_key(),
                    )
                    .col(ColumnDef::new(UserData::Description).text().default(""))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_id")
                            .from(UserData::Table, UserData::UserId)
                            .to(LoginData::Table, LoginData::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserData::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum UserData {
    Table,
    UserId,
    Description,
}
