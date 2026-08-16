use std::borrow::Cow;

use chrono::{DateTime, Utc};
use downloader::bittorrent::BITTORRENT_MIME_TYPE;
use url::Url;

use crate::{
  app::{AppContextTrait, PROJECT_NAME},
  models::{
    episodes,
    feeds::{
      self,
      rss::{RssFeedItemTrait, RssFeedTrait},
    },
    subscriptions,
  },
  web::controller,
};

pub struct SubscriptionEpisodesFeed {
  pub feed: feeds::Model,
  pub subscription: subscriptions::Model,
  pub episodes: Vec<episodes::Model>,
}

impl SubscriptionEpisodesFeed {
  pub fn from_model(feed: feeds::Model, subscription: subscriptions::Model, episodes: Vec<episodes::Model>) -> Self {
    Self { feed, subscription, episodes }
  }
}

impl RssFeedItemTrait for episodes::Model {
  fn get_guid_value(&self) -> Cow<'_, str> {
    Cow::Owned(format!("{PROJECT_NAME}:episode:{}", self.id))
  }

  fn get_xmlns(&self) -> Cow<'_, str> {
    match self.episode_type {
      episodes::EpisodeType::Mikan => Cow::Borrowed("https://mikanani.me/0.1/"),
    }
  }

  fn get_title(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.display_name)
  }

  fn get_description(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.display_name)
  }

  fn get_link(&self, _ctx: &dyn AppContextTrait, _api_base: &Url) -> Option<Cow<'_, str>> {
    self.homepage.as_deref().map(Cow::Borrowed)
  }

  fn get_enclosure_mime(&self) -> Option<Cow<'_, str>> {
    if self.enclosure_torrent_link.is_some() {
      Some(Cow::Borrowed(BITTORRENT_MIME_TYPE))
    } else {
      None
    }
  }

  fn get_enclosure_link(&self, _ctx: &dyn AppContextTrait, _api_base: &Url) -> Option<Cow<'_, str>> {
    self.enclosure_torrent_link.as_deref().map(Cow::Borrowed)
  }

  fn get_enclosure_pub_date(&self) -> Option<DateTime<Utc>> {
    self.enclosure_pub_date
  }

  fn get_enclosure_content_length(&self) -> Option<i64> {
    self.enclosure_content_length
  }
}

impl RssFeedTrait for SubscriptionEpisodesFeed {
  type Item = episodes::Model;

  fn get_description(&self) -> Cow<'_, str> {
    Cow::Owned(format!("{PROJECT_NAME} - episodes of subscription {}", self.subscription.id))
  }

  fn get_title(&self) -> Cow<'_, str> {
    Cow::Owned(format!("{PROJECT_NAME} - subscription episodes"))
  }

  fn get_link(&self, _ctx: &dyn AppContextTrait, api_base: &Url) -> Option<Cow<'_, str>> {
    let api_base = api_base
      .join(&format!("{}/rss/{}", controller::feeds::CONTROLLER_PREFIX, self.feed.token))
      .ok()?;
    Some(Cow::Owned(api_base.to_string()))
  }

  fn items(&self) -> impl Iterator<Item = &Self::Item> {
    self.episodes.iter()
  }

  fn into_items(self) -> impl Iterator<Item = Self::Item> {
    self.episodes.into_iter()
  }
}
