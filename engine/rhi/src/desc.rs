//! What the renderer asks the RHI for: formats, usages, descriptors, and what a
//! backend says it can do.

use core::ops::BitOr;

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// A GPU texel format.
///
/// Deliberately short. Every format here is one that Vulkan, Direct3D 12 and
/// Metal all sample natively; a three-channel format is absent because none of
/// them guarantees one, so an RGB image is widened by the caller before upload
/// rather than by a backend that would each do it differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureFormat {
    /// One 8-bit channel, linear.
    R8Unorm,
    /// Two 8-bit channels, linear.
    Rg8Unorm,
    /// Four 8-bit channels, linear.
    Rgba8Unorm,
    /// Four 8-bit channels, colour in sRGB and alpha linear.
    Rgba8UnormSrgb,
    /// One 32-bit float depth channel.
    Depth32Float,
}

impl TextureFormat {
    /// Whether this format holds colour, as opposed to depth.
    #[must_use]
    pub const fn is_color(self) -> bool {
        !matches!(self, Self::Depth32Float)
    }

    /// Every format, in declaration order.
    pub const ALL: [Self; 5] = [
        Self::R8Unorm,
        Self::Rg8Unorm,
        Self::Rgba8Unorm,
        Self::Rgba8UnormSrgb,
        Self::Depth32Float,
    ];

    /// Bytes per texel.
    #[must_use]
    pub const fn bytes_per_texel(self) -> u64 {
        match self {
            Self::R8Unorm => 1,
            Self::Rg8Unorm => 2,
            Self::Rgba8Unorm | Self::Rgba8UnormSrgb | Self::Depth32Float => 4,
        }
    }

    /// Stable lowercase name, safe for logs and reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::R8Unorm => "r8-unorm",
            Self::Rg8Unorm => "rg8-unorm",
            Self::Rgba8Unorm => "rgba8-unorm",
            Self::Rgba8UnormSrgb => "rgba8-unorm-srgb",
            Self::Depth32Float => "depth32-float",
        }
    }
}

/// What a resource may be used for. Combine with `|`.
///
/// A backend allocates differently for each use, and a real driver rejects, or
/// worse silently mishandles, a use the resource was not created for. The RHI
/// refuses it at submission instead, identically on every backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Usage(u32);

impl Usage {
    /// No use at all. A resource created with this is refused.
    pub const NONE: Self = Self(0);
    /// Vertex data a draw reads.
    pub const VERTEX: Self = Self(1);
    /// Index data a draw reads.
    pub const INDEX: Self = Self(1 << 1);
    /// Constants a shader reads.
    pub const UNIFORM: Self = Self(1 << 2);
    /// The source of a copy.
    pub const COPY_SRC: Self = Self(1 << 3);
    /// The destination of a copy or an upload.
    pub const COPY_DST: Self = Self(1 << 4);
    /// A texture a shader samples.
    pub const SAMPLED: Self = Self(1 << 5);
    /// A texture a draw writes to.
    pub const RENDER_TARGET: Self = Self(1 << 6);

    /// Whether every use in `other` is also in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether no use is set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Every use a buffer can have.
    pub const BUFFER: Self = Self(
        Self::VERTEX.0 | Self::INDEX.0 | Self::UNIFORM.0 | Self::COPY_SRC.0 | Self::COPY_DST.0,
    );

    /// Every use a texture can have.
    pub const TEXTURE: Self =
        Self(Self::SAMPLED.0 | Self::COPY_SRC.0 | Self::COPY_DST.0 | Self::RENDER_TARGET.0);
}

impl BitOr for Usage {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// A buffer to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferDesc {
    /// Shown in debug markers and errors. Not an identity.
    pub label: String,
    /// Size in bytes. Non-zero, and a multiple of [`COPY_ALIGNMENT`].
    pub size: u64,
    /// What it may be used for.
    pub usage: Usage,
}

