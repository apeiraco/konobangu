use std::{
  collections::HashMap,
  fmt::Debug,
  ops::{Deref, DerefMut},
  path::{self, PathBuf},
};

use chrono::{Duration, Utc};
use fetch::{FetchError, HttpClientConfig, IntoUrl, get_random_ua};
use lazy_static::lazy_static;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
  crypto::UserPassCredential,
  errors::RecorderResult,
  extract::mikan::{
    MIKAN_ACCOUNT_MANAGE_PAGE_PATH, MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH, MIKAN_BANGUMI_HOMEPAGE_PATH, MIKAN_BANGUMI_POSTER_PATH, MIKAN_BANGUMI_RSS_PATH,
    MIKAN_EPISODE_HOMEPAGE_PATH, MIKAN_EPISODE_TORRENT_PATH, MIKAN_LOGIN_PAGE_PATH, MIKAN_SEASON_FLOW_PAGE_PATH, MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH,
    MikanClient, MikanConfig, MikanCredentialForm,
  },
};

const TESTING_MIKAN_USERNAME: &str = "test_username";
const TESTING_MIKAN_PASSWORD: &str = "test_password";
const TESTING_MIKAN_ANTIFORGERY: &str = "test_antiforgery";
const TESTING_MIKAN_IDENTITY: &str = "test_identity";

const FILE_UNSAFE: &AsciiSet = &CONTROLS
  .add(b'<')
  .add(b'>')
  .add(b':')
  .add(b'"')
  .add(b'|')
  .add(b'?')
  .add(b'*')
  .add(b'\\')
  .add(b'/')
  .add(b'&')
  .add(b'=')
  .add(b'#');

pub async fn build_testing_mikan_client(base_mikan_url: impl IntoUrl) -> RecorderResult<MikanClient> {
  let mikan_client = MikanClient::from_config(MikanConfig {
    http_client: HttpClientConfig::default(),
    base_url: base_mikan_url.into_url().map_err(FetchError::from)?,
  })
  .await?;
  Ok(mikan_client)
}

pub fn build_testing_mikan_credential_form() -> MikanCredentialForm {
  MikanCredentialForm {
    username: String::from(TESTING_MIKAN_USERNAME),
    password: String::from(TESTING_MIKAN_PASSWORD),
    user_agent: get_random_ua().to_string(),
  }
}

