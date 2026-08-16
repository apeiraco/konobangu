use std::sync::Arc;

use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
  app::AppContextTrait,
  graphql::{
    domains::subscribers::restrict_subscriber_for_entity,
    infra::{
      crypto::{register_crypto_column_input_conversion_to_schema_context, register_crypto_column_output_conversion_to_schema_context},
      custom::register_entity_default_writable,
    },
  },
  models::credential_3rd,
};

pub fn register_credential3rd_to_schema_context(context: &mut BuilderContext, ctx: Arc<dyn AppContextTrait>) {
  restrict_subscriber_for_entity::<credential_3rd::Entity>(context, &credential_3rd::Column::SubscriberId);
  register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(context, ctx.clone(), &credential_3rd::Column::Cookies);
  register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(context, ctx.clone(), &credential_3rd::Column::Username);
  register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(context, ctx.clone(), &credential_3rd::Column::Password);
  register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(context, ctx.clone(), &credential_3rd::Column::Cookies);
  register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(context, ctx.clone(), &credential_3rd::Column::Username);
  register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(context, ctx, &credential_3rd::Column::Password);
}

pub fn register_credential3rd_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
  builder.register_enumeration::<credential_3rd::Credential3rdType>();
  builder = register_entity_default_writable!(builder, credential_3rd);

  builder
}
