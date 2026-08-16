mod base;
mod subscription;

pub(crate) use base::register_subscriber_task_type;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter, FromJsonQueryResult};
pub use subscription::{SyncOneSubscriptionFeedsFullTask, SyncOneSubscriptionFeedsIncrementalTask, SyncOneSubscriptionSourcesTask};

macro_rules! register_subscriber_task_types {
    (
        task_type_enum: {
            $(#[$type_enum_meta:meta])*
            $type_vis:vis enum $type_enum_name:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $variant:ident => $string_value:literal
                ),* $(,)?
            }
        },
        task_enum: {
            $(#[$task_enum_meta:meta])*
            $task_vis:vis enum $task_enum_name:ident {
                $(
                    $(#[$task_variant_meta:meta])*
                    $task_variant:ident($task_type:ty)
                ),* $(,)?
            }
        }
    ) => {
        $(#[$type_enum_meta])*
        #[derive(serde::Serialize, serde::Deserialize)]
        #[sea_orm(rs_type = "String", db_type = "Text")]
        $type_vis enum $type_enum_name {
            $(
                $(#[$variant_meta])*
                #[serde(rename = $string_value)]
                #[sea_orm(string_value = $string_value)]
                $variant,
            )*
        }


        $(#[$task_enum_meta])*
        #[derive(ts_rs::TS, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "task_type")]
        #[ts(export, rename = "SubscriberTaskType", rename_all = "camelCase", tag = "taskType")]
        $task_vis enum $task_enum_name {
            $(
                $(#[$task_variant_meta])*
                #[serde(rename = $string_value)]
                $task_variant($task_type),
            )*
        }

        paste::paste! {
            $(#[$task_enum_meta])*
            #[derive(ts_rs::TS, serde::Serialize, serde::Deserialize)]
            #[serde(tag = "taskType", rename_all = "camelCase")]
            #[ts(export, rename_all = "camelCase", tag = "taskType")]
            $task_vis enum [<$task_enum_name Input>] {
                $(
                    $(#[$task_variant_meta])*
                    #[serde(rename = $string_value)]
                    $task_variant(<$task_type as $crate::task::SubscriberTaskTrait>::InputType),
                )*
            }
        }

        impl $task_enum_name {
            pub fn task_type(&self) -> $type_enum_name {
                match self {
                    $(Self::$task_variant(_) => $type_enum_name::$variant,)*
                }
            }
        }

        #[async_trait::async_trait]
        impl $crate::task::AsyncTaskTrait for $task_enum_name {
            async fn run_async(self, ctx: std::sync::Arc<dyn $crate::app::AppContextTrait>) -> $crate::errors::RecorderResult<()> {
                match self {
                    $(Self::$task_variant(t) =>
                        <$task_type as $crate::task::AsyncTaskTrait>::run_async(t, ctx).await,)*
                }
            }
        }

        impl $crate::task::SubscriberTaskTrait for $task_enum_name {
            paste::paste! {
                type InputType = [<$task_enum_name Input>];
            }

            fn get_subscriber_id(&self) -> i32 {
                match self {
                    $(Self::$task_variant(t) =>
                        <$task_type as $crate::task::SubscriberTaskTrait>::get_subscriber_id(t),)*
                }
            }

            fn get_cron_id(&self) -> Option<i32> {
                match self {
                    $(Self::$task_variant(t) =>
                        <$task_type as $crate::task::SubscriberTaskTrait>::get_cron_id(t),)*
                }
            }

            fn set_subscriber_id(&mut self, subscriber_id: i32) {
                match self {
                    $(Self::$task_variant(t) => t.set_subscriber_id(subscriber_id),)*
                }
            }

            fn set_cron_id(&mut self, cron_id: Option<i32>) {
                match self {
                    $(Self::$task_variant(t) => t.set_cron_id(cron_id),)*
                }
            }

            fn from_input(input: Self::InputType, subscriber_id: i32) -> Self {
                match input {
                    $(Self::InputType::$task_variant(t) =>
                        Self::$task_variant(<$task_type as $crate::task::SubscriberTaskTrait>::from_input(t, subscriber_id)),)*
                }

            }
        }

        $(
            impl From<$task_type> for $task_enum_name {
                fn from(task: $task_type) -> Self {
                    Self::$task_variant(task)
                }
            }
        )*
    };
}

register_subscriber_task_types!(
    task_type_enum: {
        #[derive(
            Clone,
            Debug,
            PartialEq,
            Eq,
            Copy,
            DeriveActiveEnum,
            DeriveDisplay,
            EnumIter,
        )]
        pub enum SubscriberTaskType {
            SyncOneSubscriptionFeedsIncremental => "sync_one_subscription_feeds_incremental",
            SyncOneSubscriptionFeedsFull => "sync_one_subscription_feeds_full",
            SyncOneSubscriptionSources => "sync_one_subscription_sources"
        }
    },
    task_enum: {
        #[derive(Clone, Debug, PartialEq, FromJsonQueryResult)]
        pub enum SubscriberTask {
            SyncOneSubscriptionFeedsIncremental(SyncOneSubscriptionFeedsIncrementalTask),
            SyncOneSubscriptionFeedsFull(SyncOneSubscriptionFeedsFullTask),
            SyncOneSubscriptionSources(SyncOneSubscriptionSourcesTask),
        }
    }
);
