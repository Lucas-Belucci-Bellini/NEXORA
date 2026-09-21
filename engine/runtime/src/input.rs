//! The input system — `INPUT SYSTEM.md`, `CORE.md` §24 (ENGINE-8).
//!
//! The spec opens with the sentence the whole module is built to obey: *"Input
//! traduz sinais de hardware e entrada local em intenções sem executar
//! gameplay."* Input translates signals into **intent**. It does not move a
//! player, it does not break a block, it does not touch the world. What it
//! produces is an [`InputSnapshot`], and something above — the command system,
//! which already exists and already validates authority — decides whether the
//! intent is allowed to become a change.
//!
//! The chain the document draws, and where each link lives here:
//!
//! ```text
//! Hardware            the host's business, not this module's
//! -> Raw Input        Signal
//! -> Device Abstract. DeviceKind / DeviceId
//! -> Input Actions    ActionDefinition, named nexora:action/...
//! -> Mapping Context  set_context, resolved by priority
//! -> Modifiers/Chords Binding::with_chord
//! -> Player/UI Intent InputSnapshot
//! -> Command          above this crate, deliberately
//! ```
//!
//! ## No device is polled here
//!
//! The same rule that shapes [`crate::frame`] and `nexora_foundation::time`: a
//! system that reads the hardware itself cannot be replayed. Nothing in this
//! module opens a device, registers a callback or asks an operating system
//! anything. The host measures the world and hands it in — [`InputSystem::sample`]
//! takes an [`InputFrame`] of [`Signal`]s exactly as `WorldClock::advance` takes
//! a delta. Feed the same signals and you get the same intent, on any machine,
//! which is what `INPUT SYSTEM.md` means by *"deterministic input replay"* and
//! what makes every test below a real test rather than a rehearsal.
//!
//! ## The three decisions worth knowing before reading the code
//!
//! **Bindings name a device *kind*; signals name a device *instance*.** A
//! player binds "gamepad button 0" once, not once per controller they might
//! plug in. The snapshot is therefore the same whether the stick that moved was
//! gamepad 0 or gamepad 3 — and unplugging one is not a remap.
//!
//! **A context consumes a source.** When two active contexts bind the same
//! button, only the higher-priority one fires. That is the entire reason
//! priorities exist: a menu at priority 100 must swallow the key so the player
//! under it does not also jump. Among bindings in reach, the longest chord wins,
//! because otherwise `Ctrl+S` could never beat `S` and chords would be
//! decorative.
//!
//! **A detached device releases everything it held.** A player who unplugs a
//! controller mid-stride would otherwise walk forward until the process ends —
//! the state lives here, not in the hardware, so nothing else can clear it.
//!
//! ## What is deliberately absent
//!
//! No key names. [`ButtonCode`] is an opaque `u16` the host assigns, because
//! `CORE.md` §24 asks that *"o jogo não depende diretamente de teclas"* and a
//! scancode table in the engine is a table the engine then has to be right
//! about on four platforms. No touch gestures, no pointer deltas: an action
//! axis here is a normalised intention in `[-1, 1]`, and a mouse-look delta in
//! pixels is a different quantity that will want its own kind rather than a
//! reinterpretation of this one. No suppression of a chord's own modifiers —
//! binding `Ctrl` and `Ctrl+S` in one context fires both, and pretending
//! otherwise needs a lookahead this module does not have.
//!
//! ```
//! use nexora_foundation::ident::Identifier;
//! use nexora_runtime::input::{
//!     ActionDefinition, ActionKind, Binding, ButtonCode, DeviceId, DeviceKind, InputFrame,
//!     InputSystem, Signal, Source,
//! };
//!
//! let jump = Identifier::nexora("action/jump")?;
//! let gameplay = Identifier::nexora("context/gameplay")?;
//! let space = Source::button(DeviceKind::Keyboard, ButtonCode(57));
//!
//! let mut input = InputSystem::new();
//! input.register_action(ActionDefinition::new(jump.clone(), ActionKind::Button))?;
//! input.set_context(gameplay.clone(), 0);
//! input.bind(Binding::new(gameplay, jump.clone(), space))?;
//!
//! let keyboard = DeviceId::new(DeviceKind::Keyboard, 0);
//! let snapshot = input.sample(
//!     &InputFrame::new()
//!         .with(Signal::Attached(keyboard))
//!         .with(Signal::button(keyboard, ButtonCode(57), true)),
//! );
//! assert!(snapshot.just_pressed(&jump));
//! # Ok::<(), nexora_foundation::error::Error>(())
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

/// A class of input device.
///
/// Bindings are written against a kind rather than an instance: see the module
/// documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceKind {
    /// A keyboard.
    Keyboard,
    /// A mouse or other indirect pointer.
    Mouse,
    /// A gamepad or joystick.
    Gamepad,
    /// A touch surface.
    Touch,
}

impl DeviceKind {
    /// Every kind, in the order `INPUT SYSTEM.md` lists them.
    pub const ALL: [Self; 4] = [Self::Keyboard, Self::Mouse, Self::Gamepad, Self::Touch];

    /// Stable lowercase name, used in logs and in the binding file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Gamepad => "gamepad",
            Self::Touch => "touch",
        }
    }

    /// The kind with this name, if any.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == name)
    }
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One physical device: a kind and which one of that kind it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId {
    kind: DeviceKind,
    index: u8,
}

impl DeviceId {
    /// Name the `index`-th device of `kind`.
    #[must_use]
    pub const fn new(kind: DeviceKind, index: u8) -> Self {
        Self { kind, index }
    }

    /// Which class of device this is.
    #[must_use]
    pub const fn kind(self) -> DeviceKind {
        self.kind
    }

    /// Which one of that class this is.
    #[must_use]
    pub const fn index(self) -> u8 {
        self.index
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.kind, self.index)
    }
}

/// A digital control on a device, as the host numbers it.
///
/// Deliberately opaque: see *What is deliberately absent* in the module
/// documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ButtonCode(pub u16);

/// An analogue control on a device, as the host numbers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AxisCode(pub u16);

/// What a binding listens to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// A digital control on any device of this kind.
    Button {
        /// The class of device.
        device: DeviceKind,
        /// The control's code.
        code: ButtonCode,
    },
    /// An analogue control on any device of this kind.
    Axis {
        /// The class of device.
        device: DeviceKind,
        /// The control's code.
        code: AxisCode,
    },
}

impl Source {
    /// A digital source.
    #[must_use]
    pub const fn button(device: DeviceKind, code: ButtonCode) -> Self {
        Self::Button { device, code }
    }

    /// An analogue source.
    #[must_use]
    pub const fn axis(device: DeviceKind, code: AxisCode) -> Self {
        Self::Axis { device, code }
    }

    /// Which class of device this source is on.
    #[must_use]
    pub const fn device(self) -> DeviceKind {
        match self {
            Self::Button { device, .. } | Self::Axis { device, .. } => device,
        }
    }

    /// Whether this is a digital control.
    #[must_use]
    pub const fn is_button(self) -> bool {
        matches!(self, Self::Button { .. })
    }

    /// The canonical text form, `kind/button/code` or `kind/axis/code`.
    #[must_use]
    pub fn encode(self) -> String {
        match self {
            Self::Button { device, code } => format!("{}/button/{}", device.as_str(), code.0),
            Self::Axis { device, code } => format!("{}/axis/{}", device.as_str(), code.0),
        }
    }

