use std::{str::FromStr, time::Duration};

use color_eyre::{Result, eyre::OptionExt};
use fetch::{FetchError, HttpClientConfig, fetch_bytes, fetch_html, fetch_image, reqwest};
use inquire::{Password, Text, validator::Validation};
use recorder::{
  crypto::UserPassCredential,
  extract::mikan::{
    MikanClient, MikanConfig, MikanRssItemMeta, MikanRssRoot, build_mikan_bangumi_expand_subscribed_url,
    extract_mikan_bangumi_index_meta_list_from_season_flow_fragment, extract_mikan_bangumi_meta_from_expand_subscribed_fragment,
  },
  test_utils::mikan::{MikanDoppelMeta, MikanDoppelPath},
};
use scraper::Html;
use tokio::fs;
use url::Url;

async fn run_main() -> Result<()> {
  tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
  std::env::set_current_dir(std::path::Path::new("apps/recorder"))?;

  let mikan_scrape_client = MikanClient::from_config(MikanConfig {
    http_client: HttpClientConfig {
      exponential_backoff_max_retries: Some(3),
      leaky_bucket_max_tokens: Some(2),
      leaky_bucket_initial_tokens: Some(0),
      leaky_bucket_refill_tokens: Some(1),
      leaky_bucket_refill_interval: Some(Duration::from_millis(1000)),
      user_agent: Some(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0".to_string(),
      ),
      ..Default::default()
    },
    base_url: Url::parse("https://mikanani.me")?,
  })
  .await?;

  let username_validator = |input: &str| {
    if input.trim().is_empty() {
      Ok(Validation::Invalid("Username cannot be empty".into()))
    } else {
      Ok(Validation::Valid)
    }
  };
  let password_validator = |input: &str| {
    if input.trim().is_empty() {
      Ok(Validation::Invalid("Password cannot be empty".into()))
    } else {
      Ok(Validation::Valid)
    }
  };
  let username = Text::new("Please enter your mikan username:").with_validator(username_validator).prompt()?;
  let password = Password::new("Please enter your mikan password:")
    .without_confirmation()
    .with_display_mode(inquire::PasswordDisplayMode::Masked)
    .with_validator(password_validator)
    .prompt()?;

  let mikan_scrape_client = mikan_scrape_client
    .fork_with_userpass_credential(UserPassCredential {
      username,
      password,
      user_agent: None,
      cookies: None,
    })
    .await?;

  tracing::info!("Checking if logged in...");
  if !mikan_scrape_client.has_login().await? {
    tracing::info!("Logging in to mikan...");
    mikan_scrape_client.login().await?;
    tracing::info!("Logged in to mikan");
  }

  let mikan_base_url = mikan_scrape_client.base_url().clone();
  tracing::info!("Scraping season subscription...");
  let season_subscription = fs::read("tests/resources/mikan/BangumiCoverFlow-2025-spring.html").await?;
  let html = Html::parse_fragment(String::from_utf8(season_subscription)?.as_str());
  let bangumi_index_list = extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, &mikan_base_url);

  for bangumi_index in bangumi_index_list {
    let bangumi_meta = {
      let bangumi_expand_subscribed_url = build_mikan_bangumi_expand_subscribed_url(mikan_base_url.clone(), bangumi_index.mikan_bangumi_id.as_ref());
      let bangumi_expand_subscribed_doppel_path = MikanDoppelPath::new(bangumi_expand_subscribed_url.clone());
      tracing::info!(bangumi_title = bangumi_index.bangumi_title, "Scraping bangumi expand subscribed...");
      let bangumi_expand_subscribed_data = if !bangumi_expand_subscribed_doppel_path.exists_any() {
        let bangumi_expand_subscribed_data = fetch_html(&mikan_scrape_client, bangumi_expand_subscribed_url).await?;
        bangumi_expand_subscribed_doppel_path.write(&bangumi_expand_subscribed_data)?;
        tracing::info!(bangumi_title = bangumi_index.bangumi_title, "Bangumi expand subscribed saved");
        bangumi_expand_subscribed_data
      } else {
        tracing::info!(bangumi_title = bangumi_index.bangumi_title, "Bangumi expand subscribed already exists");
        String::from_utf8(bangumi_expand_subscribed_doppel_path.read()?)?
      };

      let html = Html::parse_fragment(&bangumi_expand_subscribed_data);
      extract_mikan_bangumi_meta_from_expand_subscribed_fragment(&html, bangumi_index.clone(), mikan_base_url.clone()).ok_or_eyre(format!(
        "Failed to extract bangumi meta from expand subscribed fragment: {:?}",
        bangumi_index.bangumi_title
      ))
    }?;
    {
      if let Some(poster_url) = bangumi_meta.origin_poster_src.as_ref() {
        let poster_doppel_path = MikanDoppelPath::new(poster_url.clone());
        tracing::info!(title = bangumi_meta.bangumi_title, "Scraping bangumi poster...");
        if !poster_doppel_path.exists_any() {
          let poster_data = fetch_image(&mikan_scrape_client, poster_url.clone()).await?;
          poster_doppel_path.write(&poster_data)?;
          tracing::info!(title = bangumi_meta.bangumi_title, "Bangumi poster saved");
        } else {
          tracing::info!(title = bangumi_meta.bangumi_title, "Bangumi poster already exists");
        }
      }
    }
    {
      let bangumi_homepage_url = bangumi_meta.bangumi_hash().build_homepage_url(mikan_base_url.clone());
      let bangumi_homepage_doppel_path = MikanDoppelPath::new(bangumi_homepage_url.clone());
      tracing::info!(title = bangumi_meta.bangumi_title, "Scraping bangumi homepage...");
      if !bangumi_homepage_doppel_path.exists_any() {
        let bangumi_homepage_data = fetch_html(&mikan_scrape_client, bangumi_homepage_url).await?;
        bangumi_homepage_doppel_path.write(&bangumi_homepage_data)?;
        tracing::info!(title = bangumi_meta.bangumi_title, "Bangumi homepage saved");
      } else {
        tracing::info!(title = bangumi_meta.bangumi_title, "Bangumi homepage already exists");
      }
    }
    let rss_items = {
      let bangumi_rss_url = bangumi_meta.bangumi_hash().build_rss_url(mikan_base_url.clone());
      let bangumi_rss_doppel_path = MikanDoppelPath::new(bangumi_rss_url.clone());
      tracing::info!(title = bangumi_meta.bangumi_title, "Scraping bangumi rss...");
      let bangumi_rss_data = if !bangumi_rss_doppel_path.exists_any() {
        let bangumi_rss_data = fetch_html(&mikan_scrape_client, bangumi_rss_url).await?;
        bangumi_rss_doppel_path.write(&bangumi_rss_data)?;
        tracing::info!(title = bangumi_meta.bangumi_title, "Bangumi rss saved");
        bangumi_rss_data
      } else {
        tracing::info!(title = bangumi_meta.bangumi_title, "Bangumi rss already exists");
        String::from_utf8(bangumi_rss_doppel_path.read()?)?
      };
      let rss_items = MikanRssRoot::from_str(&bangumi_rss_data)?.channel.items;
      rss_items.into_iter().map(MikanRssItemMeta::try_from).collect::<Result<Vec<_>, _>>()
    }?;
    for rss_item in rss_items {
      {
        let episode_homepage_url = rss_item.build_homepage_url(mikan_base_url.clone());
        let episode_homepage_doppel_path = MikanDoppelPath::new(episode_homepage_url.clone());
        tracing::info!(title = rss_item.title, "Scraping episode...");
        if !episode_homepage_doppel_path.exists_any() {
          let episode_homepage_data = fetch_html(&mikan_scrape_client, episode_homepage_url).await?;
          episode_homepage_doppel_path.write(&episode_homepage_data)?;
          tracing::info!(title = rss_item.title, "Episode saved");
        } else {
          tracing::info!(title = rss_item.title, "Episode already exists");
        };
      }
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
        } else {
          tracing::info!(title = rss_item.title, "Episode torrent already exists");
        }
      }
    }
  }
  tracing::info!("Scraping season subscription done");
  Ok(())
}

fn main() -> Result<()> {
  tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run_main())
}