pub fn build_testing_mikan_credential() -> UserPassCredential {
  UserPassCredential {
    username: String::from(TESTING_MIKAN_USERNAME),
    password: String::from(TESTING_MIKAN_PASSWORD),
    user_agent: None,
    cookies: None,
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MikanDoppelMeta {
  pub status: u16,
}

pub struct MikanDoppelPath {
  path: path::PathBuf,
}

impl MikanDoppelPath {
  pub fn new(source: impl Into<Self>) -> Self {
    source.into()
  }

  pub fn exists_any(&self) -> bool {
    self.exists() || self.exists_meta()
  }

  pub fn exists(&self) -> bool {
    self.path().exists()
  }

  pub fn exists_meta(&self) -> bool {
    self.meta_path().exists()
  }

  pub fn write(&self, content: impl AsRef<[u8]>) -> std::io::Result<()> {
    if let Some(parent) = self.as_ref().parent() {
      std::fs::create_dir_all(parent)?;
    }
    std::fs::write(self.as_ref(), content)?;
    Ok(())
  }

  pub fn write_meta(&self, meta: MikanDoppelMeta) -> std::io::Result<()> {
    self.write(serde_json::to_string(&meta)?)
  }

  pub fn read(&self) -> std::io::Result<Vec<u8>> {
    let content = std::fs::read(self.as_ref())?;
    Ok(content)
  }

  pub fn read_meta(&self) -> std::io::Result<MikanDoppelMeta> {
    let content = std::fs::read(self.meta_path())?;
    Ok(serde_json::from_slice(&content)?)
  }

  pub fn encode_path_component(component: &str) -> String {
    utf8_percent_encode(component, FILE_UNSAFE).to_string()
  }

  pub fn decode_path_component(component: &str) -> Result<String, std::str::Utf8Error> {
    Ok(percent_decode(component.as_bytes()).decode_utf8()?.to_string())
  }

  pub fn meta_path(&self) -> path::PathBuf {
    let extension = if let Some(ext) = self.path().extension() {
      format!("{}.meta.json", ext.to_string_lossy())
    } else {
      String::from("meta.json")
    };
    self.path.to_path_buf().with_extension(extension)
  }

  pub fn path(&self) -> &path::Path {
    &self.path
  }
}

impl AsRef<path::Path> for MikanDoppelPath {
  fn as_ref(&self) -> &path::Path {
    self.path()
  }
}

#[cfg(any(test, debug_assertions, feature = "test-utils"))]
lazy_static! {
  static ref TEST_RESOURCES_DIR: String = format!("{}/tests/resources", env!("CARGO_MANIFEST_DIR"));
}

#[cfg(not(any(test, debug_assertions, feature = "test-utils")))]
lazy_static! {
  static ref TEST_RESOURCES_DIR: String = "tests/resources".to_string();
}

impl From<Url> for MikanDoppelPath {
  fn from(value: Url) -> Self {
    let doppel_path = PathBuf::from(format!("{}/mikan/doppel", TEST_RESOURCES_DIR.as_str()));
    let base_path = doppel_path.join(value.path().trim_matches('/'));
    let dirname = base_path.parent();
    let stem = base_path.file_stem();
    debug_assert!(dirname.is_some() && stem.is_some());
    let extension = if let Some(ext) = base_path.extension() {
      ext.to_string_lossy().to_string()
    } else {
      String::from("html")
    };
    let mut filename = stem.unwrap().to_string_lossy().to_string();
    if let Some(query) = value.query() {
      filename.push_str(&format!("-{}", Self::encode_path_component(query)));
    }
    filename.push_str(&format!(".{extension}"));

    Self {
      path: dirname.unwrap().join(filename),
    }
  }
}

pub struct MikanMockServerLoginMock {
  pub login_get_mock: mockito::Mock,
  pub login_post_success_mock: mockito::Mock,
  pub login_post_failed_mock: mockito::Mock,
  pub account_get_success_mock: mockito::Mock,
  pub account_get_failed_mock: mockito::Mock,
}

pub struct MikanMockServerResourcesMock {
  pub shared_resource_mock: mockito::Mock,
  pub shared_resource_not_found_mock: mockito::Mock,
  pub user_resource_mock: mockito::Mock,
  pub expand_bangumi_noauth_mock: mockito::Mock,
  pub season_flow_noauth_mock: mockito::Mock,
}

pub enum MikanMockServerInner {
  Server(mockito::Server),
  ServerGuard(mockito::ServerGuard),
}

impl Deref for MikanMockServerInner {
  type Target = mockito::Server;

  fn deref(&self) -> &Self::Target {
    match self {
      MikanMockServerInner::Server(server) => server,
      MikanMockServerInner::ServerGuard(server) => server,
    }
  }
}

impl DerefMut for MikanMockServerInner {
  fn deref_mut(&mut self) -> &mut Self::Target {
    match self {
      MikanMockServerInner::Server(server) => server,
      MikanMockServerInner::ServerGuard(server) => server,
    }
  }
}

pub struct MikanMockServer {
  pub server: MikanMockServerInner,
  base_url: Url,
}

impl Debug for MikanMockServer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MikanMockServer").field("base_url", &self.base_url).finish()
  }
}

impl MikanMockServer {
  pub async fn new_with_port(port: u16) -> RecorderResult<Self> {
    let server = mockito::Server::new_with_opts_async(mockito::ServerOpts {
      host: "0.0.0.0",
      port,
      ..Default::default()
    })
    .await;
    let base_url = Url::parse(&server.url())?;

    Ok(Self {
      server: MikanMockServerInner::Server(server),
      base_url,
    })
  }

  pub async fn new() -> RecorderResult<Self> {
    let server = mockito::Server::new_async().await;
    let base_url = Url::parse(&server.url())?;

    Ok(Self {
      server: MikanMockServerInner::ServerGuard(server),
      base_url,
    })
  }

  pub fn base_url(&self) -> &Url {
    &self.base_url
  }

