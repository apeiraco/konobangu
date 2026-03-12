use std::collections::HashSet;

use async_trait::async_trait;
use sea_orm::Statement;
use sea_orm_migration::{
    prelude::{extension::postgres::IntoTypeRef, *},
    schema::timestamp_with_time_zone,
};

use crate::migrations::extension::postgres::Type;

#[derive(DeriveIden)]
pub enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Subscribers {
    Table,
    Id,
    DisplayName,
    DownloaderId,
    BangumiConf,
}

#[derive(DeriveIden)]
pub enum Subscriptions {
    Table,
    Id,
    DisplayName,
    SubscriberId,
    Category,
    SourceUrl,
    Enabled,
    CredentialId,
}

#[derive(DeriveIden)]
pub enum Bangumi {
    Table,
    Id,
    MikanBangumiId,
    DisplayName,
    SubscriberId,
    OriginName,
    Season,
    SeasonRaw,
    Fansub,
    MikanFansubId,
    Filter,
    RssLink,
    PosterLink,
    OriginPosterLink,
    /**
     * @deprecated
     */
    SavePath,
    Homepage,
    BangumiType,
}

#[derive(DeriveIden)]
pub enum SubscriptionBangumi {
    Table,
    Id,
    SubscriberId,
    SubscriptionId,
    BangumiId,
}

#[derive(DeriveIden)]
pub enum Episodes {
    Table,
    Id,
    MikanEpisodeId,
    OriginName,
    DisplayName,
    BangumiId,
    SubscriberId,
    DownloadId,
    /**
     * @deprecated
     */
    SavePath,
    Resolution,
    Season,
    SeasonRaw,
    Fansub,
    PosterLink,
    OriginPosterLink,
    EpisodeIndex,
    Homepage,
    Subtitle,
    Source,
    EpisodeType,
    EnclosureTorrentLink,
    EnclosureMagnetLink,
    EnclosurePubDate,
    EnclosureContentLength,
}

#[derive(DeriveIden)]
pub enum SubscriptionEpisode {
    Table,
    Id,
    SubscriberId,
    SubscriptionId,
    EpisodeId,
}

#[derive(DeriveIden)]
pub enum Downloads {
    Table,
    Id,
    OriginName,
    DisplayName,
    SubscriberId,
    DownloaderId,
    EpisodeId,
    Status,
    CurrSize,
    AllSize,
    Mime,
    Url,
    Homepage,
    SavePath,
}

#[derive(DeriveIden)]
pub enum Downloaders {
    Table,
    Id,
    Category,
    Endpoint,
    Password,
    Username,
    SubscriberId,
    SavePath,
}

#[derive(DeriveIden)]
pub enum Auth {
    Table,
    Id,
    Pid,
    SubscriberId,
    AuthType,
}

#[derive(DeriveIden)]
pub enum Credential3rd {
    Table,
    Id,
    SubscriberId,
    CredentialType,
    Cookies,
    Username,
    Password,
    UserAgent,
}

#[derive(DeriveIden)]
pub enum Feeds {
    Table,
    Id,
    Token,
    FeedType,
    FeedSource,
    SubscriberId,
    SubscriptionId,
}

#[derive(DeriveIden)]
pub enum Cron {
    Table,
    Id,
    SubscriberId,
    SubscriptionId,
    CronExpr,
    CronTimezone,
    NextRun,
    LastRun,
    LastError,
    Enabled,
    LockedBy,
    LockedAt,
    TimeoutMs,
    Attempts,
    MaxAttempts,
    Priority,
    Status,
    SubscriberTaskCron,
    SystemTaskCron,
}

#[derive(sea_query::Iden)]

pub enum ApalisSchema {
    #[iden = "apalis"]
    Schema,
}

#[derive(DeriveIden)]

pub enum ApalisJobs {
    #[sea_orm(iden = "jobs")]
    Table,
    SubscriberId,
    SubscriptionId,
    Job,
    JobType,
    Status,
    TaskType,
    Id,
    Attempts,
    MaxAttempts,
    RunAt,
    LastError,
    LockAt,
    LockBy,
    DoneAt,
    Priority,
    CronId,
}

macro_rules! create_postgres_enum_for_active_enum {
    ($manager: expr, $active_enum: expr, $($enum_value:expr),+) => {
        {
            use sea_orm::ActiveEnum;
            let values = [$($enum_value,)+].map(|v| ActiveEnum::to_value(&v));
            ($manager).create_postgres_enum_for_active_enum($active_enum, values)
        }
    };
}

pub fn timestamps_z(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(timestamp_with_time_zone(GeneralIds::CreatedAt).default(Expr::current_timestamp()))
        .col(timestamp_with_time_zone(GeneralIds::UpdatedAt).default(Expr::current_timestamp()))
        .take()
}

pub fn table_auto_z<T: IntoIden + 'static>(name: T) -> TableCreateStatement {
    timestamps_z(Table::create().table(name).if_not_exists().take())
}

