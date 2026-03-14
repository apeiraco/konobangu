use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::custom::register_entity_default_writable,
    },
    models::downloads,
};

pub fn register_downloads_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<downloads::Entity>(context, &downloads::Column::SubscriberId);
}

pub fn register_downloads_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<downloads::DownloadStatus>();
    builder.register_enumeration::<downloads::DownloadMime>();
    builder = register_entity_default_writable!(builder, downloads);

    builder
}
