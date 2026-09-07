//! Commands wait here before they run (§31–§35, §41).
//!
//! §31: a received command does not have to execute immediately. The queue
//! orders by priority, honours scheduling, drops what was cancelled, and refuses
//! duplicates.
//!
//! §32 carries the constraint that shapes the whole type: *"prioridade nunca
//! deve permitir violar regras de segurança."* Priority decides **order**, and
//! only order. Nothing here lets a `Critical` command skip a validation layer,
//! because the queue runs before validation and has no way to reach it.

use std::collections::HashSet;

use nexora_foundation::time::WorldTime;

use crate::identity::CommandInstanceId;
use crate::instance::CommandInstance;

/// How urgent a command is (§32).
///
/// Ordered so that `Critical` sorts highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    /// Runs when nothing else wants the budget.
    Background,
    /// Below normal.
    Low,
    /// The default.
    #[default]
    Normal,
    /// Above normal.
    High,
    /// Runs first.
    Critical,
}

/// When a command should run (§33).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Scheduling {
    /// As soon as the queue is drained.
    #[default]
    Immediate,
    /// Not before the next tick.
    NextTick,
    /// Not before a given tick.
    At(WorldTime),
}

/// A queued command and how it should be treated.
#[derive(Debug, Clone)]
pub struct Queued {
    /// The request.
    pub instance: CommandInstance,
    /// Its urgency.
    pub priority: Priority,
    /// When it may run.
    pub scheduling: Scheduling,
    /// The tick it was enqueued at, used to break ties.
    pub enqueued_at: WorldTime,
}

impl Queued {
    /// Queue a command at normal priority, to run as soon as possible.
    #[must_use]
    pub const fn now(instance: CommandInstance, tick: WorldTime) -> Self {
        Self {
            instance,
            priority: Priority::Normal,
            scheduling: Scheduling::Immediate,
            enqueued_at: tick,
        }
    }

    /// Set the priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the scheduling.
    #[must_use]
    pub const fn with_scheduling(mut self, scheduling: Scheduling) -> Self {
        self.scheduling = scheduling;
        self
    }

    /// Whether this may run at `now`.
    #[must_use]
    pub const fn is_ready(&self, now: WorldTime) -> bool {
        match self.scheduling {
            Scheduling::Immediate => true,
            Scheduling::NextTick => now.0 > self.enqueued_at.0,
            Scheduling::At(when) => now.0 >= when.0,
        }
    }
}

/// Why an enqueue attempt did not take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueRejection {
    /// This instance id has been seen before (§10, §35).
    Duplicate,
    /// The queue is at capacity (§63).
    Full,
}

/// Commands awaiting execution.
#[derive(Debug)]
pub struct CommandQueue {
    entries: Vec<Queued>,
    seen: HashSet<CommandInstanceId>,
    cancelled: HashSet<CommandInstanceId>,
    capacity: usize,
}

/// Default ceiling on queued commands.
///
/// §63's example is 100,000 `BreakBlockCommand`s: *"não pode simplesmente
/// entupir a simulation queue."* A bound means the flood is refused at the door
/// with a reason, instead of becoming unbounded memory.
pub const DEFAULT_CAPACITY: usize = 4_096;