#[async_trait]
pub trait CustomSchemaManagerExt {
    async fn create_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr>;
    async fn create_postgres_auto_update_ts_fn_for_col<C: IntoIden + 'static + Send>(
        &self,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        self.create_postgres_auto_update_ts_fn(&column_ident.to_string())
            .await?;
        Ok(())
    }

    async fn create_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr>;

    async fn create_postgres_auto_update_ts_trigger_for_col<
        T: IntoIden + 'static + Send,
        C: IntoIden + 'static + Send,
    >(
        &self,
        tab: T,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        let table_ident = tab.into_iden();
        self.create_postgres_auto_update_ts_trigger(
            &table_ident.to_string(),
            &column_ident.to_string(),
        )
        .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr>;

    async fn drop_postgres_auto_update_ts_fn_for_col<C: IntoIden + Send>(
        &self,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        self.drop_postgres_auto_update_ts_fn(&column_ident.to_string())
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr>;

    async fn drop_postgres_auto_update_ts_trigger_for_col<
        T: IntoIden + 'static + Send,
        C: IntoIden + 'static + Send,
    >(
        &self,
        tab: T,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        let table_ident = tab.into_iden();
        self.drop_postgres_auto_update_ts_trigger(
            &table_ident.to_string(),
            &column_ident.to_string(),
        )
        .await?;
        Ok(())
    }

    async fn create_postgres_enum_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        I: IntoIterator<Item = String> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr>;

    async fn add_postgres_enum_values_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        I: IntoIterator<Item = String> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr>;

    async fn drop_postgres_enum_for_active_enum<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<(), DbErr>;

    async fn if_postgres_enum_exists<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<bool, DbErr>;

    async fn get_postgres_enum_values<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<HashSet<String>, DbErr>;
}

#[async_trait]
impl CustomSchemaManagerExt for SchemaManager<'_> {
    async fn create_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr> {
        let sql = format!(
            "CREATE OR REPLACE FUNCTION update_{col_name}_column() RETURNS TRIGGER AS $$ BEGIN \
             NEW.{col_name}  = current_timestamp; RETURN NEW; END; $$ language 'plpgsql';"
        );

        self.get_connection()
            .execute_unprepared(&sql)
            .await?;

        Ok(())
    }

    async fn create_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr> {
        let sql = format!(
            "CREATE OR REPLACE TRIGGER update_{tab_name}_{col_name}_column_trigger BEFORE UPDATE \
             ON {tab_name} FOR EACH ROW EXECUTE PROCEDURE update_{col_name}_column();"
        );
        self.get_connection()
            .execute_unprepared(&sql)
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr> {
        let sql = format!("DROP FUNCTION IF EXISTS update_{col_name}_column();");
        self.get_connection()
            .execute_unprepared(&sql)
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr> {
        let sql = format!(
            "DROP TRIGGER IF EXISTS update_{tab_name}_{col_name}_column_trigger ON {tab_name};"
        );
        self.get_connection()
            .execute_unprepared(&sql)
            .await?;
        Ok(())
    }

    async fn create_postgres_enum_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        I: IntoIterator<Item = String> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr> {
        let existed = self.if_postgres_enum_exists(enum_name.clone()).await?;
        if !existed {
            let idents = values.into_iter().map(Alias::new).collect::<Vec<_>>();
            self.create_type(Type::create().as_enum(enum_name).values(idents).to_owned())
                .await?;
        } else {
            self.add_postgres_enum_values_for_active_enum(enum_name, values)
                .await?;
        }
        Ok(())
    }

    async fn add_postgres_enum_values_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        I: IntoIterator<Item = String> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr> {
        let exists_values = self.get_postgres_enum_values(enum_name.clone()).await?;
        let to_add_values = values
            .into_iter()
            .filter(|v| !exists_values.contains(v as &str))
            .collect::<Vec<_>>();

        if to_add_values.is_empty() {
            return Ok(());
        }

        let mut type_alter = Type::alter().name(enum_name);

        for v in to_add_values {
            type_alter = type_alter.add_value(Alias::new(v));
        }

        self.alter_type(type_alter.to_owned()).await?;
        Ok(())
    }

    async fn drop_postgres_enum_for_active_enum<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<(), DbErr> {
        if self.if_postgres_enum_exists(enum_name.clone()).await? {
            self.drop_type(Type::drop().name(enum_name).to_owned())
                .await?;
        }
        Ok(())
    }

    async fn if_postgres_enum_exists<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<bool, DbErr> {
        let enum_name: String = enum_name.into_iden().to_string();
        let sql = format!("SELECT 1 FROM pg_type WHERE typname = '{enum_name}'");
        let result = self
            .get_connection()
            .query_one_raw(Statement::from_string(self.get_database_backend(), sql))
            .await?;
        Ok(result.is_some())
    }

    async fn get_postgres_enum_values<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<HashSet<String>, DbErr> {
        let enum_name: String = enum_name.into_iden().to_string();
        let sql = format!(
            "SELECT pg_enum.enumlabel AS enumlabel FROM pg_type JOIN pg_enum ON pg_enum.enumtypid \
             = pg_type.oid WHERE pg_type.typname = '{enum_name}';"
        );

        let results = self
            .get_connection()
            .query_all_raw(Statement::from_string(self.get_database_backend(), sql))
            .await?;

        let mut items = HashSet::new();
        for r in results {
            items.insert(r.try_get::<String>("", "enumlabel")?);
        }

        Ok(items)
    }
}
