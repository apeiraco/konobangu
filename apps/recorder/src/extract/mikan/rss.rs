use std::{borrow::Cow, str::FromStr};

use chrono::{DateTime, Utc};
use downloader::bittorrent::defs::BITTORRENT_MIME_TYPE;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
  errors::{RecorderResult, app_error::RecorderError},
  extract::{
    bittorrent::EpisodeEnclosureMeta,
    mikan::{
      MIKAN_BANGUMI_ID_QUERY_KEY, MIKAN_BANGUMI_RSS_PATH, MIKAN_FANSUB_ID_QUERY_KEY, MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH,
      MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY, MikanEpisodeHash, build_mikan_episode_homepage_url,
    },
  },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MikanRssItemEnclosure {
  #[serde(rename = "@type")]
  pub r#type: String,
  #[serde(rename = "@length")]
  pub length: i64,
  #[serde(rename = "@url")]
  pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MikanRssItemTorrentExtension {
  pub pub_date: String,
  pub content_length: i64,
  pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MikanRssItem {
  pub torrent: MikanRssItemTorrentExtension,
  pub link: String,
  pub title: String,
  pub enclosure: MikanRssItemEnclosure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MikanRssChannel {
  #[serde(rename = "item", default)]
  pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MikanRssRoot {
  pub channel: MikanRssChannel,
}

impl FromStr for MikanRssRoot {
  type Err = RecorderError;
  fn from_str(source: &str) -> RecorderResult<Self> {
    let me = quick_xml::de::from_str(source)?;
    Ok(me)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanRssItemMeta {
  pub title: String,
  pub torrent_link: Url,
  pub content_length: i64,
  pub mime: String,
  pub pub_date: Option<DateTime<Utc>>,
  pub mikan_episode_id: String,
  pub magnet_link: Option<String>,
}

impl MikanRssItemMeta {
  pub fn build_homepage_url(&self, mikan_base_url: Url) -> Url {
    build_mikan_episode_homepage_url(mikan_base_url, &self.mikan_episode_id)
  }

  pub fn parse_pub_date(pub_date: &str) -> chrono::ParseResult<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(pub_date)
      .or_else(|_| DateTime::parse_from_rfc3339(pub_date))
      .or_else(|_| DateTime::parse_from_rfc3339(&format!("{pub_date}+08:00")))
      .map(|s| s.with_timezone(&Utc))
  }
}

impl TryFrom<MikanRssItem> for MikanRssItemMeta {
  type Error = RecorderError;

  fn try_from(item: MikanRssItem) -> Result<Self, Self::Error> {
    let torrent = item.torrent;

    let enclosure = item.enclosure;

    let mime_type = enclosure.r#type;
    if mime_type != BITTORRENT_MIME_TYPE {
      return Err(RecorderError::MimeError {
        expected: String::from(BITTORRENT_MIME_TYPE),
        found: mime_type.to_string(),
        desc: String::from("MikanRssItem"),
      });
    }

    let title = item.title;

    let enclosure_url =
      Url::parse(&enclosure.url).map_err(|err| RecorderError::from_mikan_rss_invalid_field_and_source("enclosure_url:enclosure.link".into(), err))?;

    let homepage = Url::parse(&item.link).map_err(|err| RecorderError::from_mikan_rss_invalid_field_and_source("enclosure_url:enclosure.link".into(), err))?;

    let MikanEpisodeHash { mikan_episode_id, .. } =
      MikanEpisodeHash::from_homepage_url(&homepage).ok_or_else(|| RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("mikan_episode_id")))?;

    Ok(MikanRssItemMeta {
      title,
      torrent_link: enclosure_url,
      content_length: enclosure.length,
      mime: mime_type,
      pub_date: Self::parse_pub_date(&torrent.pub_date).ok(),
      mikan_episode_id,
      magnet_link: None,
    })
  }
}

impl From<MikanRssItemMeta> for EpisodeEnclosureMeta {
  fn from(item: MikanRssItemMeta) -> Self {
    Self {
      magnet_link: item.magnet_link,
      torrent_link: Some(item.torrent_link.to_string()),
      pub_date: item.pub_date,
      content_length: Some(item.content_length),
    }
  }
}

pub fn build_mikan_subscriber_subscription_rss_url(mikan_base_url: Url, mikan_subscription_token: &str) -> Url {
  let mut url = mikan_base_url;
  url.set_path(MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH);
  url
    .query_pairs_mut()
    .append_pair(MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY, mikan_subscription_token);
  url
}

pub fn build_mikan_bangumi_subscription_rss_url(mikan_base_url: Url, mikan_bangumi_id: &str, mikan_fansub_id: Option<&str>) -> Url {
  let mut url = mikan_base_url;
  url.set_path(MIKAN_BANGUMI_RSS_PATH);
  url.query_pairs_mut().append_pair(MIKAN_BANGUMI_ID_QUERY_KEY, mikan_bangumi_id);
  if let Some(mikan_fansub_id) = mikan_fansub_id {
    url.query_pairs_mut().append_pair(MIKAN_FANSUB_ID_QUERY_KEY, mikan_fansub_id);
  };
  url
}

#[cfg(test)]
mod test {
  #![allow(unused_variables)]
  use std::fs;

  use rstest::{fixture, rstest};
  use tracing::Level;

  use super::*;
  use crate::{errors::RecorderResult, test_utils::tracing::try_init_testing_tracing};

  #[fixture]
  fn before_each() {
    try_init_testing_tracing(Level::DEBUG);
  }

  #[rstest]
  #[test]
  fn test_mikan_rss_episode_item_try_from_rss_item(before_each: ()) -> RecorderResult<()> {
    let rss_str = fs::read_to_string("tests/resources/mikan/doppel/RSS/Bangumi-bangumiId%3D3288%26subgroupid%3D370.html")?;

    let mut channel = MikanRssRoot::from_str(&rss_str)?.channel;

    assert!(!channel.items.is_empty());

    let item = channel.items.pop().unwrap();

    let episode_item = MikanRssItemMeta::try_from(item.clone())?;

    assert!(episode_item.pub_date.is_some());

    Ok(())
  }
}
