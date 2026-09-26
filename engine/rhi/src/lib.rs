//! The render hardware interface: what a renderer may ask a GPU for, and
//! nothing about which GPU API answers.
//!
//! Implements the contract of `RENDER HARDWARE INTERFACE.md` — device
//! capabilities, buffers, textures, pipelines, command submission, fences,
//! presentation — as a Rust trait, [`Rhi`], and the rules every backend must
//! keep as a suite, [`conformance::run`]. ADR-0025 records why it is built in
//! this order.
//!
//! # What exists, and what does not
//!
//! * **The contract** ([`Rhi`], [`desc`], [`api`]) — handles that cannot
//!   outlive their resource or their device, all-or-nothing submission,
//!   ordered fences, destruction deferred until the GPU is done, device loss
//!   and recreation.
//! * **The null backend** ([`NullRhi`]) — the headless backend the
//!   specification asks for: every rule enforced, device memory accounted in
//!   the ledger's `Gpu` class, and no GPU initialized. It never draws a pixel,
//!   and its capabilities say it cannot present.
//! * **The conformance suite** ([`conformance`]) — backend parity made
//!   runnable. The first native backend passes it on real hardware, or it is
//!   not a backend.
//! * **The backend kit** ([`kit`]) — resource tables, the command rules and
//!   memory accounting every backend shares, so a rule is written once.
//!
//! The first native backend is `nexora-rhi-wgpu` (ADR-0026). It lives in its
//! own crate because it takes an external dependency, and this crate does not
//! know which backend is under it. It presents to a window through
//! `nexora-window` (ADR-0027). Since ADR-0028 a pipeline declares its vertex
//! layout, its binding slots and its depth test, and a draw supplies them.

pub mod api;
pub mod conformance;
pub mod desc;
pub mod kit;
pub mod null;

pub use api::{
    Binding, BufferHandle, ClearValue, Command, CommandList, Fence, PipelineHandle, Rhi, Slot,
    TextureHandle,
};
pub use desc::{
    BindingKind, BufferDesc, Capabilities, Compare, DepthState, Filter, PipelineDesc, ShaderStage,
    TextureDesc, TextureFormat, Usage, VertexAttribute, VertexFormat, COPY_ALIGNMENT, MAX_BINDINGS,
    MAX_VERTEX_ATTRIBUTES, UNIFORM_ALIGNMENT,
};
pub use null::{NullRhi, NullStats};
