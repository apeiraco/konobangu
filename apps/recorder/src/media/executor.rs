//! One bounded queue, one owned Rayon pool, and a shared admission budget.
use std::{
  panic::{AssertUnwindSafe, catch_unwind},
  sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
  },
};

use bytes::Bytes;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, oneshot};

use crate::{
  errors::RecorderResult,
  media::{MediaConfig, invalid_options},
};

type Operation = Box<dyn FnOnce(&AtomicBool) -> RecorderResult<Bytes> + Send>;
struct Job {
  operation: Operation,
  cancelled: Arc<AtomicBool>,
  response: oneshot::Sender<RecorderResult<Bytes>>,
  /// Released once the job starts, so the queue bounds waiting work only.
  slot: OwnedSemaphorePermit,
  /// Held until the codec actually returns, even if its caller left.
  _budget: OwnedSemaphorePermit,
}
#[derive(Debug)]
struct State {
  stopped: AtomicBool,
  /// Running jobs, for test observability only; admission is the budget.
  active: AtomicUsize,
  concurrency: usize,
}
#[derive(Debug)]
pub struct MediaExecutor {
  pool: rayon::ThreadPool,
  queue: Arc<Semaphore>,
  budget: Arc<Semaphore>,
  budget_limit: u32,
  state: Arc<State>,
}
impl MediaExecutor {
  pub fn new(config: &MediaConfig) -> RecorderResult<Self> {
    config.validate()?;
    let concurrency = config.encode_concurrency;
    Ok(Self {
      pool: rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .thread_name(|index| format!("media-cpu-{index}"))
        .build()
        .map_err(|_| invalid_options("Media encoder pool initialization failed"))?,
      queue: Arc::new(Semaphore::new(config.encode_queue_capacity)),
      budget: Arc::new(Semaphore::new(config.encode_working_set_bytes as usize)),
      budget_limit: config.encode_working_set_bytes as u32,
      state: Arc::new(State {
        stopped: AtomicBool::new(false),
        active: AtomicUsize::new(0),
        concurrency,
      }),
    })
  }
  pub(super) async fn execute(&self, estimated_bytes: u64, cancelled: Arc<AtomicBool>, operation: Operation) -> RecorderResult<Bytes> {
    let weight = u32::try_from(estimated_bytes)
      .ok()
      .filter(|n| *n <= self.budget_limit)
      .ok_or_else(|| invalid_options("Image estimated working set exceeds shared admission budget"))?
      // Every admitted job holds real admission, so returning the whole budget
      // is proof that the pool has drained.
      .max(1);
    if self.state.stopped.load(Ordering::Acquire) {
      return Err(invalid_options("Image encoder is closed"));
    }
    let budget = self
      .budget
      .clone()
      .try_acquire_many_owned(weight)
      .map_err(|_| invalid_options("Image encoder admission budget unavailable (backpressure)"))?;
    // Waiting for a queue slot is the backpressure boundary; closing the queue
    // wakes every waiter instead of leaving it parked.
    let slot = self
      .queue
      .clone()
      .acquire_owned()
      .await
      .map_err(|_| invalid_options("Image encoder is closed"))?;
    if self.state.stopped.load(Ordering::Acquire) {
      return Err(invalid_options("Image encoder is closed"));
    }
    let (response, receiver) = oneshot::channel();
    let state = self.state.clone();
    self.pool.spawn(move || {
      run_job(
        Job {
          operation,
          cancelled,
          response,
          slot,
          _budget: budget,
        },
        &state,
      );
    });
    receiver.await.map_err(|_| invalid_options("Media encoder stopped without a result"))?
  }
  /// Idle capacity. Observability only: admission is the budget semaphore, and
  /// `MediaService` exposes this exclusively to tests.
  pub fn available_permits(&self) -> usize {
    self.state.concurrency.saturating_sub(self.state.active.load(Ordering::Acquire))
  }
  /// Stops admission and waits for work that already holds admission. A
  /// synchronous codec cannot be killed, so this waits for it to return.
  pub async fn shutdown(&self) -> RecorderResult<()> {
    self.stop();
    let _drained = self
      .budget
      .acquire_many(self.budget_limit)
      .await
      .map_err(|_| invalid_options("Media encoder budget closed during shutdown"))?;
    Ok(())
  }
  fn stop(&self) {
    self.state.stopped.store(true, Ordering::Release);
    self.queue.close();
  }
}
impl Drop for MediaExecutor {
  fn drop(&mut self) {
    // Queued work is skipped rather than encoded for a receiver that is gone.
    // Rayon keeps its registry alive until spawned jobs finish, so its threads
    // drain without blocking whoever dropped the service. Explicit `shutdown`
    // remains the path that waits for a running codec.
    self.stop();
  }
}
fn run_job(job: Job, state: &State) {
  let Job {
    operation,
    cancelled,
    response,
    slot,
    _budget,
  } = job;
  // Started work no longer occupies the queue.
  drop(slot);
  let abandoned = || state.stopped.load(Ordering::Acquire) || cancelled.load(Ordering::Acquire) || response.is_closed();
  if abandoned() {
    drop(operation);
    drop(_budget);
    let _ = response.send(Err(invalid_options("Media encoding cancelled")));
    return;
  }
  state.active.fetch_add(1, Ordering::AcqRel);
  // Rayon already keeps a panicking job from killing its worker; converting the
  // panic here also hands the waiting caller an error instead of a closed
  // channel. OOM, abort and native crashes remain outside this boundary.
  let result = catch_unwind(AssertUnwindSafe(|| operation(&cancelled))).unwrap_or_else(|_| Err(invalid_options("Media encoding panicked")));
  // A codec owns admission until it actually returns, even if its caller left.
  state.active.fetch_sub(1, Ordering::AcqRel);
  let result = if abandoned() {
    Err(invalid_options("Media encoding cancelled; late result rejected"))
  } else {
    result
  };
  // Completion can immediately admit another full-budget job on another thread.
  drop(_budget);
  let _ = response.send(result);
}

