//! PostgreSQL Row Level Security (RLS) support infrastructure.
//!
//! This module provides utilities for binding subscriber identity to database
//! sessions/transactions, enabling PostgreSQL RLS policies to enforce data
//! isolation at the database level.
//!
//! ## Design
//!
//! PostgreSQL RLS policies need to know the current subscriber. We use
//! `SET LOCAL app.subscriber_id = '<value>'` within transactions so that
//! RLS policies can reference `current_setting('app.subscriber_id')`.
//!
//! `SET LOCAL` is scoped to the current transaction and does not leak state
//! across connections in a pool, which is critical for connection pool safety.
//!
//! ## Usage
//!
//! This is a **preparatory** module. RLS policies and table-level `ENABLE ROW
//! LEVEL SECURITY` will be added in a future migration phase. This module
//! provides the application-side binding infrastructure.
//!
//! ## Future migration path
//!
//! 1. Add `subscriber_id` columns to tables that lack them (Phase 1)
//! 2. Enable RLS and create policies on private tables (Phase 2)
//! 3. Call `bind_subscriber_to_transaction` before executing subscriber-scoped
//!    queries (Phase 3)

use sea_orm::{ConnectionTrait, DbErr, ExecResult};

/// Binds the subscriber identity to the current transaction using
/// `SET LOCAL app.subscriber_id`.
///
/// This must be called within an active transaction. The setting will
/// automatically be reset when the transaction ends (commit or rollback),
/// ensuring no state leakage in connection pools.
///
/// # Arguments
///
/// * `db` - A database connection that supports executing raw SQL
/// * `subscriber_id` - The subscriber ID to bind
///
/// # Errors
///
/// Returns `DbErr` if the SQL execution fails.
///
/// # Example
///
/// ```rust,ignore
/// let txn = db.begin().await?;
/// bind_subscriber_to_transaction(&txn, 42).await?;
/// // All subsequent queries in this transaction will see app.subscriber_id = '42'
/// // RLS policies can use: current_setting('app.subscriber_id')::integer
/// txn.commit().await?;
/// ```
pub async fn bind_subscriber_to_transaction(
    db: &impl ConnectionTrait,
    subscriber_id: i32,
) -> Result<ExecResult, DbErr> {
    db.execute_unprepared(&format!(
        "SET LOCAL app.subscriber_id = '{subscriber_id}'"
    ))
    .await
}

/// Retrieves the current subscriber_id from the database session.
///
/// Useful for debugging and testing RLS configuration.
///
/// Returns `None` if the setting is not set (empty string).
#[allow(dead_code)]
pub async fn get_current_subscriber_id(
    db: &impl ConnectionTrait,
) -> Result<Option<i32>, DbErr> {
    use sea_orm::FromQueryResult;

    #[derive(Debug, FromQueryResult)]
    struct SettingResult {
        value: String,
    }

    let result = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT current_setting('app.subscriber_id', true) AS value",
        ))
        .await?;

    if let Some(row) = result {
        let setting = SettingResult::from_query_result(&row, "")?;
        if setting.value.is_empty() {
            Ok(None)
        } else {
            setting
                .value
                .parse::<i32>()
                .map(Some)
                .map_err(|e| DbErr::Custom(format!("Invalid subscriber_id setting: {e}")))
        }
    } else {
        Ok(None)
    }
}