/// A two-dimensional texture to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureDesc {
    /// Shown in debug markers and errors. Not an identity.
    pub label: String,
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// Texel format. Must be in the backend's [`Capabilities::formats`].
    pub format: TextureFormat,
    /// What it may be used for.
    pub usage: Usage,
}

impl TextureDesc {
    /// Bytes one full upload of this texture carries.
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.width as u64 * self.height as u64 * self.format.bytes_per_texel()
    }
}

/// One shader stage of a pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderStage {
    /// The entry point's name.
    pub entry: String,
    /// The shader, as bytes the backend interprets.
    ///
    /// Which language these bytes are in is not decided here: it is decided
    /// with the first native backend (ADR-0025). A backend that cannot check
    /// them says so in [`Capabilities::validates_shaders`].
    pub code: Vec<u8>,
}

/// A graphics pipeline to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineDesc {
    /// Shown in debug markers and errors. Not an identity.
    pub label: String,
    /// Runs once per vertex.
    pub vertex: ShaderStage,
    /// Runs once per covered sample.
    pub fragment: ShaderStage,
    /// Bytes per vertex in the vertex buffer. Non-zero, a multiple of four.
    pub vertex_stride: u32,
    /// The formats a draw with this pipeline may write to. At least one.
    pub targets: Vec<TextureFormat>,
}

/// Offsets and lengths of buffer writes are multiples of this.
///
/// Four bytes is what Vulkan's `vkCmdUpdateBuffer`, Direct3D 12's copy
/// placement and Metal's blit all accept; holding every backend to the
/// strictest common rule is what lets a write that works on one work on all.
pub const COPY_ALIGNMENT: u64 = 4;

/// What a backend can do. Read before creating anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    /// Which backend this is, e.g. `null`.
    pub backend: &'static str,
    /// The device, as the backend names it. Never a serial number.
    pub device: String,
    /// The longest edge a texture may have.
    pub max_texture_edge: u32,
    /// The largest buffer, in bytes.
    pub max_buffer_bytes: u64,
    /// Device memory the backend will hand out in total, in bytes.
    pub memory_bytes: u64,
    /// The formats textures may be created in.
    pub formats: Vec<TextureFormat>,
    /// Whether [`crate::Rhi::present`] can show anything.
    pub presents: bool,
    /// Whether pipeline creation checks shader code, or only its shape.
    pub validates_shaders: bool,
}

impl Capabilities {
    /// Whether textures may be created in `format`.
    #[must_use]
    pub fn supports(&self, format: TextureFormat) -> bool {
        self.formats.contains(&format)
    }
}

/// A descriptor or command the caller got wrong. Never retried unchanged.
pub fn refused(message: &'static str) -> Error {
    Error::new(Domain::Render, "rhi", message).with_recovery(Recovery::Reject)
}

/// Check a buffer descriptor against the rules every backend shares.
///
/// # Errors
///
/// Names the rule the descriptor breaks.
pub fn check_buffer(desc: &BufferDesc, caps: &Capabilities) -> Result<()> {
    if desc.usage.is_empty() {
        return Err(refused("a buffer needs at least one usage").with_context("label", &desc.label));
    }
    if !Usage::BUFFER.contains(desc.usage) {
        return Err(refused("a buffer was given a usage only textures have")
            .with_context("label", &desc.label));
    }
    if desc.size == 0 || desc.size % COPY_ALIGNMENT != 0 {
        return Err(
            refused("a buffer's size must be non-zero and a multiple of four")
                .with_context("label", &desc.label)
                .with_context("size", desc.size.to_string()),
        );
    }
    if desc.size > caps.max_buffer_bytes {
        return Err(refused("the buffer is larger than this backend allows")
            .with_context("label", &desc.label)
            .with_context("size", desc.size.to_string())
            .with_context("max", caps.max_buffer_bytes.to_string()));
    }
    Ok(())
}