#[cfg(test)]
mod tests {
  use std::{
    future::Future,
    task::{Context, Poll, Wake, Waker},
    time::Duration,
  };

  use super::*;

  #[test]
  fn completion_notifies_after_releasing_admission() {
    struct BudgetAtWake {
      budget: Arc<Semaphore>,
      observed: AtomicUsize,
    }
    impl Wake for BudgetAtWake {
      fn wake(self: Arc<Self>) {
        self.observed.store(self.budget.available_permits(), Ordering::Release);
      }
    }

    for outcome in ["success", "error", "panic", "cancelled", "stopped"] {
      let budget = Arc::new(Semaphore::new(4));
      let queue = Arc::new(Semaphore::new(1));
      let probe = Arc::new(BudgetAtWake {
        budget: budget.clone(),
        observed: AtomicUsize::new(usize::MAX),
      });
      let waker = Waker::from(probe.clone());
      let mut context = Context::from_waker(&waker);
      let (response, mut receiver) = oneshot::channel();
      assert!(matches!(std::pin::Pin::new(&mut receiver).poll(&mut context), Poll::Pending));
      let state = State {
        stopped: AtomicBool::new(outcome == "stopped"),
        active: AtomicUsize::new(0),
        concurrency: 1,
      };
      let ran = Arc::new(AtomicBool::new(false));
      let marker = ran.clone();
      let job = Job {
        operation: Box::new(move |_| {
          marker.store(true, Ordering::Release);
          match outcome {
            "panic" => panic!("controlled codec panic"),
            "error" => Err(invalid_options("controlled codec error")),
            _ => Ok(Bytes::from_static(b"done")),
          }
        }),
        cancelled: Arc::new(AtomicBool::new(outcome == "cancelled")),
        response,
        slot: queue.clone().try_acquire_owned().unwrap(),
        _budget: budget.clone().try_acquire_many_owned(4).unwrap(),
      };

      // oneshot wakes synchronously inside send, exposing the notification
      // boundary without depending on thread scheduling or sleeps.
      run_job(job, &state);
      assert_eq!(probe.observed.load(Ordering::Acquire), 4, "{outcome}");
      assert_eq!(state.active.load(Ordering::Acquire), 0, "{outcome}");
      assert_eq!(queue.available_permits(), 1, "{outcome}");
      assert_eq!(ran.load(Ordering::Acquire), !matches!(outcome, "cancelled" | "stopped"));
      let Poll::Ready(Ok(result)) = std::pin::Pin::new(&mut receiver).poll(&mut context) else {
        panic!("missing completion for {outcome}");
      };
      assert_eq!(result.is_ok(), outcome == "success", "{outcome}");
      assert!(budget.clone().try_acquire_many_owned(4).is_ok(), "{outcome}");
    }
  }

