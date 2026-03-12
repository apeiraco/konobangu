use std::sync::Arc;

use async_graphql::dynamic::ValueAccessor;
use sea_orm::{EntityTrait, Value as SeaValue};
use seaography::{BuilderContext, EntityColumnId, SeaResult};

use crate::app::AppContextTrait;

pub fn register_crypto_column_input_conversion_to_schema_context<T>(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
    column: &T::Column,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_column_id = EntityColumnId::of::<T>(column);
    let options = context
        .types
        .column_options
        .entry(entity_column_id)
        .or_default();

    options.input_conversion = Some(Arc::new(
        move |value: &ValueAccessor| -> SeaResult<sea_orm::Value> {
            let source = value.string()?;
            let encrypted = ctx.crypto().encrypt_string(source.into())?;
            Ok(encrypted.into())
        },
    ));
}

pub fn register_crypto_column_output_conversion_to_schema_context<T>(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
    column: &T::Column,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_column_id = EntityColumnId::of::<T>(column);
    let options = context
        .types
        .column_options
        .entry(entity_column_id)
        .or_default();

    options.output_conversion = Some(Arc::new(
        move |value: &sea_orm::sea_query::Value| -> async_graphql::Result<Option<async_graphql::dynamic::FieldValue<'static>>> {
            if let SeaValue::String(s) = value {
                if let Some(s) = s {
                    let decrypted = ctx.crypto().decrypt_string(s)?;
                    Ok(Some(async_graphql::dynamic::FieldValue::from(
                        async_graphql::Value::String(decrypted),
                    )))
                } else {
                    Ok(None)
                }
            } else {
                Err(async_graphql::Error::new("crypto column must be string column"))
            }
        },
    ));
}
