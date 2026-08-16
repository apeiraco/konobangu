use rss::Channel;
use sea_orm::{ColumnTrait, EntityTrait, JoinType, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use url::Url;

use crate::{
  app::AppContextTrait,
  errors::{RecorderError, RecorderResult},
  models::{
    episodes,
    feeds::{self, FeedSource, RssFeedTrait, SubscriptionEpisodesFeed},
    subscription_episode, subscriptions,
  },
};

pub enum Feed {
  SubscritpionEpisodes(SubscriptionEpisodesFeed),
}

impl Feed {
  pub async fn from_model(db: &sea_orm::DatabaseTransaction, m: feeds::Model) -> RecorderResult<Self> {
    match m.feed_source {
      FeedSource::SubscriptionEpisode => {
        let (subscription, episodes) = if let Some(subscription_id) = m.subscription_id
          && let Some(subscription) = subscriptions::Entity::find()
            .filter(subscriptions::Column::Id.eq(subscription_id))
            .one(db)
            .await?
        {
          let episodes = episodes::Entity::find()
            .join(JoinType::InnerJoin, episodes::Relation::SubscriptionEpisode.def())
            .join(JoinType::InnerJoin, subscription_episode::Relation::Subscription.def())
            .filter(subscriptions::Column::Id.eq(subscription_id))
            .order_by(episodes::Column::EnclosurePubDate, Order::Desc)
            .all(db)
            .await?;
          (subscription, episodes)
        } else {
          return Err(RecorderError::from_entity_not_found::<subscriptions::Entity>());
        };

        Ok(Feed::SubscritpionEpisodes(SubscriptionEpisodesFeed::from_model(m, subscription, episodes)))
      }
    }
  }

  pub fn into_rss_channel(self, ctx: &dyn AppContextTrait, api_base: &Url) -> RecorderResult<Channel> {
    match self {
      Self::SubscritpionEpisodes(feed) => feed.into_channel(ctx, api_base),
    }
  }
}
