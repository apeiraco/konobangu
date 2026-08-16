use std::collections::HashMap;

use async_graphql::dynamic::ResolverContext;
use lazy_static::lazy_static;
use maplit::btreeset;
use sea_orm::{ColumnTrait, Condition, EntityTrait};
use seaography::{
  BuilderContext, EntityColumnId, FilterInfo, FilterOperation as SeaographqlFilterOperation, FilterType, GuardAction, LifecycleHooksInterface, OperationType,
};

use super::name::GraphqlColumnKey;
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

/// A type-erased function that produces a `Condition` filtering by
/// subscriber_id. Each entity registsers its own column so `entity_filter` does
/// not need to hardcode column names.
type SubscriberIdFilterFn = Box<dyn Fn(i32, bool) -> Condition + Send + Sync>;
type SubscriberOwnerSetterFn = Box<dyn Fn(&mut dyn std::any::Any, i32) -> bool + Send + Sync>;

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
  setters: HashMap<String, SubscriberOwnerSetterFn>,
}

impl SubscriberAuthHooks {
  pub fn new() -> Self {
    Self {
      guarded_entities: HashMap::new(),
      setters: HashMap::new(),
    }
  }

  /// Register an entity and its subscriber_id column for protection.
  pub fn register_entity<T, A>(&mut self, context: &BuilderContext, column: &T::Column, inject: bool)
  where
    T: EntityTrait,
    T::Model: Sync,
    A: sea_orm::ActiveModelTrait<Entity = T> + 'static,
  {
    let name = super::name::get_entity_name::<T>(context);
    let column = *column;
    self.guarded_entities.insert(
      name.clone(),
      Box::new(move |subscriber_id, shared| {
        if shared {
          Condition::any().add(column.eq(subscriber_id)).add(column.is_null())
        } else {
          Condition::all().add(column.eq(subscriber_id))
        }
      }),
    );
    if inject {
      self.setters.insert(
        name,
        Box::new(move |model, subscriber_id| {
          model
            .downcast_mut::<A>()
            .is_some_and(|model| model.try_set(column, subscriber_id.into()).is_ok())
        }),
      );
    }
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
  fn entity_guard(&self, ctx: &ResolverContext, entity: &str, _action: OperationType) -> GuardAction {
    match ctx.ctx.data::<AuthUserInfo>() {
      Ok(user) if entity == "SystemTasks" && user.auth_type != crate::models::auth::AuthType::Basic => {
        GuardAction::Block(Some("System tasks require the explicitly configured administrator".into()))
      }
      Ok(_) => GuardAction::Allow,
      Err(err) => GuardAction::Block(Some(err.message)),
    }
  }

  fn before_active_model_save(&self, ctx: &ResolverContext, entity: &str, action: OperationType, model: &mut dyn std::any::Any) -> GuardAction {
    if entity == "Cron" && matches!(action, OperationType::Create | OperationType::Update) {
      use crate::task::{SubscriberTaskTrait, SystemTaskTrait};
      if let Some(cron) = model.downcast_mut::<crate::models::cron::ActiveModel>() {
        let Ok(user) = ctx.data::<AuthUserInfo>() else {
          return GuardAction::Block(Some("A verified owner is required".into()));
        };
        if let sea_orm::ActiveValue::Set(Some(task)) = &mut cron.subscriber_task_cron {
          task.set_subscriber_id(user.subscriber_auth.subscriber_id);
          task.set_cron_id(None);
        }
        if let sea_orm::ActiveValue::Set(Some(task)) = &mut cron.system_task_cron {
          if user.auth_type != crate::models::auth::AuthType::Basic {
            return GuardAction::Block(Some("System schedules require the explicitly configured administrator".into()));
          }
          task.set_subscriber_id(Some(user.subscriber_auth.subscriber_id));
          task.set_cron_id(None);
        }
      }
    }
    if action != OperationType::Create {
      return GuardAction::Allow;
    }
    if let Some(setter) = self.setters.get(entity) {
      match ctx.data::<AuthUserInfo>() {
        Ok(user) if setter(model, user.subscriber_auth.subscriber_id) => GuardAction::Allow,
        _ => GuardAction::Block(Some("A verified owner is required".into())),
      }
    } else {
      GuardAction::Allow
    }
  }

  /// Entity filter — auto-injects subscriber_id scope on queries.
  fn entity_filter(&self, ctx: &ResolverContext, entity: &str, action: OperationType) -> Option<Condition> {
    let filter_fn = self.guarded_entities.get(entity)?;

    match ctx.ctx.data::<AuthUserInfo>() {
      Ok(user_info) => {
        let subscriber_id = user_info.subscriber_auth.subscriber_id;
        let scope = filter_fn(subscriber_id, action == OperationType::Read && matches!(entity, "Feeds" | "Cron"));
        // Shared historical subscribers do not confer administrator privileges.
        Some(if entity == "Cron" && user_info.auth_type != crate::models::auth::AuthType::Basic {
          Condition::all().add(scope).add(crate::models::cron::Column::SystemTaskCron.is_null())
        } else {
          scope
        })
      }
      Err(_) => None,
    }
  }
}

// ---------------------------------------------------------------------------
// Configuration helper — sets up filter types and input skips
// ---------------------------------------------------------------------------

/// Registers subscriber-scoped configuration for a given entity on the
/// BuilderContext.
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
  context
    .filter_types
    .overwrites
    .insert(entity_column_id.clone(), Some(FilterType::Custom(SUBSCRIBER_ID_FILTER_INFO.type_name.clone())));

