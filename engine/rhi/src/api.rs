//! The contract: handles, command lists, fences, and the [`Rhi`] trait every
//! backend implements.

use nexora_foundation::error::Result;

use crate::desc::{BufferDesc, Capabilities, PipelineDesc, TextureDesc};

/// Where a resource lives in a backend, and which life of it this is.
///
/// Three numbers, because three different things can make a handle wrong: the
/// slot was never handed out (`index`), it was destroyed and reused
/// (`generation`), or the device it lived on was lost and recreated (`epoch`).
/// A backend refuses all three the same way, as a stale handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slot {
    /// Index into the backend's table for this kind of resource.
    pub index: u32,
    /// Bumped each time the slot is reused.
    pub generation: u32,
    /// The device life the resource was created in.
    pub epoch: u32,
}

macro_rules! handle {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub Slot);
    };
}

handle!(
    /// A buffer. Copyable, and never the owner: destroying it is explicit.
    BufferHandle
);
handle!(
    /// A texture. Copyable, and never the owner: destroying it is explicit.
    TextureHandle
);
handle!(
    /// A pipeline. Copyable, and never the owner: destroying it is explicit.
    PipelineHandle
);

/// What a draw puts in one of its pipeline's binding slots
/// ([`crate::desc::BindingKind`]), in slot order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Binding {
    /// A uniform buffer for a [`crate::desc::BindingKind::Uniform`] slot.
    Uniform(BufferHandle),
    /// A colour texture for a [`crate::desc::BindingKind::Texture`] slot.
    /// It may not also be the draw's target.
    Texture(TextureHandle),
    /// The pipeline's own sampler, for a [`crate::desc::BindingKind::Sampler`]
    /// slot. Named so a draw lists every slot, and a mismatch is visible.
    Sampler,
}

/// What a [`Command::Clear`] writes to every texel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClearValue {
    /// Red, green, blue, alpha, each finite and in `[0, 1]`, for a colour
    /// texture.
    Color([f32; 4]),
    /// A depth, finite and in `[0, 1]`, for a depth texture. `1.0` is the far
    /// plane, where a frame's depth starts.
    Depth(f32),
}

/// One recorded GPU operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Copy bytes from the CPU into a buffer at `offset`. The buffer needs
    /// [`crate::Usage::COPY_DST`]; offset and length are multiples of
    /// [`crate::COPY_ALIGNMENT`] and stay inside the buffer.
    WriteBuffer {
        /// Destination.
        buffer: BufferHandle,
        /// Byte offset into it.
        offset: u64,
        /// What to write.
        data: Vec<u8>,
    },
    /// Replace a texture's contents from the CPU. The texture needs
    /// [`crate::Usage::COPY_DST`], and `data` is exactly
    /// [`TextureDesc::byte_len`] bytes, row-major from the top-left.
    WriteTexture {
        /// Destination.
        texture: TextureHandle,
        /// Every texel.
        data: Vec<u8>,
    },
    /// Draw `vertices` vertices from `buffer` with `pipeline` into `target`.
    Draw {
        /// How to draw.
        pipeline: PipelineHandle,
        /// Vertex data; needs [`crate::Usage::VERTEX`] and must hold
        /// `vertices * stride` bytes.
        buffer: BufferHandle,
        /// Where to draw; needs [`crate::Usage::RENDER_TARGET`] and a format
        /// the pipeline declared.
        target: TextureHandle,
        /// The depth texture, exactly when the pipeline has a depth test: a
        /// [`crate::TextureFormat::Depth32Float`] render target of the
        /// target's size.
        depth: Option<TextureHandle>,
        /// One per pipeline binding slot, in order and of the slot's kind.
        bindings: Vec<Binding>,
        /// How many vertices.
        vertices: u32,
    },
    /// Set every texel of a render target to `value`: a colour for a colour
    /// target, a depth for a depth texture. How a frame starts.
    Clear {
        /// What to clear; needs [`crate::Usage::RENDER_TARGET`].
        texture: TextureHandle,
        /// What to write.
        value: ClearValue,
    },
    /// A debug marker, for captures and validation layers. No effect.
    Marker(String),
}

/// Commands recorded together and submitted as one unit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandList {
    /// Shown in debug markers and errors.
    pub label: String,
    /// In order.
    pub commands: Vec<Command>,
}

impl CommandList {
    /// An empty list.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            commands: Vec::new(),
        }
    }

    /// Record one command.
    pub fn push(&mut self, command: Command) -> &mut Self {
        self.commands.push(command);
        self
    }
}