    /// Parse the form [`Source::encode`] writes.
    ///
    /// # Errors
    ///
    /// Returns an error when the text is not three `/`-separated fields naming
    /// a known device kind, a known control class and a `u16` code.
    pub fn decode(text: &str) -> Result<Self> {
        let mut fields = text.split('/');
        let (Some(kind), Some(class), Some(code), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(reject("a source is written kind/class/code").with_context("source", text));
        };
        let Some(device) = DeviceKind::parse(kind) else {
            return Err(reject("unknown device kind").with_context("kind", kind.to_owned()));
        };
        let Ok(code) = code.parse::<u16>() else {
            return Err(reject("a control code is a u16").with_context("code", code.to_owned()));
        };
        match class {
            "button" => Ok(Self::button(device, ButtonCode(code))),
            "axis" => Ok(Self::axis(device, AxisCode(code))),
            other => {
                Err(reject("a control is a button or an axis")
                    .with_context("class", other.to_owned()))
            }
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

/// One thing the host observed, handed to [`InputSystem::sample`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// A digital control changed state.
    Button {
        /// Which device it was on.
        device: DeviceId,
        /// Which control.
        code: ButtonCode,
        /// Whether it is now down.
        pressed: bool,
    },
    /// An analogue control reported a value, before dead zone or curve.
    Axis {
        /// Which device it was on.
        device: DeviceId,
        /// Which control.
        code: AxisCode,
        /// The raw reading, nominally in `[-1, 1]`.
        value: f32,
    },
    /// A device became available.
    Attached(DeviceId),
    /// A device went away. Everything it held is released.
    Detached(DeviceId),
}

impl Signal {
    /// A digital control changing state.
    #[must_use]
    pub const fn button(device: DeviceId, code: ButtonCode, pressed: bool) -> Self {
        Self::Button {
            device,
            code,
            pressed,
        }
    }

    /// An analogue reading.
    #[must_use]
    pub const fn axis(device: DeviceId, code: AxisCode, value: f32) -> Self {
        Self::Axis {
            device,
            code,
            value,
        }
    }

    /// Which device produced this, if it names one.
    #[must_use]
    pub const fn device(self) -> DeviceId {
        match self {
            Self::Button { device, .. }
            | Self::Axis { device, .. }
            | Self::Attached(device)
            | Self::Detached(device) => device,
        }
    }
}

/// The signals of one frame.
///
/// Named for the spec's `sample(frame: InputFrame)`. It is a plain list: the
/// order inside a frame is the order the host observed things in, and two
/// presses of the same button in one frame collapse to the last one, as they
/// would on any sampled input.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputFrame {
    signals: Vec<Signal>,
}

impl InputFrame {
    /// An empty frame — nothing happened, which is still a frame.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            signals: Vec::new(),
        }
    }

    /// Add a signal, for chaining.
    #[must_use]
    pub fn with(mut self, signal: Signal) -> Self {
        self.signals.push(signal);
        self
    }

    /// Add a signal.
    pub fn push(&mut self, signal: Signal) {
        self.signals.push(signal);
    }

    /// The signals, in the order they were observed.
    #[must_use]
    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    /// How many signals this frame carries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    /// Whether the frame is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
}

/// How an analogue reading is shaped before it becomes intent.
///
/// `INPUT SYSTEM.md` asks for *"dead zones, sensitivity e curvas"*. The order
/// is dead zone, rescale, curve, sensitivity, invert — and the rescale is the
/// part that is easy to leave out and wrong to: without it a stick with a 0.2
/// dead zone jumps from `0.0` to `0.2` the instant it is nudged, and the player
/// feels a notch that is not in the hardware.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisTuning {
    dead_zone: f32,
    sensitivity: f32,
    curve: ResponseCurve,
    invert: bool,
}

/// How an axis responds between its dead zone and its limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ResponseCurve {
    /// Proportional.
    #[default]
    Linear,
    /// Squared: finer control near the centre.
    Quadratic,
    /// Cubed: finer still.
    Cubic,
}

impl ResponseCurve {
    /// Every curve.
    pub const ALL: [Self; 3] = [Self::Linear, Self::Quadratic, Self::Cubic];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Quadratic => "quadratic",
            Self::Cubic => "cubic",
        }
    }

    /// The curve with this name, if any.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|curve| curve.as_str() == name)
    }

    /// Shape a magnitude already normalised to `[0, 1]`.
    #[must_use]
    pub fn apply(self, magnitude: f32) -> f32 {
        match self {
            Self::Linear => magnitude,
            Self::Quadratic => magnitude * magnitude,
            Self::Cubic => magnitude * magnitude * magnitude,
        }
    }
}

impl Default for AxisTuning {
    fn default() -> Self {
        Self {
            dead_zone: 0.0,
            sensitivity: 1.0,
            curve: ResponseCurve::Linear,
            invert: false,
        }
    }
}

impl AxisTuning {
    /// The untouched tuning: no dead zone, unit sensitivity, linear, upright.
    pub const NONE: Self = Self {
        dead_zone: 0.0,
        sensitivity: 1.0,
        curve: ResponseCurve::Linear,
        invert: false,
    };

    /// Build a tuning.
    ///
    /// # Errors
    ///
    /// Returns an error when the dead zone is outside `[0, 1)` or the
    /// sensitivity is not a positive finite number. A dead zone of exactly 1
    /// would silence the axis entirely, which is a broken remap rather than a
    /// preference.
    pub fn new(
        dead_zone: f32,
        sensitivity: f32,
        curve: ResponseCurve,
        invert: bool,
    ) -> Result<Self> {
        if !dead_zone.is_finite() || !(0.0..1.0).contains(&dead_zone) {
            return Err(
                reject("a dead zone is in [0, 1)").with_context("dead_zone", dead_zone.to_string())
            );
        }
        if !sensitivity.is_finite() || sensitivity <= 0.0 {
            return Err(reject("sensitivity is a positive finite number")
                .with_context("sensitivity", sensitivity.to_string()));
        }
        Ok(Self {
            dead_zone,
            sensitivity,
            curve,
            invert,
        })
    }

    /// The dead zone, as a fraction of full travel.
    #[must_use]
    pub const fn dead_zone(self) -> f32 {
        self.dead_zone
    }

    /// The multiplier applied after the curve.
    #[must_use]
    pub const fn sensitivity(self) -> f32 {
        self.sensitivity
    }

    /// The response curve.
    #[must_use]
    pub const fn curve(self) -> ResponseCurve {
        self.curve
    }

    /// Whether the axis is flipped.
    #[must_use]
    pub const fn invert(self) -> bool {
        self.invert
    }

    /// Shape one raw reading into a normalised `[-1, 1]` intention.
    ///
    /// A reading that is not finite becomes `0.0` rather than propagating: a
    /// `NaN` from a driver must not reach comparisons downstream.
    #[must_use]
    pub fn apply(self, raw: f32) -> f32 {
        if !raw.is_finite() {
            return 0.0;
        }
        let clamped = raw.clamp(-1.0, 1.0);
        let magnitude = clamped.abs();
        if magnitude <= self.dead_zone {
            return 0.0;
        }
        let span = 1.0 - self.dead_zone;
        let normalised = ((magnitude - self.dead_zone) / span).clamp(0.0, 1.0);
        let shaped = (self.curve.apply(normalised) * self.sensitivity).clamp(0.0, 1.0);
        let signed = shaped.copysign(clamped);
        if self.invert {
            -signed
        } else {
            signed
        }
    }

    /// Whether this is the untouched tuning, and so need not be written out.
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::NONE
    }
}

/// Whether an action is a switch or a dial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActionKind {
    /// On or off: `JUMP`, `ATTACK`, `INVENTORY`.
    Button,
    /// A signed magnitude in `[-1, 1]`: a movement or look axis.
    Axis,
}

impl ActionKind {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Axis => "axis",
        }
    }
}

/// An action the game understands, independent of any device.
///
/// `CORE.md` §24 names the shape of these: `MOVE_FORWARD`, `JUMP`, `ATTACK`.
/// Here they are namespaced identifiers, `nexora:action/jump`, so a mod can add
/// one without colliding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDefinition {
    id: Identifier,
    kind: ActionKind,
}

impl ActionDefinition {
    /// Declare an action.
    #[must_use]
    pub const fn new(id: Identifier, kind: ActionKind) -> Self {
        Self { id, kind }
    }

    /// The action's identifier.
    #[must_use]
    pub const fn id(&self) -> &Identifier {
        &self.id
    }

    /// Whether it is a switch or a dial.
    #[must_use]
    pub const fn kind(&self) -> ActionKind {
        self.kind
    }
}

/// One mapping: in this context, this source (with these modifiers held) means
/// this action.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    context: Identifier,
    action: Identifier,
    source: Source,
    chord: Vec<Source>,
    tuning: AxisTuning,
}

impl Binding {
    /// Map a source to an action inside a context.
    #[must_use]
    pub fn new(context: Identifier, action: Identifier, source: Source) -> Self {
        Self {
            context,
            action,
            source,
            chord: Vec::new(),
            tuning: AxisTuning::NONE,
        }
    }

    /// Require these buttons to be held as well.
    ///
    /// The chord is stored sorted and deduplicated, so `Ctrl+Shift` and
    /// `Shift+Ctrl` are one binding and conflict with each other.
    ///
    /// # Errors
    ///
    /// Returns an error when a chord member is an axis: an analogue control is
    /// not something that can be *held*, and accepting one would silently
    /// produce a binding that never fires.
    pub fn with_chord(mut self, chord: &[Source]) -> Result<Self> {
        if let Some(bad) = chord.iter().find(|source| !source.is_button()) {
            return Err(reject("a chord is made of buttons").with_context("source", bad.encode()));
        }
        let mut members: Vec<Source> = chord.to_vec();
        members.sort_unstable();
        members.dedup();
        self.chord = members;
        Ok(self)
    }

    /// Shape the analogue reading this binding produces.
    #[must_use]
    pub const fn with_tuning(mut self, tuning: AxisTuning) -> Self {
        self.tuning = tuning;
        self
    }

    /// The context this mapping belongs to.
    #[must_use]
    pub const fn context(&self) -> &Identifier {
        &self.context
    }

