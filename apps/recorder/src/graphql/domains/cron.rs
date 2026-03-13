use sea_orm::Iterable;
use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::{
            subscriber_tasks::restrict_subscriber_tasks_for_entity,
            subscribers::restrict_subscriber_for_entity,
            system_tasks::restrict_system_tasks_for_entity,
        },
        infra::{custom::register_entity_default_writable, name::GraphqlColumnKey},
    },
    models::cron,
};

fn skip_columns_for_entity_input(context: &mut BuilderContext) {
    for column in cron::Column::iter() {
        if matches!(
            column,
            cron::Column::SubscriberTaskCron
                | cron::Column::SystemTaskCron
                | cron::Column::CronExpr
                | cron::Column::CronTimezone
                | cron::Column::Enabled
                | cron::Column::TimeoutMs
                | cron::Column::MaxAttempts
        ) {
            continue;
        }
        GraphqlColumnKey::of::<cron::Entity>(context, &column)
            .push_insert_skip(context);
    }
    for column in cron::Column::iter() {
        if matches!(column, |cron::Column::CronExpr| cron::Column::CronTimezone
            | cron::Column::Enabled
            | cron::Column::TimeoutMs
            | cron::Column::Priority
            | cron::Column::MaxAttempts)
        {
            continue;
        }
        GraphqlColumnKey::of::<cron::Entity>(context, &column)
            .push_update_skip(context);
    }
}

pub fn register_cron_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<cron::Entity>(context, &cron::Column::SubscriberId);

    restrict_subscriber_tasks_for_entity::<cron::Entity>(
        context,
        &cron::Column::SubscriberTaskCron,
    );
    restrict_system_tasks_for_entity::<cron::Entity>(context, &cron::Column::SystemTaskCron);
    skip_columns_for_entity_input(context);
}

pub fn register_cron_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<cron::CronStatus>();

    builder = register_entity_default_writable!(builder, cron);

    builder
}
