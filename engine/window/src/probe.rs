//! Evidence that a frame reaches a window, not just that `present` returned.
//!
//! The probe opens a window, runs the conformance suite on a backend that
//! presents, and then shows a 16x16 render target (the first visual
//! generation's size) for a number of frames. The target holds four coloured
//! quadrants, so a frame shown upside down, mirrored, or with red and blue
//! swapped is caught. On surfaces that allow it, the first frame is read back
//! from the surface image itself and compared texel by texel.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_rhi::conformance;
use nexora_rhi::{Command, CommandList, Rhi, TextureDesc, TextureFormat, TextureHandle, Usage};
use nexora_rhi_wgpu::{conformance_shaders, AdapterClass, PresentedFrame, SurfaceInfo, WgpuRhi};

use crate::{run, Client, Flow, WindowFacts, WindowSpec};

/// The shown target's edge, in texels.
pub const EDGE: u32 = 16;

/// The quadrants' colours, as RGBA8: top-left, top-right, bottom-left,
/// bottom-right. Only 0 and 255, so the colour space cannot change them.
pub const QUADRANTS: [[u8; 4]; 4] = [
    [255, 0, 0, 255],
    [0, 255, 0, 255],
    [0, 0, 255, 255],
    [255, 255, 255, 255],
];

/// What the probe observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeReport {
    /// The adapter the window's device runs on.
    pub adapter: AdapterClass,
    /// The window as opened.
    pub window: WindowFacts,
    /// The surface, after the run.
    pub surface: SurfaceInfo,
    /// Conformance cases passed, with presentation on.
    pub conformance: usize,
    /// The first frame, read back from the surface; `None` where the surface
    /// cannot be read.
    pub captured: Option<Captured>,
    /// Frames the host completed.
    pub frames: u64,
}

/// The first frame as the window received it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Captured {
    /// Window texels whose colour was determined by the target.
    pub checked: usize,
    /// Of those, how many showed the right quadrant's colour.
    pub matching: usize,
}

/// Open a window of `width` × `height`, and present `frames` frames in it.
///
/// # Errors
///
/// The window, the device or the conformance suite failed, or a frame read
/// back from the window did not show the target.
pub fn run_probe(width: u32, height: u32, frames: u64) -> Result<ProbeReport> {
    let spec = WindowSpec {
        title: "NEXORA window probe".into(),
        width,
        height,
    };
    let mut probe = Probe {
        frames: frames.max(1),
        state: None,
    };
    let session = run(&spec, &mut probe)?;
    let state = probe.state.ok_or_else(|| {
        wrong("the window closed before the probe ran").with_recovery(Recovery::Retry)
    })?;
    if session.frames < probe.frames {
        return Err(wrong("the window closed before every frame was presented")
            .with_context("presented", session.frames.to_string())
            .with_context("wanted", probe.frames.to_string()));
    }
    Ok(ProbeReport {
        adapter: state.adapter,
        window: session.window,
        surface: state
            .surface
            .ok_or_else(|| wrong("the probe ended without recording its surface"))?,
        conformance: state.conformance,
        captured: state.captured,
        frames: session.frames,
    })
}

/// Check a frame read back from the surface against [`QUADRANTS`], stretched
/// to its size by nearest sampling.
///
/// # Errors
///
/// The frame's format is not one this check reads.
pub fn check_frame(frame: &PresentedFrame) -> Result<Captured> {
    let bgra = match frame.format.as_str() {
        "Bgra8Unorm" | "Bgra8UnormSrgb" => true,
        "Rgba8Unorm" | "Rgba8UnormSrgb" => false,
        _ => {
            return Err(wrong("a frame in a format the probe does not read")
                .with_context("format", &frame.format))
        }
    };
    let (width, height) = (frame.width as usize, frame.height as usize);
    let mut checked = 0;
    let mut matching = 0;
    for (index, texel) in frame.texels.chunks_exact(4).enumerate() {
        let (x, y) = (index % width, index / width);
        // The target texel under this pixel's centre, as nearest sampling
        // picks it. A centre exactly on a texel edge could go either way.
        let (Some(tx), Some(ty)) = (texel_under(x, width), texel_under(y, height)) else {
            continue;
        };
        let quadrant = usize::from(tx >= EDGE / 2) + 2 * usize::from(ty >= EDGE / 2);
        let want = QUADRANTS[quadrant];
        let got = if bgra {
            [texel[2], texel[1], texel[0], texel[3]]
        } else {
            [texel[0], texel[1], texel[2], texel[3]]
        };
        checked += 1;
        if got == want {
            matching += 1;
        }
    }
    Ok(Captured { checked, matching })
}

