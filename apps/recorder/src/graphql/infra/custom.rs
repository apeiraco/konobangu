use std::{iter::FusedIterator, pin::Pin, sync::Arc};

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputObject, InputValue, Object, ObjectAccessor,
    ResolverContext, TypeRef,
};
use sea_orm::{ActiveModelTrait, Condition, EntityTrait, IntoActiveModel};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, EntityCreateBatchMutationBuilder,
    EntityCreateOneMutationBuilder, EntityDeleteMutationBuilder, EntityInputBuilder,
    EntityObjectBuilder, EntityUpdateMutationBuilder, GuardAction, RelationBuilder,
    RelatedEntityFilter, get_filter_conditions,
};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    graphql::infra::name::{
        get_entity_filter_input_type_name, get_entity_name,
        get_entity_renormalized_filter_field_name,
    },
};

pub type FilterMutationFn = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Condition,
        ) -> Pin<
            Box<dyn Future<Output = RecorderResult<Option<FieldValue<'a>>>> + Send + 'a>,
        > + Send
        + Sync,
>;

pub type CreateOneMutationFn<M> = Arc<
    dyn for<'a> Fn(
            &'a ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            ObjectAccessor<'a>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<M>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type CreateBatchMutationFn<M> = Arc<
    dyn for<'a> Fn(
            &'a ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Vec<ObjectAccessor<'a>>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<Vec<M>>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type UpdateMutationFn<M> = Arc<
    dyn for<'a> Fn(
            &'a ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Condition,
            ObjectAccessor<'a>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<Vec<M>>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type DeleteMutationFn = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Condition,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<u64>> + Send + 'a>>
        + Send
        + Sync,
>;

pub fn generate_entity_default_insert_input_object<T>(context: &'static BuilderContext) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_input_builder = EntityInputBuilder { context };
    entity_input_builder.insert_input_object::<T>()
}

pub fn generate_entity_default_update_input_object<T>(context: &'static BuilderContext) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_input_builder = EntityInputBuilder { context };
    entity_input_builder.update_input_object::<T>()
}

pub fn generate_entity_default_basic_entity_object<T>(context: &'static BuilderContext) -> Object
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_object_builder = EntityObjectBuilder { context };
    entity_object_builder.to_basic_object::<T>()
}

pub fn generate_entity_input_object<T>(
    context: &'static BuilderContext,
    is_insert: bool,
) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_input_builder = EntityInputBuilder { context };
    if is_insert {
        entity_input_builder.insert_input_object::<T>()
    } else {
        entity_input_builder.update_input_object::<T>()
    }
}

pub fn generate_entity_filtered_mutation_field<E, N, R>(
    builder_context: &'static BuilderContext,
    field_name: N,
    type_ref: R,
    mutation_fn: FilterMutationFn,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    N: Into<String>,
    R: Into<TypeRef>,
{
    let object_name: String = get_entity_name::<E>(builder_context);
    let hooks = &builder_context.hooks;

    Field::new(field_name, type_ref, move |resolve_context| {
        let mutation_fn = mutation_fn.clone();
        let object_name = object_name.clone();

        FieldFuture::new(async move {
            if let GuardAction::Block(reason) = hooks.entity_guard(
                &resolve_context,
                &object_name,
                seaography::OperationType::Update,
            ) {
                return Err::<Option<_>, async_graphql::Error>(async_graphql::Error::new(
                    reason.unwrap_or("Entity guard triggered.".into()),
                ));
            }

            let filters = resolve_context
                .args
                .get(get_entity_renormalized_filter_field_name());

            let filters = get_filter_conditions::<E>(builder_context, filters)?;

            let app_ctx = resolve_context.data::<Arc<dyn AppContextTrait>>()?;

            let result = mutation_fn(&resolve_context, app_ctx.clone(), filters).await?;

            Ok(result)
        })
    })
    .argument(InputValue::new(
        get_entity_renormalized_filter_field_name(),
        TypeRef::named(get_entity_filter_input_type_name::<E>(builder_context)),
    ))
}

pub fn generate_entity_default_create_one_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    let entity_create_one_mutation_builder = EntityCreateOneMutationBuilder {
        context: builder_context,
    };
    entity_create_one_mutation_builder.to_field::<E, A>()
}

