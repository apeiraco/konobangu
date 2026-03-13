use std::fmt::Display;

use sea_orm::{EntityName, EntityTrait, IdenStatic};
use seaography::BuilderContext;

pub fn get_entity_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let t = T::default();
    let name = <T as EntityName>::table_name(&t);
    context.entity_object.type_name.as_ref()(name)
}

pub fn get_column_name<T>(context: &BuilderContext, column: &T::Column) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_name::<T>(context);
    context.entity_object.column_name.as_ref()(&entity_name, column.as_str())
}

// ---------------------------------------------------------------------------
// GraphqlColumnKey — type-safe wrapper for insert_skips / update_skips keys
// ---------------------------------------------------------------------------

/// A newtype guaranteeing the string is in GraphQL format
/// "TypeName.columnName" (e.g. "Feeds.subscriberId"), suitable for
/// `insert_skips` / `update_skips`.
///
/// # SAFETY CONTRACT
/// - Construct ONLY via `GraphqlColumnKey::of::<T>(context, column)`.
/// - Do NOT construct from arbitrary strings or `EntityColumnId::to_string()`.
/// - `EntityColumnId::to_string()` returns the DATABASE format
///   ("feeds.subscriber_id") which will silently fail the skip check.
pub struct GraphqlColumnKey(String);

impl GraphqlColumnKey {
    /// Build a correctly-formatted skip key from a typed SeaORM column.
    pub fn of<T>(context: &BuilderContext, column: &T::Column) -> Self
    where
        T: EntityTrait,
        <T as EntityTrait>::Model: Sync,
    {
        let entity_name = get_entity_name::<T>(context);
        let column_name = get_column_name::<T>(context, column);
        Self(format!("{entity_name}.{column_name}"))
    }

    /// Push this key into `insert_skips` on the given context.
    pub fn push_insert_skip(self, context: &mut BuilderContext) {
        context.entity_input.insert_skips.push(self.0);
    }

    /// Push this key into `update_skips` on the given context.
    pub fn push_update_skip(self, context: &mut BuilderContext) {
        context.entity_input.update_skips.push(self.0);
    }

    /// Push into both `insert_skips` and `update_skips` (common pattern for
    /// subscriber_id and similar auto-injected fields).
    pub fn push_skip_both(self, context: &mut BuilderContext) {
        context.entity_input.insert_skips.push(self.0.clone());
        context.entity_input.update_skips.push(self.0);
    }
}

pub fn get_entity_basic_type_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let t = T::default();
    let name = <T as EntityName>::table_name(&t);
    format!(
        "{}{}",
        context.entity_object.type_name.as_ref()(name),
        context.entity_object.basic_type_suffix
    )
}

pub fn get_entity_query_field_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_name::<T>(context);
    context.entity_query_field.type_name.as_ref()(&entity_name)
}

pub fn get_entity_filter_input_type_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_name::<T>(context);
    context.filter_input.type_name.as_ref()(&entity_name)
}

pub fn get_entity_insert_input_type_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_name::<T>(context);
    format!("{entity_name}{}", context.entity_input.insert_suffix)
}

pub fn get_entity_update_input_type_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_name::<T>(context);
    format!("{entity_name}{}", context.entity_input.update_suffix)
}

pub fn get_entity_create_one_mutation_field_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let query_field_name = get_entity_query_field_name::<T>(context);
    format!(
        "{}{}",
        query_field_name, context.entity_create_one_mutation.mutation_suffix
    )
}

pub fn get_entity_create_batch_mutation_field_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let query_field_name = get_entity_query_field_name::<T>(context);
    format!(
        "{}{}",
        query_field_name, context.entity_create_batch_mutation.mutation_suffix
    )
}

pub fn get_entity_delete_mutation_field_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let query_field_name = get_entity_query_field_name::<T>(context);
    format!(
        "{}{}",
        query_field_name, context.entity_delete_mutation.mutation_suffix
    )
}

pub fn get_entity_update_mutation_field_name<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let query_field_name = get_entity_query_field_name::<T>(context);
    format!(
        "{}{}",
        query_field_name, context.entity_update_mutation.mutation_suffix
    )
}

pub fn get_entity_custom_mutation_field_name<T>(
    context: &BuilderContext,
    mutation_suffix: impl Display,
) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let query_field_name = get_entity_query_field_name::<T>(context);
    format!("{query_field_name}{mutation_suffix}")
}

pub fn get_entity_renormalized_filter_field_name() -> &'static str {
    "filter"
}

pub fn get_entity_query_filter_field_name(context: &BuilderContext) -> &str {
    &context.entity_query_field.filters
}

pub fn get_entity_update_mutation_filter_field_name(context: &BuilderContext) -> &str {
    &context.entity_update_mutation.filter_field
}

pub fn get_entity_delete_mutation_filter_field_name(context: &BuilderContext) -> &str {
    &context.entity_delete_mutation.filter_field
}

pub fn renormalize_filter_field_names_to_schema_context(context: &mut BuilderContext) {
    let renormalized_filter_field_name = get_entity_renormalized_filter_field_name();
    context.entity_query_field.filters = renormalized_filter_field_name.to_string();
    context.entity_update_mutation.filter_field = renormalized_filter_field_name.to_string();
    context.entity_delete_mutation.filter_field = renormalized_filter_field_name.to_string();
}

pub fn get_entity_renormalized_data_field_name() -> &'static str {
    "data"
}

pub fn get_entity_create_one_mutation_data_field_name(context: &BuilderContext) -> &str {
    &context.entity_create_one_mutation.data_field
}

pub fn get_entity_create_batch_mutation_data_field_name(context: &BuilderContext) -> &str {
    &context.entity_create_batch_mutation.data_field
}

pub fn get_entity_update_mutation_data_field_name(context: &BuilderContext) -> &str {
    &context.entity_update_mutation.data_field
}

pub fn renormalize_data_field_names_to_schema_context(context: &mut BuilderContext) {
    let renormalized_data_field_name = get_entity_renormalized_data_field_name();
    context.entity_create_one_mutation.data_field = renormalized_data_field_name.to_string();
    context.entity_create_batch_mutation.data_field = renormalized_data_field_name.to_string();
    context.entity_update_mutation.data_field = renormalized_data_field_name.to_string();
}
