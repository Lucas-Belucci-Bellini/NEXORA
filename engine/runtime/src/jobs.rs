//! Bounded job system.
//!
//! Implements `JOB SYSTEM.md` and the ownership rules in
//! `NEXORA THREADING AND CONCURRENCY MODEL.md`. Three of those rules are
//! enforced structurally rather than by convention:
//!
//! * **No unbounded worker creation.** The pool size is fixed at construction
//!   and validated; there is no API that spawns a thread per task.
//! * **Authoritative results cannot depend on worker order.** Jobs report into
//!   caller-owned slots, and completion order is never observable as ordering.
//! * **A failing job must not strand the runtime.** A panicking job is caught,
//!   recorded as a failure, and its worker keeps serving the queue - otherwise
//!   one bad job silently removes a worker and `wait` hangs forever.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Largest worker pool the engine will build.
pub const MAX_WORKERS: usize = 256;

/// Scheduling priority, from `CORE.md` §9.
///
/// Ordered so that a `BinaryHeap` pops [`Priority::Critical`] first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Deferred work that may wait indefinitely, e.g. distant economy ticks.
    Background,
    /// Low urgency, e.g. far chunk generation.
    Low,
    /// The default.
    Normal,
    /// Needed soon, e.g. nearby chunk generation.
    High,
    /// Needed now; blocking the frame or a save.
    Critical,
}

/// Identity of a submitted job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobHandle(pub u64);

/// A cooperative cancellation flag.
///
/// Cancellation is cooperative by design: a long job must check this itself.
/// Killing a thread mid-operation would leave shared state in an unknown
/// condition, which the concurrency model forbids.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

/// How a job finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    /// Ran to completion.
    Completed,
    /// Cancelled before or during execution.
    Cancelled,
    /// Returned an error.
    Failed(Error),
    /// Panicked. The worker survived and the failure is recorded.
    Panicked(String),
}

impl JobOutcome {
    /// Whether the job completed successfully.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// A point-in-time view of the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JobMetrics {
    /// Jobs accepted so far.
    pub submitted: u64,
    /// Jobs that ran to completion.
    pub completed: u64,
    /// Jobs cancelled.
    pub cancelled: u64,
    /// Jobs that returned an error or panicked.
    pub failed: u64,
    /// Jobs waiting in the queue.
    pub queued: usize,
    /// Jobs currently executing.
    pub running: usize,
    /// Size of the worker pool.
    pub workers: usize,
}

type Work = Box<dyn FnOnce(&CancellationToken) -> Result<()> + Send + 'static>;

struct QueuedJob {
    priority: Priority,
    sequence: u64,
    handle: JobHandle,
    token: CancellationToken,
    work: Work,
}

impl PartialEq for QueuedJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}
impl Eq for QueuedJob {}

impl Ord for QueuedJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first; within a priority, earlier submission first.
        // `BinaryHeap` is a max-heap, so the sequence comparison is reversed.
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}
impl PartialOrd for QueuedJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default)]
struct Counters {
    submitted: u64,
    completed: u64,
    cancelled: u64,
    failed: u64,
}

struct State {
    pending: BinaryHeap<QueuedJob>,
    results: HashMap<JobHandle, JobOutcome>,
    tokens: HashMap<JobHandle, CancellationToken>,
    cancelled: HashSet<JobHandle>,
    running: usize,
    stopping: bool,
    counters: Counters,
}

struct Shared {
    state: Mutex<State>,
    /// Signalled when work arrives or shutdown begins.
    work_available: Condvar,
    /// Signalled when a job finishes.
    work_finished: Condvar,
    next_handle: AtomicU64,
    next_sequence: AtomicU64,
    workers: usize,
}

impl Shared {
    /// Lock the state, recovering from a poisoned mutex.
    ///
    /// A panicking job is caught before it can unwind through the lock, so
    /// poisoning should not happen; recovering rather than propagating means a
    /// surprise cannot take down the whole scheduler.
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// A fixed-size worker pool that executes prioritized jobs.
pub struct JobSystem {
    shared: Arc<Shared>,
    workers: Vec<JoinHandle<()>>,
}

impl std::fmt::Debug for JobSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobSystem")
            .field("workers", &self.workers.len())
            .finish()
    }
}

