use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::custom::register_entity_default_writable,
    },
    models::subscription_bangumi,
};

pub fn register_subscription_bangumi_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscription_bangumi::Entity>(
        context,
        &subscription_bangumi::Column::SubscriberId,
    );
}

pub fn register_subscription_bangumi_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder = register_entity_default_writable!(builder, subscription_bangumi);

    builder
}
