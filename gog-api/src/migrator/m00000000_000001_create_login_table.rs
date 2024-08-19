use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00000000_000001_create_login_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(LoginData::Table)
                    .col(
                        ColumnDef::new(LoginData::Login)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LoginData::UserId)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(LoginData::Salt).text().not_null())
                    .col(ColumnDef::new(LoginData::Hash).text().not_null())
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LoginData::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum LoginData {
    Table,
    Login,
    Salt,
    Hash,
    UserId,
}
