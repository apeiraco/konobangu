use std::{ops::Deref, sync::Arc};

use async_graphql::dynamic::{FieldValue, Scalar, TypeRef};
use convert_case::Case;
use sea_orm::{
    ActiveModelBehavior, ColumnTrait, ConnectionTrait, EntityTrait, Iterable, QueryFilter,
    QuerySelect, QueryTrait, prelude::Expr, sea_query::Query,
};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, SeaographyError, prepare_active_model,
};
use ts_rs::TS;

use crate::{
    auth::AuthUserInfo,
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::{
                generate_entity_create_one_mutation_field,
                generate_entity_default_basic_entity_object,
                generate_entity_default_insert_input_object, generate_entity_delete_mutation_field,
                generate_entity_filtered_mutation_field, register_entity_default_readonly,
            },
            json::{convert_jsonb_output_for_entity, restrict_jsonb_filter_input_for_entity},
            name::{
                get_entity_and_column_name, get_entity_basic_type_name,
                get_entity_custom_mutation_field_name,
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
        let entity_column_key =
            get_entity_and_column_name::<subscriber_tasks::Entity>(context, &column);
        context.entity_input.insert_skips.push(entity_column_key);
    }
}

pub fn restrict_subscriber_tasks_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_and_column = get_entity_and_column_name::<T>(context, column);

    restrict_jsonb_filter_input_for_entity::<T>(context, column);
    convert_jsonb_output_for_entity::<T>(context, column, Some(Case::Camel));
    let entity_column_name = get_entity_and_column_name::<T>(context, column);

    context.types.input_type_overwrites.insert(
        entity_column_name.clone(),
        TypeRef::Named(subscriber_tasks::SubscriberTask::ident(&ts_rs::Config::default()).into()),
    );
    context.types.output_type_overwrites.insert(
        entity_column_name.clone(),
        TypeRef::Named(subscriber_tasks::SubscriberTask::ident(&ts_rs::Config::default()).into()),
    );
    context.types.input_conversions.insert(
        entity_column_name.clone(),
        Box::new(move |resolve_context, value_accessor| {
            let task: subscriber_tasks::SubscriberTaskInput = value_accessor.deserialize()?;

            let subscriber_id = resolve_context
                .data::<AuthUserInfo>()?
                .subscriber_auth
                .subscriber_id;

            let task = subscriber_tasks::SubscriberTask::from_input(task, subscriber_id);

            let json_value = serde_json::to_value(task).map_err(|err| {
                SeaographyError::TypeConversionError(
                    err.to_string(),
                    format!("Json - {entity_column_name}"),
                )
            })?;

            Ok(sea_orm::Value::Json(Some(Box::new(json_value))))
        }),
    );

    context.entity_input.update_skips.push(entity_and_column);
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
        let delete_mutation = generate_entity_delete_mutation_field::<subscriber_tasks::Entity>(
            builder_context,
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

                    let result = db.execute(delete_statement).await?;

                    Ok::<_, RecorderError>(result.rows_affected())
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
        let create_one_mutation =
            generate_entity_create_one_mutation_field::<subscriber_tasks::Entity>(
                builder_context,
                Arc::new(move |resolver_ctx, app_ctx, input_object| {
                    Box::pin(async move {
                        let active_model: Result<subscriber_tasks::ActiveModel, _> =
                            prepare_active_model(builder_context, &input_object, resolver_ctx);

                        let task_service = app_ctx.task();

                        let active_model = active_model?;

                        let db = app_ctx.db();

                        let active_model = active_model.before_save(db, true).await?;

                        let task = active_model.job.unwrap();
                        let subscriber_id = active_model.subscriber_id.unwrap();

                        if task.get_subscriber_id() != subscriber_id {
                            Err(async_graphql::Error::new(
                                "subscriber_id does not match with job.subscriber_id",
                            ))?;
                        }

                        let task_id = task_service.add_subscriber_task(task).await?.to_string();

                        let db = app_ctx.db();

                        let task = subscriber_tasks::Entity::find()
                            .filter(subscriber_tasks::Column::Id.eq(&task_id))
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                            })?;

                        Ok::<_, RecorderError>(task)
                    })
                }),
            );
        builder.mutations.push(create_one_mutation);
    }
    builder
}