impl JobSystem {
    /// Build a pool with a fixed number of workers.
    ///
    /// # Errors
    ///
    /// Returns an error when `workers` is zero or exceeds [`MAX_WORKERS`].
    pub fn new(workers: usize) -> Result<Self> {
        if workers == 0 || workers > MAX_WORKERS {
            return Err(Error::new(
                Domain::Job,
                "job-system",
                "worker count must be between one and the maximum pool size",
            )
            .with_recovery(Recovery::Reject)
            .with_context("requested", workers.to_string())
            .with_context("max", MAX_WORKERS.to_string()));
        }

        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                pending: BinaryHeap::new(),
                results: HashMap::new(),
                tokens: HashMap::new(),
                cancelled: HashSet::new(),
                running: 0,
                stopping: false,
                counters: Counters::default(),
            }),
            work_available: Condvar::new(),
            work_finished: Condvar::new(),
            next_handle: AtomicU64::new(1),
            next_sequence: AtomicU64::new(1),
            workers,
        });

        let handles = (0..workers)
            .map(|index| {
                let shared = shared.clone();
                std::thread::Builder::new()
                    .name(format!("nexora-worker-{index}"))
                    .spawn(move || worker_loop(&shared))
                    .map_err(|cause| {
                        Error::new(Domain::Job, "job-system", "failed to spawn a worker thread")
                            .fatal()
                            .with_context("worker", index.to_string())
                            .with_context("cause", cause.to_string())
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            shared,
            workers: handles,
        })
    }

    /// A pool sized to the machine, clamped to [`MAX_WORKERS`].
    ///
    /// # Errors
    ///
    /// Returns an error when the pool cannot be created.
    pub fn with_available_parallelism() -> Result<Self> {
        let workers = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1)
            .min(MAX_WORKERS);
        Self::new(workers)
    }

    /// Submit a job.
    pub fn submit<F>(&self, priority: Priority, work: F) -> JobHandle
    where
        F: FnOnce(&CancellationToken) -> Result<()> + Send + 'static,
    {
        let handle = JobHandle(self.shared.next_handle.fetch_add(1, Ordering::Relaxed));
        let sequence = self.shared.next_sequence.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();

        let mut state = self.shared.lock();
        state.counters.submitted += 1;
        state.tokens.insert(handle, token.clone());
        state.pending.push(QueuedJob {
            priority,
            sequence,
            handle,
            token,
            work: Box::new(work),
        });
        drop(state);

        self.shared.work_available.notify_one();
        handle
    }

    /// Request cancellation of a job.
    ///
    /// Returns whether the job was known. A queued job will never start; a
    /// running job sees its token flip and should stop at its next check.
    pub fn cancel(&self, handle: JobHandle) -> bool {
        let mut state = self.shared.lock();
        if state.results.contains_key(&handle) {
            return true;
        }
        let Some(token) = state.tokens.get(&handle).cloned() else {
            return false;
        };
        state.cancelled.insert(handle);
        token.cancel();
        true
    }

    /// Block until a job finishes and return its outcome.
    pub fn wait(&self, handle: JobHandle) -> JobOutcome {
        let mut state = self.shared.lock();
        loop {
            if let Some(outcome) = state.results.get(&handle) {
                return outcome.clone();
            }
            if state.stopping && state.pending.is_empty() && state.running == 0 {
                // The pool is shutting down and this job will never run.
                return JobOutcome::Cancelled;
            }
            state = self
                .shared
                .work_finished
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    /// Block until every submitted job has finished.
    ///
    /// This is the simulation barrier from `JOB SYSTEM.md`: a synchronization
    /// point where the caller knows no worker is still touching shared state.
    pub fn barrier(&self) {
        let mut state = self.shared.lock();
        while !state.pending.is_empty() || state.running > 0 {
            state = self
                .shared
                .work_finished
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    /// A snapshot of queue statistics.
    #[must_use]
    pub fn metrics(&self) -> JobMetrics {
        let state = self.shared.lock();
        JobMetrics {
            submitted: state.counters.submitted,
            completed: state.counters.completed,
            cancelled: state.counters.cancelled,
            failed: state.counters.failed,
            queued: state.pending.len(),
            running: state.running,
            workers: self.shared.workers,
        }
    }

    /// The size of the worker pool.
    #[must_use]
    pub fn worker_count(&self) -> usize {
        self.shared.workers
    }
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        {
            let mut state = self.shared.lock();
            state.stopping = true;
        }
        self.shared.work_available.notify_all();
        self.shared.work_finished.notify_all();
        for worker in self.workers.drain(..) {
            // A worker that already panicked has nothing left to join cleanly;
            // dropping the error keeps shutdown from panicking a second time.
            let _ = worker.join();
        }
    }
}

fn worker_loop(shared: &Arc<Shared>) {
    loop {
        let job = {
            let mut state = shared.lock();
            loop {
                if let Some(job) = state.pending.pop() {
                    state.running += 1;
                    break Some(job);
                }
                if state.stopping {
                    break None;
                }
                state = shared
                    .work_available
                    .wait(state)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };

        let Some(job) = job else { return };

        // The lock is released while the job runs, so jobs never serialize on
        // the scheduler and a slow job cannot block submissions.
        let outcome = if job.token.is_cancelled() {
            JobOutcome::Cancelled
        } else {
            match catch_unwind(AssertUnwindSafe(|| (job.work)(&job.token))) {
                Ok(Ok(())) => {
                    if job.token.is_cancelled() {
                        JobOutcome::Cancelled
                    } else {
                        JobOutcome::Completed
                    }
                }
                Ok(Err(cause)) => JobOutcome::Failed(cause),
                Err(payload) => JobOutcome::Panicked(describe_panic(&payload)),
            }
        };

        let mut state = shared.lock();
        state.running -= 1;
        match &outcome {
            JobOutcome::Completed => state.counters.completed += 1,
            JobOutcome::Cancelled => state.counters.cancelled += 1,
            JobOutcome::Failed(_) | JobOutcome::Panicked(_) => state.counters.failed += 1,
        }
        state.tokens.remove(&job.handle);
        state.results.insert(job.handle, outcome);
        drop(state);

        shared.work_finished.notify_all();
    }
}

fn describe_panic(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "job panicked with a non-string payload".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex as StdMutex;

    #[test]
    fn worker_counts_are_validated() {
        assert!(JobSystem::new(0).is_err());
        assert!(JobSystem::new(MAX_WORKERS + 1).is_err());
        let pool = JobSystem::new(2).unwrap();
        assert_eq!(pool.worker_count(), 2);
    }

    #[test]
    fn a_submitted_job_runs_and_reports_completion() {
        let pool = JobSystem::new(2).unwrap();
        let ran = Arc::new(AtomicBool::new(false));
        let flag = ran.clone();

        let handle = pool.submit(Priority::Normal, move |_| {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        });

        assert_eq!(pool.wait(handle), JobOutcome::Completed);
        assert!(ran.load(Ordering::SeqCst));
        assert_eq!(pool.metrics().completed, 1);
    }

    #[test]
    fn results_do_not_depend_on_worker_order() {
        // Every job writes to the slot it owns, so the aggregate is identical
        // no matter which worker happens to run which job first.
        let pool = JobSystem::new(4).unwrap();
        let slots: Arc<Vec<AtomicU64>> = Arc::new((0..256).map(|_| AtomicU64::new(0)).collect());

        let handles: Vec<JobHandle> = (0..256u64)
            .map(|index| {
                let slots = slots.clone();
                pool.submit(Priority::Normal, move |_| {
                    slots[index as usize].store(index * 3, Ordering::SeqCst);
                    Ok(())
                })
            })
            .collect();

        pool.barrier();
        for handle in handles {
            assert_eq!(pool.wait(handle), JobOutcome::Completed);
        }
        for index in 0..256u64 {
            assert_eq!(slots[index as usize].load(Ordering::SeqCst), index * 3);
        }
    }

    #[test]
    fn higher_priority_work_is_taken_first() {
        // One worker, so the queue order is directly observable. The blocker
        // occupies the worker while the rest of the queue is filled.
        let pool = JobSystem::new(1).unwrap();
        let gate = Arc::new((StdMutex::new(false), Condvar::new()));
        let order = Arc::new(StdMutex::new(Vec::new()));

        let blocker_gate = gate.clone();
        let blocker = pool.submit(Priority::Critical, move |_| {
            let (lock, condvar) = &*blocker_gate;
            let mut released = lock.lock().expect("gate lock");
            while !*released {
                released = condvar.wait(released).expect("gate wait");
            }
            Ok(())
        });

        for (label, priority) in [
            ("background", Priority::Background),
            ("critical", Priority::Critical),
            ("normal", Priority::Normal),
            ("high", Priority::High),
        ] {
            let sink = order.clone();
            pool.submit(priority, move |_| {
                sink.lock().expect("order lock").push(label);
                Ok(())
            });
        }

        // Release the blocker; the single worker now drains by priority.
        {
            let (lock, condvar) = &*gate;
            *lock.lock().expect("gate lock") = true;
            condvar.notify_all();
        }
        assert_eq!(pool.wait(blocker), JobOutcome::Completed);
        pool.barrier();

        assert_eq!(
            *order.lock().expect("order lock"),
            ["critical", "high", "normal", "background"]
        );
    }

    #[test]
    fn equal_priority_work_runs_in_submission_order() {
        let pool = JobSystem::new(1).unwrap();
        let gate = Arc::new((StdMutex::new(false), Condvar::new()));
        let order = Arc::new(StdMutex::new(Vec::new()));

        let blocker_gate = gate.clone();
        let blocker = pool.submit(Priority::Normal, move |_| {
            let (lock, condvar) = &*blocker_gate;
            let mut released = lock.lock().expect("gate lock");
            while !*released {
                released = condvar.wait(released).expect("gate wait");
            }
            Ok(())
        });

        for index in 0..8usize {
            let sink = order.clone();
            pool.submit(Priority::Normal, move |_| {
                sink.lock().expect("order lock").push(index);
                Ok(())
            });
        }

        {
            let (lock, condvar) = &*gate;
            *lock.lock().expect("gate lock") = true;
            condvar.notify_all();
        }
        pool.wait(blocker);
        pool.barrier();

        assert_eq!(
            *order.lock().expect("order lock"),
            (0..8).collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_queued_job_can_be_cancelled_before_it_starts() {
        let pool = JobSystem::new(1).unwrap();
        let gate = Arc::new((StdMutex::new(false), Condvar::new()));
        let ran = Arc::new(AtomicBool::new(false));

        let blocker_gate = gate.clone();
        let blocker = pool.submit(Priority::Critical, move |_| {
            let (lock, condvar) = &*blocker_gate;
            let mut released = lock.lock().expect("gate lock");
            while !*released {
                released = condvar.wait(released).expect("gate wait");
            }
            Ok(())
        });

        let flag = ran.clone();
        let victim = pool.submit(Priority::Normal, move |_| {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        });
        assert!(pool.cancel(victim));

        {
            let (lock, condvar) = &*gate;
            *lock.lock().expect("gate lock") = true;
            condvar.notify_all();
        }
        pool.wait(blocker);

        assert_eq!(pool.wait(victim), JobOutcome::Cancelled);
        assert!(!ran.load(Ordering::SeqCst), "a cancelled job must not run");
        assert_eq!(pool.metrics().cancelled, 1);
    }

    #[test]
    fn a_running_job_observes_cooperative_cancellation() {
        let pool = JobSystem::new(2).unwrap();
        let started = Arc::new(AtomicBool::new(false));
        let flag = started.clone();

        let handle = pool.submit(Priority::Normal, move |token| {
            flag.store(true, Ordering::SeqCst);
            // Spin until the owner asks us to stop.
            while !token.is_cancelled() {
                std::thread::yield_now();
            }
            Ok(())
        });

        while !started.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        assert!(pool.cancel(handle));
        assert_eq!(pool.wait(handle), JobOutcome::Cancelled);
    }

    #[test]
    fn cancelling_an_unknown_job_reports_false() {
        let pool = JobSystem::new(1).unwrap();
        assert!(!pool.cancel(JobHandle(9_999)));
    }

    #[test]
    fn a_failing_job_reports_its_error() {
        let pool = JobSystem::new(1).unwrap();
        let handle = pool.submit(Priority::Normal, |_| {
            Err(Error::new(
                Domain::World,
                "chunk-generator",
                "terrain source unavailable",
            ))
        });

        match pool.wait(handle) {
            JobOutcome::Failed(cause) => {
                assert_eq!(cause.owner(), "chunk-generator");
            }
            other => panic!("expected a failure, got {other:?}"),
        }
        assert_eq!(pool.metrics().failed, 1);
    }

    #[test]
    fn a_panicking_job_does_not_take_down_its_worker() {
        // The important half of this test is the second job: with a single
        // worker, a lost thread would make `wait` block forever.
        let pool = JobSystem::new(1).unwrap();
        let exploded = pool.submit(Priority::Normal, |_| panic!("simulated defect"));

        match pool.wait(exploded) {
            JobOutcome::Panicked(message) => assert!(message.contains("simulated defect")),
            other => panic!("expected a panic outcome, got {other:?}"),
        }

        let survivor = pool.submit(Priority::Normal, |_| Ok(()));
        assert_eq!(pool.wait(survivor), JobOutcome::Completed);
        assert_eq!(pool.worker_count(), 1);
    }

    #[test]
    fn barrier_waits_for_every_outstanding_job() {
        let pool = JobSystem::new(4).unwrap();
        let done = Arc::new(AtomicUsize::new(0));

        for _ in 0..200 {
            let counter = done.clone();
            pool.submit(Priority::Normal, move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
        }

        pool.barrier();
        assert_eq!(done.load(Ordering::SeqCst), 200);

        let metrics = pool.metrics();
        assert_eq!(metrics.queued, 0);
        assert_eq!(metrics.running, 0);
        assert_eq!(metrics.completed, 200);
    }

    #[test]
    fn dropping_the_pool_stops_the_workers() {
        let done = Arc::new(AtomicUsize::new(0));
        {
            let pool = JobSystem::new(3).unwrap();
            for _ in 0..50 {
                let counter = done.clone();
                pool.submit(Priority::Normal, move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                });
            }
            pool.barrier();
        }
        // Drop joined every worker; reaching here at all is the assertion.
        assert_eq!(done.load(Ordering::SeqCst), 50);
    }
}