/// A point in a backend's submission order.
///
/// Fences complete in the order they were issued: when a fence is complete,
/// every earlier fence from the same device life is complete too. That is the
/// whole synchronization promise, and [`crate::conformance`] checks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fence {
    /// Which device life issued it.
    pub epoch: u32,
    /// Its position in that life's submission order, from 1.
    pub value: u64,
}

/// The render hardware interface (`RENDER HARDWARE INTERFACE.md`).
///
/// Everything a renderer may ask a GPU for, and nothing about which GPU API
/// answers. Every rule a backend could disagree on is decided here and checked
/// by [`crate::conformance::run`], which every backend must pass: a buffer
/// write that is valid on one backend is valid on all of them.
///
/// # Errors
///
/// Every method that can fail returns an error in `Domain::Render`: `Reject`
/// for a descriptor or command the caller got wrong, `Retry` for memory that
/// may come back, and `DisableSubsystem` for a lost device: the renderer
/// stands down until [`Rhi::recreate`] brings the device back. Losing a GPU is
/// not a reason to stop the simulation.
pub trait Rhi {
    /// What this backend can do.
    fn capabilities(&self) -> &Capabilities;

    /// Create a buffer.
    ///
    /// # Errors
    ///
    /// Refused by the descriptor rules, out of device memory, or device lost.
    fn create_buffer(&mut self, desc: &BufferDesc) -> Result<BufferHandle>;

    /// Create a texture.
    ///
    /// # Errors
    ///
    /// Refused by the descriptor rules, out of device memory, or device lost.
    fn create_texture(&mut self, desc: &TextureDesc) -> Result<TextureHandle>;

    /// Create a pipeline.
    ///
    /// # Errors
    ///
    /// Refused by the descriptor rules, or device lost.
    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineHandle>;

    /// Destroy a buffer. The handle is unusable at once; its memory returns
    /// when the last submission that used it completes.
    ///
    /// # Errors
    ///
    /// The handle is stale, or the device is lost.
    fn destroy_buffer(&mut self, buffer: BufferHandle) -> Result<()>;

    /// Destroy a texture, with the same deferral as [`Rhi::destroy_buffer`].
    ///
    /// # Errors
    ///
    /// The handle is stale, or the device is lost.
    fn destroy_texture(&mut self, texture: TextureHandle) -> Result<()>;

    /// Destroy a pipeline, with the same deferral as [`Rhi::destroy_buffer`].
    ///
    /// # Errors
    ///
    /// The handle is stale, or the device is lost.
    fn destroy_pipeline(&mut self, pipeline: PipelineHandle) -> Result<()>;

    /// Validate every command, then hand the list to the GPU.
    ///
    /// All or nothing: one invalid command refuses the whole list, and none of
    /// it runs.
    ///
    /// # Errors
    ///
    /// A command breaks a rule on [`Command`], names a stale handle, or the
    /// device is lost.
    fn submit(&mut self, list: CommandList) -> Result<Fence>;

    /// Let finished work retire. Returns the latest completed fence, if any
    /// submission of this device life has completed.
    ///
    /// # Errors
    ///
    /// The device is lost.
    fn poll(&mut self) -> Result<Option<Fence>>;

    /// Whether `fence` has completed.
    fn is_complete(&self, fence: Fence) -> bool;

    /// Block until `fence` completes.
    ///
    /// # Errors
    ///
    /// The fence was never issued, it belongs to a device life that was lost,
    /// or the device is lost.
    fn wait(&mut self, fence: Fence) -> Result<()>;

    /// Show `texture` on the backend's surface.
    ///
    /// # Errors
    ///
    /// The backend has no surface ([`Capabilities::presents`] is false), the
    /// texture is stale or not a render target, or the device is lost.
    fn present(&mut self, texture: TextureHandle) -> Result<()>;

    /// Whether the device has been lost and needs [`Rhi::recreate`].
    fn device_lost(&self) -> bool;

    /// Start a new device life after a loss. Every handle and fence from the
    /// previous life is stale afterwards; the caller recreates what it needs.
    /// Does nothing when the device is not lost.
    ///
    /// # Errors
    ///
    /// The device could not be brought back.
    fn recreate(&mut self) -> Result<()>;

    /// Device memory held right now, including memory of destroyed resources
    /// that in-flight work still uses.
    fn allocated_bytes(&self) -> u64;
}
