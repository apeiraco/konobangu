use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::custom::register_entity_default_writable,
    },
    models::episodes,
};

pub fn register_episodes_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<episodes::Entity>(context, &episodes::Column::SubscriberId);
}

pub fn register_episodes_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<episodes::EpisodeType>();
    builder = register_entity_default_writable!(builder, episodes);

    builder
}