    /// The action this mapping produces.
    #[must_use]
    pub const fn action(&self) -> &Identifier {
        &self.action
    }

    /// The control this mapping listens to.
    #[must_use]
    pub const fn source(&self) -> Source {
        self.source
    }

    /// The modifiers that must also be held, sorted.
    #[must_use]
    pub fn chord(&self) -> &[Source] {
        &self.chord
    }

    /// How this mapping shapes an analogue reading.
    #[must_use]
    pub const fn tuning(&self) -> AxisTuning {
        self.tuning
    }

    /// What makes two bindings the same mapping: context, source and chord.
    ///
    /// The action is not part of it. Binding the same key to two actions in one
    /// context is exactly the conflict `INPUT SYSTEM.md` asks to detect.
    fn slot(&self) -> (&Identifier, Source, &[Source]) {
        (&self.context, self.source, &self.chord)
    }
}

/// What one action is doing this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionState {
    value: f32,
    pressed: bool,
    just_pressed: bool,
    just_released: bool,
}

impl ActionState {
    /// Not doing anything.
    pub const NEUTRAL: Self = Self {
        value: 0.0,
        pressed: false,
        just_pressed: false,
        just_released: false,
    };

    /// Build a state directly.
    ///
    /// Used by the sampler, by tests, and by a peer reconstructing a snapshot
    /// it received — which is why [`InputSystem::validate_remote`] exists.
    #[must_use]
    pub const fn new(value: f32, pressed: bool, just_pressed: bool, just_released: bool) -> Self {
        Self {
            value,
            pressed,
            just_pressed,
            just_released,
        }
    }

    /// The magnitude: `0.0` or `1.0` for a button, `[-1, 1]` for an axis.
    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }

    /// Whether the action is active this frame.
    #[must_use]
    pub const fn pressed(self) -> bool {
        self.pressed
    }

    /// Whether it became active this frame.
    #[must_use]
    pub const fn just_pressed(self) -> bool {
        self.just_pressed
    }

    /// Whether it stopped being active this frame.
    #[must_use]
    pub const fn just_released(self) -> bool {
        self.just_released
    }

    /// Whether nothing is happening and the state need not be carried.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        !self.pressed && !self.just_pressed && !self.just_released && self.value == 0.0
    }
}

/// The intent one frame produced: the output of the whole module.
///
/// Only actions that are doing something appear. Asking about any other action
/// answers [`ActionState::NEUTRAL`], so a consumer never has to know which
/// actions were registered to read one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputSnapshot {
    frame: u64,
    dropped_signals: u32,
    states: BTreeMap<Identifier, ActionState>,
}

impl InputSnapshot {
    /// An empty snapshot for frame `frame`.
    #[must_use]
    pub const fn new(frame: u64) -> Self {
        Self {
            frame,
            dropped_signals: 0,
            states: BTreeMap::new(),
        }
    }

    /// Which frame this is, counted by the sampler that produced it.
    #[must_use]
    pub const fn frame(&self) -> u64 {
        self.frame
    }

    /// How many signals were ignored because no such device is attached.
    ///
    /// A count and not a flag, for the same reason the job system counts what
    /// it forgot: the useful question is not whether signals are being dropped
    /// but how many.
    #[must_use]
    pub const fn dropped_signals(&self) -> u32 {
        self.dropped_signals
    }

    /// Record a state, replacing any previous one for that action.
    pub fn set(&mut self, action: Identifier, state: ActionState) {
        self.states.insert(action, state);
    }

    /// What this action is doing, or [`ActionState::NEUTRAL`].
    #[must_use]
    pub fn action(&self, action: &Identifier) -> ActionState {
        self.states
            .get(action)
            .copied()
            .unwrap_or(ActionState::NEUTRAL)
    }

    /// Whether the action is active.
    #[must_use]
    pub fn is_pressed(&self, action: &Identifier) -> bool {
        self.action(action).pressed()
    }

    /// Whether the action became active this frame.
    #[must_use]
    pub fn just_pressed(&self, action: &Identifier) -> bool {
        self.action(action).just_pressed()
    }

    /// Whether the action stopped being active this frame.
    #[must_use]
    pub fn just_released(&self, action: &Identifier) -> bool {
        self.action(action).just_released()
    }

    /// The action's magnitude.
    #[must_use]
    pub fn axis(&self, action: &Identifier) -> f32 {
        self.action(action).value()
    }

    /// Every action carrying a state, in identifier order.
    pub fn intents(&self) -> impl Iterator<Item = (&Identifier, ActionState)> {
        self.states.iter().map(|(id, state)| (id, *state))
    }

    /// How many actions carry a state.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether no action is doing anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

/// What decides which binding owns a source when several could.
///
/// Priority first, then chord length, then the two identifiers - the last two
/// only ever break a tie, and exist so that a tie has one answer.
type Rank<'a> = (i32, usize, &'a Identifier, &'a Identifier);

/// The input system: actions, contexts, bindings, and the device state that
/// turns signals into intent.
///
/// Implements the spec's `IInputSystem`: [`Self::register_action`],
/// [`Self::set_context`], [`Self::bind`] and [`Self::sample`].
#[derive(Debug, Clone, Default)]
pub struct InputSystem {
    actions: BTreeMap<Identifier, ActionKind>,
    bindings: Vec<Binding>,
    contexts: BTreeMap<Identifier, i32>,
    attached: BTreeSet<DeviceId>,
    held: BTreeSet<(DeviceId, ButtonCode)>,
    axes: BTreeMap<(DeviceId, AxisCode), f32>,
    active_last_frame: BTreeSet<Identifier>,
    frame: u64,
}

impl InputSystem {
    /// An input system with nothing registered and no device attached.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare an action.
    ///
    /// # Errors
    ///
    /// Returns an error when the action is already registered with a different
    /// kind. Re-registering the same action with the same kind is accepted, so
    /// that loading a binding file twice is not a failure.
    pub fn register_action(&mut self, definition: ActionDefinition) -> Result<()> {
        match self.actions.get(definition.id()) {
            Some(existing) if *existing != definition.kind() => Err(Error::new(
                Domain::Input,
                "input",
                "an action cannot change kind once registered",
            )
            .with_recovery(Recovery::Manual)
            .with_context("action", definition.id().to_string())
            .with_context("registered", existing.as_str())
            .with_context("requested", definition.kind().as_str())),
            _ => {
                self.actions.insert(definition.id, definition.kind);
                Ok(())
            }
        }
    }

    /// Whether this action is registered.
    #[must_use]
    pub fn knows_action(&self, action: &Identifier) -> bool {
        self.actions.contains_key(action)
    }

    /// How many actions are registered.
    #[must_use]
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Activate a context at a priority, or change the priority of an active one.
    ///
    /// Higher wins. A menu pushed at 100 over gameplay at 0 takes every source
    /// it binds and leaves the rest to the layer below.
    pub fn set_context(&mut self, context: Identifier, priority: i32) {
        self.contexts.insert(context, priority);
    }

    /// Deactivate a context. Returns whether it was active.
    pub fn clear_context(&mut self, context: &Identifier) -> bool {
        self.contexts.remove(context).is_some()
    }

    /// The priority of an active context.
    #[must_use]
    pub fn context_priority(&self, context: &Identifier) -> Option<i32> {
        self.contexts.get(context).copied()
    }

    /// How many contexts are active.
    #[must_use]
    pub fn active_contexts(&self) -> usize {
        self.contexts.len()
    }

    /// Add a mapping.
    ///
    /// # Errors
    ///
    /// Returns an error when the action is not registered, or when the same
    /// context already maps the same source with the same chord — the binding
    /// conflict `INPUT SYSTEM.md` requires to be detected rather than resolved
    /// by whichever binding happened to be added last.
    pub fn bind(&mut self, binding: Binding) -> Result<()> {
        if !self.actions.contains_key(binding.action()) {
            return Err(Error::new(
                Domain::Input,
                "input",
                "cannot bind an action that is not registered",
            )
            .with_recovery(Recovery::Reject)
            .with_context("action", binding.action().to_string()));
        }
        if let Some(existing) = self
            .bindings
            .iter()
            .find(|candidate| candidate.slot() == binding.slot())
        {
            return Err(Error::new(Domain::Input, "input", "binding conflict")
                .with_recovery(Recovery::Reject)
                .with_context("context", binding.context().to_string())
                .with_context("source", binding.source().encode())
                .with_context("bound", existing.action().to_string())
                .with_context("requested", binding.action().to_string()));
        }
        self.bindings.push(binding);
        Ok(())
    }

