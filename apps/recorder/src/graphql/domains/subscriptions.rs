use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::custom::register_entity_default_writable,
    },
    models::subscriptions,
};

pub fn register_subscriptions_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscriptions::Entity>(
        context,
        &subscriptions::Column::SubscriberId,
    );
}

pub fn register_subscriptions_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.register_enumeration::<subscriptions::SubscriptionCategory>();
    builder = register_entity_default_writable!(builder, subscriptions);
    builder
}
