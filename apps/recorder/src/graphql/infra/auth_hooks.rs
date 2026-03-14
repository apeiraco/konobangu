use std::collections::HashMap;

use async_graphql::dynamic::ResolverContext;
use lazy_static::lazy_static;
use maplit::btreeset;
use sea_orm::{ColumnTrait, Condition, EntityTrait};
use seaography::{
    BuilderContext, EntityColumnId, FilterInfo, FilterOperation as SeaographqlFilterOperation,
    FilterType, GuardAction, LifecycleHooksInterface, OperationType,
};

use crate::auth::AuthUserInfo;

// ---------------------------------------------------------------------------
// Filter info for subscriber_id (limited to equality check only)
// ---------------------------------------------------------------------------

lazy_static! {
    pub static ref SUBSCRIBER_ID_FILTER_INFO: FilterInfo = FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: async_graphql::dynamic::TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    };
}

// ---------------------------------------------------------------------------
// Column filter function type — used to store type-erased column references
// ---------------------------------------------------------------------------

/// A type-erased function that produces a `Condition` filtering by subscriber_id.
/// Each entity registsers its own column so `entity_filter` does not need to
/// hardcode column names.
type SubscriberIdFilterFn =
    Box<dyn Fn(i32) -> Condition + Send + Sync>;

// ---------------------------------------------------------------------------
// SubscriberAuthHooks — implements LifecycleHooksInterface
// ---------------------------------------------------------------------------

/// Lifecycle hooks that enforce subscriber-scoped authorization.
///
/// This struct implements seaography 2.0's `LifecycleHooksInterface` to:
/// 1. Guard entity access (require authentication)
/// 2. Filter queries by subscriber_id
///
/// Entities must be registered via `restrict_subscriber_for_entity` so that
/// these hooks know which entities and columns to protect.
pub struct SubscriberAuthHooks {
    /// Set of entity names that require subscriber authentication
    guarded_entities: HashMap<String, SubscriberIdFilterFn>,
}

impl SubscriberAuthHooks {
    pub fn new() -> Self {
        Self {
            guarded_entities: HashMap::new(),
        }
    }

    /// Register an entity and its subscriber_id column for protection.
    pub fn register_entity<T>(&mut self, column: &T::Column)
    where
        T: EntityTrait,
        <T as EntityTrait>::Model: Sync,
    {
        use sea_orm::EntityName;

        let t = T::default();
        let table_name = <T as EntityName>::table_name(&t).to_string();

        // Capture the concrete column value so entity_filter can use it
        // through ColumnTrait (type-safe) instead of hardcoded string.
        let col = *column;
        self.guarded_entities.insert(
            table_name,
            Box::new(move |subscriber_id: i32| {
                Condition::all().add(col.eq(subscriber_id))
            }),
        );
    }
}

impl Default for SubscriberAuthHooks {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LifecycleHooksInterface for SubscriberAuthHooks {
    /// Entity guard — checks that the caller is authenticated.
    fn entity_guard(
        &self,
        ctx: &ResolverContext,
        _entity: &str,
        _action: OperationType,
    ) -> GuardAction {
        match ctx.ctx.data::<AuthUserInfo>() {
            Ok(_) => GuardAction::Allow,
            Err(err) => GuardAction::Block(Some(err.message)),
        }
    }

    /// Entity filter — auto-injects subscriber_id scope on queries.
    fn entity_filter(
        &self,
        ctx: &ResolverContext,
        entity: &str,
        _action: OperationType,
    ) -> Option<Condition> {
        let filter_fn = self.guarded_entities.get(entity)?;

        match ctx.ctx.data::<AuthUserInfo>() {
            Ok(user_info) => {
                let subscriber_id = user_info.subscriber_auth.subscriber_id;
                Some(filter_fn(subscriber_id))
            }
            Err(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Configuration helper — sets up filter types and input skips
// ---------------------------------------------------------------------------

/// Registers subscriber-scoped configuration for a given entity on the BuilderContext.
///
/// This configures:
/// 1. Filter type overwrite (custom subscriber_id filter input)
/// 2. Update skip (prevent subscriber_id from being updated)
/// 3. Filter condition function for subscriber_id scoping
///
/// The actual guard and filter logic is in `SubscriberAuthHooks` which
/// implements `LifecycleHooksInterface`.
pub fn restrict_subscriber_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_column_id = EntityColumnId::of::<T>(column);

    // Filter type overwrite — use custom subscriber_id filter
    context.filter_types.overwrites.insert(
        entity_column_id.clone(),
        Some(FilterType::Custom(
            SUBSCRIBER_ID_FILTER_INFO.type_name.clone(),
        )),
    );

    // Filter condition — subscriber_id scoping
    {
        let column = *column;
        context.filter_types.condition_functions.insert(
            entity_column_id.clone(),
            Box::new(
                move |condition: Condition,
                      filter: &async_graphql::dynamic::ObjectAccessor<'_>|
                      -> seaography::SeaResult<Condition> {
                    // In 2.0, filter condition doesn't have ResolverContext.
                    // The subscriber_id filtering is now handled by entity_filter in hooks.
                    // This condition_function only validates explicit filter values.
                    for operation in &SUBSCRIBER_ID_FILTER_INFO.supported_operations {
                        match operation {
                            SeaographqlFilterOperation::Equals => {
                                if let Some(value) = filter.get("eq") {
                                    let value: i32 = value.i64()?.try_into()?;
                                    return Ok(condition.add(column.eq(value)));
                                }
                            }
                            _ => unreachable!("unreachable filter operation for subscriber_id"),
                        }
                    }
                    Ok(condition)
                },
            ),
        );
    }

    // Update skip — prevent subscriber_id modification
    context
        .entity_input
        .update_skips
        .push(entity_column_id.to_string());
}
