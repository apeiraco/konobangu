use async_trait::async_trait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// All user-private tables that have a NOT NULL `subscriber_id` column.
const STANDARD_RLS_TABLES: &[&str] = &[
    "subscriptions",
    "bangumi",
    "episodes",
    "subscription_bangumi",
    "subscription_episode",
    "downloaders",
    "downloads",
    "auth",
    "credential_3rd",
];

/// Tables where `subscriber_id` is nullable (system-level resources can have NULL).
const NULLABLE_RLS_TABLES: &[&str] = &["feeds", "cron"];

/// Helper: generate the `current_setting` expression used in all RLS policies.
fn subscriber_id_setting() -> String {
    "current_setting('app.subscriber_id', true)::integer".to_string()
}

/// Create standard RLS policies for a table with NOT NULL `subscriber_id`.
///
/// Policies:
/// - SELECT: `subscriber_id = current_setting(...)`
/// - INSERT: `WITH CHECK (subscriber_id = current_setting(...))`
/// - UPDATE: `USING (...) WITH CHECK (...)`
/// - DELETE: `USING (subscriber_id = current_setting(...))`
async fn create_standard_rls_policies(
    db: &impl sea_orm::ConnectionTrait,
    table: &str,
) -> Result<(), DbErr> {
    let setting = subscriber_id_setting();

    db.execute_unprepared(&format!(
        "ALTER TABLE {table} ENABLE ROW LEVEL SECURITY"
    ))
    .await?;

    db.execute_unprepared(&format!(
        "ALTER TABLE {table} FORCE ROW LEVEL SECURITY"
    ))
    .await?;

    // SELECT policy
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_select ON {table} FOR SELECT USING (subscriber_id = {setting})"
    ))
    .await?;

    // INSERT policy
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_insert ON {table} FOR INSERT WITH CHECK (subscriber_id = {setting})"
    ))
    .await?;

    // UPDATE policy
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_update ON {table} FOR UPDATE USING (subscriber_id = {setting}) WITH CHECK (subscriber_id = {setting})"
    ))
    .await?;

    // DELETE policy
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_delete ON {table} FOR DELETE USING (subscriber_id = {setting})"
    ))
    .await?;

    Ok(())
}

/// Create RLS policies for a table with nullable `subscriber_id`.
///
/// SELECT allows rows where `subscriber_id IS NULL` (system resources) OR matches
/// the current subscriber. INSERT/UPDATE/DELETE only allow rows owned by the
/// current subscriber (NULL subscriber_id rows cannot be mutated through RLS).
async fn create_nullable_rls_policies(
    db: &impl sea_orm::ConnectionTrait,
    table: &str,
) -> Result<(), DbErr> {
    let setting = subscriber_id_setting();

    db.execute_unprepared(&format!(
        "ALTER TABLE {table} ENABLE ROW LEVEL SECURITY"
    ))
    .await?;

    db.execute_unprepared(&format!(
        "ALTER TABLE {table} FORCE ROW LEVEL SECURITY"
    ))
    .await?;

    // SELECT policy — allow system resources (NULL) and own resources
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_select ON {table} FOR SELECT USING (subscriber_id IS NULL OR subscriber_id = {setting})"
    ))
    .await?;

    // INSERT policy — only allow inserting own resources
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_insert ON {table} FOR INSERT WITH CHECK (subscriber_id = {setting})"
    ))
    .await?;

    // UPDATE policy — only allow updating own resources
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_update ON {table} FOR UPDATE USING (subscriber_id = {setting}) WITH CHECK (subscriber_id = {setting})"
    ))
    .await?;

    // DELETE policy — only allow deleting own resources
    db.execute_unprepared(&format!(
        "CREATE POLICY {table}_delete ON {table} FOR DELETE USING (subscriber_id = {setting})"
    ))
    .await?;

    Ok(())
}

/// Drop all RLS policies and disable RLS for a table.
async fn drop_rls_for_table(
    db: &impl sea_orm::ConnectionTrait,
    table: &str,
) -> Result<(), DbErr> {
    for policy in &["select", "insert", "update", "delete"] {
        db.execute_unprepared(&format!(
            "DROP POLICY IF EXISTS {table}_{policy} ON {table}"
        ))
        .await?;
    }

    db.execute_unprepared(&format!(
        "ALTER TABLE {table} DISABLE ROW LEVEL SECURITY"
    ))
    .await?;

    Ok(())
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Enable RLS on standard (NOT NULL subscriber_id) tables
        for table in STANDARD_RLS_TABLES {
            create_standard_rls_policies(db, table).await?;
        }

        // Enable RLS on nullable subscriber_id tables
        for table in NULLABLE_RLS_TABLES {
            create_nullable_rls_policies(db, table).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop RLS from all tables (both standard and nullable)
        for table in STANDARD_RLS_TABLES.iter().chain(NULLABLE_RLS_TABLES.iter()) {
            drop_rls_for_table(db, table).await?;
        }

        Ok(())
    }
}
