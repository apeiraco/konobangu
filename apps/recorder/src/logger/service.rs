use std::sync::OnceLock;

use snafu::prelude::*;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use super::{LogFormat, LogLevel, LogRotation, LoggerConfig};
use crate::errors::RecorderResult;

// Function to initialize the logger based on the provided configuration
pub const MODULE_WHITELIST: &[&str] = &["sea_orm_migration", "tower_http", "sea_orm", "sea_query"];

// Keep nonblocking file appender work guard
static NONBLOCKING_WORK_GUARD_KEEP: OnceLock<WorkerGuard> = OnceLock::new();

pub struct LoggerService {}

impl LoggerService {
    pub fn init_layer<W2>(
        make_writer: W2,
        format: &LogFormat,
        ansi: bool,
    ) -> Box<dyn Layer<Registry> + Sync + Send>
    where
        W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
    {
        match format {
            LogFormat::Compact => fmt::Layer::default()
                .with_ansi(ansi)
                .with_writer(make_writer)
                .compact()
                .boxed(),
            LogFormat::Pretty => fmt::Layer::default()
                .with_ansi(ansi)
                .with_writer(make_writer)
                .pretty()
                .boxed(),
            LogFormat::Json => fmt::Layer::default()
                .with_ansi(ansi)
                .with_writer(make_writer)
                .json()
                .boxed(),
        }
    }

    fn init_env_filter(override_filter: Option<&String>, level: &LogLevel) -> EnvFilter {
        EnvFilter::try_from_default_env()
            .or_else(|_| {
                // user wanted a specific filter, don't care about our internal whitelist
                // or, if no override give them the default whitelisted filter (most common)
                override_filter.map_or_else(
                    || {
                        EnvFilter::try_new(
                            MODULE_WHITELIST
                                .iter()
                                .map(|m| format!("{m}={level}"))
                                .chain(std::iter::once(format!(
                                    "{}={}",
                                    env!("CARGO_CRATE_NAME"),
                                    level
                                )))
                                .collect::<Vec<_>>()
                                .join(","),
                        )
                    },
                    EnvFilter::try_new,
                )
            })
            .expect("logger initialization failed")
    }

    pub async fn from_config(config: LoggerConfig) -> RecorderResult<Self> {
        let mut layers: Vec<Box<dyn Layer<Registry> + Sync + Send>> = Vec::new();

        if let Some(file_appender_config) = config.file_appender.as_ref()
            && file_appender_config.enable
        {
            let dir = file_appender_config
                .dir
                .as_ref()
                .map_or_else(|| "./logs".to_string(), ToString::to_string);

            let mut rolling_builder = tracing_appender::rolling::Builder::default()
                .max_log_files(file_appender_config.max_log_files);

            rolling_builder = match file_appender_config.rotation {
                LogRotation::Minutely => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::MINUTELY)
                }
                LogRotation::Hourly => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::HOURLY)
                }
                LogRotation::Daily => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::DAILY)
                }
                LogRotation::Never => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::NEVER)
                }
            };

            let file_appender = rolling_builder
                .filename_prefix(
                    file_appender_config
                        .filename_prefix
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                )
                .filename_suffix(
                    file_appender_config
                        .filename_suffix
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                )
                .build(dir)?;

            let file_appender_layer = if file_appender_config.non_blocking {
                let (non_blocking_file_appender, work_guard) =
                    tracing_appender::non_blocking(file_appender);
                if NONBLOCKING_WORK_GUARD_KEEP.set(work_guard).is_err() {
                    whatever!("cannot lock for appender");
                };
                Self::init_layer(
                    non_blocking_file_appender,
                    &file_appender_config.format,
                    false,
                )
            } else {
                Self::init_layer(file_appender, &file_appender_config.format, false)
            };
            layers.push(file_appender_layer);
        }

        if config.enable {
            let stdout_layer = Self::init_layer(std::io::stdout, &config.format, true);
            layers.push(stdout_layer);
        }

        if !layers.is_empty() {
            let env_filter = Self::init_env_filter(config.override_filter.as_ref(), &config.level);
            let _ = tracing_subscriber::registry()
                .with(layers)
                .with(env_filter)
                .try_init();
        }

        if config.pretty_backtrace {
            unsafe {
                std::env::set_var("RUST_BACKTRACE", "1");
            }
            tracing::warn!(
                "pretty backtraces are enabled (this is great for development but has a runtime \
                 cost for production. disable with `logger.pretty_backtrace` in your config yaml)"
            );
        }

        Ok(Self {})
    }
}