  // Filter condition — subscriber_id scoping
  {
    let column = *column;
    context.filter_types.condition_functions.insert(
      entity_column_id.clone(),
      Box::new(
        move |condition: Condition, filter: &async_graphql::dynamic::ObjectAccessor<'_>| -> seaography::SeaResult<Condition> {
          // In 2.0, filter condition doesn't have ResolverContext.
          // The subscriber_id filtering is now handled by entity_filter in
          // hooks. This condition_function only validates explicit
          // filter values.
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

  // Skip subscriber_id from both insert and update inputs —
  // it is auto-injected via auth hooks / RLS.
  GraphqlColumnKey::of::<T>(context, column).push_skip_both(context);
}

pub fn install(context: &mut BuilderContext) {
  use crate::models::*;
  let mut hooks = SubscriberAuthHooks::new();
  hooks.register_entity::<subscribers::Entity, subscribers::ActiveModel>(context, &subscribers::Column::Id, false);
  hooks.register_entity::<subscriptions::Entity, subscriptions::ActiveModel>(context, &subscriptions::Column::SubscriberId, true);
  hooks.register_entity::<bangumi::Entity, bangumi::ActiveModel>(context, &bangumi::Column::SubscriberId, true);
  hooks.register_entity::<episodes::Entity, episodes::ActiveModel>(context, &episodes::Column::SubscriberId, true);
  hooks.register_entity::<subscription_bangumi::Entity, subscription_bangumi::ActiveModel>(context, &subscription_bangumi::Column::SubscriberId, true);
  hooks.register_entity::<subscription_episode::Entity, subscription_episode::ActiveModel>(context, &subscription_episode::Column::SubscriberId, true);
  hooks.register_entity::<downloaders::Entity, downloaders::ActiveModel>(context, &downloaders::Column::SubscriberId, true);
  hooks.register_entity::<downloads::Entity, downloads::ActiveModel>(context, &downloads::Column::SubscriberId, true);
  hooks.register_entity::<credential_3rd::Entity, credential_3rd::ActiveModel>(context, &credential_3rd::Column::SubscriberId, true);
  hooks.register_entity::<feeds::Entity, feeds::ActiveModel>(context, &feeds::Column::SubscriberId, true);
  hooks.register_entity::<cron::Entity, cron::ActiveModel>(context, &cron::Column::SubscriberId, true);
  hooks.register_entity::<subscriber_tasks::Entity, subscriber_tasks::ActiveModel>(context, &subscriber_tasks::Column::SubscriberId, true);
  hooks.register_entity::<system_tasks::Entity, system_tasks::ActiveModel>(context, &system_tasks::Column::SubscriberId, true);
  context.hooks = seaography::LifecycleHooks::new(hooks);
}
