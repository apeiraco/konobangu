use async_trait::async_trait;
pub use sea_orm_migration::prelude::*;

#[macro_use]
pub mod defs;
pub mod m20220101_000001_init;
pub mod m20240224_082543_add_downloads;
pub mod m20241231_000001_auth;
pub mod m20250501_021523_credential_3rd;
pub mod m20250520_021135_add_tasks;
pub mod m20250622_015618_feeds;
pub mod m20250622_020819_bangumi_and_episode_type;
pub mod m20250629_065628_add_cron;
pub mod m20260314_000001_enable_rls;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_init::Migration),
            Box::new(m20240224_082543_add_downloads::Migration),
            Box::new(m20241231_000001_auth::Migration),
            Box::new(m20250501_021523_credential_3rd::Migration),
            Box::new(m20250520_021135_add_tasks::Migration),
            Box::new(m20250622_015618_feeds::Migration),
            Box::new(m20250622_020819_bangumi_and_episode_type::Migration),
            Box::new(m20250629_065628_add_cron::Migration),
            Box::new(m20260314_000001_enable_rls::Migration),
        ]
    }
}
