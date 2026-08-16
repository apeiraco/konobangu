use std::{str::FromStr, time::Duration};

use fetch::{FetchError, HttpClientConfig, fetch_bytes, fetch_html, fetch_image, reqwest};
use recorder::{
  errors::RecorderResult,
  extract::mikan::{MikanClient, MikanConfig, MikanRssItemMeta, MikanRssRoot, extract_mikan_episode_meta_from_episode_homepage_html},
  test_utils::mikan::{MikanDoppelMeta, MikanDoppelPath},
};
use scraper::Html;
use tokio::fs;
use url::Url;

async fn run_main() -> RecorderResult<()> {
  tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
  std::env::set_current_dir(std::path::Path::new("apps/recorder"))?;

  let mikan_scrape_client = MikanClient::from_config(MikanConfig {
    http_client: HttpClientConfig {
      exponential_backoff_max_retries: Some(3),
      leaky_bucket_max_tokens: Some(2),
      leaky_bucket_initial_tokens: Some(0),
      leaky_bucket_refill_tokens: Some(1),
      leaky_bucket_refill_interval: Some(Duration::from_millis(500)),
      user_agent: Some(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0".to_string(),
      ),
      ..Default::default()
    },
    base_url: Url::parse("https://mikanani.me")?,
  })
  .await?;

  let mikan_base_url = mikan_scrape_client.base_url().clone();
  tracing::info!("Scraping subscriber subscription...");
  let subscriber_subscription = fs::read_to_string("tests/resources/mikan/doppel/RSS/MyBangumi-token%3Dtest.html").await?;
  let channel = MikanRssRoot::from_str(&subscriber_subscription)?.channel;
  let rss_items: Vec<MikanRssItemMeta> = channel.items.into_iter().map(MikanRssItemMeta::try_from).collect::<Result<Vec<_>, _>>()?;
  for rss_item in rss_items {
    let episode_homepage_meta = {
      tracing::info!(title = rss_item.title, "Scraping episode homepage...");
      let episode_homepage_url = rss_item.build_homepage_url(mikan_base_url.clone());
      let episode_homepage_doppel_path = MikanDoppelPath::new(episode_homepage_url.clone());
      let episode_homepage_data = if !episode_homepage_doppel_path.exists_any() {
        let episode_homepage_data = fetch_html(&mikan_scrape_client, episode_homepage_url.clone()).await?;
        episode_homepage_doppel_path.write(&episode_homepage_data)?;
        tracing::info!(title = rss_item.title, "Episode homepage saved");
        episode_homepage_data
      } else {
        tracing::info!(title = rss_item.title, "Episode homepage already exists");
        String::from_utf8(episode_homepage_doppel_path.read()?)?
      };
      let html = Html::parse_document(&episode_homepage_data);
      extract_mikan_episode_meta_from_episode_homepage_html(&html, mikan_base_url.clone(), episode_homepage_url)
    }?;

    {
      let episode_torrent_url = rss_item.torrent_link;
      let episode_torrent_doppel_path = MikanDoppelPath::new(episode_torrent_url.clone());
      tracing::info!(title = rss_item.title, "Scraping episode torrent...");
      if !episode_torrent_doppel_path.exists_any() {
        match fetch_bytes(&mikan_scrape_client, episode_torrent_url).await {
          Ok(episode_torrent_data) => {
            episode_torrent_doppel_path.write(&episode_torrent_data)?;
            tracing::info!(title = rss_item.title, "Episode torrent saved");
          }
          Err(e) => {
            if let FetchError::ReqwestError { source } = &e
              && source.status().is_some_and(|status| status == reqwest::StatusCode::NOT_FOUND)
            {
              tracing::warn!(title = rss_item.title, "Episode torrent not found, maybe deleted since new version");
              episode_torrent_doppel_path.write_meta(MikanDoppelMeta { status: 404 })?;
            } else {
              Err(e)?;
            }
          }
        }

        tracing::info!(title = rss_item.title, "Episode torrent saved");
      } else {
        tracing::info!(title = rss_item.title, "Episode torrent already exists");
      }
    }
    {
      if let Some(episode_poster_url) = episode_homepage_meta.origin_poster_src.as_ref() {
        let episode_poster_doppel_path = MikanDoppelPath::new(episode_poster_url.clone());
        tracing::info!(title = rss_item.title, "Scraping episode poster...");
        if !episode_poster_doppel_path.exists_any() {
          let episode_poster_data = fetch_image(&mikan_scrape_client, episode_poster_url.clone()).await?;
          episode_poster_doppel_path.write(&episode_poster_data)?;
          tracing::info!(title = rss_item.title, "Episode poster saved");
        } else {
          tracing::info!(title = rss_item.title, "Episode poster already exists");
        }
      }
    }

    {
      let bangumi_homepage_url = episode_homepage_meta.bangumi_hash().build_homepage_url(mikan_base_url.clone());
      let bangumi_homepage_doppel_path = MikanDoppelPath::new(bangumi_homepage_url.clone());
      tracing::info!(title = rss_item.title, "Scraping bangumi homepage...");
      if !bangumi_homepage_doppel_path.exists_any() {
        let bangumi_homepage_data = fetch_html(&mikan_scrape_client, bangumi_homepage_url).await?;
        bangumi_homepage_doppel_path.write(&bangumi_homepage_data)?;
        tracing::info!(title = rss_item.title, "Bangumi homepage saved");
      } else {
        tracing::info!(title = rss_item.title, "Bangumi homepage already exists");
      };
    }
    {
      let bangumi_rss_url = episode_homepage_meta.bangumi_hash().build_rss_url(mikan_base_url.clone());
      let bangumi_rss_doppel_path = MikanDoppelPath::new(bangumi_rss_url.clone());
      tracing::info!(title = rss_item.title, "Scraping bangumi rss...");
      let bangumi_rss_data = if !bangumi_rss_doppel_path.exists_any() {
        let bangumi_rss_data = fetch_html(&mikan_scrape_client, bangumi_rss_url).await?;
        bangumi_rss_doppel_path.write(&bangumi_rss_data)?;
        tracing::info!(title = rss_item.title, "Bangumi rss saved");
        bangumi_rss_data
      } else {
        tracing::info!(title = rss_item.title, "Bangumi rss already exists");
        String::from_utf8(bangumi_rss_doppel_path.read()?)?
      };

      let rss_items: Vec<MikanRssItemMeta> = MikanRssRoot::from_str(&bangumi_rss_data)?
        .channel
        .items
        .into_iter()
        .map(MikanRssItemMeta::try_from)
        .collect::<Result<Vec<_>, _>>()?;
      for rss_item in rss_items {
        {
          tracing::info!(title = rss_item.title, "Scraping episode homepage...");
          let episode_homepage_url = rss_item.build_homepage_url(mikan_base_url.clone());
          let episode_homepage_doppel_path = MikanDoppelPath::new(episode_homepage_url.clone());
          if !episode_homepage_doppel_path.exists_any() {
            let episode_homepage_data = fetch_html(&mikan_scrape_client, episode_homepage_url.clone()).await?;
            episode_homepage_doppel_path.write(&episode_homepage_data)?;
            tracing::info!(title = rss_item.title, "Episode homepage saved");
          } else {
            tracing::info!(title = rss_item.title, "Episode homepage already exists");
          };
        };

        {
          let episode_torrent_url = rss_item.torrent_link;
          let episode_torrent_doppel_path = MikanDoppelPath::new(episode_torrent_url.clone());
          tracing::info!(title = rss_item.title, "Scraping episode torrent...");
          if !episode_torrent_doppel_path.exists_any() {
            match fetch_bytes(&mikan_scrape_client, episode_torrent_url).await {
              Ok(episode_torrent_data) => {
                episode_torrent_doppel_path.write(&episode_torrent_data)?;
                tracing::info!(title = rss_item.title, "Episode torrent saved");
              }
              Err(e) => {
                if let FetchError::ReqwestError { source } = &e
                  && source.status().is_some_and(|status| status == reqwest::StatusCode::NOT_FOUND)
                {
                  tracing::warn!(title = rss_item.title, "Episode torrent not found, maybe deleted since new version");
                  episode_torrent_doppel_path.write_meta(MikanDoppelMeta { status: 404 })?;
                } else {
                  Err(e)?;
                }
              }
            }

            tracing::info!(title = rss_item.title, "Episode torrent saved");
          } else {
            tracing::info!(title = rss_item.title, "Episode torrent already exists");
          }
        }
      }
    }
  }
  tracing::info!("Scraping subscriber subscription done");
  Ok(())
}

fn main() -> RecorderResult<()> {
  tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run_main())
}
