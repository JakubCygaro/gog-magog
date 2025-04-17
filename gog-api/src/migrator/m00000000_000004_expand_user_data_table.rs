use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m00000000_000004_expand_user_data_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager
            .has_column(UserData::Table.to_string(), UserData::Created.to_string())
            .await?
        {
            let table = Table::alter()
                .table(UserData::Table)
                .add_column_if_not_exists(ColumnDef::new(UserData::Created).timestamp())
                .to_owned();
            manager.alter_table(table).await?;
        };
        if !manager
            .has_column(UserData::Table.to_string(), UserData::Gender.to_string())
            .await?
        {
            let table = Table::alter()
                .table(UserData::Table)
                .add_column_if_not_exists(
                    ColumnDef::new(UserData::Gender)
                        .text()
                        .default(Expr::value("not given")),
                )
                .to_owned();
            manager.alter_table(table).await?;
        }
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserData::Table)
                    .drop_column(UserData::Created)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(UserData::Table)
                    .drop_column(UserData::Gender)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum UserData {
    Table,
    Created,
    Gender,
}
