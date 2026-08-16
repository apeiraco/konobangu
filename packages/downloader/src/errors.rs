use std::{borrow::Cow, time::Duration};

use snafu::prelude::*;
use util::errors::OptDynErr;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum DownloaderError {
  #[snafu(transparent)]
  DownloadUrlParseError { source: url::ParseError },
  #[snafu(transparent)]
  QBitAPIError { source: qbit_rs::Error },
  #[snafu(transparent)]
  DownloaderIOError { source: std::io::Error },
  #[snafu(display("Timeout error (action = {action}, timeout = {timeout:?})"))]
  DownloadTimeoutError { action: Cow<'static, str>, timeout: Duration },
  #[snafu(display("Invalid magnet format ({message})"))]
  MagnetFormatError {
    message: String,
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
    source: OptDynErr,
  },
  #[snafu(display("Invalid torrent meta format ({message})"))]
  TorrentMetaError {
    message: String,
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
    source: OptDynErr,
  },
  #[snafu(display("Failed to fetch: {source}"))]
  DownloadFetchError {
    url: String,
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
    source: OptDynErr,
  },
  #[snafu(display("{source}"))]
  RqbitError {
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
    source: OptDynErr,
  },
  #[snafu(display("{message}"))]
  Whatever {
    message: String,
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
    source: OptDynErr,
  },
}

impl snafu::FromString for DownloaderError {
  type Source = Box<dyn std::error::Error + Send + Sync>;

  fn without_source(message: String) -> Self {
    Self::Whatever {
      message,
      source: OptDynErr::none(),
    }
  }

  fn with_source(source: Self::Source, message: String) -> Self {
    Self::Whatever {
      message,
      source: OptDynErr::some(source),
    }
  }
}
