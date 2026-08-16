use std::{borrow::Cow, fmt, str::FromStr, sync::Arc};

use async_stream::try_stream;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use downloader::bittorrent::defs::BITTORRENT_MIME_TYPE;
use fetch::{html::fetch_html, image::fetch_image};
use futures::{Stream, TryStreamExt, pin_mut};
use html_escape::decode_html_entities;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use snafu::{FromString, OptionExt};
use tracing::instrument;
use url::Url;

use crate::{
  app::AppContextTrait,
  errors::app_error::{RecorderError, RecorderResult},
  extract::{
    bittorrent::EpisodeEnclosureMeta,
    html::{extract_background_image_src_from_style_attr, extract_inner_text_from_element_ref},
    media::extract_image_src_from_str,
    mikan::{
      MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH, MIKAN_BANGUMI_HOMEPAGE_PATH, MIKAN_BANGUMI_ID_QUERY_KEY, MIKAN_BANGUMI_POSTER_PATH, MIKAN_BANGUMI_RSS_PATH,
      MIKAN_EPISODE_HOMEPAGE_PATH, MIKAN_FANSUB_HOMEPAGE_PATH, MIKAN_FANSUB_ID_QUERY_KEY, MIKAN_POSTER_BUCKET_KEY, MIKAN_SEASON_FLOW_PAGE_PATH,
      MIKAN_SEASON_STR_QUERY_KEY, MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH, MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY, MIKAN_UNKNOWN_FANSUB_ID,
      MIKAN_YEAR_QUERY_KEY, MikanClient, build_mikan_bangumi_subscription_rss_url, build_mikan_subscriber_subscription_rss_url,
    },
  },
  storage::StorageContentCategory,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanRssEpisodeItem {
  pub title: String,
  pub torrent_link: Url,
  pub content_length: Option<i64>,
  pub mime: String,
  pub pub_date: Option<DateTime<Utc>>,
  pub mikan_episode_id: String,
  pub magnet_link: Option<String>,
}

impl MikanRssEpisodeItem {
  pub fn build_homepage_url(&self, mikan_base_url: Url) -> Url {
    build_mikan_episode_homepage_url(mikan_base_url, &self.mikan_episode_id)
  }
}

impl TryFrom<rss::Item> for MikanRssEpisodeItem {
  type Error = RecorderError;

  fn try_from(item: rss::Item) -> Result<Self, Self::Error> {
    let enclosure = item
      .enclosure
      .ok_or_else(|| RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("enclosure")))?;

    let mime_type = enclosure.mime_type;
    if mime_type != BITTORRENT_MIME_TYPE {
      return Err(RecorderError::MimeError {
        expected: String::from(BITTORRENT_MIME_TYPE),
        found: mime_type.to_string(),
        desc: String::from("MikanRssItem"),
      });
    }

    let title = item
      .title
      .ok_or_else(|| RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("title:title")))?;

    let enclosure_url =
      Url::parse(&enclosure.url).map_err(|err| RecorderError::from_mikan_rss_invalid_field_and_source("enclosure_url:enclosure.link".into(), err))?;

    let homepage = item
      .link
      .and_then(|link| Url::parse(&link).ok())
      .ok_or_else(|| RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("homepage:link")))?;

    let MikanEpisodeHash { mikan_episode_id, .. } =
      MikanEpisodeHash::from_homepage_url(&homepage).ok_or_else(|| RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("mikan_episode_id")))?;

    let pub_date = item
      .extensions
      .get("torrent")
      .and_then(|t| t.get("pubDate"))
      .and_then(|e| e.first())
      .and_then(|e| e.value.as_deref());

    Ok(MikanRssEpisodeItem {
      title,
      torrent_link: enclosure_url,
      content_length: enclosure.length.parse().ok(),
      mime: mime_type,
      pub_date: pub_date.and_then(|s| {
        DateTime::parse_from_rfc2822(s)
          .ok()
          .map(|s| s.with_timezone(&Utc))
          .or_else(|| DateTime::parse_from_rfc3339(s).ok().map(|s| s.with_timezone(&Utc)))
          .or_else(|| DateTime::parse_from_rfc3339(&format!("{s}+08:00")).ok().map(|s| s.with_timezone(&Utc)))
      }),
      mikan_episode_id,
      magnet_link: None,
    })
  }
}