/// Check a texture descriptor against the rules every backend shares.
///
/// # Errors
///
/// Names the rule the descriptor breaks.
pub fn check_texture(desc: &TextureDesc, caps: &Capabilities) -> Result<()> {
    if desc.usage.is_empty() {
        return Err(
            refused("a texture needs at least one usage").with_context("label", &desc.label)
        );
    }
    if !Usage::TEXTURE.contains(desc.usage) {
        return Err(refused("a texture was given a usage only buffers have")
            .with_context("label", &desc.label));
    }
    if !caps.supports(desc.format) {
        return Err(
            refused("this backend cannot create textures in that format")
                .with_context("label", &desc.label)
                .with_context("format", desc.format.as_str()),
        );
    }
    let longest = desc.width.max(desc.height);
    if desc.width == 0 || desc.height == 0 || longest > caps.max_texture_edge {
        return Err(
            refused("the texture's size is outside what this backend allows")
                .with_context("label", &desc.label)
                .with_context("size", format!("{}x{}", desc.width, desc.height))
                .with_context("max_edge", caps.max_texture_edge.to_string()),
        );
    }
    if desc.format == TextureFormat::Depth32Float && desc.usage.contains(Usage::COPY_DST) {
        // Depth is written by draws. No backend here uploads depth from the
        // CPU the same way, so none of them is allowed to.
        return Err(refused("a depth texture cannot be an upload destination")
            .with_context("label", &desc.label));
    }
    Ok(())
}

