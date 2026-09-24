//! # NEXORA query system
//!
//! `NEXORA ARCHITECTURE RULES.md` §4 names three concepts and the engine had
//! built two:
//!
//! ```text
//! Command = intenção      engine/command   (ADR-0010)
//! Event   = fato ocorrido runtime::events
//! Query   = leitura       here
//! ```
//!
//! `Event Bus.md` §30: *"normalmente não deveria usar Event Bus para queries
//! síncronas. Preferir API direta: query(). O Event Bus não deve virar RPC
//! universal."* So a query is a direct, synchronous call — and, because
//! `NEXORA PUBLIC API AND CONTRACTS.md` lists `QUERY → read information` among
//! the public contract types, it is a **contract**: named, versioned,
//! permitted, bounded.
//!
//! # What each property is, and where it is enforced
//!
//! * **Read-only by construction.** A [`Query`] answers from `&S`. There is no
//!   path from a query to a mutable reference, so "a query that also changes
//!   something" is a type error, not a code-review finding. Writes are
//!   commands (`NEXORA DATA OWNERSHIP AND SOURCE OF TRUTH.md`: *READ → query;
//!   WRITE → command*).
//! * **Deny by default.** A definition's access policy is the command
//!   system's [`SourcePolicy`], closed until opened, and a definition that
//!   admits nobody is refused at registration.
//! * **Versioned.** A request names the version it was built against; a
//!   different one is refused by name rather than answered in a shape the
//!   caller cannot read.
//! * **Bounded.** Every definition has a finite result budget, the handler is
//!   told it, and the answer is re-counted afterwards — a handler that ignores
//!   its budget is caught, not trusted. `Mod Runtime.md` §79 wants mods to get
//!   `WorldQuery`, not raw world memory; an unbounded query is raw world memory
//!   with extra steps.
//! * **Shareable.** [`QueryService::ask`] takes `&self`; its counters are
//!   atomic. `Mod Runtime.md` §80: a worker thread *queries a snapshot* — which
//!   requires that asking does not need exclusive access to anything.
//!
//! [`SourcePolicy`]: nexora_command::definition::SourcePolicy

pub mod definition;
pub mod service;

pub use definition::{QueryDefinition, DEFAULT_MAX_RESULTS};
pub use service::{Budget, Query, QueryFailure, QueryRequest, QueryService, QueryStats};
