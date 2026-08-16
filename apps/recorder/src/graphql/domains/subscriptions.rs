use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
  graphql::{
    domains::subscribers::restrict_subscriber_for_entity,
    infra::custom::{register_entity_default_mutations_using_delete, register_entity_default_readonly},
  },
  models::subscriptions,
};

pub fn register_subscriptions_to_schema_context(context: &mut BuilderContext) {
  restrict_subscriber_for_entity::<subscriptions::Entity>(context, &subscriptions::Column::SubscriberId);
}

pub fn register_subscriptions_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
  builder.register_enumeration::<subscriptions::SubscriptionCategory>();
  builder = register_entity_default_readonly!(builder, subscriptions);
  register_entity_default_mutations_using_delete::<subscriptions::Entity, subscriptions::ActiveModel>(
    builder,
    Some(|db, filter| Box::pin(subscriptions::operation::delete_owned(db, filter))),
  )
}
