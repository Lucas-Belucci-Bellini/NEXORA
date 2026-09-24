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

use std::collections::{BinaryHeap, HashMap, VecDeque};
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
    /// The outcome is no longer known.
    ///
    /// Either the job finished long enough ago that its result was discarded to
    /// bound memory (see [`MAX_RETAINED_RESULTS`]), or this handle was never
    /// issued by this pool. The two are indistinguishable from here and both
    /// mean the same thing to a caller: **nobody is going to answer this**.
    ///
    /// It is deliberately not [`JobOutcome::Completed`]. A forgotten job may
    /// well have completed, and reporting that it did would be inventing the
    /// half of the answer that was thrown away.
    Forgotten,
}

impl JobOutcome {
    /// Whether the job completed successfully.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Whether the pool can still say what happened.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        !matches!(self, Self::Forgotten)
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
    /// Outcomes still held for a caller that has not asked for them.
    ///
    /// Bounded by [`MAX_RETAINED_RESULTS`]. A result is kept so a later `wait`
    /// can find it, and the pool cannot know that nobody will ask — so the
    /// oldest is discarded once the cap is reached rather than kept forever.
    pub retained_results: usize,
    /// Outcomes discarded to stay under the cap.
    ///
    /// Non-zero means some `wait` can now only be answered with
    /// [`JobOutcome::Forgotten`]. It is a count rather than a flag because the
    /// useful question is not *whether* the pool is dropping answers but how
    /// fast — a server that forgets a handful over a day is working as
    /// designed, and one that forgets thousands a minute has a caller
    /// submitting work it never collects.
    pub forgotten_results: u64,
}

/// How many finished jobs' outcomes the pool keeps before discarding the
/// oldest.
///
/// A result is kept so that a later `wait` can find it, and the pool cannot
/// know that nobody will ask — so without a bound this is one entry per job
/// ever submitted, for the life of the process (`DEBT-0040`). The bound is what
/// makes a long-running server possible; [`JobOutcome::Forgotten`] is what keeps
/// the bound from being a silent loss.
///
/// 65,536 is chosen against the largest wave the engine actually submits, not
/// picked for roundness: chunk generation for an interest radius of 12 is 625
/// columns, so this is about a hundred times the biggest batch any caller
/// collects from today. A completed or cancelled outcome is a discriminant and
/// a handle, so the retained set costs on the order of two megabytes; a
/// `Failed` carries its `Error` and is larger, and failures are not the common
/// case.
pub const MAX_RETAINED_RESULTS: usize = 65_536;

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
    forgotten: u64,
}

struct State {
    pending: BinaryHeap<QueuedJob>,
    /// Outcomes of finished jobs, capped at [`MAX_RETAINED_RESULTS`].
    results: HashMap<JobHandle, JobOutcome>,
    /// The order those outcomes arrived in, so the oldest can be found without
    /// searching. A `VecDeque` for the same reason the chunk change feed uses
    /// one (`DEBT-0004`): discarding from the front has to be free.
    result_order: VecDeque<JobHandle>,
    /// Jobs the pool is still holding — queued or running. Removed on
    /// completion, so this is bounded by the queue rather than by history, and
    /// it is what lets `wait` tell "not finished yet" from "finished, and the
    /// answer is gone".
    tokens: HashMap<JobHandle, CancellationToken>,
    running: usize,
    stopping: bool,
    counters: Counters,
}

