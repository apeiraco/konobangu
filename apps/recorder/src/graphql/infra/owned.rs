use async_graphql::dynamic::{Field, FieldFuture, FieldValue, InputValue, ObjectAccessor, ResolverContext, TypeRef};
use sea_orm::{
  ActiveModelBehavior, ActiveModelTrait, ColumnTrait, Condition, DatabaseTransaction, DbErr, EntityTrait, IntoActiveModel, Iterable, ModelTrait,
  PaginatorTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, QuerySelect, Related, Select,
};
use seaography::{
  BuilderContext, Connection, ConnectionObjectBuilder, Edge, EntityInputBuilder, EntityObjectBuilder, EntityQueryFieldBuilder, FilterInputBuilder, GuardAction,
  HavingInputBuilder, OperationType, OrderInputBuilder, PageInfo, PaginationInfo, PaginationInput, PaginationInputBuilder, apply_order, get_filter_conditions,
  get_having_conditions,
};

use crate::database::operation::IdentityOperation;

fn guard(context: &'static BuilderContext, resolver: &ResolverContext<'_>, object: &str, operation: OperationType) -> async_graphql::Result<()> {
  if let GuardAction::Block(reason) = context.hooks.entity_guard(resolver, object, operation) {
    return Err(async_graphql::Error::new(reason.unwrap_or_else(|| "Permission denied".into())));
  }
  Ok(())
}

pub(super) fn prepare<E, A>(
  context: &'static BuilderContext,
  resolver: &ResolverContext<'_>,
  input: &ObjectAccessor<'_>,
  operation: OperationType,
) -> async_graphql::Result<A>
where
  E: EntityTrait,
  E::Model: IntoActiveModel<A>,
  A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync + 'static,
{
  let object = EntityObjectBuilder { context }.type_name::<E>();
  for (field, _) in input.iter() {
    if let GuardAction::Block(reason) = context.hooks.field_guard(resolver, &object, field, operation) {
      return Err(async_graphql::Error::new(reason.unwrap_or_else(|| "Permission denied".into())));
    }
  }
  let mut model = A::default();
  for column in E::Column::iter() {
    if E::PrimaryKey::from_column(column).is_some() && E::PrimaryKey::auto_increment() {
      continue;
    }
    let name = EntityObjectBuilder { context }.column_name::<E>(&column);
    if let Some(value) = input.get(&name) {
      let converted = if matches!(column.def().get_column_type(), sea_orm::ColumnType::Enum { .. }) && value.enum_name().is_ok() {
        sea_orm::Value::from(value.enum_name()?.to_owned())
      } else {
        seaography::TypesMapHelper { context }.async_graphql_value_to_sea_orm_value::<E>(&column, &value)?
      };
      model.try_set(column, converted)?;
    }
  }
  if let GuardAction::Block(reason) = context.hooks.before_active_model_save(resolver, &object, operation, &mut model) {
    return Err(async_graphql::Error::new(reason.unwrap_or_else(|| "Permission denied".into())));
  }
  Ok(model)
}

async fn validate_references<E, A>(db: &DatabaseTransaction, model: &A) -> Result<(), DbErr>
where
  E: EntityTrait,
  E::Relation: Iterable,
  A: ActiveModelTrait<Entity = E>,
{
  use sea_orm::{
    ConnectionTrait, RelationTrait,
    sea_query::{Expr, ExprTrait, Query},
  };
  for relation in E::Relation::iter().map(|relation| relation.def()).collect::<Vec<_>>() {
    if relation.is_owner || relation.skip_fk {
      continue;
    }
    let mut select = Query::select();
    select.expr(Expr::val(1)).from(relation.to_tbl).limit(1);
    let mut populated = true;
    for (from, to) in relation.from_col.into_iter().zip(relation.to_col) {
      let Some(column) = E::Column::iter().find(|column| sea_orm::Iden::to_string(column) == from.to_string()) else {
        populated = false;
        break;
      };
      let value = model.get(column);
      if value.is_not_set() {
        populated = false;
        break;
      }
      let value = value.unwrap();
      if value == value.as_null() {
        populated = false;
        break;
      }
      select.and_where(Expr::col(to).eq(value));
    }
    // PostgreSQL FK checks bypass RLS; explicitly require a visible referenced
    // row.
    if populated && db.query_one_raw(db.get_database_backend().build(&select)).await?.is_none() {
      return Err(DbErr::Custom("Referenced resource is unavailable to this identity".into()));
    }
  }
  Ok(())
}

