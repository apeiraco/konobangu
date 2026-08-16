use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct EpisodeEnclosureMeta {
  pub magnet_link: Option<String>,
  pub torrent_link: Option<String>,
  pub pub_date: Option<DateTime<Utc>>,
  pub content_length: Option<i64>,
}