    /// Remove a mapping, naming it the way [`Binding::slot`] identifies one.
    ///
    /// Returns whether anything was removed. This is the other half of a remap:
    /// unbind the old source, bind the new one.
    pub fn unbind(&mut self, context: &Identifier, source: Source, chord: &[Source]) -> bool {
        let before = self.bindings.len();
        self.bindings
            .retain(|binding| binding.slot() != (context, source, chord));
        self.bindings.len() != before
    }

    /// Remove every mapping in a context. Returns how many went.
    pub fn unbind_context(&mut self, context: &Identifier) -> usize {
        let before = self.bindings.len();
        self.bindings.retain(|binding| binding.context() != context);
        before - self.bindings.len()
    }

    /// How many mappings exist.
    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// The mappings, in the order they were added.
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// Whether a device is attached.
    #[must_use]
    pub fn is_attached(&self, device: DeviceId) -> bool {
        self.attached.contains(&device)
    }

    /// How many devices are attached.
    #[must_use]
    pub fn attached_devices(&self) -> usize {
        self.attached.len()
    }

    /// How many frames have been sampled.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.frame
    }

    /// Fold one frame of signals into intent.
    ///
    /// This is the only method that advances state. Everything it needs arrives
    /// in `frame`; nothing is read from the host.
    pub fn sample(&mut self, frame: &InputFrame) -> InputSnapshot {
        self.frame = self.frame.saturating_add(1);
        let mut dropped = 0_u32;

        for signal in frame.signals() {
            match *signal {
                Signal::Attached(device) => {
                    self.attached.insert(device);
                }
                Signal::Detached(device) => self.detach(device),
                Signal::Button {
                    device,
                    code,
                    pressed,
                } => {
                    if self.attached.contains(&device) {
                        if pressed {
                            self.held.insert((device, code));
                        } else {
                            self.held.remove(&(device, code));
                        }
                    } else {
                        dropped = dropped.saturating_add(1);
                    }
                }
                Signal::Axis {
                    device,
                    code,
                    value,
                } => {
                    if self.attached.contains(&device) && value.is_finite() {
                        self.axes.insert((device, code), value);
                    } else {
                        dropped = dropped.saturating_add(1);
                    }
                }
            }
        }

        let mut snapshot = self.resolve();
        snapshot.dropped_signals = dropped;
        snapshot
    }

    /// Release everything a device was holding and forget its axes.
    fn detach(&mut self, device: DeviceId) {
        self.attached.remove(&device);
        self.held.retain(|(held_on, _)| *held_on != device);
        self.axes.retain(|(read_on, _), _| *read_on != device);
    }

    /// Whether any attached device of this kind holds this button.
    fn button_held(&self, kind: DeviceKind, code: ButtonCode) -> bool {
        self.held
            .iter()
            .any(|(device, held)| device.kind() == kind && *held == code)
    }

    /// The largest-magnitude reading of this control across attached devices of
    /// this kind. Two sticks on two pads do not add up; the one pushed further
    /// wins.
    fn axis_reading(&self, kind: DeviceKind, code: AxisCode) -> f32 {
        let mut best = 0.0_f32;
        for ((device, axis), value) in &self.axes {
            if device.kind() == kind && *axis == code && value.abs() > best.abs() {
                best = *value;
            }
        }
        best
    }

    /// What one binding contributes this frame, or `None` if it does not fire.
    fn firing(&self, binding: &Binding) -> Option<f32> {
        for member in binding.chord() {
            match member {
                Source::Button { device, code } => {
                    if !self.button_held(*device, *code) {
                        return None;
                    }
                }
                // Rejected by `with_chord`; unreachable through the public API.
                Source::Axis { .. } => return None,
            }
        }
        match binding.source() {
            Source::Button { device, code } => {
                if self.button_held(device, code) {
                    Some(if binding.tuning().invert() { -1.0 } else { 1.0 })
                } else {
                    None
                }
            }
            Source::Axis { device, code } => {
                let shaped = binding.tuning().apply(self.axis_reading(device, code));
                if shaped == 0.0 {
                    None
                } else {
                    Some(shaped)
                }
            }
        }
    }

    /// Turn the current device state into intent.
    fn resolve(&mut self) -> InputSnapshot {
        // One winner per source: the highest-priority context that binds it,
        // and within a context the longest chord. Everything else is consumed.
        //
        // The last two fields of the rank are what make this deterministic
        // rather than merely defined. Two contexts at the same priority binding
        // the same source would otherwise be settled by whichever was added
        // first, and two peers that built the same bindings in a different
        // order would disagree about what the player just did.
        let mut winners: BTreeMap<Source, (Rank<'_>, &Binding, f32)> = BTreeMap::new();
        for binding in &self.bindings {
            let Some(priority) = self.contexts.get(binding.context()).copied() else {
                continue;
            };
            let Some(value) = self.firing(binding) else {
                continue;
            };
            let rank: Rank<'_> = (
                priority,
                binding.chord().len(),
                binding.context(),
                binding.action(),
            );
            match winners.get(&binding.source()) {
                Some((best, _, _)) if *best >= rank => {}
                _ => {
                    winners.insert(binding.source(), (rank, binding, value));
                }
            }
        }

        let mut values: BTreeMap<&Identifier, f32> = BTreeMap::new();
        for (_, binding, value) in winners.values() {
            let kind = self
                .actions
                .get(binding.action())
                .copied()
                .unwrap_or(ActionKind::Button);
            let slot = values.entry(binding.action()).or_insert(0.0);
            match kind {
                // Two keys bound to one action are an OR, not a doubling.
                ActionKind::Button => *slot = 1.0,
                // W and S on one axis cancel; a stick and a key add and clamp.
                ActionKind::Axis => *slot = (*slot + value).clamp(-1.0, 1.0),
            }
        }

        let mut snapshot = InputSnapshot::new(self.frame);
        let mut active_now: BTreeSet<Identifier> = BTreeSet::new();
        for (action, value) in values {
            let pressed = value != 0.0;
            if pressed {
                active_now.insert(action.clone());
            }
            let state = ActionState::new(
                value,
                pressed,
                pressed && !self.active_last_frame.contains(action),
                false,
            );
            if !state.is_neutral() {
                snapshot.set(action.clone(), state);
            }
        }

        // Anything that was active and is not now released this frame — including
        // everything a detached device was holding.
        for action in &self.active_last_frame {
            if !active_now.contains(action) {
                snapshot.set(action.clone(), ActionState::new(0.0, false, false, true));
            }
        }

        self.active_last_frame = active_now;
        snapshot
    }

    /// Check a snapshot that arrived from outside the trust boundary.
    ///
    /// `INPUT SYSTEM.md` §Security: *"Client input is untrusted. Server never
    /// accepts client claims of final state; only validated intent/snapshots
    /// are transported."* A peer sends intent, and intent is still a claim.
    /// What can be checked without the peer's device state is checked here:
    ///
    /// * every action exists — a client may not invent one;
    /// * every value is finite and within the range its kind allows, so a
    ///   client cannot ask to move at fifty times the speed of a player who is
    ///   pushing the stick just as hard;
    /// * a button action carries `0.0` or `1.0` and nothing between;
    /// * the edges agree with the level: a press that is not held, or an action
    ///   claiming to have been pressed and released in the same frame, is a
    ///   state no sampler produces.
    ///
    /// What is **not** checked, and must not be mistaken for checked: whether
    /// the peer's contexts actually bind that action, and whether the frame
    /// number is one the server has not already accepted. Both need per-peer
    /// state that lives in the session layer, which does not exist yet.
    ///
    /// # Errors
    ///
    /// Returns the first failure found, naming the action and the claim.
    pub fn validate_remote(&self, snapshot: &InputSnapshot) -> Result<()> {
        for (action, state) in snapshot.intents() {
            let Some(kind) = self.actions.get(action).copied() else {
                return Err(untrusted("unknown action", action));
            };
            let value = state.value();
            if !value.is_finite() {
                return Err(untrusted("action value is not finite", action));
            }
            match kind {
                ActionKind::Button => {
                    if value != 0.0 && value != 1.0 {
                        return Err(untrusted("a button action is 0 or 1", action)
                            .with_context("value", value.to_string()));
                    }
                }
                ActionKind::Axis => {
                    if !(-1.0..=1.0).contains(&value) {
                        return Err(untrusted("an axis action is within [-1, 1]", action)
                            .with_context("value", value.to_string()));
                    }
                }
            }
            if state.pressed() != (value != 0.0) {
                return Err(untrusted("pressed disagrees with the value", action));
            }
            if state.just_pressed() && !state.pressed() {
                return Err(untrusted("pressed this frame but not held", action));
            }
            if state.just_released() && state.pressed() {
                return Err(untrusted("released this frame but still held", action));
            }
            if state.just_pressed() && state.just_released() {
                return Err(untrusted("pressed and released in one frame", action));
            }
        }
        Ok(())
    }

