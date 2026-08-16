use std::{any::Any, borrow::Cow, fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref, time::Duration, vec::IntoIter};

use async_trait::async_trait;

use super::DownloaderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadSimpleState {
  Paused,
  Active,
  Completed,
  Error,
  Unknown,
}

pub trait DownloadStateTrait: Sized + Debug {
  fn to_download_state(&self) -> DownloadSimpleState;
}

pub trait DownloadIdTrait: Hash + Sized + Clone + Send + Debug {}

pub trait DownloadTaskTrait: Sized + Send + Debug {
  type State: DownloadStateTrait;
  type Id: DownloadIdTrait;

  fn id(&self) -> &Self::Id;
  fn into_id(self) -> Self::Id;
  fn name(&self) -> Cow<'_, str>;
  fn speed(&self) -> Option<u64>;
  fn state(&self) -> &Self::State;
  fn dl_bytes(&self) -> Option<u64>;
  fn total_bytes(&self) -> Option<u64>;
  fn left_bytes(&self) -> Option<u64> {
    if let (Some(tt), Some(dl)) = (self.total_bytes(), self.dl_bytes()) {
      tt.checked_sub(dl)
    } else {
      None
    }
  }
  fn et(&self) -> Option<Duration>;
  fn eta(&self) -> Option<Duration> {
    if let (Some(left_bytes), Some(speed)) = (self.left_bytes(), self.speed()) {
      if speed > 0 {
        Some(Duration::from_secs_f64(left_bytes as f64 / speed as f64))
      } else {
        None
      }
    } else {
      None
    }
  }
  fn average_speed(&self) -> Option<f64> {
    if let (Some(et), Some(dl_bytes)) = (self.et(), self.dl_bytes()) {
      let secs = et.as_secs_f64();

      if secs > 0.0 { Some(dl_bytes as f64 / secs) } else { None }
    } else {
      None
    }
  }
  fn progress(&self) -> Option<f32> {
    if let (Some(dl), Some(tt)) = (self.dl_bytes(), self.total_bytes()) {
      if dl > 0 {
        if tt > 0 { Some(dl as f32 / tt as f32) } else { None }
      } else {
        Some(0.0)
      }
    } else {
      None
    }
  }
}

pub trait DownloadCreationTrait: Sized {
  type Task: DownloadTaskTrait;
}

pub trait DownloadSelectorTrait: Sized + Any + Send {
  type Id: DownloadIdTrait;
  type Task: DownloadTaskTrait<Id = Self::Id>;

  fn try_into_ids_only(self) -> Result<Vec<Self::Id>, Self> {
    Err(self)
  }
}

pub trait DownloadIdSelectorTrait:
  DownloadSelectorTrait + IntoIterator<Item = Self::Id> + FromIterator<Self::Id> + Into<Vec<Self::Id>> + From<Vec<Self::Id>>
{
  fn try_into_ids_only(self) -> Result<Vec<Self::Id>, Self> {
    Ok(Vec::from_iter(self))
  }

  fn from_id(id: Self::Id) -> Self;
}

#[derive(Debug)]
pub struct DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait,
{
  pub ids: Vec<Task::Id>,
  pub marker: PhantomData<Task>,
}

impl<Task> Deref for DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait,
{
  type Target = Vec<Task::Id>;

  fn deref(&self) -> &Self::Target {
    &self.ids
  }
}

impl<Task> IntoIterator for DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait,
{
  type Item = Task::Id;
  type IntoIter = IntoIter<Task::Id>;

  fn into_iter(self) -> Self::IntoIter {
    self.ids.into_iter()
  }
}

impl<Task> FromIterator<Task::Id> for DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait,
{
  fn from_iter<T: IntoIterator<Item = Task::Id>>(iter: T) -> Self {
    Self {
      ids: Vec::from_iter(iter),
      marker: PhantomData,
    }
  }
}

impl<Task> DownloadSelectorTrait for DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait + 'static,
{
  type Id = Task::Id;
  type Task = Task;
}

impl<Task> From<Vec<Task::Id>> for DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait + 'static,
{
  fn from(value: Vec<Task::Id>) -> Self {
    Self {
      ids: value,
      marker: PhantomData,
    }
  }
}

impl<Task> From<DownloadIdSelector<Task>> for Vec<Task::Id>
where
  Task: DownloadTaskTrait + 'static,
{
  fn from(value: DownloadIdSelector<Task>) -> Self {
    value.ids
  }
}

impl<Task> DownloadIdSelectorTrait for DownloadIdSelector<Task>
where
  Task: DownloadTaskTrait + 'static,
{
  fn try_into_ids_only(self) -> Result<Vec<Self::Id>, Self> {
    Ok(self.ids)
  }

  fn from_id(id: Self::Id) -> Self {
    Self {
      ids: vec![id],
      marker: PhantomData,
    }
  }
}

#[async_trait]
pub trait DownloaderTrait {
  type State: DownloadStateTrait;
  type Id: DownloadIdTrait;
  type Task: DownloadTaskTrait<State = Self::State, Id = Self::Id>;
  type Creation: DownloadCreationTrait<Task = Self::Task>;
  type Selector: DownloadSelectorTrait<Task = Self::Task>;

  async fn add_downloads(&self, creation: Self::Creation) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError>;
  async fn pause_downloads(&self, selector: Self::Selector) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError>;
  async fn resume_downloads(&self, selector: Self::Selector) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError>;
  async fn remove_downloads(&self, selector: Self::Selector) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError>;
  async fn query_downloads(&self, selector: Self::Selector) -> Result<impl IntoIterator<Item = Self::Task>, DownloaderError>;
}
