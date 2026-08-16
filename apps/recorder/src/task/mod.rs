mod config;
mod core;
pub mod delivery;
pub mod execution;
mod registry;
mod service;

pub use core::{
  AsyncTaskTrait, SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_TRIGGER_NAME, SUBSCRIBER_TASK_APALIS_NAME,
  SYSTEM_TASK_APALIS_NAME, StreamTaskTrait, SubscriberTaskTrait, SystemTaskTrait,
};

pub use config::TaskConfig;
pub use registry::{
  EchoTask, OptimizeImageTask, SubscriberTask, SubscriberTaskInput, SubscriberTaskType, SubscriberTaskTypeEnum, SubscriberTaskTypeVariant,
  SubscriberTaskTypeVariantIter, SyncOneSubscriptionFeedsFullTask, SyncOneSubscriptionFeedsIncrementalTask, SyncOneSubscriptionSourcesTask, SystemTask,
  SystemTaskInput, SystemTaskType, SystemTaskTypeEnum, SystemTaskTypeVariant, SystemTaskTypeVariantIter,
};
#[allow(unused_imports)]
pub(crate) use registry::{register_subscriber_task_type, register_system_task_type};
pub use service::TaskService;

pub mod operation;
