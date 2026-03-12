use std::{ops::Deref, sync::Arc};

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputValue, Scalar, TypeRef,
};
use convert_case::Case;
use sea_orm::{
    ActiveModelBehavior, ColumnTrait, ConnectionTrait, EntityTrait, Iterable, QueryFilter,
    QuerySelect, QueryTrait, prelude::Expr, sea_query::{ExprTrait, Query},
};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, EntityColumnId,
    EntityInputBuilder, EntityObjectBuilder, SeaographyError, prepare_active_model,
};
use ts_rs::TS;

use crate::{
    auth::AuthUserInfo,
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::{
                generate_entity_default_basic_entity_object,
                generate_entity_default_insert_input_object,
                generate_entity_filtered_mutation_field, register_entity_default_readonly,
            },
            json::{convert_jsonb_output_for_entity, restrict_jsonb_filter_input_for_entity},
            name::{
                get_entity_basic_type_name, get_entity_custom_mutation_field_name,
                get_entity_delete_mutation_field_name,
                get_entity_create_one_mutation_field_name,
                get_entity_insert_input_type_name,
                get_entity_renormalized_data_field_name,
            },
        },
    },
    migrations::defs::{ApalisJobs, ApalisSchema},
    models::subscriber_tasks,
    task::SubscriberTaskTrait,
};

fn skip_columns_for_entity_input(context: &mut BuilderContext) {
    for column in subscriber_tasks::Column::iter() {
        if matches!(
            column,
            subscriber_tasks::Column::Job | subscriber_tasks::Column::SubscriberId
        ) {
            continue;
        }
        let entity_column_id =
            EntityColumnId::of::<subscriber_tasks::Entity>(&column);
        context.entity_input.insert_skips.push(entity_column_id.to_string());
    }
}

pub fn restrict_subscriber_tasks_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_column_id = EntityColumnId::of::<T>(column);

    restrict_jsonb_filter_input_for_entity::<T>(context, column);
    convert_jsonb_output_for_entity::<T>(context, column, Some(Case::Camel));

    let entity_column_name = entity_column_id.to_string();
    let options = context
        .types
        .column_options
        .entry(entity_column_id.clone())
        .or_default();

    options.input_type = Some(TypeRef::Named(
        subscriber_tasks::SubscriberTask::ident(&ts_rs::Config::default()).into(),
    ));
    options.output_type = Some(TypeRef::Named(
        subscriber_tasks::SubscriberTask::ident(&ts_rs::Config::default()).into(),
    ));

    // Input conversion — note: in seaography 2.0, input_conversion no longer
    // receives ResolverContext. The subscriber_id injection is handled in the
    // custom create_one mutation closure instead.
    options.input_conversion = Some(Arc::new(
        move |value_accessor: &async_graphql::dynamic::ValueAccessor| -> seaography::SeaResult<sea_orm::Value> {
            let task: subscriber_tasks::SubscriberTaskInput = value_accessor.deserialize()?;

            // We cannot access subscriber_id here (no ResolverContext in 2.0).
            // Use subscriber_id=0 as placeholder; the custom mutation closure
            // will override it with the actual subscriber_id.
            let task = subscriber_tasks::SubscriberTask::from_input(task, 0);

            let json_value = serde_json::to_value(task).map_err(|err| {
                SeaographyError::TypeConversionError(
                    err.to_string(),
                    format!("Json - {entity_column_name}"),
                )
            })?;

            Ok(sea_orm::Value::Json(Some(Box::new(json_value))))
        },
    ));

    context.entity_input.update_skips.push(entity_column_id.to_string());
}

pub fn register_subscriber_tasks_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::SubscriberId,
    );
    restrict_subscriber_tasks_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::Job,
    );

    skip_columns_for_entity_input(context);
}