  pub fn get_has_auth_matcher() -> impl Fn(&mockito::Request) -> bool {
    |req: &mockito::Request| -> bool {
      let test_identity_cookie = format!(".AspNetCore.Identity.Application={TESTING_MIKAN_IDENTITY}");
      req
        .header("Cookie")
        .iter()
        .any(|cookie| cookie.to_str().is_ok_and(|c| c.contains(&test_identity_cookie)))
    }
  }

  pub fn mock_get_login_page(&mut self) -> MikanMockServerLoginMock {
    let login_get_mock = self
      .server
      .mock("GET", MIKAN_LOGIN_PAGE_PATH)
      .match_query(mockito::Matcher::Any)
      .with_status(200)
      .with_header("Content-Type", "text/html; charset=utf-8")
      .with_header(
        "Set-Cookie",
        &format!(".AspNetCore.Antiforgery.test_app_id={TESTING_MIKAN_ANTIFORGERY}; HttpOnly; SameSite=Strict; Path=/"),
      )
      .with_body_from_file(format!("{}/mikan/LoginPage.html", TEST_RESOURCES_DIR.as_str()))
      .create();

    let test_identity_expires = (Utc::now() + Duration::days(30)).to_rfc2822();

    let match_post_login_body = |req: &mockito::Request| {
      req.body().map(|b| url::form_urlencoded::parse(b)).is_ok_and(|queires| {
        let qs = queires.collect::<HashMap<_, _>>();
        qs.get("UserName").is_some_and(|s| s == TESTING_MIKAN_USERNAME)
          && qs.get("Password").is_some_and(|s| s == TESTING_MIKAN_PASSWORD)
          && qs.get("__RequestVerificationToken").is_some_and(|s| s == TESTING_MIKAN_ANTIFORGERY)
      })
    };

    let login_post_success_mock = {
      let mikan_base_url = self.base_url().clone();
      self
        .server
        .mock("POST", MIKAN_LOGIN_PAGE_PATH)
        .match_query(mockito::Matcher::Any)
        .match_request(match_post_login_body)
        .with_status(302)
        .with_header(
          "Set-Cookie",
          &format!(".AspNetCore.Identity.Application={TESTING_MIKAN_IDENTITY}; HttpOnly; SameSite=Lax; Path=/; Expires=${test_identity_expires}"),
        )
        .with_header_from_request("Location", move |req| {
          let request_url = mikan_base_url.join(req.path_and_query()).ok();
          request_url
            .and_then(|u| u.query_pairs().find(|(key, _)| key == "ReturnUrl").map(|(_, value)| value.to_string()))
            .unwrap_or(String::from("/"))
        })
        .create()
    };

    let login_post_failed_mock = self
      .server
      .mock("POST", MIKAN_LOGIN_PAGE_PATH)
      .match_query(mockito::Matcher::Any)
      .match_request(move |req| !match_post_login_body(req))
      .with_status(200)
      .with_body_from_file(format!("{}/mikan/LoginError.html", TEST_RESOURCES_DIR.as_str()))
      .create();

    let account_get_success_mock = self
      .server
      .mock("GET", MIKAN_ACCOUNT_MANAGE_PAGE_PATH)
      .match_query(mockito::Matcher::Any)
      .match_request(move |req| Self::get_has_auth_matcher()(req))
      .with_status(200)
      .create();

    let account_get_failed_mock = self
      .server
      .mock("GET", MIKAN_ACCOUNT_MANAGE_PAGE_PATH)
      .match_query(mockito::Matcher::Any)
      .match_request(move |req| !Self::get_has_auth_matcher()(req))
      .with_status(302)
      .with_header("Location", MIKAN_LOGIN_PAGE_PATH)
      .create();

    MikanMockServerLoginMock {
      login_get_mock,
      login_post_success_mock,
      login_post_failed_mock,
      account_get_success_mock,
      account_get_failed_mock,
    }
  }