impl From<MikanRssEpisodeItem> for EpisodeEnclosureMeta {
  fn from(item: MikanRssEpisodeItem) -> Self {
    Self {
      magnet_link: item.magnet_link,
      torrent_link: Some(item.torrent_link.to_string()),
      pub_date: item.pub_date,
      content_length: item.content_length,
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSubscriberSubscriptionUrlMeta {
  pub mikan_subscription_token: String,
}

impl MikanSubscriberSubscriptionUrlMeta {
  pub fn from_rss_url(url: &Url) -> Option<Self> {
    if url.path() == MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH {
      url
        .query_pairs()
        .find(|(k, _)| k == MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY)
        .map(|(_, v)| MikanSubscriberSubscriptionUrlMeta {
          mikan_subscription_token: v.to_string(),
        })
    } else {
      None
    }
  }

  pub fn build_rss_url(self, mikan_base_url: Url) -> Url {
    build_mikan_subscriber_subscription_rss_url(mikan_base_url, &self.mikan_subscription_token)
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct MikanBangumiIndexMeta {
  pub homepage: Url,
  pub origin_poster_src: Option<Url>,
  pub bangumi_title: String,
  pub mikan_bangumi_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct MikanFansubMeta {
  pub mikan_fansub_id: String,
  pub fansub: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct MikanBangumiMeta {
  pub homepage: Url,
  pub origin_poster_src: Option<Url>,
  pub bangumi_title: String,
  pub mikan_bangumi_id: String,
  pub mikan_fansub_id: String,
  pub fansub: String,
}

impl MikanBangumiMeta {
  pub fn bangumi_hash(&self) -> MikanBangumiHash {
    MikanBangumiHash {
      mikan_bangumi_id: self.mikan_bangumi_id.clone(),
      mikan_fansub_id: self.mikan_fansub_id.clone(),
    }
  }
}

impl From<MikanEpisodeMeta> for MikanBangumiMeta {
  fn from(episode_meta: MikanEpisodeMeta) -> Self {
    Self {
      homepage: episode_meta.homepage,
      origin_poster_src: episode_meta.origin_poster_src,
      bangumi_title: episode_meta.bangumi_title,
      mikan_bangumi_id: episode_meta.mikan_bangumi_id,
      mikan_fansub_id: episode_meta.mikan_fansub_id,
      fansub: episode_meta.fansub,
    }
  }
}

impl MikanBangumiMeta {
  pub fn from_bangumi_index_and_fansub_meta(bangumi_index_meta: MikanBangumiIndexMeta, fansub_meta: MikanFansubMeta) -> Self {
    Self {
      homepage: bangumi_index_meta.homepage,
      origin_poster_src: bangumi_index_meta.origin_poster_src,
      bangumi_title: bangumi_index_meta.bangumi_title,
      mikan_bangumi_id: bangumi_index_meta.mikan_bangumi_id,
      mikan_fansub_id: fansub_meta.mikan_fansub_id,
      fansub: fansub_meta.fansub,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MikanFansubHash {
  pub mikan_fansub_id: String,
}

impl MikanFansubHash {
  pub fn from_homepage_url(url: &Url) -> Option<Self> {
    let path = url.path();
    if path.starts_with(MIKAN_FANSUB_HOMEPAGE_PATH) {
      let mikan_fansub_id = path.replace(&format!("{MIKAN_FANSUB_HOMEPAGE_PATH}/"), "");
      Some(Self { mikan_fansub_id })
    } else {
      None
    }
  }

  pub fn build_homepage_url(self, mikan_base_url: Url) -> Url {
    let mut url = mikan_base_url;
    url.set_path(&format!("{MIKAN_FANSUB_HOMEPAGE_PATH}/{}", self.mikan_fansub_id));
    url
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpisodeMeta {
  pub homepage: Url,
  pub origin_poster_src: Option<Url>,
  pub bangumi_title: String,
  pub episode_title: String,
  pub fansub: String,
  pub mikan_bangumi_id: String,
  pub mikan_fansub_id: String,
  pub mikan_episode_id: String,
}

impl MikanEpisodeMeta {
  pub fn bangumi_hash(&self) -> MikanBangumiHash {
    MikanBangumiHash {
      mikan_bangumi_id: self.mikan_bangumi_id.clone(),
      mikan_fansub_id: self.mikan_fansub_id.clone(),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiPosterMeta {
  pub origin_poster_src: Url,
  pub poster_src: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiIndexHash {
  pub mikan_bangumi_id: String,
}

impl MikanBangumiIndexHash {
  pub fn from_homepage_url(url: &Url) -> Option<Self> {
    if url.path().starts_with(MIKAN_BANGUMI_HOMEPAGE_PATH) {
      let mikan_bangumi_id = url.path().replace(&format!("{MIKAN_BANGUMI_HOMEPAGE_PATH}/"), "");

      Some(Self { mikan_bangumi_id })
    } else {
      None
    }
  }

  pub fn build_homepage_url(self, mikan_base_url: Url) -> Url {
    build_mikan_bangumi_homepage_url(mikan_base_url, &self.mikan_bangumi_id, None)
  }

  pub fn from_rss_url(url: &Url) -> Option<Self> {
    if url.path() == MIKAN_BANGUMI_RSS_PATH {
      url
        .query_pairs()
        .find(|(k, _)| k == MIKAN_BANGUMI_ID_QUERY_KEY)
        .map(|(_, v)| v.to_string())
        .map(|mikan_bangumi_id| Self { mikan_bangumi_id })
    } else {
      None
    }
  }

  pub fn build_rss_url(self, mikan_base_url: Url) -> Url {
    build_mikan_bangumi_subscription_rss_url(mikan_base_url, &self.mikan_bangumi_id, None)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MikanBangumiHash {
  pub mikan_bangumi_id: String,
  pub mikan_fansub_id: String,
}

impl MikanBangumiHash {
  pub fn from_homepage_url(url: &Url) -> Option<Self> {
    if url.path().starts_with(MIKAN_BANGUMI_HOMEPAGE_PATH) {
      let mikan_bangumi_id = url.path().replace(&format!("{MIKAN_BANGUMI_HOMEPAGE_PATH}/"), "");

      let url_fragment = url.fragment()?;

      Some(Self {
        mikan_bangumi_id,
        mikan_fansub_id: String::from(url_fragment),
      })
    } else {
      None
    }
  }

  pub fn from_rss_url(url: &Url) -> Option<Self> {
    if url.path() == MIKAN_BANGUMI_RSS_PATH {
      if let (Some(mikan_fansub_id), Some(mikan_bangumi_id)) = (
        url.query_pairs().find(|(k, _)| k == MIKAN_FANSUB_ID_QUERY_KEY).map(|(_, v)| v.to_string()),
        url.query_pairs().find(|(k, _)| k == MIKAN_BANGUMI_ID_QUERY_KEY).map(|(_, v)| v.to_string()),
      ) {
        Some(Self {
          mikan_bangumi_id,
          mikan_fansub_id,
        })
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn build_rss_url(self, mikan_base_url: Url) -> Url {
    build_mikan_bangumi_subscription_rss_url(mikan_base_url, &self.mikan_bangumi_id, Some(&self.mikan_fansub_id))
  }

  pub fn build_homepage_url(self, mikan_base_url: Url) -> Url {
    build_mikan_bangumi_homepage_url(mikan_base_url, &self.mikan_bangumi_id, Some(&self.mikan_fansub_id))
  }
}

pub fn build_mikan_episode_homepage_url(mikan_base_url: Url, mikan_episode_id: &str) -> Url {
  let mut url = mikan_base_url;
  url.set_path(&format!("{MIKAN_EPISODE_HOMEPAGE_PATH}/{mikan_episode_id}"));
  url
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MikanEpisodeHash {
  pub mikan_episode_id: String,
}

impl MikanEpisodeHash {
  pub fn from_homepage_url(url: &Url) -> Option<Self> {
    if url.path().starts_with(MIKAN_EPISODE_HOMEPAGE_PATH) {
      let mikan_episode_id = url.path().replace(&format!("{MIKAN_EPISODE_HOMEPAGE_PATH}/"), "");
      Some(Self { mikan_episode_id })
    } else {
      None
    }
  }

  pub fn build_homepage_url(self, mikan_base_url: Url) -> Url {
    build_mikan_episode_homepage_url(mikan_base_url, &self.mikan_episode_id)
  }
}

#[derive(async_graphql::Enum, Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MikanSeasonStr {
  #[serde(rename = "春")]
  #[graphql(name = "spring")]
  Spring,
  #[serde(rename = "夏")]
  #[graphql(name = "summer")]
  Summer,
  #[serde(rename = "秋")]
  #[graphql(name = "autumn")]
  Autumn,
  #[serde(rename = "冬")]
  #[graphql(name = "winter")]
  Winter,
}

impl fmt::Display for MikanSeasonStr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Spring => write!(f, "春"),
      Self::Summer => write!(f, "夏"),
      Self::Autumn => write!(f, "秋"),
      Self::Winter => write!(f, "冬"),
    }
  }
}

impl FromStr for MikanSeasonStr {
  type Err = RecorderError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "春" => Ok(MikanSeasonStr::Spring),
      "夏" => Ok(MikanSeasonStr::Summer),
      "秋" => Ok(MikanSeasonStr::Autumn),
      "冬" => Ok(MikanSeasonStr::Winter),
      _ => Err(RecorderError::without_source(format!(
        "MikanSeasonStr must be one of '春', '夏', '秋', '冬', but got '{s}'"
      ))),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanSeasonFlowUrlMeta {
  pub year: i32,
  pub season_str: MikanSeasonStr,
}

impl MikanSeasonFlowUrlMeta {
  pub fn from_url(url: &Url) -> Option<Self> {
    if url.path().starts_with(MIKAN_SEASON_FLOW_PAGE_PATH) {
      if let (Some(year), Some(season_str)) = (
        url
          .query_pairs()
          .find(|(key, _)| key == MIKAN_YEAR_QUERY_KEY)
          .and_then(|(_, value)| value.parse::<i32>().ok()),
        url
          .query_pairs()
          .find(|(key, _)| key == MIKAN_SEASON_STR_QUERY_KEY)
          .and_then(|(_, value)| MikanSeasonStr::from_str(&value).ok()),
      ) {
        Some(Self { year, season_str })
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn build_season_flow_url(self, mikan_base_url: Url) -> Url {
    build_mikan_season_flow_url(mikan_base_url, self.year, self.season_str)
  }
}
pub fn build_mikan_bangumi_homepage_url(mikan_base_url: Url, mikan_bangumi_id: &str, mikan_fansub_id: Option<&str>) -> Url {
  let mut url = mikan_base_url;
  url.set_path(&format!("{MIKAN_BANGUMI_HOMEPAGE_PATH}/{mikan_bangumi_id}"));
  url.set_fragment(mikan_fansub_id);
  url
}

pub fn build_mikan_season_flow_url(mikan_base_url: Url, year: i32, season_str: MikanSeasonStr) -> Url {
  let mut url = mikan_base_url;
  url.set_path(MIKAN_SEASON_FLOW_PAGE_PATH);
  url
    .query_pairs_mut()
    .append_pair(MIKAN_YEAR_QUERY_KEY, &year.to_string())
    .append_pair(MIKAN_SEASON_STR_QUERY_KEY, &season_str.to_string());
  url
}

pub fn build_mikan_bangumi_expand_subscribed_url(mikan_base_url: Url, mikan_bangumi_id: &str) -> Url {
  let mut url = mikan_base_url;
  url.set_path(MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH);
  url
    .query_pairs_mut()
    .append_pair(MIKAN_BANGUMI_ID_QUERY_KEY, mikan_bangumi_id)
    .append_pair("showSubscribed", "true");
  url
}

#[instrument(err, skip_all, fields(mikan_episode_homepage_url = mikan_episode_homepage_url.as_str()))]
pub fn extract_mikan_episode_meta_from_episode_homepage_html(
  html: &Html,
  mikan_base_url: Url,
  mikan_episode_homepage_url: Url,
) -> RecorderResult<MikanEpisodeMeta> {
  let bangumi_title_selector = &Selector::parse(".bangumi-title > a[href^='/Home/Bangumi/']").unwrap();
  let mikan_bangumi_id_selector = &Selector::parse(".bangumi-title > a.mikan-rss[data-original-title='RSS']").unwrap();
  let bangumi_poster_selector = &Selector::parse(".bangumi-poster").unwrap();

  let bangumi_title = html
    .select(bangumi_title_selector)
    .next()
    .map(extract_inner_text_from_element_ref)
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title")))?;

  let MikanBangumiHash {
    mikan_bangumi_id,
    mikan_fansub_id,
    ..
  } = html
    .select(mikan_bangumi_id_selector)
    .next()
    .and_then(|el| el.value().attr("href"))
    .and_then(|s| mikan_episode_homepage_url.join(s).ok())
    .and_then(|rss_link_url| MikanBangumiHash::from_rss_url(&rss_link_url))
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_bangumi_id")))?;

  let episode_title = html
    .select(&Selector::parse("title").unwrap())
    .next()
    .map(extract_inner_text_from_element_ref)
    .map(|s| s.replace(" - Mikan Project", ""))
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("episode_title")))?;

  let MikanEpisodeHash { mikan_episode_id, .. } = MikanEpisodeHash::from_homepage_url(&mikan_episode_homepage_url)
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_episode_id")))?;

  let fansub_name = if mikan_fansub_id == MIKAN_UNKNOWN_FANSUB_ID {
    MIKAN_UNKNOWN_FANSUB_ID.to_string()
  } else {
    html
      .select(&Selector::parse(".bangumi-info a.magnet-link-wrap[href^='/Home/PublishGroup/']").unwrap())
      .next()
      .map(extract_inner_text_from_element_ref)
      .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("fansub_name")))?
  };

  let origin_poster_src = html.select(bangumi_poster_selector).next().and_then(|el| {
    el.value()
      .attr("data-src")
      .and_then(|data_src| extract_image_src_from_str(data_src, &mikan_base_url))
      .or_else(|| {
        el.value()
          .attr("style")
          .and_then(|style| extract_background_image_src_from_style_attr(style, &mikan_base_url))
      })
  });

  tracing::debug!(
    bangumi_title,
    mikan_bangumi_id,
    episode_title,
    mikan_episode_id,
    origin_poster_src = origin_poster_src.as_ref().map(|url| url.as_str()),
    fansub_name,
    mikan_fansub_id,
    "mikan episode meta extracted"
  );

  Ok(MikanEpisodeMeta {
    mikan_bangumi_id,
    mikan_fansub_id,
    bangumi_title,
    episode_title,
    homepage: mikan_episode_homepage_url,
    origin_poster_src,
    fansub: fansub_name,
    mikan_episode_id,
  })
}

#[instrument(err, skip_all, fields(mikan_episode_homepage_url = mikan_episode_homepage_url.as_str()))]
pub async fn scrape_mikan_episode_meta_from_episode_homepage_url(
  http_client: &MikanClient,
  mikan_episode_homepage_url: Url,
) -> RecorderResult<MikanEpisodeMeta> {
  let mikan_base_url = http_client.base_url().clone();
  let content = fetch_html(http_client, mikan_episode_homepage_url.as_str()).await?;

  let html = Html::parse_document(&content);

  extract_mikan_episode_meta_from_episode_homepage_html(&html, mikan_base_url, mikan_episode_homepage_url)
}

pub fn extract_mikan_bangumi_index_meta_from_bangumi_homepage_html(
  html: &Html,
  mikan_bangumi_homepage_url: Url,
  mikan_base_url: &Url,
) -> RecorderResult<MikanBangumiIndexMeta> {
  let bangumi_title_selector = &Selector::parse(".bangumi-title").unwrap();
  let mikan_bangumi_id_selector = &Selector::parse(".bangumi-title > .mikan-rss[data-original-title='RSS']").unwrap();
  let bangumi_poster_selector = &Selector::parse(".bangumi-poster").unwrap();

  let bangumi_title = html
    .select(bangumi_title_selector)
    .next()
    .map(extract_inner_text_from_element_ref)
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title")))?;

  let mikan_bangumi_id = html
    .select(mikan_bangumi_id_selector)
    .next()
    .and_then(|el| el.value().attr("href"))
    .and_then(|s| mikan_bangumi_homepage_url.join(s).ok())
    .and_then(|rss_link_url| MikanBangumiIndexHash::from_rss_url(&rss_link_url))
    .map(|MikanBangumiIndexHash { mikan_bangumi_id }| mikan_bangumi_id)
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_bangumi_id")))?;

  let origin_poster_src = html.select(bangumi_poster_selector).next().and_then(|el| {
    el.value()
      .attr("data-src")
      .and_then(|data_src| extract_image_src_from_str(data_src, mikan_base_url))
      .or_else(|| {
        el.value()
          .attr("style")
          .and_then(|style| extract_background_image_src_from_style_attr(style, mikan_base_url))
      })
  });

  tracing::trace!(
    bangumi_title,
    mikan_bangumi_id,
    origin_poster_src = origin_poster_src.as_ref().map(|url| url.as_str()),
    "mikan bangumi index meta extracted"
  );

  Ok(MikanBangumiIndexMeta {
    homepage: mikan_bangumi_homepage_url,
    bangumi_title,
    origin_poster_src,
    mikan_bangumi_id,
  })
}

pub fn extract_mikan_fansub_meta_from_bangumi_homepage_html(html: &Html, mikan_fansub_id: String) -> Option<MikanFansubMeta> {
  if mikan_fansub_id == MIKAN_UNKNOWN_FANSUB_ID {
    return Some(MikanFansubMeta {
      mikan_fansub_id,
      fansub: MIKAN_UNKNOWN_FANSUB_ID.to_string(),
    });
  }

  html
    .select(&Selector::parse(&format!("a.subgroup-name[data-anchor='#{mikan_fansub_id}']")).unwrap())
    .next()
    .map(extract_inner_text_from_element_ref)
    .map(|fansub_name| MikanFansubMeta {
      mikan_fansub_id,
      fansub: fansub_name,
    })
}

#[instrument(err, skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub fn extract_mikan_bangumi_meta_from_bangumi_homepage_html(
  html: &Html,
  mikan_bangumi_homepage_url: Url,
  mikan_base_url: &Url,
) -> RecorderResult<MikanBangumiMeta> {
  let mikan_fansub_id = MikanBangumiHash::from_homepage_url(&mikan_bangumi_homepage_url)
    .map(|s| s.mikan_fansub_id)
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_fansub_id")))?;

  let bangumi_index_meta = extract_mikan_bangumi_index_meta_from_bangumi_homepage_html(html, mikan_bangumi_homepage_url, mikan_base_url)?;

  let fansub_meta = extract_mikan_fansub_meta_from_bangumi_homepage_html(html, mikan_fansub_id)
    .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("fansub_name")))?;

  Ok(MikanBangumiMeta::from_bangumi_index_and_fansub_meta(bangumi_index_meta, fansub_meta))
}

#[instrument(err, skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub async fn scrape_mikan_bangumi_meta_from_bangumi_homepage_url(
  mikan_client: &MikanClient,
  mikan_bangumi_homepage_url: Url,
) -> RecorderResult<MikanBangumiMeta> {
  let mikan_base_url = mikan_client.base_url();
  let content = fetch_html(mikan_client, mikan_bangumi_homepage_url.as_str()).await?;
  let html = Html::parse_document(&content);

  extract_mikan_bangumi_meta_from_bangumi_homepage_html(&html, mikan_bangumi_homepage_url, mikan_base_url)
}

#[allow(dead_code)]
#[instrument(err, skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub async fn scrape_mikan_bangumi_index_meta_from_bangumi_homepage_url(
  mikan_client: &MikanClient,
  mikan_bangumi_homepage_url: Url,
) -> RecorderResult<MikanBangumiIndexMeta> {
  let mikan_base_url = mikan_client.base_url();
  let content = fetch_html(mikan_client, mikan_bangumi_homepage_url.as_str()).await?;
  let html = Html::parse_document(&content);

  extract_mikan_bangumi_index_meta_from_bangumi_homepage_html(&html, mikan_bangumi_homepage_url, mikan_base_url)
}

#[instrument(skip_all, fields(origin_poster_src_url = origin_poster_src_url.as_str()))]
pub async fn scrape_mikan_poster_data_from_image_url(mikan_client: &MikanClient, origin_poster_src_url: Url) -> RecorderResult<Bytes> {
  let poster_data = fetch_image(mikan_client, origin_poster_src_url.clone()).await?;
  Ok(poster_data)
}

#[instrument(skip_all, fields(origin_poster_src_url = origin_poster_src_url.as_str()))]
pub async fn scrape_mikan_poster_meta_from_image_url(ctx: &dyn AppContextTrait, origin_poster_src_url: Url) -> RecorderResult<MikanBangumiPosterMeta> {
  let storage_service = ctx.storage();
  let media_service = ctx.media();
  let mikan_client = ctx.mikan();

  let storage_path = storage_service.build_public_object_path(
    StorageContentCategory::Image,
    MIKAN_POSTER_BUCKET_KEY,
    &origin_poster_src_url.path().replace(&format!("{MIKAN_BANGUMI_POSTER_PATH}/"), ""),
  );
  let meta = if let Some(poster_src) = storage_service.exists(&storage_path).await? {
    MikanBangumiPosterMeta {
      origin_poster_src: origin_poster_src_url,
      poster_src: Some(poster_src.to_string()),
    }
  } else {
    let poster_data = scrape_mikan_poster_data_from_image_url(mikan_client, origin_poster_src_url.clone()).await?;

    let poster_str = storage_service
      .write_for_task(ctx.task().queue_database()?, storage_path.clone(), poster_data)
      .await?;

    MikanBangumiPosterMeta {
      origin_poster_src: origin_poster_src_url,
      poster_src: Some(poster_str.to_string()),
    }
  };

  if meta.poster_src.is_some()
    && storage_path.extension().is_some_and(|ext| media_service.is_legacy_image_format(ext))
    && let Err(_error) = crate::media::derivative::enqueue_missing(ctx, storage_path.as_str()).await
  {
    tracing::warn!(
      category = "image_derivative_enqueue",
      "Original image retained; optimization could not be scheduled"
    );
  }

  Ok(meta)
}

pub fn extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(html: &Html, mikan_base_url: &Url) -> Vec<MikanBangumiIndexMeta> {
  let bangumi_empty_selector = &Selector::parse(".no-subscribe-bangumi").unwrap();

  if html.select(bangumi_empty_selector).next().is_some() {
    return vec![];
  }

  let bangumi_item_selector = &Selector::parse(".mine.an-box ul.an-ul>li").unwrap();
  let bangumi_poster_span_selector = &Selector::parse("span[data-src][data-bangumiid]").unwrap();
  let bangumi_title_a_selector = &Selector::parse(".an-info-group a.an-text[title]").unwrap();

  let mut items = vec![];
  for bangumi_item in html.select(bangumi_item_selector) {
    let bangumi_poster_span = bangumi_item.select(bangumi_poster_span_selector).next();
    let bangumi_title_a = bangumi_item.select(bangumi_title_a_selector).next();
    if let (Some(bangumi_poster_span), Some(bangumi_title_a)) = (bangumi_poster_span, bangumi_title_a) {
      let origin_poster_src = bangumi_poster_span
        .attr("data-src")
        .and_then(|data_src| extract_image_src_from_str(data_src, mikan_base_url));
      let bangumi_title = bangumi_title_a.attr("title").map(|title| decode_html_entities(&title).trim().to_string());
      let mikan_bangumi_id = bangumi_poster_span.attr("data-bangumiid").map(|id| id.to_string());

      if let (Some(bangumi_title), Some(mikan_bangumi_id)) = (bangumi_title, mikan_bangumi_id) {
        let homepage = build_mikan_bangumi_homepage_url(mikan_base_url.clone(), &mikan_bangumi_id, None);
        if let Some(origin_poster_src) = origin_poster_src.as_ref() {
          tracing::trace!(
            origin_poster_src = origin_poster_src.as_str(),
            bangumi_title,
            mikan_bangumi_id,
            "bangumi index meta extracted"
          );
        } else {
          tracing::warn!(
            bangumi_title,
            mikan_bangumi_id,
            "bangumi index meta extracted, but failed to extract poster_src"
          );
        }
        items.push(MikanBangumiIndexMeta {
          homepage,
          origin_poster_src,
          bangumi_title,
          mikan_bangumi_id,
        });
      }
    }
  }
  items
}

#[instrument(skip_all, fields(mikan_bangumi_index = mikan_bangumi_index.mikan_bangumi_id.as_str()))]
pub fn extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
  html: &Html,
  mikan_bangumi_index: MikanBangumiIndexMeta,
  mikan_base_url: Url,
) -> Option<MikanBangumiMeta> {
  let fansub_container_selector = &Selector::parse(".js-expand_bangumi-subgroup.js-subscribed").unwrap();
  let fansub_title_selector = &Selector::parse(".tag-res-name[title]").unwrap();
  let fansub_id_selector = &Selector::parse(".active[data-subtitlegroupid][data-bangumiid]").unwrap();

  if let Some((fansub_name, mikan_fansub_id)) = {
    let subscribed_subgroup = html.select(fansub_container_selector).next();

    if subscribed_subgroup.is_none() {
      tracing::warn!(
        mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id,
        bangumi_title = mikan_bangumi_index.bangumi_title,
        "subscribed fansub subgroup not found, maybe you have not logged in or not subscribed to this bangumi"
      );
    }

    subscribed_subgroup.and_then(|fansub_info| {
      if let (Some(fansub_name), Some(mikan_fansub_id)) = (
        fansub_info
          .select(fansub_title_selector)
          .next()
          .and_then(|ele| ele.attr("title"))
          .map(String::from),
        fansub_info
          .select(fansub_id_selector)
          .next()
          .and_then(|ele| ele.attr("data-subtitlegroupid"))
          .map(String::from),
      ) {
        Some((fansub_name, mikan_fansub_id))
      } else {
        None
      }
    })
  } {
    tracing::trace!(
      mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id,
      bangumi_title = mikan_bangumi_index.bangumi_title,
      fansub_name,
      mikan_fansub_id,
      "subscribed fansub extracted"
    );

    let mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id;
    let bangumi_title = mikan_bangumi_index.bangumi_title;
    let origin_poster_src = mikan_bangumi_index.origin_poster_src;

    Some(MikanBangumiMeta {
      homepage: build_mikan_bangumi_homepage_url(mikan_base_url, &mikan_bangumi_id, Some(&mikan_fansub_id)),
      bangumi_title: bangumi_title.to_string(),
      mikan_bangumi_id: mikan_bangumi_id.to_string(),
      mikan_fansub_id: mikan_fansub_id.to_string(),
      fansub: fansub_name.to_string(),
      origin_poster_src: origin_poster_src.clone(),
    })
  } else {
    tracing::trace!(
      mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id,
      bangumi_title = mikan_bangumi_index.bangumi_title,
      "subscribed fansub failed to extract"
    );
    None
  }
}

pub fn scrape_mikan_bangumi_meta_stream_from_season_flow_url(
  ctx: Arc<dyn AppContextTrait>,
  mikan_season_flow_url: Url,
  credential_id: i32,
  subscriber_id: i32,
) -> impl Stream<Item = RecorderResult<MikanBangumiMeta>> {
  try_stream! {
      let mikan_base_url = ctx.mikan().base_url().clone();
      let mikan_client = ctx.mikan().fork_with_credential_id(ctx.as_ref(), credential_id, subscriber_id).await?;

      let content = fetch_html(&mikan_client, mikan_season_flow_url.clone()).await?;


      let mut bangumi_indices_meta = {
          let html = Html::parse_document(&content);
          extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, &mikan_base_url)
      };

      if bangumi_indices_meta.is_empty() && !mikan_client.has_login().await? {
          mikan_client.login().await?;
          let content = fetch_html(&mikan_client, mikan_season_flow_url).await?;
          let html = Html::parse_document(&content);
          bangumi_indices_meta =
              extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, &mikan_base_url);
      }


      mikan_client
          .sync_credential_cookies(ctx.as_ref(), credential_id, subscriber_id)
          .await?;

      for bangumi_index in bangumi_indices_meta {
          let bangumi_title = bangumi_index.bangumi_title.clone();
          let bangumi_expand_subscribed_fragment_url = build_mikan_bangumi_expand_subscribed_url(
              mikan_base_url.clone(),
              &bangumi_index.mikan_bangumi_id,
          );
          let bangumi_expand_subscribed_fragment =
              fetch_html(&mikan_client, bangumi_expand_subscribed_fragment_url).await?;

          let bangumi_meta = {
              let html = Html::parse_document(&bangumi_expand_subscribed_fragment);

              extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
                  &html,
                  bangumi_index,
                  mikan_base_url.clone(),
              )
              .with_whatever_context::<_, String, RecorderError>(|| {
                  format!("failed to extract mikan bangumi fansub of title = {bangumi_title}")
              })
          }?;

          yield bangumi_meta;
      }

      mikan_client
          .sync_credential_cookies(ctx.as_ref(), credential_id, subscriber_id)
      .await?;
  }
}

pub async fn scrape_mikan_bangumi_meta_list_from_season_flow_url(
  ctx: Arc<dyn AppContextTrait>,
  mikan_season_flow_url: Url,
  credential_id: i32,
  subscriber_id: i32,
) -> RecorderResult<Vec<MikanBangumiMeta>> {
  let stream = scrape_mikan_bangumi_meta_stream_from_season_flow_url(ctx, mikan_season_flow_url, credential_id, subscriber_id);

  pin_mut!(stream);

  stream.try_collect().await
}

#[cfg(test)]
mod test {
  #![allow(unused_variables)]
  use std::{fs, io::Cursor, sync::Arc};

  use futures::StreamExt;
  use image::{ImageFormat, ImageReader};
  use rstest::{fixture, rstest};
  use tracing::Level;
  use url::Url;

  use super::*;
  use crate::test_utils::{
    app::{TestingAppContext, TestingPreset},
    crypto::build_testing_crypto_service,
    database::build_testing_database_service,
    mikan::{MikanMockServer, build_testing_mikan_client, build_testing_mikan_credential, build_testing_mikan_credential_form},
    tracing::try_init_testing_tracing,
  };

  #[fixture]
  fn before_each() {
    try_init_testing_tracing(Level::DEBUG);
  }

  #[rstest]
  #[tokio::test]
  async fn test_scrape_mikan_poster_data_from_image_url(before_each: ()) -> RecorderResult<()> {
    let mut mikan_server = MikanMockServer::new().await?;

    let resources_mock = mikan_server.mock_resources_with_doppel();

    let mikan_base_url = mikan_server.base_url().clone();
    let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

    let bangumi_poster_url = mikan_base_url.join("/images/Bangumi/202309/5ce9fed1.jpg")?;

    let bgm_poster_data = scrape_mikan_poster_data_from_image_url(&mikan_client, bangumi_poster_url).await?;

    resources_mock.shared_resource_mock.expect(1);

    let image = {
      let c = Cursor::new(bgm_poster_data);
      ImageReader::new(c)
    };
    let image_format = image.with_guessed_format().ok().and_then(|i| i.format());
    assert!(
      image_format.is_some_and(|fmt| matches!(fmt, ImageFormat::Jpeg)),
      "should start with valid jpeg data magic number"
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_scrape_mikan_poster_meta_from_image_url(before_each: ()) -> RecorderResult<()> {
    let mut preset = TestingPreset::default().await?;

    let app_ctx = preset.app_ctx.clone();

    let mikan_base_url = preset.mikan_server.base_url().clone();

    let resources_mock = preset.mikan_server.mock_resources_with_doppel();

    let bangumi_poster_url = mikan_base_url.join("/images/Bangumi/202309/5ce9fed1.jpg")?;

    let bgm_poster = scrape_mikan_poster_meta_from_image_url(app_ctx.as_ref(), bangumi_poster_url).await?;

    resources_mock.shared_resource_mock.expect(1);

    let storage_service = app_ctx.storage();

    let storage_fullname = storage_service.build_public_object_path(StorageContentCategory::Image, MIKAN_POSTER_BUCKET_KEY, "202309/5ce9fed1.jpg");

    assert!(
      storage_service.exists(&storage_fullname).await?.is_some(),
      "storage_fullename_str = {}, list public = {:?}",
      storage_fullname,
      storage_service.list_public().await?
    );

    let bgm_poster_data = storage_service.read(&storage_fullname).await?;

    let image = {
      let c = Cursor::new(bgm_poster_data.to_vec());
      ImageReader::new(c)
    };
    let image_format = image.with_guessed_format().ok().and_then(|i| i.format());
    assert!(
      image_format.is_some_and(|fmt| matches!(fmt, ImageFormat::Jpeg)),
      "should start with valid jpeg data magic number"
    );

    Ok(())
  }

  #[rstest]
  #[test]
  fn test_extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(before_each: ()) -> RecorderResult<()> {
    let fragment_str = fs::read_to_string("tests/resources/mikan/BangumiCoverFlow-2025-spring.html")?;

    let mikan_base_url = Url::parse("https://mikanani.me/")?;

    let bangumi_index_meta_list = extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&Html::parse_document(&fragment_str), &mikan_base_url);

    assert_eq!(bangumi_index_meta_list.len(), 49);
    let first = &bangumi_index_meta_list[0];
    assert_eq!(first.bangumi_title, "吉伊卡哇");
    assert_eq!(first.mikan_bangumi_id, "3288");
    assert_eq!(first.homepage.to_string(), String::from("https://mikanani.me/Home/Bangumi/3288"));
    assert_eq!(
      first.origin_poster_src.as_ref().map(|s| s.to_string()).unwrap_or_default(),
      String::from("https://mikanani.me/images/Bangumi/202204/d8ef46c0.jpg")
    );

    Ok(())
  }

  #[rstest]
  #[test]
  fn test_extract_mikan_bangumi_index_meta_list_from_season_flow_fragment_noauth(before_each: ()) -> RecorderResult<()> {
    let fragment_str = fs::read_to_string("tests/resources/mikan/BangumiCoverFlow-noauth.html")?;

    let bangumi_index_meta_list =
      extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&Html::parse_document(&fragment_str), &Url::parse("https://mikanani.me/")?);

    assert!(bangumi_index_meta_list.is_empty());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_extract_mikan_bangumi_meta_from_expand_subscribed_fragment(before_each: ()) -> RecorderResult<()> {
    let mut mikan_server = MikanMockServer::new().await?;

    let login_mock = mikan_server.mock_get_login_page();
    let resources_mock = mikan_server.mock_resources_with_doppel();

    let mikan_base_url = mikan_server.base_url().clone();

    let mikan_client = build_testing_mikan_client(mikan_base_url.clone())
      .await?
      .fork_with_userpass_credential(build_testing_mikan_credential())
      .await?;

    mikan_client.login().await?;

    let origin_poster_src = Url::parse("https://mikanani.me/images/Bangumi/202504/076c1094.jpg")?;

    let bangumi_index_meta = MikanBangumiIndexMeta {
      homepage: Url::parse("https://mikanani.me/Home/Bangumi/3599")?,
      origin_poster_src: Some(origin_poster_src.clone()),
      bangumi_title: "夏日口袋".to_string(),
      mikan_bangumi_id: "3599".to_string(),
    };

    let fragment_str = fetch_html(
      &mikan_client,
      build_mikan_bangumi_expand_subscribed_url(mikan_base_url.clone(), &bangumi_index_meta.mikan_bangumi_id),
    )
    .await?;

    let bangumi = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
      &Html::parse_fragment(&fragment_str),
      bangumi_index_meta.clone(),
      Url::parse("https://mikanani.me/")?,
    )
    .unwrap_or_else(|| {
      panic!("bangumi should not be None");
    });

    assert_eq!(bangumi.homepage, Url::parse("https://mikanani.me/Home/Bangumi/3599#370")?);
    assert_eq!(bangumi.bangumi_title, bangumi_index_meta.bangumi_title);
    assert_eq!(bangumi.mikan_bangumi_id, bangumi_index_meta.mikan_bangumi_id);
    assert_eq!(bangumi.origin_poster_src, bangumi_index_meta.origin_poster_src);
    assert_eq!(bangumi.mikan_fansub_id, String::from("370"));
    assert_eq!(bangumi.fansub, String::from("LoliHouse"));

    Ok(())
  }

  #[rstest]
  #[test]
  fn test_extract_mikan_bangumi_meta_from_expand_subscribed_fragment_noauth(before_each: ()) -> RecorderResult<()> {
    let origin_poster_src = Url::parse("https://mikanani.me/images/Bangumi/202504/076c1094.jpg")?;

    let bangumi_index_meta = MikanBangumiIndexMeta {
      homepage: Url::parse("https://mikanani.me/Home/Bangumi/3599")?,
      origin_poster_src: Some(origin_poster_src.clone()),
      bangumi_title: "夏日口袋".to_string(),
      mikan_bangumi_id: "3599".to_string(),
    };

    let fragment_str = fs::read_to_string("tests/resources/mikan/ExpandBangumi-noauth.html")?;

    let bangumi = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
      &Html::parse_fragment(&fragment_str),
      bangumi_index_meta.clone(),
      Url::parse("https://mikanani.me/")?,
    );

    assert!(bangumi.is_none());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_scrape_mikan_bangumi_meta_stream_from_season_flow_url(before_each: ()) -> RecorderResult<()> {
    let mut mikan_server = MikanMockServer::new().await?;

    let mikan_base_url = mikan_server.base_url().clone();

    let app_ctx = {
      let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;
      let db_service = build_testing_database_service(Default::default()).await?;
      let crypto_service = build_testing_crypto_service().await?;
      let app_ctx = TestingAppContext::builder().mikan(mikan_client).db(db_service).crypto(crypto_service).build();

      Arc::new(app_ctx)
    };

    let resources_mock = mikan_server.mock_resources_with_doppel();

    let login_mock = mikan_server.mock_get_login_page();

    let mikan_client = app_ctx.mikan();

    let subscriber_id = 1;

    let credential = mikan_client
      .submit_credential_form(app_ctx.as_ref(), subscriber_id, build_testing_mikan_credential_form())
      .await?;

    let mikan_season_flow_url = build_mikan_season_flow_url(mikan_base_url.clone(), 2025, MikanSeasonStr::Spring);

    let bangumi_meta_stream = scrape_mikan_bangumi_meta_stream_from_season_flow_url(app_ctx.clone(), mikan_season_flow_url, credential.id, subscriber_id);

    pin_mut!(bangumi_meta_stream);

    if let Some(Ok(bangumi)) = bangumi_meta_stream.next().await {
      assert!(bangumi.homepage.to_string().ends_with("/Home/Bangumi/3288#370"),);
      assert_eq!(bangumi.bangumi_title, "吉伊卡哇");
      assert_eq!(bangumi.mikan_bangumi_id, "3288");
      assert!(
        bangumi
          .origin_poster_src
          .as_ref()
          .map_or(String::new(), |u| u.to_string())
          .ends_with("/images/Bangumi/202204/d8ef46c0.jpg")
      );
      assert_eq!(bangumi.mikan_fansub_id, String::from("370"));
      assert_eq!(bangumi.fansub, String::from("LoliHouse"));
      Ok(())
    } else {
      panic!("bangumi stream should not be empty or error");
    }
  }

  #[rstest]
  #[tokio::test]
  async fn test_scrape_mikan_episode_meta_from_episode_homepage_url(before_each: ()) -> RecorderResult<()> {
    let mut mikan_server = MikanMockServer::new().await?;

    let mikan_base_url = mikan_server.base_url().clone();

    let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

    let resources_mock = mikan_server.mock_resources_with_doppel();

    let episode_homepage_url = MikanEpisodeHash {
      mikan_episode_id: "475184dce83ea2b82902592a5ac3343f6d54b36a".to_string(),
    }
    .build_homepage_url(mikan_base_url.clone());

    let episode_meta = scrape_mikan_episode_meta_from_episode_homepage_url(&mikan_client, episode_homepage_url.clone()).await?;

    assert_eq!(episode_meta.homepage, episode_homepage_url);
    assert_eq!(episode_meta.bangumi_title, "葬送的芙莉莲");
    assert_eq!(
      episode_meta.origin_poster_src.as_ref().map(|s| s.path().to_string()),
      Some(String::from("/images/Bangumi/202309/5ce9fed1.jpg"))
    );
    assert_eq!(episode_meta.fansub, "LoliHouse");
    assert_eq!(episode_meta.mikan_fansub_id, "370");
    assert_eq!(episode_meta.mikan_bangumi_id, "3141");

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_scrape_mikan_bangumi_meta_from_bangumi_homepage_url(before_each: ()) -> RecorderResult<()> {
    let mut mikan_server = MikanMockServer::new().await?;

    let mikan_base_url = mikan_server.base_url().clone();

    let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

    let resources_mock = mikan_server.mock_resources_with_doppel();

    let bangumi_homepage_url = MikanBangumiHash {
      mikan_bangumi_id: "3416".to_string(),
      mikan_fansub_id: "370".to_string(),
    }
    .build_homepage_url(mikan_base_url.clone());

    let bangumi_meta = scrape_mikan_bangumi_meta_from_bangumi_homepage_url(&mikan_client, bangumi_homepage_url.clone()).await?;

    assert_eq!(bangumi_meta.homepage, bangumi_homepage_url);
    assert_eq!(bangumi_meta.bangumi_title, "叹气的亡灵想隐退");
    assert_eq!(
      bangumi_meta.origin_poster_src.as_ref().map(|s| s.path().to_string()),
      Some(String::from("/images/Bangumi/202410/480ef127.jpg"))
    );
    assert_eq!(bangumi_meta.fansub, String::from("LoliHouse"));
    assert_eq!(bangumi_meta.mikan_fansub_id, String::from("370"));
    assert_eq!(bangumi_meta.mikan_bangumi_id, "3416");

    Ok(())
  }
}