pub fn generate_entity_default_create_batch_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    let entity_create_batch_mutation_builder = EntityCreateBatchMutationBuilder {
        context: builder_context,
    };
    entity_create_batch_mutation_builder.to_field::<E, A>()
}

pub fn generate_entity_default_update_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    let entity_update_mutation_builder = EntityUpdateMutationBuilder {
        context: builder_context,
    };
    entity_update_mutation_builder.to_field::<E, A>()
}

pub fn generate_entity_default_delete_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    let entity_delete_mutation_builder = EntityDeleteMutationBuilder {
        context: builder_context,
    };
    entity_delete_mutation_builder.to_field::<E, A>()
}

pub fn register_entity_default_mutations<E, A>(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    let builder_context = builder.context;
    builder
        .outputs
        .push(generate_entity_default_basic_entity_object::<E>(
            builder_context,
        ));

    builder.inputs.extend([
        generate_entity_default_insert_input_object::<E>(builder_context),
        generate_entity_default_update_input_object::<E>(builder_context),
    ]);

    builder.mutations.extend([
        generate_entity_default_create_one_mutation_field::<E, A>(builder_context),
        generate_entity_default_create_batch_mutation_field::<E, A>(builder_context),
        generate_entity_default_update_mutation_field::<E, A>(builder_context),
        generate_entity_default_delete_mutation_field::<E, A>(builder_context),
    ]);

    builder
}

pub(crate) fn register_entity_default_readonly_impl<T, RE, I>(
    mut builder: SeaographyBuilder,
    entity: T,
) -> SeaographyBuilder
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
    RE: sea_orm::Iterable<Iterator = I> + RelationBuilder,
    I: Iterator<Item = RE> + Clone + DoubleEndedIterator + ExactSizeIterator + FusedIterator,
{
    let related_entity_filter =
        RelatedEntityFilter::<T>::build::<RE>(builder.context);
    builder.register_entity::<T>(
        <RE as sea_orm::Iterable>::iter()
            .map(|rel| RelationBuilder::get_relation(&rel, builder.context))
            .collect(),
        &related_entity_filter,
    );
    builder = builder.register_entity_dataloader_one_to_one(entity, tokio::spawn);
    builder = builder.register_entity_dataloader_one_to_many(entity, tokio::spawn);
    builder =
        builder.register_related_entity_filter::<T>(related_entity_filter);
    builder
}

pub(crate) fn register_entity_default_writable_impl<T, RE, A, I>(
    mut builder: SeaographyBuilder,
    entity: T,
) -> SeaographyBuilder
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = T> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
    RE: sea_orm::Iterable<Iterator = I> + RelationBuilder,
    I: Iterator<Item = RE> + Clone + DoubleEndedIterator + ExactSizeIterator + FusedIterator,
{
    builder = register_entity_default_readonly_impl::<T, RE, I>(builder, entity);
    builder = register_entity_default_mutations::<T, A>(builder);
    builder
}

macro_rules! register_entity_default_readonly {
    ($builder:expr, $module_path:ident) => {
        $crate::graphql::infra::custom::register_entity_default_readonly_impl::<
            $module_path::Entity,
            $module_path::RelatedEntity,
            _,
        >($builder, $module_path::Entity)
    };
}

macro_rules! register_entity_default_writable {
    ($builder:expr, $module_path:ident) => {
        $crate::graphql::infra::custom::register_entity_default_writable_impl::<
            $module_path::Entity,
            $module_path::RelatedEntity,
            $module_path::ActiveModel,
            _,
        >($builder, $module_path::Entity)
    };
}

pub(crate) use register_entity_default_readonly;
pub(crate) use register_entity_default_writable;