  pub fn mock_resources_with_doppel(&mut self) -> MikanMockServerResourcesMock {
    let shared_resource_mock = self
      .server
      .mock("GET", mockito::Matcher::Any)
      .match_request({
        let mikan_base_url = self.base_url().clone();
        move |request| {
          let path = request.path();
          if !path.starts_with(MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH)
            && !path.starts_with(MIKAN_SEASON_FLOW_PAGE_PATH)
            && (path.starts_with(MIKAN_BANGUMI_RSS_PATH)
              || path.starts_with(MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH)
              || path.starts_with(MIKAN_BANGUMI_HOMEPAGE_PATH)
              || path.starts_with(MIKAN_EPISODE_HOMEPAGE_PATH)
              || path.starts_with(MIKAN_BANGUMI_POSTER_PATH)
              || path.starts_with(MIKAN_EPISODE_TORRENT_PATH))
          {
            if let Ok(url) = mikan_base_url.join(request.path_and_query()) {
              let doppel_path = MikanDoppelPath::from(url);
              doppel_path.exists()
            } else {
              false
            }
          } else {
            false
          }
        }
      })
      .with_status(200)
      .with_body_from_request({
        let mikan_base_url = self.base_url().clone();
        move |req| {
          let path_and_query = req.path_and_query();
          let url = mikan_base_url.join(path_and_query).unwrap();
          let doppel_path = MikanDoppelPath::from(url);
          doppel_path.read().unwrap()
        }
      })
      .create();

    let shared_resource_not_found_mock = self
      .server
      .mock("GET", mockito::Matcher::Any)
      .match_request({
        let mikan_base_url = self.base_url().clone();
        move |request| {
          let path = request.path();
          if !path.starts_with(MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH)
            && !path.starts_with(MIKAN_SEASON_FLOW_PAGE_PATH)
            && (path.starts_with(MIKAN_BANGUMI_RSS_PATH)
              || path.starts_with(MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH)
              || path.starts_with(MIKAN_BANGUMI_HOMEPAGE_PATH)
              || path.starts_with(MIKAN_EPISODE_HOMEPAGE_PATH)
              || path.starts_with(MIKAN_BANGUMI_POSTER_PATH)
              || path.starts_with(MIKAN_EPISODE_TORRENT_PATH))
          {
            if let Ok(url) = mikan_base_url.join(request.path_and_query()) {
              let doppel_path = MikanDoppelPath::from(url);
              doppel_path.exists_meta() && doppel_path.read_meta().unwrap().status == 404
            } else {
              false
            }
          } else {
            false
          }
        }
      })
      .with_status(404)
      .create();

    let user_resource_mock = self
      .server
      .mock("GET", mockito::Matcher::Any)
      .match_request({
        let mikan_base_url = self.base_url().clone();
        move |req| {
          if !Self::get_has_auth_matcher()(req) {
            return false;
          }
          let path = req.path();
          if path.starts_with(MIKAN_SEASON_FLOW_PAGE_PATH) || path.starts_with(MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH) {
            if let Ok(url) = mikan_base_url.join(req.path_and_query()) {
              let doppel_path = MikanDoppelPath::from(url);
              doppel_path.exists()
            } else {
              false
            }
          } else {
            false
          }
        }
      })
      .with_status(200)
      .with_body_from_request({
        let mikan_base_url = self.base_url().clone();
        move |req| {
          let path_and_query = req.path_and_query();
          let url = mikan_base_url.join(path_and_query).unwrap();
          let doppel_path = MikanDoppelPath::from(url);
          doppel_path.read().unwrap()
        }
      })
      .create();

    let expand_bangumi_noauth_mock = self
      .server
      .mock("GET", mockito::Matcher::Any)
      .match_request(move |req| !Self::get_has_auth_matcher()(req) && req.path().starts_with(MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH))
      .with_status(200)
      .with_body_from_file(format!("{}/mikan/ExpandBangumi-noauth.html", TEST_RESOURCES_DIR.as_str()))
      .create();

    let season_flow_noauth_mock = self
      .server
      .mock("GET", mockito::Matcher::Any)
      .match_request(move |req| !Self::get_has_auth_matcher()(req) && req.path().starts_with(MIKAN_SEASON_FLOW_PAGE_PATH))
      .with_status(200)
      .with_body_from_file(format!("{}/mikan/BangumiCoverFlow-noauth.html", TEST_RESOURCES_DIR.as_str()))
      .create();

    MikanMockServerResourcesMock {
      shared_resource_mock,
      shared_resource_not_found_mock,
      user_resource_mock,
      expand_bangumi_noauth_mock,
      season_flow_noauth_mock,
    }
  }
}
