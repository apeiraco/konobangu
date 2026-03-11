//! Subscriber-scoped authorization hooks for GraphQL.
//!
//! This module centralizes the subscriber-based authorization logic that was
//! previously spread across seaography fork's `BuilderContext` configuration
//! (entity_guards, field_guards, condition_functions, input_none_conversions).
//!
//! The design intentionally aligns with seaography 2.0's `LifecycleHooksInterface`
//! trait pattern (`entity_guard`, `field_guard`, `entity_filter`,
//! `before_active_model_save`) to minimize future migration friction.
//!
//! ## Responsibilities
//!
//! - **Entity guard**: Requires authentication before accessing any subscriber-scoped entity
//! - **Field guard**: Validates subscriber_id in mutation payloads matches the authenticated user
//! - **Query filter**: Auto-injects `subscriber_id = current_user` condition on queries
//! - **Default value injection**: Auto-fills subscriber_id on create mutations
//!
//! ## Relationship to seaography extensions
//!
//! These hooks still register into seaography's `BuilderContext` extension points
//! (guards, condition_functions, input_none_conversions) for now. When migrating
//! to seaography 2.0, the logic here can be moved to implement
//! `LifecycleHooksInterface` directly.

use std::sync::Arc;

use async_graphql::dynamic::{ObjectAccessor, ResolverContext, TypeRef, ValueAccessor};
use lazy_static::lazy_static;
use maplit::btreeset;
use sea_orm::{ColumnTrait, Condition, EntityTrait, Value as SeaValue};
use seaography::{
    BuilderContext, FilterInfo, FilterOperation as SeaographqlFilterOperation, FilterType,
    FnFilterCondition, FnGuard, FnInputTypeNoneConversion, GuardAction, SeaResult,
};

use crate::{
    auth::{AuthError, AuthUserInfo},
    graphql::infra::name::{
        get_column_name, get_entity_and_column_name,
        get_entity_create_batch_mutation_data_field_name,
        get_entity_create_batch_mutation_field_name,
        get_entity_create_one_mutation_data_field_name,
        get_entity_create_one_mutation_field_name, get_entity_name,
        get_entity_update_mutation_data_field_name, get_entity_update_mutation_field_name,
    },
};

// ---------------------------------------------------------------------------
// Filter info for subscriber_id (limited to equality check only)
// ---------------------------------------------------------------------------

lazy_static! {
    pub static ref SUBSCRIBER_ID_FILTER_INFO: FilterInfo = FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    };
}

// ---------------------------------------------------------------------------
// Entity guard — checks that the caller is authenticated
// ---------------------------------------------------------------------------

/// Creates an entity-level guard that requires authentication.
///
/// Aligned with seaography 2.0's `LifecycleHooksInterface::entity_guard`.
pub fn create_entity_auth_guard<T>(_context: &BuilderContext, _column: &T::Column) -> FnGuard
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    Box::new(move |context: &ResolverContext| -> GuardAction {
        match context.ctx.data::<AuthUserInfo>() {
            Ok(_) => GuardAction::Allow,
            Err(err) => GuardAction::Block(Some(err.message)),
        }
    })
}

// ---------------------------------------------------------------------------
// Field guard — validates subscriber_id in mutation payloads
// ---------------------------------------------------------------------------

fn validate_subscriber_id_in_object(
    value: ValueAccessor<'_>,
    column_name: &str,
    subscriber_id: i32,
) -> async_graphql::Result<()> {
    let obj = value.object()?;

    let subscriber_id_value = obj.try_get(column_name)?;

    let id = subscriber_id_value.i64()?;

    if id == subscriber_id as i64 {
        Ok(())
    } else {
        Err(async_graphql::Error::new("subscriber not match"))
    }
}

fn validate_optional_subscriber_id_in_object(
    value: ValueAccessor<'_>,
    column_name: &str,
    subscriber_id: i32,
) -> async_graphql::Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let obj = value.object()?;

    if let Some(subscriber_id_value) = obj.get(column_name) {
        let id = subscriber_id_value.i64()?;
        if id == subscriber_id as i64 {
            Ok(())
        } else {
            Err(async_graphql::Error::new("subscriber not match"))
        }
    } else {
        Ok(())
    }
}

