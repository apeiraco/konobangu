use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  str::FromStr,
  sync::Arc,
};

use async_graphql::{InputObject, SimpleObject};
use async_stream::try_stream;
use fetch::fetch_html;
use futures::{Stream, TryStreamExt, pin_mut, try_join};
use maplit::hashmap;
use sea_orm::{ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};
use url::Url;

use super::scrape_mikan_bangumi_meta_stream_from_season_flow_url;
use crate::{
  app::AppContextTrait,
  errors::{RecorderError, RecorderResult},
  extract::{
    bittorrent::EpisodeEnclosureMeta,
    mikan::{
      MikanBangumiHash, MikanBangumiMeta, MikanEpisodeHash, MikanEpisodeMeta, MikanRssItemMeta, MikanRssRoot, MikanSeasonFlowUrlMeta, MikanSeasonStr,
      MikanSubscriberSubscriptionUrlMeta, build_mikan_bangumi_subscription_rss_url, build_mikan_season_flow_url, build_mikan_subscriber_subscription_rss_url,
      scrape_mikan_episode_meta_from_episode_homepage_url,
    },
  },
  models::{
    bangumi, episodes, subscription_bangumi, subscription_episode,
    subscriptions::{self, SubscriptionTrait},
  },
};

#[tracing::instrument(err, skip(ctx, rss_item_list))]
async fn sync_mikan_feeds_from_rss_item_list(
  ctx: &dyn AppContextTrait,
  rss_item_list: Vec<MikanRssItemMeta>,
  subscriber_id: i32,
  subscription_id: i32,
) -> RecorderResult<()> {
  let mikan_base_url = ctx.mikan().base_url().clone();
  let (new_episode_meta_list, existed_episode_hash2id_map) = {
    let existed_episode_hash2id_map = episodes::Model::get_existed_mikan_episode_list(
      ctx,
      rss_item_list.iter().map(|s| MikanEpisodeHash {
        mikan_episode_id: s.mikan_episode_id.clone(),
      }),
      subscriber_id,
      subscription_id,
    )
    .await?
    .map(|(episode_id, hash, bangumi_id)| (hash.mikan_episode_id, (episode_id, bangumi_id)))
    .collect::<HashMap<_, _>>();

    let mut new_episode_meta_list: Vec<(MikanEpisodeMeta, EpisodeEnclosureMeta)> = vec![];

    let mikan_client = ctx.mikan();
    for to_insert_rss_item in rss_item_list
      .into_iter()
      .filter(|rss_item| !existed_episode_hash2id_map.contains_key(&rss_item.mikan_episode_id))
    {
      let episode_meta =
        scrape_mikan_episode_meta_from_episode_homepage_url(mikan_client, to_insert_rss_item.build_homepage_url(mikan_base_url.clone())).await?;
      let episode_enclosure_meta = EpisodeEnclosureMeta::from(to_insert_rss_item);
      new_episode_meta_list.push((episode_meta, episode_enclosure_meta));
    }

    (new_episode_meta_list, existed_episode_hash2id_map)
  };

  // subscribe existed but not subscribed episode and bangumi
  let (existed_episode_id_list, existed_episode_bangumi_id_set): (Vec<i32>, HashSet<i32>) = existed_episode_hash2id_map.into_values().unzip();

  try_join!(
    subscription_episode::Model::add_episodes_for_subscription(ctx, existed_episode_id_list.into_iter(), subscriber_id, subscription_id,),
    subscription_bangumi::Model::add_bangumis_for_subscription(ctx, existed_episode_bangumi_id_set.into_iter(), subscriber_id, subscription_id,),
  )?;

  let new_episode_meta_list_group_by_bangumi_hash: HashMap<MikanBangumiHash, Vec<(MikanEpisodeMeta, EpisodeEnclosureMeta)>> = {
    let mut m = hashmap! {};
    for (episode_meta, episode_enclosure_meta) in new_episode_meta_list {
      let bangumi_hash = episode_meta.bangumi_hash();

      m.entry(bangumi_hash).or_insert_with(Vec::new).push((episode_meta, episode_enclosure_meta));
    }
    m
  };

  for (group_bangumi_hash, group_episode_meta_list) in new_episode_meta_list_group_by_bangumi_hash {
    let (first_episode_meta, _) = group_episode_meta_list.first().unwrap();
    let group_bangumi_model = bangumi::Model::get_or_insert_from_mikan(ctx, group_bangumi_hash, subscriber_id, subscription_id, async || {
      let bangumi_meta: MikanBangumiMeta = first_episode_meta.clone().into();
      let bangumi_am = bangumi::ActiveModel::from_mikan_bangumi_meta(ctx, bangumi_meta, subscriber_id, subscription_id).await?;
      Ok(bangumi_am)
    })
    .await?;
    let group_episode_creation_list = group_episode_meta_list
      .into_iter()
      .map(|(episode_meta, episode_enclosure_meta)| (&group_bangumi_model, episode_meta, episode_enclosure_meta));

    episodes::Model::add_mikan_episodes_for_subscription(ctx, group_episode_creation_list.into_iter(), subscriber_id, subscription_id).await?;
  }
  Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSubscriberSubscription {
  pub subscription_id: i32,
  pub mikan_subscription_token: String,
  pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriptionTrait for MikanSubscriberSubscription {
  fn get_subscriber_id(&self) -> i32 {
    self.subscriber_id
  }

  fn get_subscription_id(&self) -> i32 {
    self.subscription_id
  }

  async fn sync_feeds_incremental(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    let rss_item_list = self.get_rss_item_list_from_source_url(ctx.as_ref()).await?;

    sync_mikan_feeds_from_rss_item_list(ctx.as_ref(), rss_item_list, self.get_subscriber_id(), self.get_subscription_id()).await?;

    Ok(())
  }

  async fn sync_feeds_full(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    self.sync_feeds_incremental(ctx.clone()).await?;

    let rss_item_list = self.get_rss_item_list_from_subsribed_url_rss_link(ctx.as_ref()).await?;

    sync_mikan_feeds_from_rss_item_list(ctx.as_ref(), rss_item_list, self.get_subscriber_id(), self.get_subscription_id()).await
  }

  async fn sync_sources(&self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    Ok(())
  }

  fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
    let source_url = Url::parse(&model.source_url)?;

    let meta = MikanSubscriberSubscriptionUrlMeta::from_rss_url(&source_url).with_whatever_context::<_, String, RecorderError>(|| {
      format!(
        "MikanSubscriberSubscription should extract mikan_subscription_token from source_url = {}, subscription_id = {}",
        source_url, model.id
      )
    })?;

    Ok(Self {
      subscription_id: model.id,
      mikan_subscription_token: meta.mikan_subscription_token,
      subscriber_id: model.subscriber_id,
    })
  }
}

impl MikanSubscriberSubscription {
  #[tracing::instrument(err, skip(ctx))]
  async fn get_rss_item_list_from_source_url(&self, ctx: &dyn AppContextTrait) -> RecorderResult<Vec<MikanRssItemMeta>> {
    let mikan_base_url = ctx.mikan().base_url().clone();
    let rss_url = build_mikan_subscriber_subscription_rss_url(mikan_base_url.clone(), &self.mikan_subscription_token);
    let html = fetch_html(ctx.mikan(), rss_url).await?;

    let channel = MikanRssRoot::from_str(&html)?.channel;

    let mut result = vec![];
    for (idx, item) in channel.items.into_iter().enumerate() {
      let item = MikanRssItemMeta::try_from(item).with_whatever_context::<_, String, RecorderError>(|_| format!("failed to extract rss item at idx {idx}"))?;
      result.push(item);
    }
    Ok(result)
  }

  #[tracing::instrument(err, skip(ctx))]
  async fn get_rss_item_list_from_subsribed_url_rss_link(&self, ctx: &dyn AppContextTrait) -> RecorderResult<Vec<MikanRssItemMeta>> {
    let subscribed_bangumi_list = bangumi::Model::get_subsribed_bangumi_list_from_subscription(ctx, self.subscription_id, self.subscriber_id).await?;

    let mut rss_item_list = vec![];
    for subscribed_bangumi in subscribed_bangumi_list {
      let rss_url = subscribed_bangumi.rss_link.with_whatever_context::<_, String, RecorderError>(|| {
        format!(
          "rss link is required, subscription_id = {:?}, bangumi_name = {}",
          self.subscription_id, subscribed_bangumi.display_name
        )
      })?;
      let html = fetch_html(ctx.mikan(), rss_url).await?;

      let channel = MikanRssRoot::from_str(&html)?.channel;

      for (idx, item) in channel.items.into_iter().enumerate() {
        let item =
          MikanRssItemMeta::try_from(item).with_whatever_context::<_, String, RecorderError>(|_| format!("failed to extract rss item at idx {idx}"))?;
        rss_item_list.push(item);
      }
    }
    Ok(rss_item_list)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSeasonSubscription {
  pub subscription_id: i32,
  pub year: i32,
  pub season_str: MikanSeasonStr,
  pub credential_id: i32,
  pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriptionTrait for MikanSeasonSubscription {
  fn get_subscriber_id(&self) -> i32 {
    self.subscriber_id
  }

  fn get_subscription_id(&self) -> i32 {
    self.subscription_id
  }

  async fn sync_feeds_incremental(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    let rss_item_stream = self.get_rss_item_stream_from_subsribed_url_rss_link(ctx.as_ref());

    pin_mut!(rss_item_stream);

    while let Some(rss_item_chunk_list) = rss_item_stream.try_next().await? {
      sync_mikan_feeds_from_rss_item_list(ctx.as_ref(), rss_item_chunk_list, self.get_subscriber_id(), self.get_subscription_id()).await?;
    }

    Ok(())
  }

  async fn sync_feeds_full(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    self.sync_sources(ctx.clone()).await?;
    self.sync_feeds_incremental(ctx).await
  }

  async fn sync_sources(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    let bangumi_meta_list = self.get_bangumi_meta_stream_from_source_url(ctx.clone());

    pin_mut!(bangumi_meta_list);

    while let Some(bangumi_meta) = bangumi_meta_list.try_next().await? {
      let bangumi_hash = bangumi_meta.bangumi_hash();
      bangumi::Model::get_or_insert_from_mikan(ctx.as_ref(), bangumi_hash, self.get_subscriber_id(), self.get_subscription_id(), async || {
        let bangumi_am =
          bangumi::ActiveModel::from_mikan_bangumi_meta(ctx.as_ref(), bangumi_meta, self.get_subscriber_id(), self.get_subscription_id()).await?;
        Ok(bangumi_am)
      })
      .await?;
    }

    Ok(())
  }

  fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
    let source_url = Url::parse(&model.source_url)?;

    let source_url_meta = MikanSeasonFlowUrlMeta::from_url(&source_url).with_whatever_context::<_, String, RecorderError>(|| {
      format!(
        "season_str and year is required when extracting MikanSeasonSubscription from source_url, source_url = {}, subscription_id = {}",
        source_url, model.id
      )
    })?;

    let credential_id = model.credential_id.with_whatever_context::<_, String, RecorderError>(|| {
      format!(
        "credential_id is required when extracting MikanSeasonSubscription, subscription_id = {}",
        model.id
      )
    })?;

    Ok(Self {
      subscription_id: model.id,
      year: source_url_meta.year,
      season_str: source_url_meta.season_str,
      credential_id,
      subscriber_id: model.subscriber_id,
    })
  }
}

impl MikanSeasonSubscription {
  pub fn get_bangumi_meta_stream_from_source_url(&self, ctx: Arc<dyn AppContextTrait>) -> impl Stream<Item = RecorderResult<MikanBangumiMeta>> {
    let credential_id = self.credential_id;
    let year = self.year;
    let season_str = self.season_str;

    let mikan_base_url = ctx.mikan().base_url().clone();
    let mikan_season_flow_url = build_mikan_season_flow_url(mikan_base_url, year, season_str);

    scrape_mikan_bangumi_meta_stream_from_season_flow_url(ctx, mikan_season_flow_url, credential_id, self.get_subscriber_id())
  }

  fn get_rss_item_stream_from_subsribed_url_rss_link(&self, ctx: &dyn AppContextTrait) -> impl Stream<Item = RecorderResult<Vec<MikanRssItemMeta>>> {
    try_stream! {

        let transaction = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), self.subscriber_id).await?;
        let db = &transaction;

        let subscribed_bangumi_list = bangumi::Entity::find()
            .filter(
                Condition::all()
                    .add(subscription_bangumi::Column::SubscriptionId.eq(self.subscription_id)),
            )
            .join_rev(
                JoinType::InnerJoin,
                subscription_bangumi::Relation::Bangumi.def(),
            )
            .all(db)
            .await?;


        transaction.rollback().await?;
        for subscribed_bangumi in subscribed_bangumi_list {
            let rss_url = subscribed_bangumi
                .rss_link
                .with_whatever_context::<_, String, RecorderError>(|| {
                    format!(
                        "rss_link is required, subscription_id = {}, bangumi_name = {}",
                        self.subscription_id, subscribed_bangumi.display_name
                    )
                })?;
            let html = fetch_html(ctx.mikan(), rss_url).await?;

            let channel = MikanRssRoot::from_str(&html)?.channel;

            let mut rss_item_list = vec![];

            for (idx, item) in channel.items.into_iter().enumerate() {
                let item = MikanRssItemMeta::try_from(item)
                    .with_whatever_context::<_, String, RecorderError>(|_| {
                        format!("failed to extract rss item at idx {idx}")
                    })?;
                rss_item_list.push(item);
            }

            yield rss_item_list;
        }
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, InputObject, SimpleObject)]
pub struct MikanBangumiSubscription {
  pub subscription_id: i32,
  pub mikan_bangumi_id: String,
  pub mikan_fansub_id: String,
  pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriptionTrait for MikanBangumiSubscription {
  fn get_subscriber_id(&self) -> i32 {
    self.subscriber_id
  }

  fn get_subscription_id(&self) -> i32 {
    self.subscription_id
  }

  async fn sync_feeds_incremental(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    let rss_item_list = self.get_rss_item_list_from_source_url(ctx.as_ref()).await?;

    sync_mikan_feeds_from_rss_item_list(ctx.as_ref(), rss_item_list, self.get_subscriber_id(), self.get_subscription_id()).await?;

    Ok(())
  }

  async fn sync_feeds_full(&self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    self.sync_feeds_incremental(_ctx).await
  }

  async fn sync_sources(&self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    Ok(())
  }

  fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
    let source_url = Url::parse(&model.source_url)?;

    let meta = MikanBangumiHash::from_rss_url(&source_url).with_whatever_context::<_, String, RecorderError>(|| {
      format!(
        "bangumi_id and fansub_id is required when extracting MikanBangumiSubscription, source_url = {}, subscription_id = {}",
        source_url, model.id
      )
    })?;

    Ok(Self {
      subscription_id: model.id,
      mikan_bangumi_id: meta.mikan_bangumi_id,
      mikan_fansub_id: meta.mikan_fansub_id,
      subscriber_id: model.subscriber_id,
    })
  }
}

impl MikanBangumiSubscription {
  #[tracing::instrument(err, skip(ctx))]
  async fn get_rss_item_list_from_source_url(&self, ctx: &dyn AppContextTrait) -> RecorderResult<Vec<MikanRssItemMeta>> {
    let mikan_base_url = ctx.mikan().base_url().clone();
    let rss_url = build_mikan_bangumi_subscription_rss_url(mikan_base_url.clone(), &self.mikan_bangumi_id, Some(&self.mikan_fansub_id));
    let html = fetch_html(ctx.mikan(), rss_url).await?;

    let channel = MikanRssRoot::from_str(&html)?.channel;

    let mut result = vec![];
    for (idx, item) in channel.items.into_iter().enumerate() {
      let item = MikanRssItemMeta::try_from(item).with_whatever_context::<_, String, RecorderError>(|_| format!("failed to extract rss item at idx {idx}"))?;
      result.push(item);
    }
    Ok(result)
  }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {

  use rstest::{fixture, rstest};
  use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
  use tracing::Level;

  use crate::{
    errors::RecorderResult,
    extract::mikan::{MikanBangumiHash, MikanSeasonFlowUrlMeta, MikanSeasonStr, MikanSubscriberSubscriptionUrlMeta},
    models::{
      bangumi, episodes,
      subscriptions::{self, SubscriptionTrait},
    },
    test_utils::{app::TestingPreset, mikan::build_testing_mikan_credential_form, tracing::try_init_testing_tracing},
  };

  #[fixture]
  fn before_each() {
    try_init_testing_tracing(Level::DEBUG);
  }

  #[rstest]
  #[tokio::test]
  async fn test_mikan_season_subscription_sync_feeds(before_each: ()) -> RecorderResult<()> {
    let mut preset = TestingPreset::default().await?;
    let app_ctx = preset.app_ctx.clone();

    let mikan_server = &mut preset.mikan_server;

    let _resources_mock = mikan_server.mock_resources_with_doppel();

    let _login_mock = mikan_server.mock_get_login_page();

    let mikan_client = app_ctx.mikan();

    let subscriber_id = 1;

    let credential = mikan_client
      .submit_credential_form(app_ctx.as_ref(), subscriber_id, build_testing_mikan_credential_form())
      .await?;

    let subscription_am = subscriptions::ActiveModel {
      display_name: ActiveValue::Set("test subscription".to_string()),
      subscriber_id: ActiveValue::Set(subscriber_id),
      category: ActiveValue::Set(subscriptions::SubscriptionCategory::MikanSeason),
      source_url: ActiveValue::Set(
        MikanSeasonFlowUrlMeta {
          year: 2025,
          season_str: MikanSeasonStr::Spring,
        }
        .build_season_flow_url(mikan_server.base_url().clone())
        .to_string(),
      ),
      enabled: ActiveValue::Set(true),
      credential_id: ActiveValue::Set(Some(credential.id)),
      ..Default::default()
    };

    let subscription_model = subscription_am.insert(app_ctx.db()).await?;

    let subscription = subscriptions::Subscription::try_from_model(&subscription_model)?;

    {
      subscription.sync_feeds_incremental(app_ctx.clone()).await?;
      let bangumi_list = bangumi::Entity::find().all(app_ctx.db()).await?;

      assert!(bangumi_list.is_empty());
    }

    {
      subscription.sync_feeds_full(app_ctx.clone()).await?;
      let bangumi_list = bangumi::Entity::find().all(app_ctx.db()).await?;

      assert!(!bangumi_list.is_empty());
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_mikan_subscriber_subscription_sync_feeds(before_each: ()) -> RecorderResult<()> {
    let mut preset = TestingPreset::default().await?;

    let app_ctx = preset.app_ctx.clone();

    let mikan_server = &mut preset.mikan_server;

    let _resources_mock = mikan_server.mock_resources_with_doppel();

    let _login_mock = mikan_server.mock_get_login_page();

    let subscriber_id = 1;

    let subscription_am = subscriptions::ActiveModel {
      display_name: ActiveValue::Set("test subscription".to_string()),
      subscriber_id: ActiveValue::Set(subscriber_id),
      category: ActiveValue::Set(subscriptions::SubscriptionCategory::MikanSubscriber),
      source_url: ActiveValue::Set(
        MikanSubscriberSubscriptionUrlMeta {
          mikan_subscription_token: "test".into(),
        }
        .build_rss_url(mikan_server.base_url().clone())
        .to_string(),
      ),
      enabled: ActiveValue::Set(true),
      ..Default::default()
    };

    let subscription_model = subscription_am.insert(app_ctx.db()).await?;

    let subscription = subscriptions::Subscription::try_from_model(&subscription_model)?;

    let (incremental_bangumi_list, incremental_episode_list) = {
      subscription.sync_feeds_incremental(app_ctx.clone()).await?;

      let bangumi_list = bangumi::Entity::find().all(app_ctx.db()).await?;

      assert!(!bangumi_list.is_empty());

      let episode_list = episodes::Entity::find().all(app_ctx.db()).await?;

      assert!(!episode_list.is_empty());

      (bangumi_list, episode_list)
    };

    let (full_bangumi_list, full_episode_list) = {
      subscription.sync_feeds_full(app_ctx.clone()).await?;

      let bangumi_list = bangumi::Entity::find().all(app_ctx.db()).await?;

      assert!(!bangumi_list.is_empty());

      let episode_list = episodes::Entity::find().all(app_ctx.db()).await?;

      assert!(!episode_list.is_empty());

      (bangumi_list, episode_list)
    };

    assert_eq!(incremental_bangumi_list.len(), full_bangumi_list.len());
    assert!(incremental_episode_list.len() < full_episode_list.len());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_mikan_bangumi_subscription_sync_feeds(before_each: ()) -> RecorderResult<()> {
    let mut preset = TestingPreset::default().await?;

    let app_ctx = preset.app_ctx.clone();

    let mikan_server = &mut preset.mikan_server;

    let _resources_mock = mikan_server.mock_resources_with_doppel();

    let _login_mock = mikan_server.mock_get_login_page();

    let subscriber_id = 1;

    let subscription_am = subscriptions::ActiveModel {
      display_name: ActiveValue::Set("test subscription".to_string()),
      subscriber_id: ActiveValue::Set(subscriber_id),
      category: ActiveValue::Set(subscriptions::SubscriptionCategory::MikanBangumi),
      source_url: ActiveValue::Set(
        MikanBangumiHash {
          mikan_bangumi_id: "3600".into(),
          mikan_fansub_id: "370".into(),
        }
        .build_rss_url(mikan_server.base_url().clone())
        .to_string(),
      ),
      enabled: ActiveValue::Set(true),
      ..Default::default()
    };

    let subscription_model = subscription_am.insert(app_ctx.db()).await?;

    let subscription = subscriptions::Subscription::try_from_model(&subscription_model)?;

    {
      subscription.sync_feeds_incremental(app_ctx.clone()).await?;
      let bangumi_list = bangumi::Entity::find().all(app_ctx.db()).await?;

      assert!(!bangumi_list.is_empty());
    };

    {
      subscription.sync_feeds_full(app_ctx.clone()).await?;
      let bangumi_list = bangumi::Entity::find().all(app_ctx.db()).await?;

      assert!(!bangumi_list.is_empty());
    }

    Ok(())
  }
}