/// The texel under pixel `at`'s centre, across `span` pixels; `None` on an
/// edge between two texels.
fn texel_under(at: usize, span: usize) -> Option<u32> {
    let numerator = (2 * at + 1) * EDGE as usize;
    let denominator = 2 * span;
    (!numerator.is_multiple_of(denominator)).then(|| (numerator / denominator) as u32)
}

struct Probe {
    frames: u64,
    state: Option<State>,
}

struct State {
    adapter: AdapterClass,
    conformance: usize,
    target: TextureHandle,
    shown: u64,
    captured: Option<Captured>,
    surface: Option<SurfaceInfo>,
}

impl Client for Probe {
    fn opened(&mut self, rhi: &mut WgpuRhi, _window: &WindowFacts) -> Result<()> {
        if !rhi.capabilities().presents {
            return Err(wrong("a backend opened on a window says it cannot present"));
        }
        let report = conformance::run(rhi, &conformance_shaders())?;
        let target = rhi.create_texture(&TextureDesc {
            label: "window probe".into(),
            width: EDGE,
            height: EDGE,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: Usage::RENDER_TARGET | Usage::COPY_DST,
        })?;
        let mut texels = Vec::with_capacity((EDGE * EDGE * 4) as usize);
        for y in 0..EDGE {
            for x in 0..EDGE {
                let quadrant = usize::from(x >= EDGE / 2) + 2 * usize::from(y >= EDGE / 2);
                texels.extend_from_slice(&QUADRANTS[quadrant]);
            }
        }
        let mut upload = CommandList::new("window probe");
        upload.push(Command::WriteTexture {
            texture: target,
            data: texels,
        });
        let fence = rhi.submit(upload)?;
        rhi.wait(fence)?;
        self.state = Some(State {
            adapter: rhi.adapter().clone(),
            conformance: report.passed.len(),
            target,
            shown: 0,
            captured: None,
            surface: None,
        });
        Ok(())
    }

    fn frame(&mut self, rhi: &mut WgpuRhi) -> Result<Flow> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| wrong("a frame before the window opened"))?;
        if state.shown == 0 {
            if let Some(frame) = rhi.present_and_capture(state.target)? {
                let captured = check_frame(&frame)?;
                if captured.checked == 0 || captured.matching != captured.checked {
                    return Err(wrong("the window did not show the target")
                        .with_context("matching", captured.matching.to_string())
                        .with_context("checked", captured.checked.to_string())
                        .with_context("format", &frame.format));
                }
                state.captured = Some(captured);
            }
        } else {
            rhi.present(state.target)?;
        }
        state.shown += 1;
        if state.shown < self.frames {
            return Ok(Flow::Continue);
        }
        state.surface = rhi.surface();
        rhi.destroy_texture(state.target)?;
        Ok(Flow::Exit)
    }
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "window-probe", message).with_recovery(Recovery::DisableSubsystem)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(width: u32, height: u32, bgra: bool, flip: bool) -> PresentedFrame {
        let mut texels = Vec::new();
        for y in 0..height as usize {
            for x in 0..width as usize {
                let tx = texel_under(x, width as usize).unwrap_or(0);
                let mut ty = texel_under(y, height as usize).unwrap_or(0);
                if flip {
                    ty = EDGE - 1 - ty;
                }
                let q = QUADRANTS[usize::from(tx >= EDGE / 2) + 2 * usize::from(ty >= EDGE / 2)];
                if bgra {
                    texels.extend_from_slice(&[q[2], q[1], q[0], q[3]]);
                } else {
                    texels.extend_from_slice(&q);
                }
            }
        }
        PresentedFrame {
            width,
            height,
            format: if bgra { "Bgra8UnormSrgb" } else { "Rgba8Unorm" }.into(),
            texels,
        }
    }

    #[test]
    fn a_frame_shown_as_drawn_matches_in_either_channel_order() {
        for bgra in [true, false] {
            let captured = check_frame(&frame(64, 48, bgra, false)).unwrap();
            assert_eq!(captured.checked, 64 * 48);
            assert_eq!(captured.matching, captured.checked);
        }
    }

    #[test]
    fn a_flipped_or_swizzled_frame_does_not() {
        let flipped = check_frame(&frame(64, 64, true, true)).unwrap();
        assert!(flipped.matching < flipped.checked);
        let mut swapped = frame(64, 64, true, false);
        swapped.format = "Rgba8Unorm".into();
        let swapped = check_frame(&swapped).unwrap();
        assert!(swapped.matching < swapped.checked);
    }

    #[test]
    fn pixels_on_a_texel_edge_are_not_judged() {
        // 16 texels over 8 pixels: every pixel centre lands on an edge.
        assert_eq!(texel_under(0, 8), None);
        assert_eq!(texel_under(0, 32), Some(0));
        assert_eq!(texel_under(31, 32), Some(15));
        assert_eq!(texel_under(128, 256), Some(8));
    }
}