  fn queued(executor: &MediaExecutor, capacity: usize) -> usize {
    capacity - executor.queue.available_permits()
  }
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
    let n = config.encode_concurrency;
    let capacity = config.encode_queue_capacity;
    // An admitted job starts immediately; nothing waits to fill a batch.
    let (first, started, release) = blocked(executor.clone(), 1);
    started.await.unwrap();
    // A second job runs beside it, and both leave the queue once started.
    let (second, ready2, release2) = blocked(executor.clone(), 1);
    ready2.await.unwrap();
    wait_for(|| queued(&executor, capacity) == 0).await;
    assert_eq!(executor.available_permits(), n - 2);
    // A third job has no free worker, so it waits in the queue.
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
    wait_for(|| queued(&executor, capacity) == 1).await;
    let (fourth, ready4, release4) = blocked(executor.clone(), 1);
    // A waiting job holds its queue slot, and admission is taken before it.
    wait_for(|| queued(&executor, capacity) == 2).await;
    assert_eq!(executor.budget.available_permits(), 0);
    assert!(!fourth.is_finished());
    // The budget is exhausted, so a fifth job is refused instead of queued.
    assert!(
      executor
        .execute(1, Arc::new(AtomicBool::new(false)), Box::new(|_| Ok(Bytes::new())))
        .await
        .unwrap_err()
        .to_string()
        .contains("backpressure")
    );
    cancelled.abort();
    let _ = cancelled.await;
    first.abort();
    let _ = first.await;
    assert_eq!(executor.available_permits(), n - 2, "cancelling a waiter cannot free running capacity");
    release.send(()).unwrap();
    assert!(!ran_cancelled.load(Ordering::Acquire), "a queued cancelled job must not run");
    release2.send(()).unwrap();
    assert_eq!(second.await.unwrap().unwrap(), Bytes::from_static(b"done"));
    ready4.await.unwrap();
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
    // A finished job is delivered while its sibling still runs. Both formats
    // enter this same path.
    let (fast, fast_started, finish_fast) = blocked(executor.clone(), 1);
    let (slow, slow_started, finish_slow) = blocked(executor.clone(), 1);
    fast_started.await.unwrap();
    slow_started.await.unwrap();
    finish_fast.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), fast).await.unwrap().unwrap().unwrap();
    assert!(!slow.is_finished(), "one result does not wait for its sibling");
    finish_slow.send(()).unwrap();
    slow.await.unwrap().unwrap();
    // An already cancelled job never reaches its operation.
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
    // Cancellation between stages is observed by the running operation.
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
    // A result produced after cancellation is rejected rather than returned.
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
    // A panicking codec becomes an error and leaves the pool usable.
    for _ in 0..8 {
      assert!(
        executor
          .execute(1, Arc::new(AtomicBool::new(false)), Box::new(|_| panic!("controlled codec panic")))
          .await
          .unwrap_err()
          .to_string()
          .contains("panicked")
      );
      assert_eq!(
        executor
          .execute(1, Arc::new(AtomicBool::new(false)), Box::new(|_| Ok(Bytes::from_static(b"healthy"))))
          .await
          .unwrap(),
        Bytes::from_static(b"healthy")
      );
    }
    // Shutdown stops admission and waits for work that already holds it.
    let (active, started, release) = blocked(executor.clone(), 4);
    started.await.unwrap();
    let owner = executor.clone();
    let shutdown = tokio::spawn(async move { owner.shutdown().await });
    wait_for(|| executor.state.stopped.load(Ordering::Acquire)).await;
    assert!(!shutdown.is_finished(), "shutdown waits for a running codec");
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