    /// Write the bindings as canonical text, for a remap the player keeps.
    ///
    /// Sorted rather than in insertion order, so the same set of bindings
    /// always produces byte-identical text — two players who chose the same
    /// remap get the same file, and a diff shows what changed rather than what
    /// moved. Identifiers are written in their `namespace:path` form for the
    /// same reason the registry persists those and never runtime integers.
    #[must_use]
    pub fn encode_bindings(&self) -> String {
        let mut lines: Vec<String> = self.bindings.iter().map(encode_binding).collect();
        lines.sort_unstable();
        lines.dedup();
        let mut out = String::new();
        for line in lines {
            out.push_str(&line);
            out.push('\n');
        }
        out
    }

    /// Read bindings written by [`Self::encode_bindings`] and add them.
    ///
    /// Blank lines and lines beginning with `#` are ignored, so the file stays
    /// something a person can edit. Returns how many bindings were added.
    ///
    /// # Errors
    ///
    /// Returns an error on the first malformed line, or on the first binding
    /// that [`Self::bind`] refuses — an unregistered action or a conflict. A
    /// partially applied file is left applied: the caller learns which line
    /// failed and the bindings before it are real, which is more useful than a
    /// rollback that hides where the file went wrong.
    pub fn load_bindings(&mut self, text: &str) -> Result<usize> {
        let mut added = 0;
        for (number, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let binding = decode_binding(trimmed)
                .map_err(|error| error.with_context("line", (number + 1).to_string()))?;
            self.bind(binding)
                .map_err(|error| error.with_context("line", (number + 1).to_string()))?;
            added += 1;
        }
        Ok(added)
    }
}

/// One binding as a line of the remap file.
fn encode_binding(binding: &Binding) -> String {
    let mut line = format!(
        "bind {} {} {}",
        binding.context(),
        binding.action(),
        binding.source().encode()
    );
    for member in binding.chord() {
        line.push('+');
        line.push_str(&member.encode());
    }
    let tuning = binding.tuning();
    if !tuning.is_default() {
        line.push_str(&format!(
            " dead={} sens={} curve={}",
            tuning.dead_zone(),
            tuning.sensitivity(),
            tuning.curve().as_str()
        ));
        if tuning.invert() {
            line.push_str(" invert");
        }
    }
    line
}

/// Parse one line of the remap file.
fn decode_binding(line: &str) -> Result<Binding> {
    let mut fields = line.split_whitespace();
    if fields.next() != Some("bind") {
        return Err(
            reject("a binding line begins with `bind`").with_context("line", line.to_owned())
        );
    }
    let (Some(context), Some(action), Some(sources)) =
        (fields.next(), fields.next(), fields.next())
    else {
        return Err(
            reject("a binding line is `bind <context> <action> <source>`")
                .with_context("line", line.to_owned()),
        );
    };
    let context = Identifier::parse(context)?;
    let action = Identifier::parse(action)?;

    let mut parts = sources.split('+');
    let Some(primary) = parts.next() else {
        return Err(reject("a binding needs a source").with_context("line", line.to_owned()));
    };
    let source = Source::decode(primary)?;
    let chord = parts.map(Source::decode).collect::<Result<Vec<_>>>()?;

    let mut dead_zone = 0.0_f32;
    let mut sensitivity = 1.0_f32;
    let mut curve = ResponseCurve::Linear;
    let mut invert = false;
    for field in fields {
        if field == "invert" {
            invert = true;
            continue;
        }
        let Some((key, value)) = field.split_once('=') else {
            return Err(reject("a tuning field is `key=value` or `invert`")
                .with_context("field", field.to_owned()));
        };
        match key {
            "dead" => dead_zone = parse_float(value, "dead")?,
            "sens" => sensitivity = parse_float(value, "sens")?,
            "curve" => {
                curve = ResponseCurve::parse(value).ok_or_else(|| {
                    reject("unknown response curve").with_context("curve", value.to_owned())
                })?;
            }
            other => {
                return Err(reject("unknown tuning field").with_context("field", other.to_owned()))
            }
        }
    }

    let binding = Binding::new(context, action, source)
        .with_chord(&chord)?
        .with_tuning(AxisTuning::new(dead_zone, sensitivity, curve, invert)?);
    Ok(binding)
}

/// Parse a tuning number, saying which field failed.
fn parse_float(text: &str, field: &'static str) -> Result<f32> {
    text.parse::<f32>()
        .map_err(|_| reject("a tuning value is a number").with_context(field, text.to_owned()))
}

/// An input the system refuses.
fn reject(message: &'static str) -> Error {
    Error::new(Domain::Input, "input", message).with_recovery(Recovery::Reject)
}

