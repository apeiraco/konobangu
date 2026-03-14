use std::sync::Arc;

use async_graphql::dynamic::{Field, FieldFuture, FieldValue, Object, TypeRef};
use sea_orm::{EntityTrait, QueryFilter};
use seaography::{Builder as SeaographyBuilder, BuilderContext};
use serde::{Deserialize, Serialize};
use util_derive::DynamicGraphql;

use crate::{
    app::AppContextTrait,
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            crypto::{
                register_crypto_column_input_conversion_to_schema_context,
                register_crypto_column_output_conversion_to_schema_context,
            },
            custom::{generate_entity_filtered_mutation_field, register_entity_default_writable},
            name::get_entity_custom_mutation_field_name,
        },
    },
    models::credential_3rd,
};

#[derive(DynamicGraphql, Serialize, Deserialize, Clone, Debug)]
pub struct Credential3rdCheckAvailableInfo {
    pub available: bool,
}

impl Credential3rdCheckAvailableInfo {
    fn object_type_name() -> &'static str {
        "Credential3rdCheckAvailableInfo"
    }

    fn generate_output_object() -> Object {
        Object::new(Self::object_type_name())
            .description("The output of the credential3rdCheckAvailable query")
            .field(Field::new(
                Credential3rdCheckAvailableInfoFieldEnum::Available,
                TypeRef::named_nn(TypeRef::BOOLEAN),
                move |ctx| {
                    FieldFuture::new(async move {
                        let subscription_info = ctx.parent_value.try_downcast_ref::<Self>()?;
                        Ok(Some(async_graphql::Value::from(
                            subscription_info.available,
                        )))
                    })
                },
            ))
    }
}

pub fn register_credential3rd_to_schema_context(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
) {
    restrict_subscriber_for_entity::<credential_3rd::Entity>(
        context,
        &credential_3rd::Column::SubscriberId,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Password,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx,
        &credential_3rd::Column::Password,
    );
}

pub fn register_credential3rd_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.register_enumeration::<credential_3rd::Credential3rdType>();
    builder = register_entity_default_writable!(builder, credential_3rd);

    builder.schema = builder
        .schema
        .register(Credential3rdCheckAvailableInfo::generate_output_object());

    let builder_context = &builder.context;
    {
        let check_available_mutation_name = get_entity_custom_mutation_field_name::<
            credential_3rd::Entity,
        >(builder_context, "CheckAvailable");
        let check_available_mutation =
            generate_entity_filtered_mutation_field::<credential_3rd::Entity, _, _>(
                builder_context,
                check_available_mutation_name,
                TypeRef::named_nn(Credential3rdCheckAvailableInfo::object_type_name()),
                Arc::new(|_resolver_ctx, app_ctx, filters| {
                    Box::pin(async move {
                        let db = app_ctx.db();

                        let credential_model = credential_3rd::Entity::find()
                            .filter(filters)
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<credential_3rd::Entity>()
                            })?;

                        let available = credential_model.check_available(app_ctx.as_ref()).await?;
                        Ok(Some(FieldValue::owned_any(
                            Credential3rdCheckAvailableInfo { available },
                        )))
                    })
                }),
            );
        builder.mutations.push(check_available_mutation);
    }

    builder
}
