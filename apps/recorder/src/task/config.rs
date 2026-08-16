use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "SchedulerWire", into = "SchedulerWire")]
pub struct TaskConfig {
  #[serde(default)]
  pub queue_database_uri: Option<String>,
  #[serde(default = "default_subscriber_task_workers")]
  pub subscriber_task_concurrency: u32,
  #[serde(default = "default_system_task_workers")]
  pub system_task_concurrency: u32,
  #[serde(default = "default_cron_interval_duration")]
  pub cron_interval_duration: Duration,
}

impl Default for TaskConfig {
  fn default() -> Self {
    Self {
      queue_database_uri: None,
      subscriber_task_concurrency: default_subscriber_task_workers(),
      system_task_concurrency: default_system_task_workers(),
      cron_interval_duration: default_cron_interval_duration(),
    }
  }
}

pub fn default_subscriber_task_workers() -> u32 {
  if cfg!(test) {
    1
  } else {
    ((num_cpus::get_physical() as f32 / 2.0).floor() as u32).max(1)
  }
}

pub fn default_system_task_workers() -> u32 {
  if cfg!(test) {
    1
  } else {
    ((num_cpus::get_physical() as f32 / 2.0).floor() as u32).max(1)
  }
}

pub fn default_cron_interval_duration() -> Duration {
  Duration::from_secs(30)
}

impl std::fmt::Debug for TaskConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug = f.debug_struct("TaskConfig");
    debug.field("subscriber_task_concurrency", &self.subscriber_task_concurrency);
    debug.field("system_task_concurrency", &self.system_task_concurrency);
    debug.field("cron_interval_duration", &self.cron_interval_duration);
    debug.field("queue_database_uri", &"[REDACTED]");
    debug.finish()
  }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct SchedulerWire {
  database: SchedulerDatabaseWire,
  workers: WorkersWire,
  cron: CronWire,
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct SchedulerDatabaseWire {
  url: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct WorkersWire {
  subscriber_concurrency: u32,
  system_concurrency: u32,
}
impl Default for WorkersWire {
  fn default() -> Self {
    Self {
      subscriber_concurrency: default_subscriber_task_workers(),
      system_concurrency: default_system_task_workers(),
    }
  }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct CronWire {
  poll_interval_seconds: u64,
}
impl Default for CronWire {
  fn default() -> Self {
    Self { poll_interval_seconds: 30 }
  }
}
impl From<SchedulerWire> for TaskConfig {
  fn from(w: SchedulerWire) -> Self {
    Self {
      queue_database_uri: w.database.url,
      subscriber_task_concurrency: w.workers.subscriber_concurrency,
      system_task_concurrency: w.workers.system_concurrency,
      cron_interval_duration: Duration::from_secs(w.cron.poll_interval_seconds),
    }
  }
}
impl From<TaskConfig> for SchedulerWire {
  fn from(c: TaskConfig) -> Self {
    Self {
      database: SchedulerDatabaseWire { url: c.queue_database_uri },
      workers: WorkersWire {
        subscriber_concurrency: c.subscriber_task_concurrency,
        system_concurrency: c.system_task_concurrency,
      },
      cron: CronWire {
        poll_interval_seconds: c.cron_interval_duration.as_secs(),
      },
    }
  }
}
