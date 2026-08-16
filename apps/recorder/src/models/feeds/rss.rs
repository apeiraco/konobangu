use std::borrow::Cow;

use chrono::{DateTime, Utc};
use downloader::bittorrent::BITTORRENT_MIME_TYPE;
use maplit::btreemap;
use rss::{
  Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item, ItemBuilder,
  extension::{ExtensionBuilder, ExtensionMap},
};
use url::Url;

use crate::{
  app::AppContextTrait,
  errors::{RecorderError, RecorderResult},
};

pub trait RssFeedItemTrait: Sized {
  fn get_guid_value(&self) -> Cow<'_, str>;
  fn get_title(&self) -> Cow<'_, str>;
  fn get_description(&self) -> Cow<'_, str>;
  fn get_link(&self, ctx: &dyn AppContextTrait, api_base: &Url) -> Option<Cow<'_, str>>;
  fn get_enclosure_mime(&self) -> Option<Cow<'_, str>>;
  fn get_enclosure_link(&self, ctx: &dyn AppContextTrait, api_base: &Url) -> Option<Cow<'_, str>>;
  fn get_enclosure_pub_date(&self) -> Option<DateTime<Utc>>;
  fn get_enclosure_content_length(&self) -> Option<i64>;
  fn get_xmlns(&self) -> Cow<'_, str>;
  fn into_item(self, ctx: &dyn AppContextTrait, api_base: &Url) -> RecorderResult<Item> {
    let enclosure_mime_type = self.get_enclosure_mime().ok_or_else(|| RecorderError::MikanRssInvalidFieldError {
      field: "enclosure_mime_type".into(),
      source: None.into(),
    })?;
    let enclosure_link = self.get_enclosure_link(ctx, api_base).ok_or_else(|| RecorderError::MikanRssInvalidFieldError {
      field: "enclosure_link".into(),
      source: None.into(),
    })?;
    let enclosure_content_length = self.get_enclosure_content_length().ok_or_else(|| RecorderError::MikanRssInvalidFieldError {
      field: "enclosure_content_length".into(),
      source: None.into(),
    })?;
    let enclosure_pub_date = self.get_enclosure_pub_date();
    let link = self.get_link(ctx, api_base).ok_or_else(|| RecorderError::MikanRssInvalidFieldError {
      field: "link".into(),
      source: None.into(),
    })?;

    let mut extensions = ExtensionMap::default();
    if enclosure_mime_type == BITTORRENT_MIME_TYPE {
      let xmlns = self.get_xmlns();

      let torrent_extension = ExtensionBuilder::default()
        .name("torrent")
        .attrs(btreemap! {
            "xmlns".to_string() => xmlns.to_string()
        })
        .children({
          let mut m = btreemap! {
              "link".to_string() => vec![
                  ExtensionBuilder::default()
                      .name("link")
                      .value(link.to_string())
                      .build()
              ],
              "contentLength".to_string() => vec![
                  ExtensionBuilder::default()
                      .name("contentLength")
                      .value(enclosure_content_length.to_string())
                      .build()
              ]
          };
          if let Some(pub_date) = enclosure_pub_date {
            m.insert(
              "pubDate".to_string(),
              vec![ExtensionBuilder::default().name("pubDate").value(pub_date.to_rfc3339()).build()],
            );
          };
          m
        })
        .build();

      extensions.insert(
        "".to_string(),
        btreemap! {
            "torrent".to_string() => vec![torrent_extension]
        },
      );
    };

    let enclosure = EnclosureBuilder::default()
      .mime_type(enclosure_mime_type)
      .url(enclosure_link.to_string())
      .length(enclosure_content_length.to_string())
      .build();

    let guid = GuidBuilder::default().value(self.get_guid_value()).permalink(false).build();

    let item = ItemBuilder::default()
      .guid(guid)
      .title(self.get_title().to_string())
      .description(self.get_description().to_string())
      .link(link.to_string())
      .enclosure(enclosure)
      .extensions(extensions)
      .build();

    Ok(item)
  }
}

pub trait RssFeedTrait: Sized {
  type Item: RssFeedItemTrait;

  fn get_description(&self) -> Cow<'_, str>;

  fn get_title(&self) -> Cow<'_, str>;

  fn get_link(&self, ctx: &dyn AppContextTrait, api_base: &Url) -> Option<Cow<'_, str>>;

  fn items(&self) -> impl Iterator<Item = &Self::Item>;

  fn into_items(self) -> impl Iterator<Item = Self::Item>;

  fn into_channel(self, ctx: &dyn AppContextTrait, api_base: &Url) -> RecorderResult<Channel> {
    let link = self.get_link(ctx, api_base).ok_or_else(|| RecorderError::MikanRssInvalidFieldError {
      field: "link".into(),
      source: None.into(),
    })?;

    let channel = ChannelBuilder::default()
      .title(self.get_title())
      .link(link.to_string())
      .description(self.get_description())
      .items({
        self
          .into_items()
          .map(|item| item.into_item(ctx, api_base))
          .collect::<RecorderResult<Vec<_>>>()?
      })
      .build();

    Ok(channel)
  }
}
