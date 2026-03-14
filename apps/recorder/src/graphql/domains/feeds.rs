use seaography::{Builder as SeaographyBuilder, BuilderContext, EntityColumnId};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::custom::register_entity_default_writable,
    },
    models::feeds,
};

pub fn register_feeds_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<feeds::Entity>(context, &feeds::Column::SubscriberId);

    // In seaography 2.0, input_none_conversions no longer exists.
    // The token field auto-generation on create is handled via
    // ActiveModelBehavior::before_save in the feeds model.
    // We skip the token from insert input so users don't need to provide it.
    context
        .entity_input
        .insert_skips
        .push(EntityColumnId::of::<feeds::Entity>(&feeds::Column::Token).to_string());
}

pub fn register_feeds_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<feeds::FeedType>();
    builder.register_enumeration::<feeds::FeedSource>();

    builder = register_entity_default_writable!(builder, feeds);

    builder
}