impl CommandQueue {
    /// A queue with the default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// A queue with an explicit capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            seen: HashSet::new(),
            cancelled: HashSet::new(),
            capacity,
        }
    }

    /// Add a command.
    ///
    /// # Errors
    ///
    /// Returns [`EnqueueRejection::Duplicate`] when this instance id was
    /// already queued, and [`EnqueueRejection::Full`] at capacity.
    pub fn enqueue(&mut self, entry: Queued) -> Result<(), EnqueueRejection> {
        if !self.seen.insert(entry.instance.instance) {
            return Err(EnqueueRejection::Duplicate);
        }
        if self.entries.len() >= self.capacity {
            // Undo the dedup insert: the command was not accepted, so a retry
            // must not be mistaken for a duplicate of something never queued.
            self.seen.remove(&entry.instance.instance);
            return Err(EnqueueRejection::Full);
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Ask for a queued command to be abandoned (§41).
    ///
    /// Returns whether anything was waiting under that id.
    pub fn cancel(&mut self, instance: CommandInstanceId) -> bool {
        let present = self.entries.iter().any(|e| e.instance.instance == instance);
        self.cancelled.insert(instance);
        present
    }

    /// Take the commands that may run now, most urgent first.
    ///
    /// Cancelled entries are dropped rather than returned, and entries not yet
    /// due stay queued. Expired ones are *not* filtered here — see the note on
    /// [`DrainedBatch`]. Ties break on enqueue tick and then on
    /// instance id, so the order is total and does not depend on hashing —
    /// `NEXORA REPLAY AND DETERMINISM.md` requires the same inputs to produce
    /// the same state, and "whatever order the queue happened to be in" does
    /// not survive a replay.
    pub fn drain_ready(&mut self, now: WorldTime, limit: usize) -> DrainedBatch {
        let mut cancelled = 0usize;

        let mut ready: Vec<Queued> = Vec::new();
        let mut waiting: Vec<Queued> = Vec::new();

        for entry in self.entries.drain(..) {
            if self.cancelled.contains(&entry.instance.instance) {
                cancelled += 1;
                continue;
            }
            if entry.is_ready(now) {
                ready.push(entry);
            } else {
                waiting.push(entry);
            }
        }

        ready.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(a.enqueued_at.0.cmp(&b.enqueued_at.0))
                .then(a.instance.instance.cmp(&b.instance.instance))
        });

        let overflow = ready.split_off(ready.len().min(limit));
        waiting.extend(overflow);
        self.entries = waiting;
        self.cancelled.clear();

        DrainedBatch { ready, cancelled }
    }

    /// How many commands are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether an instance id has ever been enqueued.
    #[must_use]
    pub fn has_seen(&self, instance: CommandInstanceId) -> bool {
        self.seen.contains(&instance)
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// What one drain produced.
#[derive(Debug, Clone)]
pub struct DrainedBatch {
    /// Ready to execute, most urgent first.
    pub ready: Vec<Queued>,
    /// How many were dropped because they had been cancelled.
    pub cancelled: usize,
}

// Deliberately no `expired` count here. Expiry needs the definition's
// `max_age_ticks`, which the queue cannot see -- and it is already checked by
// `ExpiryValidator`, which runs on every drained command and does have it.
// A counter here would be structurally always zero: a number that cannot be
// anything but wrong is worse than an absent one.

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::diagnostics::CorrelationId;

    use crate::identity::{Actor, CommandId, Source};
    use crate::instance::{CommandContext, Target};

    fn instance(id: u64, tick: u64) -> CommandInstance {
        CommandInstance::new(
            CommandId::nexora("break_block").expect("valid"),
            CommandInstanceId(id),
            Target::None,
            Vec::new(),
            CommandContext::new(
                Actor::player(1),
                Source::Network,
                WorldTime(tick),
                CorrelationId(1),
            ),
        )
    }

    #[test]
    fn urgent_commands_come_out_first() {
        let mut queue = CommandQueue::new();
        queue
            .enqueue(Queued::now(instance(1, 0), WorldTime(0)))
            .expect("accepted");
        queue
            .enqueue(Queued::now(instance(2, 0), WorldTime(0)).with_priority(Priority::Critical))
            .expect("accepted");
        queue
            .enqueue(Queued::now(instance(3, 0), WorldTime(0)).with_priority(Priority::Low))
            .expect("accepted");

        let batch = queue.drain_ready(WorldTime(0), 10);
        let order: Vec<u64> = batch.ready.iter().map(|e| e.instance.instance.0).collect();
        assert_eq!(order, vec![2, 1, 3]);
    }

    #[test]
    fn ties_break_deterministically() {
        // Same priority and same tick: the order must still be total, or a
        // replay at the same inputs produces a different world.
        let mut queue = CommandQueue::new();
        for id in [7, 3, 5] {
            queue
                .enqueue(Queued::now(instance(id, 0), WorldTime(0)))
                .expect("accepted");
        }
        let order: Vec<u64> = queue
            .drain_ready(WorldTime(0), 10)
            .ready
            .iter()
            .map(|e| e.instance.instance.0)
            .collect();
        assert_eq!(order, vec![3, 5, 7]);
    }

    #[test]
    fn a_duplicate_instance_id_is_refused() {
        let mut queue = CommandQueue::new();
        queue
            .enqueue(Queued::now(instance(1, 0), WorldTime(0)))
            .expect("accepted");
        assert_eq!(
            queue.enqueue(Queued::now(instance(1, 0), WorldTime(0))),
            Err(EnqueueRejection::Duplicate)
        );
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn a_full_queue_refuses_rather_than_growing() {
        // §63: a flood must not become unbounded memory.
        let mut queue = CommandQueue::with_capacity(2);
        queue
            .enqueue(Queued::now(instance(1, 0), WorldTime(0)))
            .expect("accepted");
        queue
            .enqueue(Queued::now(instance(2, 0), WorldTime(0)))
            .expect("accepted");
        assert_eq!(
            queue.enqueue(Queued::now(instance(3, 0), WorldTime(0))),
            Err(EnqueueRejection::Full)
        );
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn a_command_refused_for_being_full_can_be_retried_later() {
        // The dedup set must not remember something that was never queued,
        // or the retry looks like a duplicate and is dropped forever.
        let mut queue = CommandQueue::with_capacity(1);
        queue
            .enqueue(Queued::now(instance(1, 0), WorldTime(0)))
            .expect("accepted");
        assert_eq!(
            queue.enqueue(Queued::now(instance(2, 0), WorldTime(0))),
            Err(EnqueueRejection::Full)
        );
        assert!(!queue.has_seen(CommandInstanceId(2)));

        queue.drain_ready(WorldTime(0), 10);
        assert!(queue
            .enqueue(Queued::now(instance(2, 0), WorldTime(0)))
            .is_ok());
    }

    #[test]
    fn a_scheduled_command_waits_for_its_tick() {
        let mut queue = CommandQueue::new();
        queue
            .enqueue(
                Queued::now(instance(1, 0), WorldTime(0))
                    .with_scheduling(Scheduling::At(WorldTime(5))),
            )
            .expect("accepted");

        assert!(queue.drain_ready(WorldTime(3), 10).ready.is_empty());
        assert_eq!(queue.len(), 1, "still waiting");
        assert_eq!(queue.drain_ready(WorldTime(5), 10).ready.len(), 1);
    }

    #[test]
    fn next_tick_means_not_this_one() {
        let mut queue = CommandQueue::new();
        queue
            .enqueue(
                Queued::now(instance(1, 0), WorldTime(4)).with_scheduling(Scheduling::NextTick),
            )
            .expect("accepted");
        assert!(queue.drain_ready(WorldTime(4), 10).ready.is_empty());
        assert_eq!(queue.drain_ready(WorldTime(5), 10).ready.len(), 1);
    }

    #[test]
    fn a_cancelled_command_never_runs() {
        let mut queue = CommandQueue::new();
        queue
            .enqueue(Queued::now(instance(1, 0), WorldTime(0)))
            .expect("accepted");
        assert!(queue.cancel(CommandInstanceId(1)));

        let batch = queue.drain_ready(WorldTime(0), 10);
        assert!(batch.ready.is_empty());
        assert_eq!(batch.cancelled, 1);
    }

    #[test]
    fn the_batch_limit_leaves_the_rest_queued() {
        let mut queue = CommandQueue::new();
        for id in 1..=5 {
            queue
                .enqueue(Queued::now(instance(id, 0), WorldTime(0)))
                .expect("accepted");
        }
        let batch = queue.drain_ready(WorldTime(0), 2);
        assert_eq!(batch.ready.len(), 2);
        assert_eq!(queue.len(), 3, "the rest stay for the next tick");
    }

    #[test]
    fn priority_orders_and_nothing_else() {
        // §32: "prioridade nunca deve permitir violar regras de segurança."
        // A Critical command comes out first and is otherwise identical; the
        // queue has no path to validation, so it cannot skip one.
        let mut queue = CommandQueue::new();
        queue
            .enqueue(Queued::now(instance(1, 0), WorldTime(0)).with_priority(Priority::Critical))
            .expect("accepted");
        let batch = queue.drain_ready(WorldTime(0), 10);
        assert_eq!(batch.ready.len(), 1);
        assert_eq!(batch.ready[0].priority, Priority::Critical);
    }
}
