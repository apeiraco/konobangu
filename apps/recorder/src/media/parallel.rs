//! Backend ownership stays here; business scheduling only uses par-core join.
use crate::errors::RecorderResult;
#[cfg(any(feature = "media-par-rayon", feature = "media-par-chili"))]
use crate::media::invalid_options;

#[derive(Debug)]
pub(super) struct Backend {
  #[cfg(feature = "media-par-rayon")]
  pool: rayon::ThreadPool,
}
impl Backend {
  pub fn new(threads: usize) -> RecorderResult<Self> {
    #[cfg(feature = "media-par-rayon")]
    return Ok(Self {
      pool: rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|n| format!("media-cpu-{n}"))
        .build()
        .map_err(|_| invalid_options("Media Rayon pool initialization failed"))?,
    });
    #[cfg(all(feature = "media-par-chili", not(feature = "media-par-rayon")))]
    {
      // Chili owns a process-global pool. Reuse only an identical
      // configuration; a second owner cannot silently resize it or select
      // a different budget.
      static CONFIG: std::sync::Mutex<Option<usize>> = std::sync::Mutex::new(None);
      let mut configured = CONFIG.lock().map_err(|_| invalid_options("Media Chili configuration lock poisoned"))?;
      if configured.is_some_and(|n| n != threads) {
        return Err(invalid_options(
          "Media Chili global pool already has a different concurrency; restart to change it",
        ));
      }
      if configured.is_none() {
        chili::ThreadPool::with_config(chili::Config {
          thread_count: std::num::NonZero::new(threads),
          ..Default::default()
        })
        .set_global()
        .map_err(|_| invalid_options("Media Chili global pool was initialized outside its owner"))?;
        *configured = Some(threads);
      }
      Ok(Self {})
    }
    #[cfg(not(any(feature = "media-par-rayon", feature = "media-par-chili")))]
    {
      let _ = threads;
      Ok(Self {})
    }
  }
  pub fn install(&self, operation: impl FnOnce() + Send) {
    #[cfg(feature = "media-par-rayon")]
    self.pool.install(operation);
    #[cfg(not(feature = "media-par-rayon"))]
    operation();
  }
  pub fn concurrency(requested: usize) -> usize {
    if cfg!(any(feature = "media-par-rayon", feature = "media-par-chili")) {
      requested
    } else {
      1
    }
  }
}
