use std::{collections::HashMap, sync::Arc};

use async_graphql::dynamic::{FieldValue, TypeRef};
use sea_orm::{ActiveEnum, EntityTrait, Value};
use seaography::{BuilderContext, EntityColumnId, FilterType};

/// Text-backed ActiveEnums still expose their existing GraphQL enum contracts.
pub fn register<E: EntityTrait, A: ActiveEnum>(context: &mut BuilderContext, column: &E::Column) {
  let name = A::name().to_string();
  let output = (context.active_enum.type_name)(&name);
  let values: HashMap<_, _> = A::values()
    .into_iter()
    .filter_map(|value| {
      let value: Value = value.into();
      if let Value::String(Some(value)) = value {
        let variant = (context.active_enum.variant_name)(&output, &value);
        Some((value, variant))
      } else {
        None
      }
    })
    .collect();
  let key = EntityColumnId::of::<E>(column);
  context.filter_types.overwrites.insert(key.clone(), Some(FilterType::Enumeration(name)));
  let options = context.types.column_options.entry(key).or_default();
  options.output_type = Some(TypeRef::named_nn(output));
  options.output_conversion = Some(Arc::new(move |value| match value {
    Value::String(Some(value)) => {
      let variant = values
        .get(value.as_str())
        .ok_or_else(|| async_graphql::Error::new("Invalid stored enumeration value"))?;
      Ok(Some(FieldValue::value(async_graphql::Value::Enum(async_graphql::Name::new(variant)))))
    }
    Value::String(None) => Ok(None),
    _ => Err(async_graphql::Error::new("Enumeration column must be text")),
  }));
}
