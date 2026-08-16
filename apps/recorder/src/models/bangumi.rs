use async_graphql::SimpleObject;
use async_trait::async_trait;
use sea_orm::{
  ActiveValue, Condition, FromJsonQueryResult, IntoSimpleExpr, JoinType, QuerySelect,
  entity::prelude::*,
  sea_query::{ExprTrait, OnConflict},
};
use serde::{Deserialize, Serialize};

use super::subscription_bangumi;
use crate::{
  app::AppContextTrait,
  errors::RecorderResult,
  extract::{
    mikan::{MikanBangumiHash, MikanBangumiMeta, build_mikan_bangumi_subscription_rss_url, scrape_mikan_poster_meta_from_image_url},
    origin::{BangumiComps, OriginCompTrait},
  },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, SimpleObject)]
pub struct BangumiFilter {
  pub name: Option<Vec<String>>,
  pub group: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "bangumi_type")]
pub enum BangumiType {
  #[sea_orm(string_value = "mikan")]
  Mikan,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "bangumi")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  pub mikan_bangumi_id: Option<String>,
  pub bangumi_type: BangumiType,
  pub subscriber_id: i32,
  pub display_name: String,
  pub origin_name: String,
  pub season: i32,
  pub season_raw: Option<String>,
  pub fansub: Option<String>,
  pub mikan_fansub_id: Option<String>,
  pub filter: Option<BangumiFilter>,
  pub rss_link: Option<String>,
  pub poster_link: Option<String>,
  pub origin_poster_link: Option<String>,
  pub homepage: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::subscriptions::Entity")]
  Subscription,
  #[sea_orm(
    belongs_to = "super::subscribers::Entity",
    from = "Column::SubscriberId",
    to = "super::subscribers::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Subscriber,
  #[sea_orm(has_many = "super::episodes::Entity")]
  Episode,
  #[sea_orm(has_many = "super::subscription_bangumi::Entity")]
  SubscriptionBangumi,
}

impl Related<super::episodes::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Episode.def()
  }
}

impl Related<super::subscription_bangumi::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SubscriptionBangumi.def()
  }
}

impl Related<super::subscriptions::Entity> for Entity {
  fn to() -> RelationDef {
    super::subscription_bangumi::Relation::Subscription.def()
  }

  fn via() -> Option<RelationDef> {
    Some(super::subscription_bangumi::Relation::Bangumi.def().rev())
  }
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscriptions::Entity")]
  Subscription,
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::episodes::Entity")]
  Episode,
  #[sea_orm(entity = "super::subscription_bangumi::Entity")]
  SubscriptionBangumi,
}

