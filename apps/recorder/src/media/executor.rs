//! One bounded queue, one coordinator, and a shared admission budget.
use std::{
  panic::{AssertUnwindSafe, catch_unwind},
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
  },
  thread::JoinHandle,
};

use bytes::Bytes;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc, oneshot};

use super::parallel::Backend;
use crate::{
  errors::RecorderResult,
  media::{MediaConfig, invalid_options},
};

type Operation = Box<dyn FnOnce(&AtomicBool) -> RecorderResult<Bytes> + Send>;
struct Job {
  operation: Operation,
  cancelled: Arc<AtomicBool>,
  response: oneshot::Sender<RecorderResult<Bytes>>,
  _budget: OwnedSemaphorePermit,
}
#[derive(Debug)]
struct State {
  stopped: AtomicBool,
  active: AtomicUsize,
  concurrency: usize,
}
#[derive(Debug)]
pub struct MediaExecutor {
  sender: Mutex<Option<mpsc::Sender<Job>>>,
  thread: Mutex<Option<JoinHandle<()>>>,
  budget: Arc<Semaphore>,
  budget_limit: u32,
  state: Arc<State>,
}
impl MediaExecutor {
  pub fn new(config: &MediaConfig) -> RecorderResult<Self> {
    config.validate()?;
    let concurrency = Backend::concurrency(config.encode_concurrency);
    let backend = Backend::new(concurrency)?;
    let state = Arc::new(State {
      stopped: AtomicBool::new(false),
      active: AtomicUsize::new(0),
      concurrency,
    });
    let (sender, mut receiver) = mpsc::channel(config.encode_queue_capacity);
    let owner = state.clone();
    let thread = std::thread::Builder::new()
      .name("media-coordinator".into())
      .spawn(move || {
        while let Some(job) = receiver.blocking_recv() {
          let mut batch = vec![job];
          // Dispatch whatever is ready. Waiting for a full batch delays a lone
          // job.
          while batch.len() < concurrency {
            match receiver.try_recv() {
              Ok(job) => batch.push(job),
              Err(_) => break,
            }
          }
          backend.install(|| run_batch(batch, &owner));
        }
      })
      .map_err(|_| invalid_options("Media coordinator initialization failed"))?;
    Ok(Self {
      sender: Mutex::new(Some(sender)),
      thread: Mutex::new(Some(thread)),
      budget: Arc::new(Semaphore::new(config.encode_working_set_bytes as usize)),
      budget_limit: config.encode_working_set_bytes as u32,
      state,
    })
  }
  pub(super) async fn execute(&self, estimated_bytes: u64, cancelled: Arc<AtomicBool>, operation: Operation) -> RecorderResult<Bytes> {
    let weight = u32::try_from(estimated_bytes)
      .ok()
      .filter(|n| *n <= self.budget_limit)
      .ok_or_else(|| invalid_options("Image estimated working set exceeds shared admission budget"))?;
    let budget = self
      .budget
      .clone()
      .try_acquire_many_owned(weight)
      .map_err(|_| invalid_options("Image encoder admission budget unavailable (closed or backpressure)"))?;
    if self.state.stopped.load(Ordering::Acquire) {
      return Err(invalid_options("Image encoder is closed"));
    }
    let sender = self
      .sender
      .lock()
      .map_err(|_| invalid_options("Media queue lock poisoned"))?
      .clone()
      .ok_or_else(|| invalid_options("Image encoder is closed"))?;
    let (response, receiver) = oneshot::channel();
    sender
      .send(Job {
        operation,
        cancelled,
        response,
        _budget: budget,
      })
      .await
      .map_err(|_| invalid_options("Image encoder is closed"))?;
    receiver.await.map_err(|_| invalid_options("Media coordinator stopped without a result"))?
  }
  pub fn available_permits(&self) -> usize {
    self.state.concurrency.saturating_sub(self.state.active.load(Ordering::Acquire))
  }
  pub async fn shutdown(&self) -> RecorderResult<()> {
    self.stop();
    let thread = self.thread.lock().map_err(|_| invalid_options("Media thread lock poisoned"))?.take();
    if let Some(thread) = thread {
      tokio::task::spawn_blocking(move || thread.join())
        .await
        .map_err(|_| invalid_options("Media shutdown wait failed"))?
        .map_err(|_| invalid_options("Media coordinator panicked"))?;
    }
    Ok(())
  }
  fn stop(&self) {
    self.state.stopped.store(true, Ordering::Release);
    self.budget.close();
    if let Ok(mut sender) = self.sender.lock() {
      sender.take();
    }
  }
}
impl Drop for MediaExecutor {
  fn drop(&mut self) {
    self.stop();
    // Explicit application shutdown joins off Tokio. Dropping an embedding
    // service still drains its coordinator; capacity is never handed to a new
    // pool.
    if let Ok(thread) = self.thread.get_mut()
      && let Some(thread) = thread.take()
    {
      let _ = thread.join();
    }
  }
}
fn run_batch(mut jobs: Vec<Job>, state: &State) {
  if jobs.len() == 1 {
    run_leaf(jobs.pop().unwrap(), state);
    return;
  }
  let right = jobs.split_off(jobs.len() / 2);
  par_core::join(
    || {
      let _ = catch_unwind(AssertUnwindSafe(|| run_batch(jobs, state)));
    },
    || {
      let _ = catch_unwind(AssertUnwindSafe(|| run_batch(right, state)));
    },
  );
}
fn run_leaf(job: Job, state: &State) {
  if state.stopped.load(Ordering::Acquire) || job.cancelled.load(Ordering::Acquire) || job.response.is_closed() {
    let _ = job.response.send(Err(invalid_options("Media encoding cancelled")));
    return;
  }
  state.active.fetch_add(1, Ordering::AcqRel);
  let result = catch_unwind(AssertUnwindSafe(|| (job.operation)(&job.cancelled))).unwrap_or_else(|_| Err(invalid_options("Media encoding panicked")));
  // A codec owns admission until it actually returns, even if its caller left.
  state.active.fetch_sub(1, Ordering::AcqRel);
  let result = if state.stopped.load(Ordering::Acquire) || job.cancelled.load(Ordering::Acquire) || job.response.is_closed() {
    Err(invalid_options("Media encoding cancelled; late result rejected"))
  } else {
    result
  };
  let _ = job.response.send(result);
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::*;

  async fn wait_for(mut predicate: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(10), async {
      while !predicate() {
        tokio::task::yield_now().await;
      }
    })
    .await
    .unwrap();
  }
  fn blocked(
    executor: Arc<MediaExecutor>,
    weight: u64,
  ) -> (
    tokio::task::JoinHandle<RecorderResult<Bytes>>,
    oneshot::Receiver<()>,
    std::sync::mpsc::Sender<()>,
  ) {
    let (started, ready) = oneshot::channel();
    let (release, wait) = std::sync::mpsc::channel();
    let task = tokio::spawn(async move {
      executor
        .execute(
          weight,
          Arc::new(AtomicBool::new(false)),
          Box::new(move |_| {
            let _ = started.send(());
            wait.recv().unwrap();
            Ok(Bytes::from_static(b"done"))
          }),
        )
        .await
    });
    (task, ready, release)
  }
  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn bounded_lifecycle_backpressure_cancel_deadline_late_panic_shutdown() {
    let config = MediaConfig {
      encode_concurrency: 2,
      encode_queue_capacity: 2,
      encode_working_set_bytes: 4,
      ..Default::default()
    };
    let executor = Arc::new(MediaExecutor::new(&config).unwrap());
    let n = Backend::concurrency(2);
    // A lone ready item starts immediately; no wait to fill the first batch.
    let (first, started, release) = blocked(executor.clone(), 1);
    started.await.unwrap();
    let (second, ready2, release2) = blocked(executor.clone(), 1);
    wait_for(|| executor.sender.lock().unwrap().as_ref().unwrap().capacity() == 1).await;
    let ran_cancelled = Arc::new(AtomicBool::new(false));
    let marker = ran_cancelled.clone();
    let owner = executor.clone();
    let cancelled = tokio::spawn(async move {
      owner
        .execute(
          1,
          Arc::new(AtomicBool::new(false)),
          Box::new(move |_| {
            marker.store(true, Ordering::Release);
            Ok(Bytes::new())
          }),
        )
        .await
    });
    wait_for(|| executor.sender.lock().unwrap().as_ref().unwrap().capacity() == 0).await;
    let (fourth, ready4, release4) = blocked(executor.clone(), 1);
    wait_for(|| executor.budget.available_permits() == 0).await;
    assert!(!fourth.is_finished(), "full queue applies backpressure");
    assert_eq!(executor.state.active.load(Ordering::Acquire), 1);
    cancelled.abort();
    let _ = cancelled.await;
    first.abort();
    let _ = first.await;
    assert_eq!(executor.available_permits(), n - 1, "cancelling a waiter cannot free running capacity");
    release.send(()).unwrap();
    ready2.await.unwrap();
    assert!(!ran_cancelled.load(Ordering::Acquire), "queued cancelled task must not run");
    assert!(executor.state.active.load(Ordering::Acquire) <= n);
    release2.send(()).unwrap();
    assert_eq!(second.await.unwrap().unwrap(), Bytes::from_static(b"done"));
    ready4.await.unwrap();
    assert!(executor.state.active.load(Ordering::Acquire) <= n);
    // A deadline leaves the actual job and its working-set reservation alive.
    assert!(
      tokio::time::timeout(Duration::ZERO, async {
        while !fourth.is_finished() {
          tokio::task::yield_now().await;
        }
      })
      .await
      .is_err()
    );
    fourth.abort();
    let _ = fourth.await;
    assert_eq!(executor.available_permits(), n - 1);
    release4.send(()).unwrap();
    wait_for(|| executor.budget.available_permits() == 4).await;
    // Ready batches respect N, and a completed leaf is delivered while its
    // sibling still runs. Both formats enter this same executor path.
    let (gate, gate_started, open_gate) = blocked(executor.clone(), 1);
    gate_started.await.unwrap();
    let (fast, mut fast_started, finish_fast) = blocked(executor.clone(), 1);
    wait_for(|| executor.sender.lock().unwrap().as_ref().unwrap().capacity() == 1).await;
    let (slow, mut slow_started, finish_slow) = blocked(executor.clone(), 1);
    wait_for(|| executor.sender.lock().unwrap().as_ref().unwrap().capacity() == 0).await;
    open_gate.send(()).unwrap();
    gate.await.unwrap().unwrap();
    let fast_first = tokio::select! { _ = &mut fast_started => true, _ = &mut slow_started => false };
    assert!(executor.state.active.load(Ordering::Acquire) <= n);
    // Join may execute either branch first, including sequentially. Deliver
    // that leaf while its sibling is still blocked or has yet to begin.
    if fast_first {
      finish_fast.send(()).unwrap();
      tokio::time::timeout(Duration::from_secs(10), fast).await.unwrap().unwrap().unwrap();
      assert!(!slow.is_finished());
      finish_slow.send(()).unwrap();
      slow.await.unwrap().unwrap();
    } else {
      finish_slow.send(()).unwrap();
      tokio::time::timeout(Duration::from_secs(10), slow).await.unwrap().unwrap().unwrap();
      assert!(!fast.is_finished());
      finish_fast.send(()).unwrap();
      fast.await.unwrap().unwrap();
    }
    let pre = Arc::new(AtomicBool::new(true));
    let entered = Arc::new(AtomicBool::new(false));
    let marker = entered.clone();
    assert!(
      executor
        .execute(
          1,
          pre,
          Box::new(move |_| {
            marker.store(true, Ordering::Release);
            Ok(Bytes::new())
          })
        )
        .await
        .is_err()
    );
    assert!(!entered.load(Ordering::Acquire));
    let cancel = Arc::new(AtomicBool::new(false));
    let token = cancel.clone();
    let (at_stage, stage_ready) = oneshot::channel();
    let (advance, stage_wait) = std::sync::mpsc::channel();
    let reached_next_stage = Arc::new(AtomicBool::new(false));
    let marker = reached_next_stage.clone();
    let owner = executor.clone();
    let staged = tokio::spawn(async move {
      owner
        .execute(
          4,
          token,
          Box::new(move |cancelled| {
            let _ = at_stage.send(());
            stage_wait.recv().unwrap();
            if cancelled.load(Ordering::Acquire) {
              return Err(invalid_options("cancelled between stages"));
            }
            marker.store(true, Ordering::Release);
            Ok(Bytes::new())
          }),
        )
        .await
    });
    stage_ready.await.unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(executor.budget.available_permits(), 0);
    advance.send(()).unwrap();
    assert!(staged.await.unwrap().is_err());
    assert!(!reached_next_stage.load(Ordering::Acquire));
    let post = Arc::new(AtomicBool::new(false));
    let token = post.clone();
    assert!(
      executor
        .execute(
          1,
          post,
          Box::new(move |_| {
            token.store(true, Ordering::Release);
            Ok(Bytes::from_static(b"late"))
          })
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("late result rejected")
    );
    // Both join branches catch panics before par-core/Chili restores TLS state.
    let backend = Backend::new(n).unwrap();
    for _ in 0..8 {
      let mut jobs = Vec::new();
      let mut responses = Vec::new();
      for panic in [true, false] {
        let (response, receiver) = oneshot::channel();
        responses.push(receiver);
        jobs.push(Job {
          operation: Box::new(move |_| {
            if panic {
              panic!("controlled codec panic");
            }
            Ok(Bytes::from_static(b"healthy"))
          }),
          cancelled: Arc::new(AtomicBool::new(false)),
          response,
          _budget: executor.budget.clone().acquire_owned().await.unwrap(),
        });
      }
      backend.install(|| run_batch(jobs, &executor.state));
      assert!(responses.remove(0).await.unwrap().is_err());
      assert_eq!(responses.remove(0).await.unwrap().unwrap(), Bytes::from_static(b"healthy"));
    }
    let (active, started, release) = blocked(executor.clone(), 4);
    started.await.unwrap();
    let owner = executor.clone();
    let shutdown = tokio::spawn(async move { owner.shutdown().await });
    wait_for(|| executor.state.stopped.load(Ordering::Acquire)).await;
    assert!(!shutdown.is_finished());
    assert!(
      executor
        .execute(1, Arc::new(AtomicBool::new(false)), Box::new(|_| Ok(Bytes::new())))
        .await
        .is_err()
    );
    release.send(()).unwrap();
    assert!(active.await.unwrap().unwrap_err().to_string().contains("late result rejected"));
    shutdown.await.unwrap().unwrap();
    executor.shutdown().await.unwrap();
  }
}
