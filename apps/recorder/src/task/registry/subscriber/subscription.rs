use sea_orm::prelude::*;

use super::base::register_subscriber_task_type;
use crate::{errors::RecorderResult, models::subscriptions::SubscriptionTrait};

macro_rules! register_subscription_task_type {
    (
        $(#[$type_meta:meta])* pub struct $task_name:ident {
            $($(#[$field_meta:meta])* pub $field_name:ident: $field_type:ty),* $(,)?
        } => async |$subscription_param:ident, $ctx_param:ident| -> $task_return_type:ty $method_body:block
    ) => {
        register_subscriber_task_type! {
            $(#[$type_meta])*
            pub struct $task_name {
                $($(#[$field_meta])* pub $field_name: $field_type,)*
                pub subscription_id: i32,
            }
        }

        #[async_trait::async_trait]
        impl $crate::task::AsyncTaskTrait for $task_name {
            async fn run_async(self, ctx: std::sync::Arc<dyn $crate::app::AppContextTrait>) -> $task_return_type {
                use $crate::models::subscriptions::{
                    Entity, Column, Subscription,
                };
                let transaction = $crate::database::operation::begin_task_transaction(ctx.db().as_ref(), self.subscriber_id).await?;
                let subscription_model = Entity::find()
                    .filter(Column::Id.eq(self.subscription_id))
                    .filter(Column::SubscriberId.eq(self.subscriber_id))
                    .one(&transaction)
                    .await?
                    .ok_or_else(|| $crate::errors::RecorderError::from_entity_not_found::<Entity>())?;

                transaction.rollback().await?;
                let $subscription_param = Subscription::try_from_model(&subscription_model)?;
                let $ctx_param = ctx;
                $method_body
            }
        }
    }
}

register_subscription_task_type! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SyncOneSubscriptionFeedsIncrementalTask {
    } => async |subscription, ctx| -> RecorderResult<()> {
        subscription.sync_feeds_incremental(ctx).await?;
        Ok(())
    }
}

register_subscription_task_type! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SyncOneSubscriptionFeedsFullTask {
    } => async |subscription, ctx| -> RecorderResult<()> {
        subscription.sync_feeds_full(ctx).await?;
        Ok(())
    }
}

register_subscription_task_type! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SyncOneSubscriptionSourcesTask {
    } => async |subscription, ctx| -> RecorderResult<()> {
        subscription.sync_sources(ctx).await?;
        Ok(())
    }
}