impl State {
    /// Record an outcome, discarding the oldest if that puts us over the cap.
    fn record(&mut self, handle: JobHandle, outcome: JobOutcome) {
        if self.results.insert(handle, outcome).is_none() {
            self.result_order.push_back(handle);
        }
        while self.result_order.len() > MAX_RETAINED_RESULTS {
            if let Some(oldest) = self.result_order.pop_front() {
                if self.results.remove(&oldest).is_some() {
                    self.counters.forgotten += 1;
                }
            }
        }
    }
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
                result_order: VecDeque::new(),
                tokens: HashMap::new(),
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
    ///
    /// # Cost
    ///
    /// Enqueuing is cheap; **waking a worker to run it is not**. Measured on
    /// the benchmark pool, one submission costs ~288 ns when every worker is
    /// already busy and ~12 µs when one is parked on the queue — the producer
    /// ends up serialized behind the woken worker reacquiring the same lock.
    /// Submitting a wave of jobs one at a time therefore pays that wake-up once
    /// per job; [`JobSystem::submit_all`] pays it once per wave. See
    /// `DEBT-0009` and finding 23 in `docs/benchmarks/PHASE-0-BASELINE.md`.
    pub fn submit<F>(&self, priority: Priority, work: F) -> JobHandle
    where
        F: FnOnce(&CancellationToken) -> Result<()> + Send + 'static,
    {
        let handle = {
            let mut state = self.shared.lock();
            self.enqueue(&mut state, priority, Box::new(work))
        };
        // Notified after the lock is released: a worker woken while the
        // producer still holds it wakes only to block again.
        self.shared.work_available.notify_one();
        handle
    }

    /// Submit many jobs under one lock and one round of wake-ups.
    ///
    /// Same jobs, same priority, same order as calling [`JobSystem::submit`] in
    /// a loop — and one wake-up for the wave instead of one per job. That is
    /// the whole difference, and it is nearly the whole cost: submission is
    /// about 2% enqueue and 98% waking a worker (`DEBT-0009`).
    ///
    /// Returns a handle per job, in the order the jobs were given.
    pub fn submit_all<I, F>(&self, priority: Priority, jobs: I) -> Vec<JobHandle>
    where
        I: IntoIterator<Item = F>,
        F: FnOnce(&CancellationToken) -> Result<()> + Send + 'static,
    {
        // Boxed before the lock is taken: the allocation is the caller's to pay
        // and has no business happening inside the scheduler's critical section.
        let work: Vec<Work> = jobs.into_iter().map(|job| Box::new(job) as Work).collect();
        if work.is_empty() {
            return Vec::new();
        }

        let count = work.len();
        let handles = {
            let mut state = self.shared.lock();
            work.into_iter()
                .map(|job| self.enqueue(&mut state, priority, job))
                .collect()
        };

        // One wake per job would be the thing this exists to avoid, and one
        // wake for a thousand jobs would leave three workers asleep with work
        // queued. Wake as many as there is work for, capped at the pool.
        if count >= self.shared.workers {
            self.shared.work_available.notify_all();
        } else {
            for _ in 0..count {
                self.shared.work_available.notify_one();
            }
        }
        handles
    }

    /// Put one job in the queue. The caller holds the lock and does the waking.
    fn enqueue(&self, state: &mut State, priority: Priority, work: Work) -> JobHandle {
        let handle = JobHandle(self.shared.next_handle.fetch_add(1, Ordering::Relaxed));
        let sequence = self.shared.next_sequence.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();

        state.counters.submitted += 1;
        state.tokens.insert(handle, token.clone());
        state.pending.push(QueuedJob {
            priority,
            sequence,
            handle,
            token,
            work,
        });
        handle
    }

    /// Request cancellation of a job.
    ///
    /// Returns whether the job was known. A queued job will never start; a
    /// running job sees its token flip and should stop at its next check.
    ///
    /// A job whose result has already been discarded reads as unknown, the same
    /// as a handle this pool never issued — cancelling something that finished
    /// long ago was never going to do anything anyway.
    pub fn cancel(&self, handle: JobHandle) -> bool {
        let state = self.shared.lock();
        if state.results.contains_key(&handle) {
            return true;
        }
        let Some(token) = state.tokens.get(&handle).cloned() else {
            return false;
        };
        token.cancel();
        true
    }