pub fn register_subscriber_tasks_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.schema = builder.schema.register(
        Scalar::new(subscriber_tasks::SubscriberTask::ident(&ts_rs::Config::default()))
            .description(subscriber_tasks::SubscriberTask::decl(&ts_rs::Config::default())),
    );
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskType>();
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskStatus>();

    builder = register_entity_default_readonly!(builder, subscriber_tasks);
    let builder_context = builder.context;

    {
        builder
            .outputs
            .push(generate_entity_default_basic_entity_object::<
                subscriber_tasks::Entity,
            >(builder_context));
    }
    {
        // Custom delete mutation — deletes from apalis jobs table
        let delete_field_name = get_entity_delete_mutation_field_name::<subscriber_tasks::Entity>(builder_context);
        let delete_mutation = generate_entity_filtered_mutation_field::<subscriber_tasks::Entity, _, _>(
            builder_context,
            delete_field_name,
            TypeRef::named_nn(TypeRef::INT),
            Arc::new(|_resolver_ctx, app_ctx, filters| {
                Box::pin(async move {
                    let db = app_ctx.db();

                    let select_subquery = subscriber_tasks::Entity::find()
                        .select_only()
                        .column(subscriber_tasks::Column::Id)
                        .filter(filters);

                    let delete_query = Query::delete()
                        .from_table((ApalisSchema::Schema, ApalisJobs::Table))
                        .and_where(
                            Expr::col(ApalisJobs::Id).in_subquery(select_subquery.into_query()),
                        )
                        .to_owned();

                    let db_backend = db.deref().get_database_backend();
                    let delete_statement = db_backend.build(&delete_query);

                    let result = db.execute_raw(delete_statement).await?;

                    Ok::<_, RecorderError>(Some(FieldValue::value(
                        result.rows_affected() as i64,
                    )))
                })
            }),
        );
        builder.mutations.push(delete_mutation);
    }
    {
        let entity_retry_one_mutation_name = get_entity_custom_mutation_field_name::<
            subscriber_tasks::Entity,
        >(builder_context, "RetryOne");
        let retry_one_mutation =
            generate_entity_filtered_mutation_field::<subscriber_tasks::Entity, _, _>(
                builder_context,
                entity_retry_one_mutation_name,
                TypeRef::named_nn(get_entity_basic_type_name::<subscriber_tasks::Entity>(
                    builder_context,
                )),
                Arc::new(|_resolver_ctx, app_ctx, filters| {
                    Box::pin(async move {
                        let db = app_ctx.db();

                        let job_id = subscriber_tasks::Entity::find()
                            .filter(filters)
                            .select_only()
                            .column(subscriber_tasks::Column::Id)
                            .into_tuple::<String>()
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                            })?;

                        let task = app_ctx.task();
                        task.retry_subscriber_task(job_id.clone()).await?;

                        let task_model = subscriber_tasks::Entity::find()
                            .filter(subscriber_tasks::Column::Id.eq(&job_id))
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                            })?;

                        Ok::<_, RecorderError>(Some(FieldValue::owned_any(task_model)))
                    })
                }),
            );
        builder.mutations.push(retry_one_mutation);
    }
    {
        builder
            .inputs
            .push(generate_entity_default_insert_input_object::<
                subscriber_tasks::Entity,
            >(builder_context));

        // Custom create_one mutation — creates via task service
        let create_one_field_name =
            get_entity_create_one_mutation_field_name::<subscriber_tasks::Entity>(builder_context);
        let create_one_mutation = Field::new(
            create_one_field_name,
            TypeRef::named_nn(get_entity_basic_type_name::<subscriber_tasks::Entity>(
                builder_context,
            )),
            move |resolve_context| {
                FieldFuture::new(async move {
                    let guard_result = builder_context.hooks.entity_guard(
                        &resolve_context,
                        &crate::graphql::infra::name::get_entity_name::<subscriber_tasks::Entity>(
                            builder_context,
                        ),
                        seaography::OperationType::Create,
                    );
                    if let seaography::GuardAction::Block(reason) = guard_result {
                        return Err(async_graphql::Error::new(
                            reason.unwrap_or("Entity guard triggered.".into()),
                        ));
                    }

                    let input_object = resolve_context
                        .args
                        .get(get_entity_renormalized_data_field_name())
                        .ok_or_else(|| async_graphql::Error::new("Missing data field"))?
                        .object()?;

                    let entity_input_builder = EntityInputBuilder { context: builder_context };
                    let entity_object_builder = EntityObjectBuilder { context: builder_context };
                    let active_model: Result<subscriber_tasks::ActiveModel, _> =
                        prepare_active_model(&entity_input_builder, &entity_object_builder, &input_object);

                    let app_ctx = resolve_context
                        .data::<Arc<dyn crate::app::AppContextTrait>>()?;
                    let task_service = app_ctx.task();

                    let active_model = active_model?;

                    let db = app_ctx.db();

                    let active_model = active_model.before_save(db, true).await?;

                    let mut task = active_model.job.unwrap();

                    // Inject subscriber_id from auth context (since input_conversion
                    // can no longer access ResolverContext in seaography 2.0).
                    // This restores the old behavior where the real subscriber_id was
                    // injected during input_conversion via ResolverContext.
                    let auth_subscriber_id = resolve_context
                        .data::<AuthUserInfo>()?
                        .subscriber_auth
                        .subscriber_id;
                    task.set_subscriber_id(auth_subscriber_id);

                    let task_id = task_service.add_subscriber_task(task).await?.to_string();

                    let db = app_ctx.db();

                    let task = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(&task_id))
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                        })?;

                    Ok(Some(FieldValue::owned_any(task)))
                })
            },
        )
        .argument(InputValue::new(
            get_entity_renormalized_data_field_name(),
            TypeRef::named_nn(get_entity_insert_input_type_name::<subscriber_tasks::Entity>(
                builder_context,
            )),
        ));
        builder.mutations.push(create_one_mutation);
    }
    builder
}