fn query_args<E: EntityTrait>(context: &'static BuilderContext, field: Field, having: Option<&seaography::RelatedEntityFilter<E>>) -> Field {
  let object = EntityObjectBuilder { context }.type_name::<E>();
  let field = field
    .argument(InputValue::new(
      &context.entity_query_field.filters,
      TypeRef::named(FilterInputBuilder { context }.type_name(&object)),
    ))
    .argument(InputValue::new(
      &context.entity_query_field.order_by,
      TypeRef::named(OrderInputBuilder { context }.type_name(&object)),
    ))
    .argument(InputValue::new(
      &context.entity_query_field.pagination,
      TypeRef::named(PaginationInputBuilder { context }.type_name()),
    ));
  if having.is_some() {
    field.argument(InputValue::new(
      &context.entity_query_field.having,
      TypeRef::named(HavingInputBuilder { context }.type_name(&object)),
    ))
  } else {
    field
  }
}

/// Pagination executes only against the operation's borrowed transaction.
pub async fn paginate<E: EntityTrait>(
  context: &'static BuilderContext,
  db: &DatabaseTransaction,
  statement: Select<E>,
  mut input: PaginationInput,
) -> Result<Connection<E>, DbErr>
where
  E::Model: Sync,
{
  if input.cursor.is_none()
    && input.page.is_none()
    && input.offset.is_none()
    && let Some(limit) = context.pagination_input.default_limit.or(context.pagination_input.max_limit)
  {
    input.page = Some(seaography::PageInput { page: 0, limit });
  }
  let limit = input
    .cursor
    .as_ref()
    .map(|value| value.limit)
    .or_else(|| input.page.as_ref().map(|value| value.limit))
    .or_else(|| input.offset.as_ref().map(|value| value.limit));
  if limit.is_some_and(|limit| limit == 0 || context.pagination_input.max_limit.is_some_and(|max| limit > max)) {
    return Err(DbErr::Custom("Pagination limit must be positive and within the configured maximum".into()));
  }
  let (rows, previous, next, pagination_info) = if let Some(input) = input.cursor {
    let columns: Vec<_> = E::PrimaryKey::iter().map(|key| key.into_column()).collect();
    let cursor_for = |statement: Select<E>| -> Result<sea_orm::Cursor<sea_orm::SelectModel<E::Model>>, DbErr> {
      match columns.as_slice() {
        [first] => Ok(statement.cursor_by(*first)),
        [first, second] => Ok(statement.cursor_by((*first, *second))),
        [first, second, third] => Ok(statement.cursor_by((*first, *second, *third))),
        _ => Err(DbErr::Custom("Unsupported primary key arity for cursor pagination".into())),
      }
    };
    let mut cursor = cursor_for(statement.clone())?;
    if let Some(after) = input.cursor {
      cursor.after(seaography::decode_cursor(&after)?);
    }
    let rows = cursor.first(input.limit).all(db).await?;
    let next = if let Some(row) = rows.last() {
      !cursor_for(statement.clone())?
        .after(row.get_primary_key_value())
        .first(1)
        .all(db)
        .await?
        .is_empty()
    } else {
      false
    };
    let previous = if let Some(row) = rows.first() {
      !cursor_for(statement)?.before(row.get_primary_key_value()).first(1).all(db).await?.is_empty()
    } else {
      false
    };
    (rows, previous, next, None)
  } else if let Some(input) = input.page {
    let offset = input
      .page
      .checked_mul(input.limit)
      .ok_or_else(|| DbErr::Custom("Pagination offset overflow".into()))?;
    let paginator = statement.paginate(db, input.limit);
    let totals = paginator.num_items_and_pages().await?;
    let rows = paginator.fetch_page(input.page).await?;
    (
      rows,
      input.page != 0,
      input.page.saturating_add(1) < totals.number_of_pages,
      Some(PaginationInfo {
        pages: totals.number_of_pages,
        current: input.page,
        offset,
        total: totals.number_of_items,
      }),
    )
  } else if let Some(input) = input.offset {
    let total = statement.clone().count(db).await?;
    let rows = statement.offset(input.offset).limit(input.limit).all(db).await?;
    (
      rows,
      input.offset != 0,
      input.offset.saturating_add(input.limit) < total,
      Some(PaginationInfo {
        pages: total.div_ceil(input.limit),
        current: input.offset.div_ceil(input.limit),
        offset: input.offset,
        total,
      }),
    )
  } else {
    let rows = statement.all(db).await?;
    let total = rows.len() as u64;
    (
      rows,
      false,
      false,
      Some(PaginationInfo {
        pages: 1,
        current: 1,
        offset: 0,
        total,
      }),
    )
  };
  let edges: Vec<_> = rows
    .into_iter()
    .map(|node| Edge::<E> {
      cursor: seaography::encode_cursor(node.get_primary_key_value()),
      node,
    })
    .collect();
  let page_info = PageInfo {
    has_previous_page: previous,
    has_next_page: next,
    start_cursor: edges.first().map(|edge| edge.cursor.clone()),
    end_cursor: edges.last().map(|edge| edge.cursor.clone()),
  };
  Ok(Connection {
    edges,
    page_info,
    pagination_info,
  })
}