/// Creates a field-level guard that validates subscriber_id in mutation payloads.
///
/// Aligned with seaography 2.0's `LifecycleHooksInterface::field_guard`.
pub fn create_field_subscriber_id_guard<T>(
    context: &BuilderContext,
    column: &T::Column,
) -> FnGuard
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column_name = Arc::new(get_column_name::<T>(context, column));
    let entity_create_one_mutation_field_name =
        Arc::new(get_entity_create_one_mutation_field_name::<T>(context));
    let entity_create_one_mutation_data_field_name =
        Arc::new(get_entity_create_one_mutation_data_field_name(context).to_string());
    let entity_create_batch_mutation_field_name =
        Arc::new(get_entity_create_batch_mutation_field_name::<T>(context));
    let entity_create_batch_mutation_data_field_name =
        Arc::new(get_entity_create_batch_mutation_data_field_name(context).to_string());
    let entity_update_mutation_field_name =
        Arc::new(get_entity_update_mutation_field_name::<T>(context));
    let entity_update_mutation_data_field_name =
        Arc::new(get_entity_update_mutation_data_field_name(context).to_string());

    Box::new(move |context: &ResolverContext| -> GuardAction {
        match context.ctx.data::<AuthUserInfo>() {
            Ok(user_info) => {
                let subscriber_id = user_info.subscriber_auth.subscriber_id;
                let validation_result = match context.field().name() {
                    field if field == entity_create_one_mutation_field_name.as_str() => {
                        if let Some(data_value) = context
                            .args
                            .get(&entity_create_one_mutation_data_field_name)
                        {
                            validate_subscriber_id_in_object(
                                data_value,
                                &column_name,
                                subscriber_id,
                            )
                            .map_err(|inner_error| {
                                AuthError::from_graphql_dynamic_subscribe_id_guard(
                                    inner_error,
                                    context,
                                    &entity_create_one_mutation_data_field_name,
                                    &column_name,
                                )
                            })
                        } else {
                            Ok(())
                        }
                    }
                    field if field == entity_create_batch_mutation_field_name.as_str() => {
                        if let Some(data_value) = context
                            .args
                            .get(&entity_create_batch_mutation_data_field_name)
                        {
                            data_value
                                .list()
                                .and_then(|data_list| {
                                    data_list.iter().try_for_each(|data_item_value| {
                                        validate_optional_subscriber_id_in_object(
                                            data_item_value,
                                            &column_name,
                                            subscriber_id,
                                        )
                                    })
                                })
                                .map_err(|inner_error| {
                                    AuthError::from_graphql_dynamic_subscribe_id_guard(
                                        inner_error,
                                        context,
                                        &entity_create_batch_mutation_data_field_name,
                                        &column_name,
                                    )
                                })
                        } else {
                            Ok(())
                        }
                    }
                    field if field == entity_update_mutation_field_name.as_str() => {
                        if let Some(data_value) =
                            context.args.get(&entity_update_mutation_data_field_name)
                        {
                            validate_optional_subscriber_id_in_object(
                                data_value,
                                &column_name,
                                subscriber_id,
                            )
                            .map_err(|inner_error| {
                                AuthError::from_graphql_dynamic_subscribe_id_guard(
                                    inner_error,
                                    context,
                                    &entity_update_mutation_data_field_name,
                                    &column_name,
                                )
                            })
                        } else {
                            Ok(())
                        }
                    }
                    _ => Ok(()),
                };
                match validation_result {
                    Ok(_) => GuardAction::Allow,
                    Err(err) => GuardAction::Block(Some(err.to_string())),
                }
            }
            Err(err) => GuardAction::Block(Some(err.message)),
        }
    })
}

// ---------------------------------------------------------------------------
// Query filter — auto-injects subscriber_id scope on queries
// ---------------------------------------------------------------------------