/// Check a pipeline descriptor's shape against the rules every backend shares.
///
/// # Errors
///
/// Names the rule the descriptor breaks.
pub fn check_pipeline(desc: &PipelineDesc, caps: &Capabilities) -> Result<()> {
    for (stage, shader) in [("vertex", &desc.vertex), ("fragment", &desc.fragment)] {
        if shader.entry.is_empty() || shader.code.is_empty() {
            return Err(refused("a shader stage needs an entry point and code")
                .with_context("label", &desc.label)
                .with_context("stage", stage));
        }
    }
    if desc.vertex_stride == 0 || u64::from(desc.vertex_stride) % COPY_ALIGNMENT != 0 {
        return Err(
            refused("a vertex stride must be non-zero and a multiple of four")
                .with_context("label", &desc.label)
                .with_context("stride", desc.vertex_stride.to_string()),
        );
    }
    if desc.targets.is_empty() {
        return Err(refused("a pipeline needs at least one target format")
            .with_context("label", &desc.label));
    }
    if let Some(format) = desc.targets.iter().find(|format| !format.is_color()) {
        return Err(
            refused("a pipeline's targets are colour formats; depth is not one")
                .with_context("label", &desc.label)
                .with_context("format", format.as_str()),
        );
    }
    if let Some(format) = desc.targets.iter().find(|format| !caps.supports(**format)) {
        return Err(refused("this backend cannot render to that format")
            .with_context("label", &desc.label)
            .with_context("format", format.as_str()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps() -> Capabilities {
        Capabilities {
            backend: "test",
            device: "test".into(),
            max_texture_edge: 64,
            max_buffer_bytes: 1024,
            memory_bytes: 4096,
            formats: vec![TextureFormat::Rgba8Unorm],
            presents: false,
            validates_shaders: false,
        }
    }

    #[test]
    fn usages_combine_and_contain() {
        let usage = Usage::VERTEX | Usage::COPY_DST;
        assert!(usage.contains(Usage::VERTEX));
        assert!(usage.contains(Usage::COPY_DST));
        assert!(!usage.contains(Usage::INDEX));
        assert!(!usage.contains(Usage::VERTEX | Usage::INDEX));
        assert!(Usage::NONE.is_empty());
    }

    #[test]
    fn a_texture_knows_its_upload_size() {
        let desc = TextureDesc {
            label: "t".into(),
            width: 16,
            height: 8,
            format: TextureFormat::Rg8Unorm,
            usage: Usage::SAMPLED,
        };
        assert_eq!(desc.byte_len(), 16 * 8 * 2);
    }

    #[test]
    fn buffers_are_refused_for_each_rule_they_break() {
        let caps = caps();
        let ok = BufferDesc {
            label: "b".into(),
            size: 16,
            usage: Usage::VERTEX,
        };
        assert!(check_buffer(&ok, &caps).is_ok());
        for bad in [
            BufferDesc {
                usage: Usage::VERTEX | Usage::RENDER_TARGET,
                ..ok.clone()
            },
            BufferDesc {
                size: 0,
                ..ok.clone()
            },
            BufferDesc {
                size: 6,
                ..ok.clone()
            },
            BufferDesc {
                size: 2048,
                ..ok.clone()
            },
            BufferDesc {
                usage: Usage::NONE,
                ..ok.clone()
            },
        ] {
            let error = check_buffer(&bad, &caps).expect_err("refused");
            assert_eq!(error.domain(), Domain::Render);
        }
    }

    #[test]
    fn textures_are_refused_for_each_rule_they_break() {
        let caps = caps();
        let ok = TextureDesc {
            label: "t".into(),
            width: 16,
            height: 16,
            format: TextureFormat::Rgba8Unorm,
            usage: Usage::SAMPLED | Usage::COPY_DST,
        };
        assert!(check_texture(&ok, &caps).is_ok());
        for bad in [
            TextureDesc {
                usage: Usage::SAMPLED | Usage::VERTEX,
                ..ok.clone()
            },
            TextureDesc {
                width: 0,
                ..ok.clone()
            },
            TextureDesc {
                height: 65,
                ..ok.clone()
            },
            TextureDesc {
                format: TextureFormat::R8Unorm,
                ..ok.clone()
            },
            TextureDesc {
                usage: Usage::NONE,
                ..ok.clone()
            },
        ] {
            assert!(check_texture(&bad, &caps).is_err(), "{bad:?}");
        }
        let mut depth = caps.clone();
        depth.formats.push(TextureFormat::Depth32Float);
        let upload_depth = TextureDesc {
            format: TextureFormat::Depth32Float,
            ..ok
        };
        assert!(check_texture(&upload_depth, &depth).is_err());
    }

    #[test]
    fn pipelines_are_refused_for_each_rule_they_break() {
        let caps = caps();
        let stage = ShaderStage {
            entry: "main".into(),
            code: vec![1, 2, 3, 4],
        };
        let ok = PipelineDesc {
            label: "p".into(),
            vertex: stage.clone(),
            fragment: stage.clone(),
            vertex_stride: 12,
            targets: vec![TextureFormat::Rgba8Unorm],
        };
        assert!(check_pipeline(&ok, &caps).is_ok());
        let no_entry = ShaderStage {
            entry: String::new(),
            ..stage.clone()
        };
        let no_code = ShaderStage {
            code: Vec::new(),
            ..stage
        };
        let mut with_depth = caps.clone();
        with_depth.formats.push(TextureFormat::Depth32Float);
        let depth_target = PipelineDesc {
            targets: vec![TextureFormat::Depth32Float],
            ..ok.clone()
        };
        assert!(
            check_pipeline(&depth_target, &with_depth).is_err(),
            "depth is not a colour target, even where it is supported"
        );
        for bad in [
            PipelineDesc {
                vertex: no_entry,
                ..ok.clone()
            },
            PipelineDesc {
                fragment: no_code,
                ..ok.clone()
            },
            PipelineDesc {
                vertex_stride: 0,
                ..ok.clone()
            },
            PipelineDesc {
                vertex_stride: 6,
                ..ok.clone()
            },
            PipelineDesc {
                targets: Vec::new(),
                ..ok.clone()
            },
            PipelineDesc {
                targets: vec![TextureFormat::R8Unorm],
                ..ok.clone()
            },
        ] {
            assert!(check_pipeline(&bad, &caps).is_err(), "{bad:?}");
        }
    }
}
