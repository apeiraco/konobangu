use sea_orm::Iterable;
use seaography::{Builder as SeaographyBuilder, BuilderContext, EntityColumnId, FilterTypesMapHelper};

// Re-export restrict_subscriber_for_entity from the centralized auth_hooks module
// so that existing domain files can continue importing from `subscribers::`
pub use crate::graphql::infra::auth_hooks::restrict_subscriber_for_entity;
use crate::{
    graphql::infra::{
        auth_hooks::SUBSCRIBER_ID_FILTER_INFO,
        custom::register_entity_default_readonly,
    },
    models::subscribers,
};

pub fn register_subscribers_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscribers::Entity>(context, &subscribers::Column::Id);
    for column in subscribers::Column::iter() {
        if !matches!(column, subscribers::Column::Id) {
            let key = EntityColumnId::of::<subscribers::Entity>(&column);
            context.filter_types.overwrites.insert(key, None);
        }
    }
}

pub fn register_subscribers_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    {
        let filter_helper = FilterTypesMapHelper {
            context: builder.context,
        };
        builder.schema = builder
            .schema
            .register(filter_helper.generate_filter_input(
                &SUBSCRIBER_ID_FILTER_INFO,
            ));
    }

    builder = register_entity_default_readonly!(builder, subscribers);

    builder
}