impl ActiveModel {
  #[tracing::instrument(err, skip_all, fields(mikan_bangumi_id = %meta.mikan_bangumi_id, mikan_fansub_id = %meta.mikan_fansub_id, subscriber_id = %subscriber_id))]
  pub async fn from_mikan_bangumi_meta(ctx: &dyn AppContextTrait, meta: MikanBangumiMeta, subscriber_id: i32, _subscription_id: i32) -> RecorderResult<Self> {
    let mikan_client = ctx.mikan();
    let mikan_base_url = mikan_client.base_url();
    let season_comp = BangumiComps::parse_comp(&meta.bangumi_title).ok().map(|(_, s)| s).and_then(|s| s.season);
    let season_index = season_comp.as_ref().map(|s| s.num).unwrap_or(1);
    let season_raw = season_comp.map(|s| s.source.to_string());

    let rss_url = build_mikan_bangumi_subscription_rss_url(mikan_base_url.clone(), &meta.mikan_bangumi_id, Some(&meta.mikan_fansub_id));

    let poster_link = if let Some(origin_poster_src) = meta.origin_poster_src.clone() {
      let poster_meta = scrape_mikan_poster_meta_from_image_url(ctx, origin_poster_src).await?;
      poster_meta.poster_src
    } else {
      None
    };

    Ok(Self {
      mikan_bangumi_id: ActiveValue::Set(Some(meta.mikan_bangumi_id)),
      mikan_fansub_id: ActiveValue::Set(Some(meta.mikan_fansub_id)),
      subscriber_id: ActiveValue::Set(subscriber_id),
      display_name: ActiveValue::Set(meta.bangumi_title.clone()),
      origin_name: ActiveValue::Set(meta.bangumi_title),
      season: ActiveValue::Set(season_index),
      season_raw: ActiveValue::Set(season_raw),
      fansub: ActiveValue::Set(Some(meta.fansub)),
      poster_link: ActiveValue::Set(poster_link),
      origin_poster_link: ActiveValue::Set(meta.origin_poster_src.map(|src| src.to_string())),
      homepage: ActiveValue::Set(Some(meta.homepage.to_string())),
      rss_link: ActiveValue::Set(Some(rss_url.to_string())),
      bangumi_type: ActiveValue::Set(BangumiType::Mikan),
      ..Default::default()
    })
  }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
  pub fn get_or_insert_from_mikan<'a, F, Fut>(
    ctx: &'a dyn AppContextTrait,
    hash: MikanBangumiHash,
    subscriber_id: i32,
    subscription_id: i32,
    create_bangumi_fn: F,
  ) -> futures::future::BoxFuture<'a, RecorderResult<Self>>
  where
    F: FnOnce() -> Fut + Send + 'a,
    Fut: std::future::Future<Output = RecorderResult<ActiveModel>> + Send + 'a,
  {
    // Bound the caller Send proof at the task stage rather than nesting ORM
    // futures.
    Box::pin(async move {
      let read = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
      crate::database::operation::require_visible::<super::subscriptions::Entity>(&read, subscription_id).await?;
      let existing = Entity::find()
        .filter(Column::MikanBangumiId.eq(&hash.mikan_bangumi_id))
        .filter(Column::MikanFansubId.eq(&hash.mikan_fansub_id))
        .filter(Column::SubscriberId.eq(subscriber_id))
        .one(&read)
        .await?;
      read.rollback().await?;
      // Poster fetching/encoding runs before the database stage starts.
      let creation = if existing.is_none() { Some(create_bangumi_fn().await?) } else { None };
      let write = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
      crate::database::operation::require_visible::<super::subscriptions::Entity>(&write, subscription_id).await?;
      let model = if let Some(model) = existing {
        model
      } else {
        let mut creation = creation.expect("missing row requires prepared creation");
        creation.subscriber_id = ActiveValue::Set(subscriber_id);
        Entity::insert(creation)
          .on_conflict(
            OnConflict::columns([Column::MikanBangumiId, Column::MikanFansubId, Column::SubscriberId])
              .update_columns([
                Column::OriginName,
                Column::Fansub,
                Column::PosterLink,
                Column::OriginPosterLink,
                Column::Season,
                Column::SeasonRaw,
                Column::RssLink,
                Column::Homepage,
              ])
              .to_owned(),
          )
          .exec_with_returning(&write)
          .await?
      };
      subscription_bangumi::Entity::insert(subscription_bangumi::ActiveModel {
        subscription_id: ActiveValue::Set(subscription_id),
        bangumi_id: ActiveValue::Set(model.id),
        subscriber_id: ActiveValue::Set(subscriber_id),
        ..Default::default()
      })
      .on_conflict(
        OnConflict::columns([subscription_bangumi::Column::SubscriptionId, subscription_bangumi::Column::BangumiId])
          .do_nothing()
          .to_owned(),
      )
      .exec_without_returning(&write)
      .await?;
      crate::database::operation::commit_task_transaction(write).await?;
      Ok(model)
    })
  }

  pub async fn get_existed_mikan_bangumi_list(
    ctx: &dyn AppContextTrait,
    hashes: impl Iterator<Item = MikanBangumiHash>,
    subscriber_id: i32,
    _subscription_id: i32,
  ) -> RecorderResult<impl Iterator<Item = (i32, MikanBangumiHash)>> {
    let transaction = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
    let rows = Entity::find()
      .select_only()
      .column(Column::Id)
      .column(Column::MikanBangumiId)
      .column(Column::MikanFansubId)
      .filter(
        Expr::tuple([
          Column::MikanBangumiId.into_simple_expr(),
          Column::MikanFansubId.into_simple_expr(),
          Column::SubscriberId.into_simple_expr(),
        ])
        .in_tuples(hashes.map(|hash| (hash.mikan_bangumi_id.clone(), hash.mikan_fansub_id.clone(), subscriber_id))),
      )
      .into_tuple::<(i32, String, String)>()
      .all(&transaction)
      .await?;
    transaction.rollback().await?;
    Ok(rows.into_iter().map(|(bangumi_id, mikan_bangumi_id, mikan_fansub_id)| {
      (
        bangumi_id,
        MikanBangumiHash {
          mikan_bangumi_id,
          mikan_fansub_id,
        },
      )
    }))
  }

  pub async fn get_subsribed_bangumi_list_from_subscription(ctx: &dyn AppContextTrait, subscription_id: i32, subscriber_id: i32) -> RecorderResult<Vec<Self>> {
    let transaction = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
    let db = &transaction;
    let bangumi_list = Entity::find()
      .filter(Condition::all().add(subscription_bangumi::Column::SubscriptionId.eq(subscription_id)))
      .join_rev(JoinType::InnerJoin, subscription_bangumi::Relation::Bangumi.def())
      .all(db)
      .await?;

    transaction.rollback().await?;
    Ok(bangumi_list)
  }
}