pub fn query<E: EntityTrait>(context: &'static BuilderContext, having: &seaography::RelatedEntityFilter<E>) -> Field
where
  E::Model: Sync,
{
  let name = EntityQueryFieldBuilder { context }.type_name::<E>();
  let object = EntityObjectBuilder { context }.type_name::<E>();
  let connection = ConnectionObjectBuilder { context }.type_name(&object);
  let field = Field::new(name, TypeRef::named_nn(connection), move |resolver| {
    let object = object.clone();
    FieldFuture::new(async move {
      guard(context, &resolver, &object, OperationType::Read)?;
      let filter = get_filter_conditions::<E>(context, resolver.args.get(&context.entity_query_field.filters))?;
      let filter = get_having_conditions::<E>(context, &resolver, filter, resolver.args.get(&context.entity_query_field.having))?;
      let mut statement = E::find().filter(filter);
      if let Some(filter) = context.hooks.entity_filter(&resolver, &object, OperationType::Read) {
        statement = statement.filter(filter);
      }
      let order = OrderInputBuilder { context }.parse_object::<E>(resolver.args.get(&context.entity_query_field.order_by))?;
      let statement = apply_order(statement, order);
      let pagination = PaginationInputBuilder { context }.parse_object(resolver.args.get(&context.entity_query_field.pagination))?;
      let connection = resolver
        .data::<IdentityOperation>()?
        .run(move |db| Box::pin(paginate::<E>(context, db, statement, pagination)))
        .await?;
      Ok(Some(FieldValue::owned_any(connection)))
    })
  });
  query_args::<E>(context, field, Some(having))
}

pub trait OwnedRelations {
  fn field(&self, context: &'static BuilderContext) -> Field;
}

pub fn relation<E, R>(context: &'static BuilderContext, name: &str) -> Field
where
  E: EntityTrait + Related<R>,
  E::Model: Sync,
  R: EntityTrait,
  R::Model: Sync,
{
  let relation = <E as Related<R>>::to();
  let owner = <E as Related<R>>::via().unwrap_or_else(|| relation.clone());
  let many = owner.is_owner && (<E as Related<R>>::via().is_some() || relation.rel_type != sea_orm::RelationType::HasOne);
  let name = seaography::EntityObjectRelationBuilder { context }.get_relation_name::<E, R>(name, relation);
  let parent_name = EntityObjectBuilder { context }.type_name::<E>();
  let object = EntityObjectBuilder { context }.type_name::<R>();
  let output = if many {
    TypeRef::named_nn(ConnectionObjectBuilder { context }.type_name(&object))
  } else {
    TypeRef::named(&object)
  };
  let field_name = name.clone();
  let field = Field::new(name, output, move |resolver| {
    let object = object.clone();
    let parent_name = parent_name.clone();
    let field_name = field_name.clone();
    FieldFuture::new(async move {
      guard(context, &resolver, &object, OperationType::Read)?;
      if let GuardAction::Block(reason) = context.hooks.field_guard(&resolver, &parent_name, &field_name, OperationType::Read) {
        return Err(async_graphql::Error::new(reason.unwrap_or_else(|| "Permission denied".into())));
      }
      let parent = resolver.parent_value.try_downcast_ref::<E::Model>()?;
      let filter = get_filter_conditions::<R>(context, resolver.args.get(&context.entity_query_field.filters))?;
      let mut statement = parent.find_related(R::default()).filter(filter);
      if let Some(filter) = context.hooks.entity_filter(&resolver, &object, OperationType::Read) {
        statement = statement.filter(filter);
      }
      let order = OrderInputBuilder { context }.parse_object::<R>(resolver.args.get(&context.entity_query_field.order_by))?;
      let statement = apply_order(statement, order);
      let operation = resolver.data::<IdentityOperation>()?;
      if many {
        let pagination = PaginationInputBuilder { context }.parse_object(resolver.args.get(&context.entity_query_field.pagination))?;
        let connection = operation.run(move |db| Box::pin(paginate::<R>(context, db, statement, pagination))).await?;
        Ok(Some(FieldValue::owned_any(connection)))
      } else {
        let row = resolver.data::<super::loader::RequestLoaders>()?.one(operation, statement).await?;
        Ok(row.map(FieldValue::owned_any))
      }
    })
  });
  query_args::<R>(context, field, None)
}

pub fn create<E, A>(context: &'static BuilderContext, batch: bool) -> Field
where
  E: EntityTrait,
  E::Relation: Iterable,
  E::Model: Sync + IntoActiveModel<A>,
  A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync + 'static,
{
  let object = EntityObjectBuilder { context }.type_name::<E>();
  let input = EntityInputBuilder { context }.insert_type_name::<E>();
  let basic = EntityObjectBuilder { context }.basic_type_name::<E>();
  let field_name = if batch {
    seaography::EntityCreateBatchMutationBuilder { context }.type_name::<E>()
  } else {
    seaography::EntityCreateOneMutationBuilder { context }.type_name::<E>()
  };
  let data_name = if batch {
    &context.entity_create_batch_mutation.data_field
  } else {
    &context.entity_create_one_mutation.data_field
  };
  Field::new(
    field_name,
    if batch { TypeRef::named_nn_list_nn(basic) } else { TypeRef::named_nn(basic) },
    move |resolver| {
      let object = object.clone();
      FieldFuture::new(async move {
        guard(context, &resolver, &object, OperationType::Create)?;
        let value = resolver.args.try_get(data_name)?;
        let models = if batch {
          value
            .list()?
            .iter()
            .map(|value| prepare::<E, A>(context, &resolver, &value.object()?, OperationType::Create))
            .collect::<async_graphql::Result<Vec<_>>>()?
        } else {
          vec![prepare::<E, A>(context, &resolver, &value.object()?, OperationType::Create)?]
        };
        let mut rows = resolver
          .data::<IdentityOperation>()?
          .run(move |db| {
            Box::pin(async move {
              let mut rows = Vec::with_capacity(models.len());
              for model in models {
                validate_references::<E, A>(db, &model).await?;
                rows.push(model.insert(db).await?);
              }
              Ok::<_, DbErr>(rows)
            })
          })
          .await?;
        if batch {
          Ok(Some(FieldValue::list(rows.into_iter().map(FieldValue::owned_any))))
        } else {
          Ok(rows.pop().map(FieldValue::owned_any))
        }
      })
    },
  )
  .argument(InputValue::new(
    data_name,
    if batch { TypeRef::named_nn_list_nn(input) } else { TypeRef::named_nn(input) },
  ))
}

pub fn update<E, A>(context: &'static BuilderContext) -> Field
where
  E: EntityTrait,
  E::Relation: Iterable,
  E::Model: Sync + IntoActiveModel<A>,
  A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + Sync + 'static,
{
  let object = EntityObjectBuilder { context }.type_name::<E>();
  let filter_type = FilterInputBuilder { context }.type_name(&object);
  let field = Field::new(
    seaography::EntityUpdateMutationBuilder { context }.type_name::<E>(),
    TypeRef::named_nn_list_nn(EntityObjectBuilder { context }.basic_type_name::<E>()),
    move |resolver| {
      let object = object.clone();
      FieldFuture::new(async move {
        guard(context, &resolver, &object, OperationType::Update)?;
        let input = resolver.args.try_get(&context.entity_update_mutation.data_field)?.object()?;
        let model = prepare::<E, A>(context, &resolver, &input, OperationType::Update)?;
        let filter = get_filter_conditions::<E>(context, resolver.args.get(&context.entity_update_mutation.filter_field))?;
        let mut statement = E::find().filter(filter);
        if let Some(filter) = context.hooks.entity_filter(&resolver, &object, OperationType::Update) {
          statement = statement.filter(filter);
        }
        let rows = resolver
          .data::<IdentityOperation>()?
          .run(move |db| {
            Box::pin(async move {
              let existing = statement.all(db).await?;
              let mut rows = Vec::with_capacity(existing.len());
              for row in existing {
                let mut active = row.into_active_model();
                for column in E::Column::iter() {
                  if let sea_orm::ActiveValue::Set(value) = model.get(column) {
                    active.try_set(column, value)?;
                  }
                }
                validate_references::<E, A>(db, &active).await?;
                rows.push(active.update(db).await?);
              }
              Ok::<_, DbErr>(rows)
            })
          })
          .await?;
        Ok(Some(FieldValue::list(rows.into_iter().map(FieldValue::owned_any))))
      })
    },
  );
  field
    .argument(InputValue::new(
      &context.entity_update_mutation.data_field,
      TypeRef::named_nn(EntityInputBuilder { context }.update_type_name::<E>()),
    ))
    .argument(InputValue::new(&context.entity_update_mutation.filter_field, TypeRef::named(filter_type)))
}

pub fn delete<E: EntityTrait>(context: &'static BuilderContext) -> Field
where
  E::Model: Sync,
{
  delete_using::<E>(context, |db, filter| {
    Box::pin(async move { Ok(E::delete_many().filter(filter).exec(db).await?.rows_affected) })
  })
}

pub type DeleteCommand = for<'a> fn(&'a DatabaseTransaction, Condition) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, DbErr>> + Send + 'a>>;

pub fn delete_using<E: EntityTrait>(context: &'static BuilderContext, command: DeleteCommand) -> Field
where
  E::Model: Sync,
{
  let object = EntityObjectBuilder { context }.type_name::<E>();
  let filter_type = FilterInputBuilder { context }.type_name(&object);
  Field::new(
    seaography::EntityDeleteMutationBuilder { context }.type_name::<E>(),
    TypeRef::named_nn(TypeRef::INT),
    move |resolver| {
      let object = object.clone();
      FieldFuture::new(async move {
        guard(context, &resolver, &object, OperationType::Delete)?;
        let filter = get_filter_conditions::<E>(context, resolver.args.get(&context.entity_delete_mutation.filter_field))?;
        let mut filter = filter;
        if let Some(scope) = context.hooks.entity_filter(&resolver, &object, OperationType::Delete) {
          filter = filter.add(scope);
        }
        let result = resolver.data::<IdentityOperation>()?.run(move |db| command(db, filter)).await?;
        Ok(Some(FieldValue::value(result)))
      })
    },
  )
  .argument(InputValue::new(&context.entity_delete_mutation.filter_field, TypeRef::named(filter_type)))
}
