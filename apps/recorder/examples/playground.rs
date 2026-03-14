use std::{sync::Arc, time::Duration};

use apalis_sql::postgres::PostgresStorage;
use recorder::{
    app::AppContextTrait,
    errors::RecorderResult,
    test_utils::{
        app::TestingAppContext,
        database::{TestingDatabaseServiceConfig, build_testing_database_service},
    },
};

#[tokio::main]
async fn main() -> RecorderResult<()> {
    let app_ctx = {
        let db_service = build_testing_database_service(TestingDatabaseServiceConfig {
            auto_migrate: false,
        })
        .await?;
        Arc::new(TestingAppContext::builder().db(db_service).build())
    };

    let db = app_ctx.db();

    PostgresStorage::setup(db.get_postgres_connection_pool()).await?;

    dbg!(db.get_postgres_connection_pool().connect_options());

    tokio::time::sleep(Duration::from_hours(1)).await;

    Ok(())
}
