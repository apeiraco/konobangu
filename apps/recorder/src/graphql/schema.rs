use std::sync::Arc;

use async_graphql::dynamic::*;
use once_cell::sync::OnceCell;
use seaography::{Builder, BuilderContext};

use crate::{
    app::AppContextTrait,
    graphql::{
        domains::{
            bangumi::{register_bangumi_to_schema_builder, register_bangumi_to_schema_context},
            credential_3rd::{
                register_credential3rd_to_schema_builder, register_credential3rd_to_schema_context,
            },
            cron::{register_cron_to_schema_builder, register_cron_to_schema_context},
            downloaders::{
                register_downloaders_to_schema_builder, register_downloaders_to_schema_context,
            },
            downloads::{
                register_downloads_to_schema_builder, register_downloads_to_schema_context,
            },
            episodes::{register_episodes_to_schema_builder, register_episodes_to_schema_context},
            feeds::{register_feeds_to_schema_builder, register_feeds_to_schema_context},
            subscriber_tasks::{
                register_subscriber_tasks_to_schema_builder,
                register_subscriber_tasks_to_schema_context,
            },
            subscribers::{
                register_subscribers_to_schema_builder, register_subscribers_to_schema_context,
            },
            subscription_bangumi::{
                register_subscription_bangumi_to_schema_builder,
                register_subscription_bangumi_to_schema_context,
            },
            subscription_episode::{
                register_subscription_episode_to_schema_builder,
                register_subscription_episode_to_schema_context,
            },
            subscriptions::{
                register_subscriptions_to_schema_builder, register_subscriptions_to_schema_context,
            },
            system_tasks::{
                register_system_tasks_to_schema_builder, register_system_tasks_to_schema_context,
            },
        },
        infra::{
            json::register_jsonb_input_filter_to_schema_builder,
            name::{
                renormalize_data_field_names_to_schema_context,
                renormalize_filter_field_names_to_schema_context,
            },
        },
    },
};

pub static CONTEXT: OnceCell<BuilderContext> = OnceCell::new();

pub fn build_schema(
    app_ctx: Arc<dyn AppContextTrait>,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let database = app_ctx.db().as_ref().clone();

    let context = CONTEXT.get_or_init(|| {
        let mut context = BuilderContext::default();

        renormalize_filter_field_names_to_schema_context(&mut context);
        renormalize_data_field_names_to_schema_context(&mut context);

        {
            // domains
            register_feeds_to_schema_context(&mut context);
            register_subscribers_to_schema_context(&mut context);
            register_subscriptions_to_schema_context(&mut context);
            register_subscriber_tasks_to_schema_context(&mut context);
            register_credential3rd_to_schema_context(&mut context, app_ctx.clone());
            register_downloaders_to_schema_context(&mut context);
            register_downloads_to_schema_context(&mut context);
            register_episodes_to_schema_context(&mut context);
            register_subscription_bangumi_to_schema_context(&mut context);
            register_subscription_episode_to_schema_context(&mut context);
            register_bangumi_to_schema_context(&mut context);
            register_cron_to_schema_context(&mut context);
            register_system_tasks_to_schema_context(&mut context);
        }
        context
    });

    let mut builder = Builder::new(context, database.clone());

    {
        // infra
        builder = register_jsonb_input_filter_to_schema_builder(builder);
    }
    {
        // domains
        builder = register_subscribers_to_schema_builder(builder);
        builder = register_feeds_to_schema_builder(builder);
        builder = register_episodes_to_schema_builder(builder);
        builder = register_subscription_bangumi_to_schema_builder(builder);
        builder = register_subscription_episode_to_schema_builder(builder);
        builder = register_downloaders_to_schema_builder(builder);
        builder = register_downloads_to_schema_builder(builder);
        builder = register_subscriptions_to_schema_builder(builder);
        builder = register_credential3rd_to_schema_builder(builder);
        builder = register_subscriber_tasks_to_schema_builder(builder);
        builder = register_bangumi_to_schema_builder(builder);
        builder = register_cron_to_schema_builder(builder);
        builder = register_system_tasks_to_schema_builder(builder);
    }

    let schema = builder.schema_builder();

    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    schema
        .data(database)
        .data(app_ctx)
        .finish()
        .inspect_err(|e| tracing::error!(e = ?e))
}

#[cfg(test)]
mod tests {
    use seaography::BuilderContext;

    use crate::graphql::{
        domains::{
            bangumi::register_bangumi_to_schema_context,
            cron::register_cron_to_schema_context,
            downloaders::register_downloaders_to_schema_context,
            downloads::register_downloads_to_schema_context,
            episodes::register_episodes_to_schema_context,
            feeds::register_feeds_to_schema_context,
            subscriber_tasks::register_subscriber_tasks_to_schema_context,
            subscribers::register_subscribers_to_schema_context,
            subscription_bangumi::register_subscription_bangumi_to_schema_context,
            subscription_episode::register_subscription_episode_to_schema_context,
            subscriptions::register_subscriptions_to_schema_context,
            system_tasks::register_system_tasks_to_schema_context,
        },
        infra::name::get_entity_name,
    };