/// Creates a filter condition that auto-scopes queries to the current subscriber.
///
/// Aligned with seaography 2.0's `LifecycleHooksInterface::entity_filter`.
pub fn create_subscriber_id_filter_condition<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterCondition
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(
        move |context: &ResolverContext,
              mut condition: Condition,
              filter: Option<&ObjectAccessor<'_>>|
              -> SeaResult<Condition> {
            match context.ctx.data::<AuthUserInfo>() {
                Ok(user_info) => {
                    let subscriber_id = user_info.subscriber_auth.subscriber_id;

                    if let Some(filter) = filter {
                        for operation in &SUBSCRIBER_ID_FILTER_INFO.supported_operations {
                            match operation {
                                SeaographqlFilterOperation::Equals => {
                                    if let Some(value) = filter.get("eq") {
                                        let value: i32 = value.i64()?.try_into()?;
                                        if value != subscriber_id {
                                            return Err(async_graphql::Error::new(
                                                "subscriber_id and auth_info does not match",
                                            )
                                            .into());
                                        }
                                    }
                                }
                                _ => unreachable!("unreachable filter operation for subscriber_id"),
                            }
                        }
                    } else {
                        condition = condition.add(column.eq(subscriber_id));
                    }

                    Ok(condition)
                }
                Err(err) => unreachable!("auth user info must be guarded: {:?}", err),
            }
        },
    )
}

// ---------------------------------------------------------------------------
// Default value injection — auto-fills subscriber_id on create mutations
// ---------------------------------------------------------------------------

/// Creates a conversion that auto-injects subscriber_id from auth context
/// when the field is not provided in create mutations.
///
/// Aligned with seaography 2.0's `LifecycleHooksInterface::before_active_model_save`.
pub fn create_subscriber_id_default_injection<T>(
    context: &BuilderContext,
    _column: &T::Column,
) -> FnInputTypeNoneConversion
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_create_one_mutation_field_name =
        Arc::new(get_entity_create_one_mutation_field_name::<T>(context));
    let entity_create_batch_mutation_field_name =
        Arc::new(get_entity_create_batch_mutation_field_name::<T>(context));
    Box::new(
        move |context: &ResolverContext| -> SeaResult<Option<SeaValue>> {
            let field_name = context.field().name();
            if field_name == entity_create_one_mutation_field_name.as_str()
                || field_name == entity_create_batch_mutation_field_name.as_str()
            {
                match context.ctx.data::<AuthUserInfo>() {
                    Ok(user_info) => {
                        let subscriber_id = user_info.subscriber_auth.subscriber_id;
                        Ok(Some(SeaValue::Int(Some(subscriber_id))))
                    }
                    Err(err) => unreachable!("auth user info must be guarded: {:?}", err),
                }
            } else {
                Ok(None)
            }
        },
    )
}

// ---------------------------------------------------------------------------
// Combined registration — configures all subscriber auth hooks for an entity
// ---------------------------------------------------------------------------

/// Registers all subscriber-scoped authorization hooks for a given entity.
///
/// This is the entry point that domain modules call to protect a subscriber-owned
/// entity. It configures:
/// 1. Entity guard (authentication check)
/// 2. Field guard (subscriber_id validation in mutations)
/// 3. Filter type overwrite (custom subscriber_id filter input)
/// 4. Filter condition (auto-scope queries to current subscriber)
/// 5. Input default injection (auto-fill subscriber_id on creates)
/// 6. Update skip (prevent subscriber_id from being updated)
pub fn restrict_subscriber_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_and_column = get_entity_and_column_name::<T>(context, column);

    // 1. Entity guard — require authentication
    context.guards.entity_guards.insert(
        get_entity_name::<T>(context),
        create_entity_auth_guard::<T>(context, column),
    );

    // 2. Field guard — validate subscriber_id in mutation payloads
    context.guards.field_guards.insert(
        get_entity_and_column_name::<T>(context, column),
        create_field_subscriber_id_guard::<T>(context, column),
    );

    // 3. Filter type overwrite — use custom subscriber_id filter
    context.filter_types.overwrites.insert(
        get_entity_and_column_name::<T>(context, column),
        Some(FilterType::Custom(
            SUBSCRIBER_ID_FILTER_INFO.type_name.clone(),
        )),
    );

    // 4. Filter condition — auto-scope queries
    context.filter_types.condition_functions.insert(
        entity_and_column.clone(),
        create_subscriber_id_filter_condition::<T>(context, column),
    );

    // 5. Input default — auto-fill subscriber_id
    context.types.input_none_conversions.insert(
        entity_and_column.clone(),
        create_subscriber_id_default_injection::<T>(context, column),
    );

    // 6. Update skip — prevent subscriber_id modification
    context.entity_input.update_skips.push(entity_and_column);
}