    /// Block until a job finishes and return its outcome.
    ///
    /// Returns [`JobOutcome::Forgotten`] when the pool cannot answer: the job
    /// finished long enough ago that its result was discarded, or the handle
    /// was never issued here. **This call always terminates** — before the cap
    /// existed, waiting on a handle the pool had never seen blocked forever,
    /// because the result it was waiting for was never going to arrive.
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
            // No result, and the pool is not holding the job either. Whatever
            // happened to it, waiting longer cannot find out: `tokens` holds
            // exactly the queued and running jobs, so falling through here
            // means the answer was discarded or never existed.
            if !state.tokens.contains_key(&handle) {
                return JobOutcome::Forgotten;
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
            retained_results: state.results.len(),
            forgotten_results: state.counters.forgotten,
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
            // Not a way a job can finish: `Forgotten` is what `wait` says when
            // the pool has no answer, and a worker always has one. Named rather
            // than swallowed by a catch-all, so the next variant added to
            // `JobOutcome` stops the build here instead of going uncounted.
            JobOutcome::Forgotten => {}
        }
        state.tokens.remove(&job.handle);
        state.record(job.handle, outcome);
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

#[cfg(test)]
mod batch_tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn a_batch_runs_every_job_it_was_given() {
        let pool = JobSystem::new(4).expect("pool");
        let ran = Arc::new(AtomicUsize::new(0));

        let jobs: Vec<_> = (0..256)
            .map(|_| {
                let ran = ran.clone();
                move |_: &CancellationToken| {
                    ran.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            })
            .collect();

        let handles = pool.submit_all(Priority::Normal, jobs);
        assert_eq!(handles.len(), 256);
        pool.barrier();

        assert_eq!(ran.load(Ordering::Relaxed), 256);
        for handle in handles {
            assert_eq!(pool.wait(handle), JobOutcome::Completed);
        }
    }

    #[test]
    fn a_batch_hands_back_distinct_handles_in_the_order_it_was_given() {
        let pool = JobSystem::new(2).expect("pool");
        let handles = pool.submit_all(
            Priority::Normal,
            (0..64).map(|_| |_: &CancellationToken| Ok(())),
        );
        pool.barrier();

        // Handles come from one counter, so "in order" is "strictly ascending".
        // A batch that shuffled them would still pass a length check.
        for pair in handles.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{:?} then {:?}", pair[0], pair[1]);
        }
        let unique: std::collections::HashSet<u64> = handles.iter().map(|h| h.0).collect();
        assert_eq!(unique.len(), handles.len());
    }

    #[test]
    fn an_empty_batch_is_not_a_wake_up() {
        let pool = JobSystem::new(2).expect("pool");
        let empty: Vec<fn(&CancellationToken) -> Result<()>> = Vec::new();
        assert!(pool.submit_all(Priority::Normal, empty).is_empty());
        assert_eq!(pool.metrics().submitted, 0);
    }

