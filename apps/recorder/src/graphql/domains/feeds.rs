use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{custom::register_entity_default_writable, name::GraphqlColumnKey},
    },
    models::feeds,
};

pub fn register_feeds_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<feeds::Entity>(context, &feeds::Column::SubscriberId);

    // Token is auto-generated via ActiveModelBehavior::before_save.
    // Skip it from insert input so users don't need to provide it.
    GraphqlColumnKey::of::<feeds::Entity>(context, &feeds::Column::Token)
        .push_insert_skip(context);
}

pub fn register_feeds_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<feeds::FeedType>();
    builder.register_enumeration::<feeds::FeedSource>();

    builder = register_entity_default_writable!(builder, feeds);

    builder
}