    /// Build a BuilderContext with all domain registrations applied
    /// (except credential_3rd which requires AppContext).
    fn build_test_context() -> BuilderContext {
        let mut context = BuilderContext::default();
        register_feeds_to_schema_context(&mut context);
        register_subscribers_to_schema_context(&mut context);
        register_subscriptions_to_schema_context(&mut context);
        register_subscriber_tasks_to_schema_context(&mut context);
        register_downloaders_to_schema_context(&mut context);
        register_downloads_to_schema_context(&mut context);
        register_episodes_to_schema_context(&mut context);
        register_subscription_bangumi_to_schema_context(&mut context);
        register_subscription_episode_to_schema_context(&mut context);
        register_bangumi_to_schema_context(&mut context);
        register_cron_to_schema_context(&mut context);
        register_system_tasks_to_schema_context(&mut context);
        context
    }

    /// Helper to build [TypeName].[columnName] for assertions.
    fn graphql_key<T: sea_orm::EntityTrait>(
        ctx: &BuilderContext,
        column: &T::Column,
    ) -> String
    where
        <T as sea_orm::EntityTrait>::Model: Sync,
    {
        use crate::graphql::infra::name::get_column_name;
        let entity_name = get_entity_name::<T>(ctx);
        let column_name = get_column_name::<T>(ctx, column);
        format!("{entity_name}.{column_name}")
    }

    /// Asserts that auto-generated/auto-injected fields are correctly
    /// excluded from insert input types. This catches regressions where
    /// someone adds `before_save` logic but forgets to add insert_skips.
    #[test]
    fn insert_skips_exclude_auto_generated_fields() {
        let ctx = build_test_context();
        let skips = &ctx.entity_input.insert_skips;

        // feeds.token — auto-generated via before_save (UUID v7)
        use crate::models::feeds;
        assert!(
            skips.contains(&graphql_key::<feeds::Entity>(&ctx, &feeds::Column::Token)),
            "feeds.token should be in insert_skips (auto-generated via before_save)"
        );

        // subscriber_id — auto-injected via auth hooks for all subscriber-bound entities
        use crate::models::{
            bangumi, downloaders, downloads, episodes, subscriptions,
        };

        let entities_with_subscriber_id: Vec<String> = vec![
            graphql_key::<feeds::Entity>(&ctx, &feeds::Column::SubscriberId),
            graphql_key::<subscriptions::Entity>(&ctx, &subscriptions::Column::SubscriberId),
            graphql_key::<bangumi::Entity>(&ctx, &bangumi::Column::SubscriberId),
            graphql_key::<episodes::Entity>(&ctx, &episodes::Column::SubscriberId),
            graphql_key::<downloaders::Entity>(&ctx, &downloaders::Column::SubscriberId),
            graphql_key::<downloads::Entity>(&ctx, &downloads::Column::SubscriberId),
        ];

        for key in &entities_with_subscriber_id {
            assert!(
                skips.contains(key),
                "{key} should be in insert_skips (auto-injected subscriber_id)"
            );
        }
    }

    /// Asserts that subscriber_id is excluded from update input types.
    #[test]
    fn update_skips_exclude_subscriber_id() {
        let ctx = build_test_context();
        let skips = &ctx.entity_input.update_skips;

        use crate::models::{
            bangumi, downloaders, downloads, episodes, feeds, subscriptions,
        };

        let entities_with_subscriber_id: Vec<String> = vec![
            graphql_key::<feeds::Entity>(&ctx, &feeds::Column::SubscriberId),
            graphql_key::<subscriptions::Entity>(&ctx, &subscriptions::Column::SubscriberId),
            graphql_key::<bangumi::Entity>(&ctx, &bangumi::Column::SubscriberId),
            graphql_key::<episodes::Entity>(&ctx, &episodes::Column::SubscriberId),
            graphql_key::<downloaders::Entity>(&ctx, &downloaders::Column::SubscriberId),
            graphql_key::<downloads::Entity>(&ctx, &downloads::Column::SubscriberId),
        ];

        for key in &entities_with_subscriber_id {
            assert!(
                skips.contains(key),
                "{key} should be in update_skips (subscriber_id should be immutable)"
            );
        }
    }

    /// Asserts that SubscriberTasks and SystemTasks only expose the 'job'
    /// field in insert input (all other fields are skipped).
    #[test]
    fn task_entities_only_expose_job_in_insert() {
        let ctx = build_test_context();
        let skips = &ctx.entity_input.insert_skips;

        use crate::models::{subscriber_tasks, system_tasks};

        // subscriber_tasks: all columns except Job should be in insert_skips
        for column in <subscriber_tasks::Column as sea_orm::Iterable>::iter() {
            let key = graphql_key::<subscriber_tasks::Entity>(&ctx, &column);
            if matches!(column, subscriber_tasks::Column::Job) {
                assert!(
                    !skips.contains(&key),
                    "subscriber_tasks.job should NOT be in insert_skips"
                );
            } else {
                assert!(
                    skips.contains(&key),
                    "{key} should be in insert_skips for subscriber_tasks"
                );
            }
        }

        // system_tasks: all columns except Job should be in insert_skips
        for column in <system_tasks::Column as sea_orm::Iterable>::iter() {
            let key = graphql_key::<system_tasks::Entity>(&ctx, &column);
            if matches!(column, system_tasks::Column::Job) {
                assert!(
                    !skips.contains(&key),
                    "system_tasks.job should NOT be in insert_skips"
                );
            } else {
                assert!(
                    skips.contains(&key),
                    "{key} should be in insert_skips for system_tasks"
                );
            }
        }
    }
}