    #[test]
    fn a_batch_smaller_than_the_pool_still_runs_all_of_it() {
        // The branch that wakes one worker per job rather than the whole pool.
        // Two jobs against eight workers: six must stay asleep and both jobs
        // must still run, which is what a miscounted wake would break.
        let pool = JobSystem::new(8).expect("pool");
        let ran = Arc::new(AtomicUsize::new(0));
        let jobs: Vec<_> = (0..2)
            .map(|_| {
                let ran = ran.clone();
                move |_: &CancellationToken| {
                    ran.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            })
            .collect();
        pool.submit_all(Priority::Normal, jobs);
        pool.barrier();
        assert_eq!(ran.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn a_batched_job_can_be_cancelled_like_any_other() {
        let pool = JobSystem::new(1).expect("pool");
        let gate = Arc::new(AtomicBool::new(false));

        // One blocker so the rest of the batch is still queued when we cancel.
        let blocker = {
            let gate = gate.clone();
            pool.submit(Priority::Critical, move |_| {
                while !gate.load(Ordering::Relaxed) {
                    std::hint::spin_loop();
                }
                Ok(())
            })
        };

        let ran = Arc::new(AtomicUsize::new(0));
        let jobs: Vec<_> = (0..4)
            .map(|_| {
                let ran = ran.clone();
                move |_: &CancellationToken| {
                    ran.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            })
            .collect();
        let handles = pool.submit_all(Priority::Normal, jobs);
        // Recorded, not asserted, until the gate is open. An assertion here
        // panics while the blocker is still spinning, and the pool's `Drop`
        // then joins a worker that never returns — the test deadlocks instead
        // of failing, which reports nothing at all. Found by breaking it.
        let cancel_accepted = pool.cancel(handles[1]);

        gate.store(true, Ordering::Relaxed);
        pool.barrier();

        assert!(cancel_accepted, "cancel did not recognise a batched handle");
        assert_eq!(pool.wait(blocker), JobOutcome::Completed);
        assert_eq!(pool.wait(handles[1]), JobOutcome::Cancelled);
        assert_eq!(
            ran.load(Ordering::Relaxed),
            3,
            "the cancelled job must not have run"
        );
    }

    #[test]
    fn a_batch_queues_behind_higher_priority_work_the_same_way_one_job_does() {
        let pool = JobSystem::new(1).expect("pool");
        let gate = Arc::new(AtomicBool::new(false));
        let order = Arc::new(Mutex::new(Vec::new()));

        let blocker = {
            let gate = gate.clone();
            pool.submit(Priority::Critical, move |_| {
                while !gate.load(Ordering::Relaxed) {
                    std::hint::spin_loop();
                }
                Ok(())
            })
        };

        // Queued while the single worker is held: the heap, not arrival, decides.
        let low: Vec<_> = (0..2)
            .map(|index| {
                let order = order.clone();
                move |_: &CancellationToken| {
                    order.lock().unwrap().push(format!("low{index}"));
                    Ok(())
                }
            })
            .collect();
        pool.submit_all(Priority::Background, low);

        let order_high = order.clone();
        pool.submit(Priority::Critical, move |_| {
            order_high.lock().unwrap().push("high".to_owned());
            Ok(())
        });

        gate.store(true, Ordering::Relaxed);
        pool.barrier();
        let _ = pool.wait(blocker);

        let seen = order.lock().unwrap().clone();
        assert_eq!(seen, vec!["high", "low0", "low1"], "got {seen:?}");
    }

    #[test]
    fn metrics_report_the_results_nobody_has_collected() {
        let pool = JobSystem::new(2).expect("pool");
        pool.submit_all(
            Priority::Normal,
            (0..32).map(|_| |_: &CancellationToken| Ok(())),
        );
        pool.barrier();
        // Nothing drains `results`, and this is the number that says so.
        assert_eq!(pool.metrics().retained_results, 32);
    }
}

#[cfg(test)]
mod retention_tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    /// Drive `count` trivial jobs through a pool and wait for all of them.
    fn run(pool: &JobSystem, count: usize) -> Vec<JobHandle> {
        let handles = pool.submit_all(
            Priority::Normal,
            (0..count).map(|_| |_: &CancellationToken| Ok(())),
        );
        pool.barrier();
        handles
    }

    #[test]
    fn retention_stops_at_the_cap_instead_of_growing_forever() {
        let pool = JobSystem::new(2).expect("pool");
        // Two full caps plus a bit, so eviction has to have happened more than
        // once and the count has to be right both times.
        let total = MAX_RETAINED_RESULTS * 2 + 1_000;
        let mut done = 0;
        while done < total {
            let chunk = (total - done).min(20_000);
            run(&pool, chunk);
            done += chunk;
        }

        let metrics = pool.metrics();
        assert_eq!(metrics.submitted, total as u64);
        assert_eq!(
            metrics.retained_results, MAX_RETAINED_RESULTS,
            "retention did not stop at the cap"
        );
        assert_eq!(
            metrics.forgotten_results,
            (total - MAX_RETAINED_RESULTS) as u64,
            "every discarded outcome must be counted"
        );
    }

    #[test]
    fn the_oldest_outcome_is_the_one_discarded() {
        let pool = JobSystem::new(2).expect("pool");
        let first = run(&pool, 1);
        assert_eq!(pool.wait(first[0]), JobOutcome::Completed);

        // Exactly enough newer work to push that first result out.
        let mut done = 0;
        while done < MAX_RETAINED_RESULTS {
            let chunk = (MAX_RETAINED_RESULTS - done).min(20_000);
            run(&pool, chunk);
            done += chunk;
        }

        assert_eq!(
            pool.wait(first[0]),
            JobOutcome::Forgotten,
            "the oldest result should have been the one to go"
        );
        // And the newest is still answerable, which is what says the eviction
        // took from the correct end.
        let newest = run(&pool, 1);
        assert_eq!(pool.wait(newest[0]), JobOutcome::Completed);
    }

    #[test]
    fn a_forgotten_outcome_does_not_claim_the_job_succeeded() {
        // The tempting shortcut is to treat a missing result as "fine, it must
        // have worked". A job that failed and was then discarded would read as
        // a success, which is the one answer the pool must never invent.
        assert!(!JobOutcome::Forgotten.is_success());
        assert!(!JobOutcome::Forgotten.is_known());
        assert!(JobOutcome::Completed.is_known());
        assert!(JobOutcome::Cancelled.is_known());
    }

    #[test]
    fn waiting_on_a_handle_this_pool_never_issued_returns_rather_than_hanging() {
        // Before the cap existed this blocked forever: no result would ever
        // arrive, and the loop had no other way out while the pool was healthy.
        //
        // Waited on another thread with a deadline, because the failure this
        // guards against is a hang — and asserting inline would mean the test
        // hangs too, which reports nothing at all.
        let pool = Arc::new(JobSystem::new(2).expect("pool"));
        let (sender, receiver) = std::sync::mpsc::channel();
        let answering = pool.clone();
        std::thread::spawn(move || {
            let _ = sender.send(answering.wait(JobHandle(999_999)));
        });

        match receiver.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(outcome) => assert_eq!(outcome, JobOutcome::Forgotten),
            Err(_) => panic!("wait on an unknown handle never returned"),
        }
        assert_eq!(pool.metrics().forgotten_results, 0, "nothing was discarded");
    }

    #[test]
    fn a_result_that_is_still_retained_can_be_waited_on_more_than_once() {
        // The property that stopped `wait` from consuming in the first place.
        let pool = JobSystem::new(2).expect("pool");
        let handles = run(&pool, 4);
        for _ in 0..3 {
            for handle in &handles {
                assert_eq!(pool.wait(*handle), JobOutcome::Completed);
            }
        }
        assert_eq!(pool.metrics().retained_results, 4);
    }

    #[test]
    fn waiting_still_blocks_until_a_running_job_actually_finishes() {
        // The dangerous failure mode of the new early return: a job that is
        // queued or running is not in `results` either, and answering
        // `Forgotten` there would turn every wait into an instant wrong answer.
        let pool = JobSystem::new(1).expect("pool");
        let gate = Arc::new(AtomicBool::new(false));
        let ran = Arc::new(AtomicUsize::new(0));

        let handle = {
            let gate = gate.clone();
            let ran = ran.clone();
            pool.submit(Priority::Normal, move |_| {
                while !gate.load(Ordering::Relaxed) {
                    std::hint::spin_loop();
                }
                ran.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
        };

        let waiter = {
            let gate = gate.clone();
            std::thread::spawn(move || {
                // Let the waiter park first, then release the job.
                std::thread::sleep(std::time::Duration::from_millis(50));
                gate.store(true, Ordering::Relaxed);
            })
        };

        assert_eq!(pool.wait(handle), JobOutcome::Completed);
        assert_eq!(ran.load(Ordering::Relaxed), 1);
        waiter.join().expect("gate thread");
    }

    #[test]
    fn cancelling_a_forgotten_handle_reports_it_as_unknown() {
        let pool = JobSystem::new(2).expect("pool");
        assert!(!pool.cancel(JobHandle(888_888)));
    }
}
