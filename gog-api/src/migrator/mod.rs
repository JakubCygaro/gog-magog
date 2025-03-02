mod m00000000_000001_create_login_table;
mod m00000000_000002_create_user_data_table;
mod m00000000_000003_expand_user_data_table;
mod m00000000_000004_expand_user_data_table;
mod m00000000_000005_create_pfp_table;
mod m00000000_000006_create_posts_table;
mod m00000000_000007_create_comments_table;
use sea_orm_migration::prelude::*;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00000000_000001_create_login_table::Migration {}),
            Box::new(m00000000_000002_create_user_data_table::Migration {}),
            Box::new(m00000000_000003_expand_user_data_table::Migration {}),
            Box::new(m00000000_000004_expand_user_data_table::Migration {}),
            Box::new(m00000000_000005_create_pfp_table::Migration {}),
            Box::new(m00000000_000006_create_posts_table::Migration {}),
            Box::new(m00000000_000007_create_comments_table::Migration {}),
        ]
    }
}