/// A claim from outside the trust boundary that the system refuses.
fn untrusted(message: &'static str, action: &Identifier) -> Error {
    Error::new(Domain::Security, "input", message)
        .with_recovery(Recovery::Reject)
        .with_context("action", action.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);
    const PAD: DeviceId = DeviceId::new(DeviceKind::Gamepad, 0);
    const SECOND_PAD: DeviceId = DeviceId::new(DeviceKind::Gamepad, 1);

    const W: ButtonCode = ButtonCode(17);
    const S: ButtonCode = ButtonCode(31);
    const CTRL: ButtonCode = ButtonCode(29);
    const STICK_X: AxisCode = AxisCode(0);

    fn id(path: &str) -> Identifier {
        Identifier::nexora(path).expect("a first-party identifier")
    }

    fn gameplay() -> Identifier {
        id("context/gameplay")
    }

    fn menu() -> Identifier {
        id("context/menu")
    }

    /// A system with the two devices attached and one context active.
    fn ready() -> InputSystem {
        let mut input = InputSystem::new();
        input.set_context(gameplay(), 0);
        let _ = input.sample(
            &InputFrame::new()
                .with(Signal::Attached(KEYBOARD))
                .with(Signal::Attached(PAD)),
        );
        input
    }

    fn button_action(input: &mut InputSystem, path: &str) -> Identifier {
        let action = id(path);
        input
            .register_action(ActionDefinition::new(action.clone(), ActionKind::Button))
            .expect("a fresh action");
        action
    }

    fn axis_action(input: &mut InputSystem, path: &str) -> Identifier {
        let action = id(path);
        input
            .register_action(ActionDefinition::new(action.clone(), ActionKind::Axis))
            .expect("a fresh action");
        action
    }

    fn press(code: ButtonCode) -> InputFrame {
        InputFrame::new().with(Signal::button(KEYBOARD, code, true))
    }

    fn release(code: ButtonCode) -> InputFrame {
        InputFrame::new().with(Signal::button(KEYBOARD, code, false))
    }

    fn close(left: f32, right: f32) -> bool {
        (left - right).abs() < 1e-6
    }

    // --- the abstraction itself -------------------------------------------

    #[test]
    fn a_source_survives_the_round_trip_through_text() {
        for kind in DeviceKind::ALL {
            for source in [
                Source::button(kind, ButtonCode(0)),
                Source::button(kind, ButtonCode(u16::MAX)),
                Source::axis(kind, AxisCode(7)),
            ] {
                let text = source.encode();
                assert_eq!(
                    Source::decode(&text).expect("what encode wrote"),
                    source,
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn a_malformed_source_is_refused_rather_than_guessed_at() {
        for bad in [
            "keyboard/button",
            "keyboard/button/17/extra",
            "trackball/button/1",
            "keyboard/dial/1",
            "keyboard/button/70000",
            "",
        ] {
            assert!(Source::decode(bad).is_err(), "{bad} should not parse");
        }
    }

    #[test]
    fn an_action_may_be_declared_twice_but_not_redefined() {
        let mut input = InputSystem::new();
        let jump = id("action/jump");
        let definition = ActionDefinition::new(jump.clone(), ActionKind::Button);
        assert!(input.register_action(definition.clone()).is_ok());
        assert!(
            input.register_action(definition).is_ok(),
            "loading the same declaration twice is not a failure"
        );
        assert!(
            input
                .register_action(ActionDefinition::new(jump, ActionKind::Axis))
                .is_err(),
            "an action that changes kind invalidates every binding to it"
        );
        assert_eq!(input.action_count(), 1);
    }

    #[test]
    fn a_binding_to_an_unregistered_action_is_refused() {
        let mut input = ready();
        let error = input
            .bind(Binding::new(
                gameplay(),
                id("action/typo"),
                Source::button(DeviceKind::Keyboard, W),
            ))
            .expect_err("the action does not exist");
        assert_eq!(error.domain(), Domain::Input);
    }

    // --- the six the spec asks for ----------------------------------------

    /// `INPUT SYSTEM.md` §Tests: binding conflict.
    #[test]
    fn two_bindings_on_one_source_in_one_context_are_a_conflict() {
        let mut input = ready();
        let jump = button_action(&mut input, "action/jump");
        let attack = button_action(&mut input, "action/attack");
        let space = Source::button(DeviceKind::Keyboard, W);

        input
            .bind(Binding::new(gameplay(), jump, space))
            .expect("the first binding");
        let error = input
            .bind(Binding::new(gameplay(), attack.clone(), space))
            .expect_err("the second binding takes the same source");
        assert_eq!(error.domain(), Domain::Input);
        assert_eq!(input.binding_count(), 1, "the conflict was not stored");

        // The same source in a *different* context is not a conflict: that is
        // what contexts are for.
        input.set_context(menu(), 10);
        input
            .bind(Binding::new(menu(), attack, space))
            .expect("a different context may claim the same key");
        assert_eq!(input.binding_count(), 2);
    }

    #[test]
    fn a_chord_is_a_different_binding_from_the_key_alone() {
        let mut input = ready();
        let save = button_action(&mut input, "action/save");
        let step = button_action(&mut input, "action/step");
        let key = Source::button(DeviceKind::Keyboard, S);
        let ctrl = Source::button(DeviceKind::Keyboard, CTRL);

        input
            .bind(Binding::new(gameplay(), step, key))
            .expect("the plain key");
        input
            .bind(
                Binding::new(gameplay(), save, key)
                    .with_chord(&[ctrl])
                    .expect("a button chord"),
            )
            .expect("the same key with a modifier is a different mapping");
        assert_eq!(input.binding_count(), 2);
    }

    #[test]
    fn a_chord_made_of_an_axis_is_refused() {
        let stick = Source::axis(DeviceKind::Gamepad, STICK_X);
        assert!(Binding::new(gameplay(), id("action/save"), stick)
            .with_chord(&[stick])
            .is_err());
    }

    #[test]
    fn a_chord_is_the_same_mapping_whichever_order_it_was_written_in() {
        let mut input = ready();
        let save = button_action(&mut input, "action/save");
        let other = button_action(&mut input, "action/other");
        let key = Source::button(DeviceKind::Keyboard, S);
        let ctrl = Source::button(DeviceKind::Keyboard, CTRL);
        let shift = Source::button(DeviceKind::Keyboard, ButtonCode(42));

        input
            .bind(
                Binding::new(gameplay(), save, key)
                    .with_chord(&[ctrl, shift])
                    .expect("a chord"),
            )
            .expect("the first");
        assert!(
            input
                .bind(
                    Binding::new(gameplay(), other, key)
                        .with_chord(&[shift, ctrl])
                        .expect("the same chord, written backwards")
                )
                .is_err(),
            "Ctrl+Shift and Shift+Ctrl are one mapping"
        );
    }

    /// `INPUT SYSTEM.md` §Tests: context priority.
    ///
    /// The menu is bound **first** and named so that it loses every tiebreak
    /// the rank falls back on - `context/menu` sorts before `context/gameplay`,
    /// and it was added first. Priority is then the only thing that can make it
    /// win, so reverting the priority comparison in `resolve` fails this test
    /// rather than passing it by accident.
    #[test]
    fn the_higher_context_takes_the_key_and_the_lower_one_does_not_see_it() {
        let mut input = ready();
        let walk = button_action(&mut input, "action/walk");
        let scroll = button_action(&mut input, "action/scroll");
        let key = Source::button(DeviceKind::Keyboard, W);

        input.set_context(menu(), 100);
        input
            .bind(Binding::new(menu(), scroll.clone(), key))
            .expect("the menu");
        input
            .bind(Binding::new(gameplay(), walk.clone(), key))
            .expect("gameplay");

        let snapshot = input.sample(&press(W));
        assert!(snapshot.is_pressed(&scroll), "the menu is on top");
        assert!(
            !snapshot.is_pressed(&walk),
            "the player must not also walk while the menu has the key"
        );

        // Close the menu: the same key reaches the player again.
        assert!(input.clear_context(&menu()));
        let snapshot = input.sample(&InputFrame::new());
        assert!(snapshot.is_pressed(&walk));
        assert!(snapshot.just_released(&scroll));
    }

    #[test]
    fn the_longest_chord_in_reach_wins() {
        let mut input = ready();
        let step = button_action(&mut input, "action/step");
        let save = button_action(&mut input, "action/save");
        let key = Source::button(DeviceKind::Keyboard, S);
        let ctrl = Source::button(DeviceKind::Keyboard, CTRL);

        input
            .bind(Binding::new(gameplay(), step.clone(), key))
            .expect("the plain key");
        input
            .bind(
                Binding::new(gameplay(), save.clone(), key)
                    .with_chord(&[ctrl])
                    .expect("a chord"),
            )
            .expect("the chord");

        let alone = input.sample(&press(S));
        assert!(alone.is_pressed(&step), "S alone steps");
        assert!(!alone.is_pressed(&save));

        let with_modifier = input.sample(&press(CTRL));
        assert!(
            with_modifier.is_pressed(&save),
            "Ctrl+S saves, or the chord could never fire"
        );
        assert!(
            !with_modifier.is_pressed(&step),
            "and the plain binding is consumed"
        );
        assert!(with_modifier.just_released(&step));
    }

    /// `INPUT SYSTEM.md` §Tests: device disconnect.
    ///
    /// Remove the `held.retain` in `detach` and this fails: the player walks
    /// forward for the rest of the process.
    #[test]
    fn unplugging_a_device_releases_everything_it_was_holding() {
        let mut input = ready();
        let fire = button_action(&mut input, "action/fire");
        input
            .bind(Binding::new(
                gameplay(),
                fire.clone(),
                Source::button(DeviceKind::Gamepad, ButtonCode(0)),
            ))
            .expect("the trigger");

        let held = input.sample(&InputFrame::new().with(Signal::button(PAD, ButtonCode(0), true)));
        assert!(held.is_pressed(&fire));

        let gone = input.sample(&InputFrame::new().with(Signal::Detached(PAD)));
        assert!(!gone.is_pressed(&fire), "the trigger cannot still be held");
        assert!(
            gone.just_released(&fire),
            "and the release is an edge, so a consumer that only watches edges sees it"
        );
        assert_eq!(input.attached_devices(), 1);
    }

    #[test]
    fn signals_from_a_device_that_is_not_attached_are_dropped_and_counted() {
        let mut input = ready();
        let fire = button_action(&mut input, "action/fire");
        input
            .bind(Binding::new(
                gameplay(),
                fire.clone(),
                Source::button(DeviceKind::Gamepad, ButtonCode(0)),
            ))
            .expect("the trigger");

        let snapshot = input.sample(
            &InputFrame::new()
                .with(Signal::Detached(PAD))
                .with(Signal::button(PAD, ButtonCode(0), true))
                .with(Signal::axis(PAD, STICK_X, 1.0)),
        );
        assert!(
            !snapshot.is_pressed(&fire),
            "a button-down after the unplug is the driver catching up, not the player"
        );
        assert_eq!(snapshot.dropped_signals(), 2);
    }

    #[test]
    fn an_axis_reading_that_is_not_a_number_never_reaches_intent() {
        let mut input = ready();
        let look = axis_action(&mut input, "action/look");
        input
            .bind(Binding::new(
                gameplay(),
                look.clone(),
                Source::axis(DeviceKind::Gamepad, STICK_X),
            ))
            .expect("the stick");

        let snapshot = input.sample(&InputFrame::new().with(Signal::axis(PAD, STICK_X, f32::NAN)));
        assert_eq!(snapshot.axis(&look), 0.0);
        assert_eq!(snapshot.dropped_signals(), 1);
    }

    /// `INPUT SYSTEM.md` §Tests: remap persistence.
    #[test]
    fn a_remap_survives_being_written_out_and_read_back() {
        let mut input = ready();
        let walk = axis_action(&mut input, "action/walk");
        let save = button_action(&mut input, "action/save");
        let tuning = AxisTuning::new(0.15, 1.5, ResponseCurve::Quadratic, true).expect("a tuning");

        input
            .bind(
                Binding::new(
                    gameplay(),
                    walk.clone(),
                    Source::axis(DeviceKind::Gamepad, STICK_X),
                )
                .with_tuning(tuning),
            )
            .expect("the stick");
        input
            .bind(Binding::new(
                gameplay(),
                walk.clone(),
                Source::button(DeviceKind::Keyboard, W),
            ))
            .expect("the key");
        input
            .bind(
                Binding::new(gameplay(), save, Source::button(DeviceKind::Keyboard, S))
                    .with_chord(&[Source::button(DeviceKind::Keyboard, CTRL)])
                    .expect("a chord"),
            )
            .expect("the chord");

        let written = input.encode_bindings();
        let mut restored = InputSystem::new();
        restored
            .register_action(ActionDefinition::new(walk.clone(), ActionKind::Axis))
            .expect("the action is declared by the game, not by the file");
        restored
            .register_action(ActionDefinition::new(id("action/save"), ActionKind::Button))
            .expect("declared");
        assert_eq!(restored.load_bindings(&written).expect("a valid file"), 3);
        assert_eq!(
            restored.encode_bindings(),
            written,
            "the text is canonical: what comes back out is what went in"
        );

        // And the restored tuning is the same number, not a rounded one.
        let stick = restored
            .bindings()
            .iter()
            .find(|binding| !binding.source().is_button())
            .expect("the stick binding");
        assert_eq!(stick.tuning(), tuning);
    }

    #[test]
    fn the_binding_file_does_not_depend_on_the_order_the_bindings_were_added() {
        let sources = [
            Source::button(DeviceKind::Keyboard, W),
            Source::button(DeviceKind::Keyboard, S),
            Source::axis(DeviceKind::Gamepad, STICK_X),
        ];
        let mut forwards = ready();
        let walk = axis_action(&mut forwards, "action/walk");
        let mut backwards = ready();
        let _ = axis_action(&mut backwards, "action/walk");

        for source in sources {
            forwards
                .bind(Binding::new(gameplay(), walk.clone(), source))
                .expect("forwards");
        }
        for source in sources.into_iter().rev() {
            backwards
                .bind(Binding::new(gameplay(), walk.clone(), source))
                .expect("backwards");
        }
        assert_eq!(forwards.encode_bindings(), backwards.encode_bindings());
    }

    #[test]
    fn a_binding_file_says_which_line_it_failed_on() {
        let mut input = ready();
        let _ = button_action(&mut input, "action/save");
        let error = input
            .load_bindings(
                "# a remap\n\nbind nexora:context/gameplay nexora:action/save keyboard/button/31\nbind nexora:context/gameplay nexora:action/save keyboard/nonsense/31\n",
            )
            .expect_err("the fourth line is malformed");
        assert!(format!("{error}").contains("line=4"), "{error}");
        assert_eq!(
            input.binding_count(),
            1,
            "the lines before the bad one were applied, and say so"
        );
    }

    #[test]
    fn comments_and_blank_lines_are_not_bindings() {
        let mut input = ready();
        let _ = button_action(&mut input, "action/save");
        assert_eq!(
            input
                .load_bindings("\n  # nothing here\n\n\t\n")
                .expect("a file of nothing"),
            0
        );
    }

    /// `INPUT SYSTEM.md` §Tests: deterministic input replay.
    ///
    /// The property the whole module is arranged around: the same signals in,
    /// the same intent out — no clock, no device, nothing else consulted.
    #[test]
    fn the_same_signals_replay_to_the_same_intent() {
        let script = || {
            vec![
                InputFrame::new()
                    .with(Signal::Attached(KEYBOARD))
                    .with(Signal::Attached(PAD)),
                press(W),
                InputFrame::new().with(Signal::axis(PAD, STICK_X, 0.8)),
                press(CTRL),
                release(W),
                InputFrame::new().with(Signal::axis(PAD, STICK_X, -0.4)),
                InputFrame::new().with(Signal::Detached(PAD)),
                release(CTRL),
                InputFrame::new(),
            ]
        };

        let build = || {
            let mut input = InputSystem::new();
            input.set_context(gameplay(), 0);
            let walk = axis_action(&mut input, "action/walk");
            let crouch = button_action(&mut input, "action/crouch");
            input
                .bind(Binding::new(
                    gameplay(),
                    walk.clone(),
                    Source::button(DeviceKind::Keyboard, W),
                ))
                .expect("the key");
            input
                .bind(Binding::new(
                    gameplay(),
                    walk,
                    Source::axis(DeviceKind::Gamepad, STICK_X),
                ))
                .expect("the stick");
            input
                .bind(Binding::new(
                    gameplay(),
                    crouch,
                    Source::button(DeviceKind::Keyboard, CTRL),
                ))
                .expect("the modifier");
            input
        };

        let run = |mut input: InputSystem| {
            script()
                .iter()
                .map(|frame| input.sample(frame))
                .collect::<Vec<_>>()
        };

        let first = run(build());
        let second = run(build());
        assert_eq!(first, second, "a replay is not a re-enactment");
        assert!(
            first.iter().any(|snapshot| !snapshot.is_empty()),
            "a script that produces nothing would pass this test without proving anything"
        );
        assert_eq!(first.last().map(InputSnapshot::frame), Some(9));
    }

    /// `INPUT SYSTEM.md` §Security: client input is untrusted.
    #[test]
    fn a_peer_cannot_claim_an_action_the_server_does_not_know() {
        let mut input = ready();
        let _ = button_action(&mut input, "action/fire");
        let mut forged = InputSnapshot::new(1);
        forged.set(id("action/win"), ActionState::new(1.0, true, true, false));
        let error = input.validate_remote(&forged).expect_err("no such action");
        assert_eq!(error.domain(), Domain::Security);
    }

    #[test]
    fn a_peer_cannot_claim_more_of_an_axis_than_an_axis_has() {
        let mut input = ready();
        let walk = axis_action(&mut input, "action/walk");
        for value in [50.0_f32, -1.5, f32::INFINITY, f32::NAN] {
            let mut forged = InputSnapshot::new(1);
            forged.set(walk.clone(), ActionState::new(value, true, false, false));
            assert!(
                input.validate_remote(&forged).is_err(),
                "a client claiming {value} is claiming a stick that does not exist"
            );
        }
        let mut honest = InputSnapshot::new(1);
        honest.set(walk, ActionState::new(-1.0, true, false, false));
        assert!(input.validate_remote(&honest).is_ok());
    }

    #[test]
    fn a_peer_cannot_send_a_state_no_sampler_produces() {
        let mut input = ready();
        let fire = button_action(&mut input, "action/fire");
        let impossible = [
            ActionState::new(0.5, true, false, false),  // half a button
            ActionState::new(1.0, false, false, false), // held without being pressed
            ActionState::new(0.0, false, true, false),  // pressed without being held
            ActionState::new(1.0, true, false, true),   // released while still held
            ActionState::new(1.0, true, true, true),    // both edges in one frame
        ];
        for state in impossible {
            let mut forged = InputSnapshot::new(1);
            forged.set(fire.clone(), state);
            assert!(
                input.validate_remote(&forged).is_err(),
                "{state:?} should not survive validation"
            );
        }
    }

    #[test]
    fn what_the_sampler_produces_passes_its_own_validation() {
        let mut input = ready();
        let walk = axis_action(&mut input, "action/walk");
        let fire = button_action(&mut input, "action/fire");
        input
            .bind(Binding::new(
                gameplay(),
                walk,
                Source::axis(DeviceKind::Gamepad, STICK_X),
            ))
            .expect("the stick");
        input
            .bind(Binding::new(
                gameplay(),
                fire,
                Source::button(DeviceKind::Keyboard, W),
            ))
            .expect("the key");

        for frame in [
            InputFrame::new().with(Signal::axis(PAD, STICK_X, 0.6)),
            press(W),
            release(W),
            InputFrame::new().with(Signal::axis(PAD, STICK_X, 0.0)),
        ] {
            let snapshot = input.sample(&frame);
            input
                .validate_remote(&snapshot)
                .expect("the sampler must not produce what the validator rejects");
        }
    }

    // --- the analogue path -------------------------------------------------

    #[test]
    fn a_dead_zone_rescales_so_the_axis_does_not_start_at_the_dead_zone() {
        let tuning = AxisTuning::new(0.25, 1.0, ResponseCurve::Linear, false).expect("a tuning");
        assert_eq!(tuning.apply(0.25), 0.0, "inside the dead zone is nothing");
        assert!(
            close(tuning.apply(0.5), 1.0 / 3.0),
            "just past it is just past zero, not 0.5"
        );
        assert!(
            close(tuning.apply(1.0), 1.0),
            "and full travel is still full"
        );
        assert!(close(tuning.apply(-1.0), -1.0), "in both directions");
    }

    #[test]
    fn the_curve_shapes_the_travel_and_sensitivity_scales_it() {
        let linear = AxisTuning::default();
        let squared = AxisTuning::new(0.0, 1.0, ResponseCurve::Quadratic, false).expect("a tuning");
        assert!(close(linear.apply(0.5), 0.5));
        assert!(close(squared.apply(0.5), 0.25), "finer near the centre");
        assert!(close(squared.apply(1.0), 1.0), "and unchanged at the limit");

        let sensitive = AxisTuning::new(0.0, 2.0, ResponseCurve::Linear, false).expect("a tuning");
        assert!(close(sensitive.apply(0.25), 0.5));
        assert!(
            close(sensitive.apply(0.9), 1.0),
            "sensitivity cannot push an intention past full"
        );

        let inverted = AxisTuning::new(0.0, 1.0, ResponseCurve::Linear, true).expect("a tuning");
        assert!(close(inverted.apply(0.5), -0.5));
    }

    #[test]
    fn a_tuning_that_would_silence_or_reverse_an_axis_is_refused() {
        assert!(AxisTuning::new(1.0, 1.0, ResponseCurve::Linear, false).is_err());
        assert!(AxisTuning::new(-0.1, 1.0, ResponseCurve::Linear, false).is_err());
        assert!(AxisTuning::new(f32::NAN, 1.0, ResponseCurve::Linear, false).is_err());
        assert!(AxisTuning::new(0.0, 0.0, ResponseCurve::Linear, false).is_err());
        assert!(AxisTuning::new(0.0, -1.0, ResponseCurve::Linear, false).is_err());
        assert!(AxisTuning::new(0.0, f32::INFINITY, ResponseCurve::Linear, false).is_err());
    }

    #[test]
    fn two_keys_on_one_axis_cancel_and_two_keys_on_one_button_do_not_double() {
        let mut input = ready();
        let walk = axis_action(&mut input, "action/walk");
        let fire = button_action(&mut input, "action/fire");
        let forward = Source::button(DeviceKind::Keyboard, W);
        let back = Source::button(DeviceKind::Keyboard, S);

        input
            .bind(Binding::new(gameplay(), walk.clone(), forward))
            .expect("forward");
        input
            .bind(Binding::new(gameplay(), walk.clone(), back).with_tuning(
                AxisTuning::new(0.0, 1.0, ResponseCurve::Linear, true).expect("inverted"),
            ))
            .expect("back");
        input
            .bind(Binding::new(
                gameplay(),
                fire.clone(),
                Source::button(DeviceKind::Keyboard, CTRL),
            ))
            .expect("fire");
        input
            .bind(Binding::new(
                gameplay(),
                fire.clone(),
                Source::button(DeviceKind::Gamepad, ButtonCode(0)),
            ))
            .expect("fire, on the pad too");

        let forward_only = input.sample(&press(W));
        assert!(close(forward_only.axis(&walk), 1.0));

        let both = input.sample(&press(S));
        assert_eq!(both.axis(&walk), 0.0, "forward and back cancel");
        assert!(both.just_released(&walk), "and that is a release");

        let two_fire_keys = input.sample(
            &InputFrame::new()
                .with(Signal::button(KEYBOARD, CTRL, true))
                .with(Signal::button(PAD, ButtonCode(0), true)),
        );
        assert!(close(two_fire_keys.axis(&fire), 1.0), "a button is not 2.0");
    }

    #[test]
    fn the_stick_pushed_furthest_is_the_one_that_answers() {
        let mut input = ready();
        let _ = input.sample(&InputFrame::new().with(Signal::Attached(SECOND_PAD)));
        let walk = axis_action(&mut input, "action/walk");
        input
            .bind(Binding::new(
                gameplay(),
                walk.clone(),
                Source::axis(DeviceKind::Gamepad, STICK_X),
            ))
            .expect("the stick");

        let snapshot = input.sample(
            &InputFrame::new()
                .with(Signal::axis(PAD, STICK_X, 0.3))
                .with(Signal::axis(SECOND_PAD, STICK_X, -0.9)),
        );
        assert!(
            close(snapshot.axis(&walk), -0.9),
            "two pads do not add up to 1.2 of a stick"
        );
    }

    #[test]
    fn a_button_action_bound_to_a_stick_fires_once_the_stick_leaves_the_dead_zone() {
        let mut input = ready();
        let fire = button_action(&mut input, "action/fire");
        input
            .bind(
                Binding::new(
                    gameplay(),
                    fire.clone(),
                    Source::axis(DeviceKind::Gamepad, AxisCode(5)),
                )
                .with_tuning(
                    AxisTuning::new(0.5, 1.0, ResponseCurve::Linear, false).expect("a trigger"),
                ),
            )
            .expect("the trigger");

        let light = input.sample(&InputFrame::new().with(Signal::axis(PAD, AxisCode(5), 0.4)));
        assert!(!light.is_pressed(&fire), "a resting trigger is not a shot");
        let pulled = input.sample(&InputFrame::new().with(Signal::axis(PAD, AxisCode(5), 0.9)));
        assert!(pulled.just_pressed(&fire));
        assert_eq!(pulled.axis(&fire), 1.0, "a button action is 0 or 1");
    }

    // --- edges and housekeeping -------------------------------------------

    #[test]
    fn an_edge_happens_once_and_the_hold_continues() {
        let mut input = ready();
        let fire = button_action(&mut input, "action/fire");
        input
            .bind(Binding::new(
                gameplay(),
                fire.clone(),
                Source::button(DeviceKind::Keyboard, W),
            ))
            .expect("the key");

        let down = input.sample(&press(W));
        assert!(down.just_pressed(&fire) && down.is_pressed(&fire));

        let still_down = input.sample(&InputFrame::new());
        assert!(
            !still_down.just_pressed(&fire),
            "a held key is not pressed again every frame"
        );
        assert!(still_down.is_pressed(&fire));

        let up = input.sample(&release(W));
        assert!(up.just_released(&fire) && !up.is_pressed(&fire));

        let quiet = input.sample(&InputFrame::new());
        assert!(quiet.is_empty(), "nothing is happening, so nothing is said");
    }

    #[test]
    fn a_remap_is_an_unbind_and_a_bind() {
        let mut input = ready();
        let fire = button_action(&mut input, "action/fire");
        let old = Source::button(DeviceKind::Keyboard, W);
        let new = Source::button(DeviceKind::Keyboard, S);
        input
            .bind(Binding::new(gameplay(), fire.clone(), old))
            .expect("the old key");

        assert!(input.unbind(&gameplay(), old, &[]));
        assert!(!input.unbind(&gameplay(), old, &[]), "and only once");
        input
            .bind(Binding::new(gameplay(), fire.clone(), new))
            .expect("the new key");

        assert!(!input.sample(&press(W)).is_pressed(&fire));
        assert!(input.sample(&press(S)).is_pressed(&fire));
        assert_eq!(input.unbind_context(&gameplay()), 1);
    }

    #[test]
    fn a_binding_in_an_inactive_context_does_nothing_at_all() {
        let mut input = ready();
        let scroll = button_action(&mut input, "action/scroll");
        input.set_context(menu(), 50);
        input
            .bind(Binding::new(
                menu(),
                scroll.clone(),
                Source::button(DeviceKind::Keyboard, W),
            ))
            .expect("the menu key");
        assert!(input.clear_context(&menu()));
        assert!(!input.clear_context(&menu()), "and it was only active once");

        assert!(
            !input.sample(&press(W)).is_pressed(&scroll),
            "a closed menu does not scroll"
        );
        assert_eq!(input.binding_count(), 1, "but its bindings are still there");
        assert_eq!(input.active_contexts(), 1);
    }

    #[test]
    fn a_source_bound_in_two_contexts_at_the_same_priority_has_one_answer() {
        // Not a recommendation - a tie is a configuration mistake. What matters
        // is that it resolves the same way every time and on every peer.
        let build = |reversed: bool| {
            let mut input = ready();
            let first = button_action(&mut input, "action/first");
            let second = button_action(&mut input, "action/second");
            let key = Source::button(DeviceKind::Keyboard, W);
            let alpha = id("context/alpha");
            let beta = id("context/beta");
            input.set_context(alpha.clone(), 7);
            input.set_context(beta.clone(), 7);
            let bindings = if reversed {
                [
                    Binding::new(beta, second.clone(), key),
                    Binding::new(alpha, first.clone(), key),
                ]
            } else {
                [
                    Binding::new(alpha, first.clone(), key),
                    Binding::new(beta, second.clone(), key),
                ]
            };
            for binding in bindings {
                input.bind(binding).expect("different contexts");
            }
            let snapshot = input.sample(&press(W));
            (snapshot.is_pressed(&first), snapshot.is_pressed(&second))
        };
        assert_eq!(build(false), build(true));
        assert_eq!(build(false), (false, true), "beta sorts after alpha");
    }

    #[test]
    fn the_frame_counter_counts_frames_and_not_signals() {
        let mut input = InputSystem::new();
        assert_eq!(input.frames(), 0);
        let _ = input.sample(&InputFrame::new().with(Signal::Attached(KEYBOARD)));
        let _ = input.sample(&InputFrame::new());
        assert_eq!(input.frames(), 2);
        assert_eq!(input.sample(&InputFrame::new()).frame(), 3);
    }

    #[test]
    fn an_empty_frame_is_still_a_frame() {
        let frame = InputFrame::new();
        assert!(frame.is_empty());
        assert_eq!(frame.len(), 0);
        let frame = frame.with(Signal::Attached(KEYBOARD));
        assert_eq!(frame.len(), 1);
        assert_eq!(
            frame.signals().first().copied().map(Signal::device),
            Some(KEYBOARD)
        );
    }
}
