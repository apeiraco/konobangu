use std::{fmt::Debug, sync::Arc};

use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter};
use serde::{Deserialize, Serialize};

use crate::{
  errors::RecorderResult,
  extract::mikan::{MikanBangumiSubscription, MikanSeasonSubscription, MikanSubscriberSubscription},
  models::subscriptions::{self, SubscriptionTrait},
};

macro_rules! register_subscription_type {
    (
        subscription_category_enum: {
            $(#[$subscription_category_enum_meta:meta])*
            pub enum $type_enum_name:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $variant:ident => $string_value:literal
                ),* $(,)?
            }
        }$(,)?
        subscription_enum: {
            $(#[$subscription_enum_meta:meta])*
            pub enum $subscription_enum_name:ident {
                $(
                    $subscription_variant:ident($subscription_type:ty)
                ),* $(,)?
            }
        }
    ) => {
        $(#[$subscription_category_enum_meta])*
        #[sea_orm(
            rs_type = "String",
            db_type = "Enum",
            enum_name = "subscription_category"
        )]
        pub enum $type_enum_name {
            $(
                $(#[$variant_meta])*
                #[serde(rename = $string_value)]
                #[sea_orm(string_value = $string_value)]
                $variant,
            )*
        }


        $(#[$subscription_enum_meta])*
        #[serde(tag = "category")]
        pub enum $subscription_enum_name {
            $(
                #[serde(rename = $string_value)]
                $subscription_variant($subscription_type),
            )*
        }

        impl $subscription_enum_name {
            pub fn category(&self) -> $type_enum_name {
                match self {
                    $(Self::$subscription_variant(_) => $type_enum_name::$variant,)*
                }
            }
        }

        #[async_trait::async_trait]
        impl $crate::models::subscriptions::SubscriptionTrait for $subscription_enum_name {
            fn get_subscriber_id(&self) -> i32 {
                match self {
                    $(Self::$subscription_variant(subscription) => subscription.get_subscriber_id(),)*
                }
            }

            fn get_subscription_id(&self) -> i32 {
                match self {
                    $(Self::$subscription_variant(subscription) => subscription.get_subscription_id(),)*
                }
            }

            async fn sync_feeds_incremental(&self, ctx: Arc<dyn $crate::app::AppContextTrait>) -> $crate::errors::RecorderResult<()> {
                match self {
                    $(Self::$subscription_variant(subscription) => subscription.sync_feeds_incremental(ctx).await,)*
                }
            }

            async fn sync_feeds_full(&self, ctx: Arc<dyn $crate::app::AppContextTrait>) -> $crate::errors::RecorderResult<()> {
                match self {
                    $(Self::$subscription_variant(subscription) => subscription.sync_feeds_full(ctx).await,)*
                }
            }

            async fn sync_sources(&self, ctx: Arc<dyn $crate::app::AppContextTrait>) -> $crate::errors::RecorderResult<()> {
                match self {
                    $(Self::$subscription_variant(subscription) => subscription.sync_sources(ctx).await,)*
                }
            }

            fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {

                match model.category {
                    $($type_enum_name::$variant => {
                        <$subscription_type as $crate::models::subscriptions::SubscriptionTrait>::try_from_model(model).map(Self::$subscription_variant)
                    })*
                }
            }
        }

        impl TryFrom<&$crate::models::subscriptions::Model> for $subscription_enum_name {
            type Error = $crate::errors::RecorderError;

            fn try_from(model: &$crate::models::subscriptions::Model) -> Result<Self, Self::Error> {
                Self::try_from_model(model)
            }
        }
    };
}

register_subscription_type! {
    subscription_category_enum: {
        #[derive(
            Clone,
            Debug,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            Copy,
            DeriveActiveEnum,
            DeriveDisplay,
            EnumIter,
        )]
        pub enum SubscriptionCategory {
            MikanSubscriber => "mikan_subscriber",
            MikanSeason => "mikan_season",
            MikanBangumi => "mikan_bangumi",
        }
    }
    subscription_enum: {
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
        pub enum Subscription {
            MikanSubscriber(MikanSubscriberSubscription),
            MikanSeason(MikanSeasonSubscription),
            MikanBangumi(MikanBangumiSubscription)
        }
    }
}
