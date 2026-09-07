//! NEXORA command system: the boundary between *"I want to do something"* and
//! *"the world actually changed"*.
//!
//! Implements `Command System.md` stages **CMD-0 to CMD-4** — core contracts,
//! registry, dispatch, layered validation and the queue. See
//! [ADR-0010](https://github.com/Lucas-Belucci-Bellini/NEXORA/blob/main/docs/adr/ADR-0010-commands-are-intent-and-carry-their-own-authority.md)
//! for what is deliberately absent and why.
//!
//! # The three concepts, kept apart
//!
//! `NEXORA ARCHITECTURE RULES.md` §4 and `Command System.md` §2–§3:
//!
//! ```text
//! COMMAND -> intent, "I want to break this block"
//! EVENT   -> fact,   "this block was broken"
//! QUERY   -> read
//! ```
//!
//! A `BreakBlockEvent` that performs the break is the specific mistake the rule
//! forbids, and `DEBT-0007` existed because the engine had events and no
//! commands — which is exactly the pressure that produces one.
//!
//! # What this crate must not contain
//!
//! §134 lists it: combat logic, AI logic, inventory rules, block rules,
//! crafting rules, economy rules, physics, worldgen. None of those crates are
//! reachable from here — `nexora-command` depends on `nexora-foundation` and
//! `nexora-runtime` and nothing else, so the boundary is a build error rather
//! than a review comment.
//!
//! Domain handlers therefore live above, in the crate that may see the world.
//!
//! # The pipeline
//!
//! ```text
//! instance -> structural -> identity -> authorization -> quota
//!          -> queue -> domain validation -> handler -> result + events
//! ```
//!
//! The order is a security property: nothing reaches a layer that does real
//! work until the cheap, universal checks have passed. And there is no trusted
//! bypass — §72 is explicit that internal commands are validated too.

pub mod definition;
pub mod dispatcher;
pub mod handler;
pub mod identity;
pub mod instance;
pub mod lifecycle;
pub mod queue;
pub mod registry;
pub mod result;
pub mod validation;
