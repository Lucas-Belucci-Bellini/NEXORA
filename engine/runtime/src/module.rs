//! Engine module system.
//!
//! Implements `ENGINE MODULE SYSTEM.md`. Engine modules are *trusted* internal
//! units (renderer, physics, networking); external content goes through the Mod
//! Runtime, which is deliberately outside this boundary.
//!
//! The module graph enforces four of that document's rules mechanically rather
//! than by review: no circular dependencies, no hidden initialization order,
//! deterministic startup and shutdown sequencing, and shutdown in reverse
//! dependency order.

use std::collections::{BTreeMap, BTreeSet};

use nexora_foundation::diagnostics::{Category, Diagnostics, Level, Record};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

use crate::lifecycle::RuntimeMode;

/// A module's stable identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(Identifier);

impl ModuleId {
    /// Wrap an identifier as a module id.
    #[must_use]
    pub const fn new(id: Identifier) -> Self {
        Self(id)
    }

    /// Parse the canonical `namespace:path` form.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is malformed.
    pub fn parse(raw: &str) -> Result<Self> {
        Ok(Self(Identifier::parse(raw)?))
    }

    /// The underlying identifier.
    #[must_use]
    pub const fn identifier(&self) -> &Identifier {
        &self.0
    }
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A declared dependency on another module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDependency {
    id: ModuleId,
    optional: bool,
}

impl ModuleDependency {
    /// A dependency that must be present.
    #[must_use]
    pub const fn required(id: ModuleId) -> Self {
        Self {
            id,
            optional: false,
        }
    }

    /// A dependency that is used when present and skipped otherwise.
    #[must_use]
    pub const fn optional(id: ModuleId) -> Self {
        Self { id, optional: true }
    }

    /// The depended-upon module.
    #[must_use]
    pub const fn id(&self) -> &ModuleId {
        &self.id
    }

    /// Whether absence is tolerated.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.optional
    }
}

/// Which runtime modes a module participates in.
///
/// This is what lets a dedicated server start without a renderer: a client-only
/// module is filtered out of the graph entirely, rather than loaded and then
/// asked to no-op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleSides {
    /// Loaded in every mode.
    Always,
    /// Loaded only in the listed modes.
    Only(Vec<RuntimeMode>),
}

impl ModuleSides {
    /// Whether the module belongs in a given mode.
    #[must_use]
    pub fn includes(&self, mode: RuntimeMode) -> bool {
        match self {
            Self::Always => true,
            Self::Only(modes) => modes.contains(&mode),
        }
    }
}

/// Lifecycle phases of a single module (`ENGINE MODULE SYSTEM.md`, "Phases").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModulePhase {
    /// Registered but not yet checked.
    Discovered,
    /// Declaration checked.
    Validated,
    /// Placed in dependency order.
    Resolved,
    /// Present in the active module set.
    Loaded,
    /// `initialize` returned successfully.
    Initialized,
    /// Participating in the running runtime.
    Running,
    /// `shutdown` is in progress.
    Stopping,
    /// Torn down.
    Unloaded,
}

/// Services handed to a module during initialization and shutdown.
///
/// Modules receive this rather than reaching for globals, which is what keeps
/// "no hidden initialization order" enforceable.
#[derive(Debug)]
pub struct ModuleContext {
    mode: RuntimeMode,
    diagnostics: Diagnostics,
}

impl ModuleContext {
    /// Build a context.
    #[must_use]
    pub const fn new(mode: RuntimeMode, diagnostics: Diagnostics) -> Self {
        Self { mode, diagnostics }
    }

    /// The runtime mode being initialized.
    #[must_use]
    pub const fn mode(&self) -> RuntimeMode {
        self.mode
    }

    /// The diagnostics facade.
    #[must_use]
    pub const fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}

/// A trusted engine module.
pub trait EngineModule: Send {
    /// Stable identity.
    fn id(&self) -> ModuleId;

    /// Declared dependencies. Defaults to none.
    fn dependencies(&self) -> Vec<ModuleDependency> {
        Vec::new()
    }

    /// Modes this module participates in. Defaults to all of them.
    fn sides(&self) -> ModuleSides {
        ModuleSides::Always
    }

    /// Bring the module up.
    ///
    /// # Errors
    ///
    /// Returns an error when the module cannot initialize. The manager will
    /// then roll back every module it has already initialized.
    fn initialize(&mut self, context: &ModuleContext) -> Result<()>;

    /// Tear the module down. Defaults to doing nothing.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup fails. Shutdown continues regardless, so
    /// that one failing module cannot strand the rest.
    fn shutdown(&mut self, context: &ModuleContext) -> Result<()> {
        let _ = context;
        Ok(())
    }
}

/// Owns the module set and drives it through its lifecycle.
pub struct ModuleManager {
    modules: Vec<Box<dyn EngineModule>>,
    phases: BTreeMap<ModuleId, ModulePhase>,
    order: Vec<usize>,
    initialized: Vec<usize>,
}

impl std::fmt::Debug for ModuleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleManager")
            .field("modules", &self.modules.len())
            .field("phases", &self.phases)
            .finish()
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleManager {
    /// Create an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            phases: BTreeMap::new(),
            order: Vec::new(),
            initialized: Vec::new(),
        }
    }

    /// Register a module.
    ///
    /// # Errors
    ///
    /// Returns an error when a module with the same id is already registered.
    pub fn register(&mut self, module: Box<dyn EngineModule>) -> Result<()> {
        let id = module.id();
        if self.phases.contains_key(&id) {
            return Err(error("a module with this id is already registered")
                .with_recovery(Recovery::Reject)
                .with_context("module", id.to_string()));
        }
        self.phases.insert(id, ModulePhase::Discovered);
        self.modules.push(module);
        Ok(())
    }

    /// The phase of a module, if registered.
    #[must_use]
    pub fn phase_of(&self, id: &ModuleId) -> Option<ModulePhase> {
        self.phases.get(id).copied()
    }

    /// The resolved initialization order, once [`ModuleManager::resolve`] has run.
    #[must_use]
    pub fn resolved_order(&self) -> Vec<ModuleId> {
        self.order
            .iter()
            .map(|index| self.modules[*index].id())
            .collect()
    }

    /// Validate declarations and compute a deterministic initialization order.
    ///
    /// # Errors
    ///
    /// Returns an error when a required dependency is missing or when the graph
    /// contains a cycle. Both name the modules involved, because "module load
    /// failed" without the chain is not a diagnosable failure.
    pub fn resolve(&mut self, mode: RuntimeMode) -> Result<()> {
        // Only modules that belong in this mode take part.
        let active: Vec<usize> = (0..self.modules.len())
            .filter(|index| self.modules[*index].sides().includes(mode))
            .collect();

        let mut index_of: BTreeMap<ModuleId, usize> = BTreeMap::new();
        for &index in &active {
            index_of.insert(self.modules[index].id(), index);
            self.phases
                .insert(self.modules[index].id(), ModulePhase::Validated);
        }

        // Edges point from dependency to dependent, so a node becomes ready once
        // everything it needs has been emitted.
        let mut dependents: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();
        let mut remaining: BTreeMap<ModuleId, usize> = BTreeMap::new();

        for &index in &active {
            let id = self.modules[index].id();
            let mut count = 0usize;
            for dependency in self.modules[index].dependencies() {
                if index_of.contains_key(dependency.id()) {
                    dependents
                        .entry(dependency.id().clone())
                        .or_default()
                        .insert(id.clone());
                    count += 1;
                } else if !dependency.is_optional() {
                    // Distinguish "absent entirely" from "excluded by mode": the
                    // second is a configuration error worth naming precisely.
                    let known = self.phases.contains_key(dependency.id());
                    return Err(error("required module dependency is not available")
                        .with_recovery(Recovery::Manual)
                        .with_context("module", id.to_string())
                        .with_context("requires", dependency.id().to_string())
                        .with_context("mode", mode.as_str())
                        .with_context(
                            "reason",
                            if known {
                                "excluded by runtime mode"
                            } else {
                                "not registered"
                            },
                        ));
                }
            }
            remaining.insert(id, count);
        }

        // Kahn's algorithm over a BTreeMap: ties break by module id, so the same
        // module set always produces the same order on every machine.
        let mut ready: BTreeSet<ModuleId> = remaining
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut order = Vec::with_capacity(active.len());
        while let Some(id) = ready.iter().next().cloned() {
            ready.remove(&id);
            order.push(index_of[&id]);
            self.phases.insert(id.clone(), ModulePhase::Resolved);

            if let Some(children) = dependents.get(&id) {
                for child in children {
                    let count = remaining.get_mut(child).expect("dependent must be tracked");
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(child.clone());
                    }
                }
            }
        }

        if order.len() != active.len() {
            let cycle: Vec<String> = remaining
                .iter()
                .filter(|(_, count)| **count > 0)
                .map(|(id, _)| id.to_string())
                .collect();
            return Err(error("module dependency graph contains a cycle")
                .with_recovery(Recovery::Manual)
                .with_context("modules", cycle.join(", ")));
        }

        for &index in &order {
            self.phases
                .insert(self.modules[index].id(), ModulePhase::Loaded);
        }
        self.order = order;
        Ok(())
    }

    /// Initialize every resolved module in dependency order.
    ///
    /// # Errors
    ///
    /// Returns the first initialization failure. Before returning, every module
    /// already initialized is shut down in reverse order, so a failed startup
    /// leaves nothing half-alive.
    pub fn initialize_all(&mut self, context: &ModuleContext) -> Result<()> {
        for position in 0..self.order.len() {
            let index = self.order[position];
            let id = self.modules[index].id();

            match self.modules[index].initialize(context) {
                Ok(()) => {
                    self.phases.insert(id.clone(), ModulePhase::Initialized);
                    self.initialized.push(index);
                    context.diagnostics().record(
                        Record::new(
                            Level::Debug,
                            Category::Engine,
                            "module-manager",
                            "module initialized",
                        )
                        .with_field("module", id.to_string()),
                    );
                }
                Err(cause) => {
                    context.diagnostics().record(
                        Record::new(
                            Level::Error,
                            Category::Engine,
                            "module-manager",
                            "module failed to initialize",
                        )
                        .with_field("module", id.to_string())
                        .with_field("cause", cause.to_string()),
                    );
                    self.shutdown_all(context);
                    return Err(error("module initialization failed")
                        .with_recovery(Recovery::Manual)
                        .with_context("module", id.to_string())
                        .with_source(cause));
                }
            }
        }

        for &index in &self.order {
            self.phases
                .insert(self.modules[index].id(), ModulePhase::Running);
        }
        Ok(())
    }

    /// Shut down every initialized module in reverse initialization order.
    ///
    /// Shutdown never aborts early: a module that fails to clean up is recorded
    /// and the remaining modules are still torn down.
    pub fn shutdown_all(&mut self, context: &ModuleContext) {
        while let Some(index) = self.initialized.pop() {
            let id = self.modules[index].id();
            self.phases.insert(id.clone(), ModulePhase::Stopping);
            if let Err(cause) = self.modules[index].shutdown(context) {
                context.diagnostics().record(
                    Record::new(
                        Level::Warn,
                        Category::Engine,
                        "module-manager",
                        "module shutdown failed",
                    )
                    .with_field("module", id.to_string())
                    .with_field("cause", cause.to_string()),
                );
            }
            self.phases.insert(id, ModulePhase::Unloaded);
        }
    }
}

fn error(message: &'static str) -> Error {
    Error::new(Domain::Module, "module-manager", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Shared log of initialization and shutdown, so ordering can be asserted.
    #[derive(Default)]
    struct Journal {
        entries: std::sync::Mutex<Vec<String>>,
    }

    impl Journal {
        fn push(&self, entry: String) {
            self.entries.lock().expect("journal lock").push(entry);
        }
        fn entries(&self) -> Vec<String> {
            self.entries.lock().expect("journal lock").clone()
        }
    }

    struct TestModule {
        id: ModuleId,
        dependencies: Vec<ModuleDependency>,
        sides: ModuleSides,
        journal: Arc<Journal>,
        fail_on_init: bool,
    }

    impl TestModule {
        fn new(id: &str, journal: Arc<Journal>) -> Self {
            Self {
                id: ModuleId::parse(id).expect("valid module id"),
                dependencies: Vec::new(),
                sides: ModuleSides::Always,
                journal,
                fail_on_init: false,
            }
        }

        fn needing(mut self, dependencies: &[&str]) -> Self {
            self.dependencies = dependencies
                .iter()
                .map(|raw| ModuleDependency::required(ModuleId::parse(raw).expect("valid id")))
                .collect();
            self
        }

        fn optionally_needing(mut self, dependency: &str) -> Self {
            self.dependencies.push(ModuleDependency::optional(
                ModuleId::parse(dependency).expect("valid id"),
            ));
            self
        }

        fn only_on(mut self, modes: &[RuntimeMode]) -> Self {
            self.sides = ModuleSides::Only(modes.to_vec());
            self
        }

        fn failing(mut self) -> Self {
            self.fail_on_init = true;
            self
        }
    }

    impl EngineModule for TestModule {
        fn id(&self) -> ModuleId {
            self.id.clone()
        }
        fn dependencies(&self) -> Vec<ModuleDependency> {
            self.dependencies.clone()
        }
        fn sides(&self) -> ModuleSides {
            self.sides.clone()
        }
        fn initialize(&mut self, _context: &ModuleContext) -> Result<()> {
            if self.fail_on_init {
                return Err(Error::new(
                    Domain::Module,
                    "test-module",
                    "deliberate failure",
                ));
            }
            self.journal.push(format!("init {}", self.id));
            Ok(())
        }
        fn shutdown(&mut self, _context: &ModuleContext) -> Result<()> {
            self.journal.push(format!("stop {}", self.id));
            Ok(())
        }
    }

    fn context() -> ModuleContext {
        ModuleContext::new(RuntimeMode::Test, Diagnostics::silent())
    }

    #[test]
    fn modules_initialize_in_dependency_order() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        // Registered in an order that is deliberately wrong for initialization.
        manager
            .register(Box::new(
                TestModule::new("nexora:world", journal.clone()).needing(&["nexora:core"]),
            ))
            .unwrap();
        manager
            .register(Box::new(
                TestModule::new("nexora:physics", journal.clone()).needing(&["nexora:world"]),
            ))
            .unwrap();
        manager
            .register(Box::new(TestModule::new("nexora:core", journal.clone())))
            .unwrap();

        manager.resolve(RuntimeMode::Test).unwrap();
        assert_eq!(
            manager
                .resolved_order()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["nexora:core", "nexora:world", "nexora:physics"]
        );

        manager.initialize_all(&context()).unwrap();
        assert_eq!(
            journal.entries(),
            [
                "init nexora:core",
                "init nexora:world",
                "init nexora:physics"
            ]
        );
    }

    #[test]
    fn shutdown_runs_in_reverse_dependency_order() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(TestModule::new("nexora:core", journal.clone())))
            .unwrap();
        manager
            .register(Box::new(
                TestModule::new("nexora:world", journal.clone()).needing(&["nexora:core"]),
            ))
            .unwrap();

        manager.resolve(RuntimeMode::Test).unwrap();
        manager.initialize_all(&context()).unwrap();
        manager.shutdown_all(&context());

        assert_eq!(
            journal.entries(),
            [
                "init nexora:core",
                "init nexora:world",
                "stop nexora:world",
                "stop nexora:core"
            ]
        );
        assert_eq!(
            manager.phase_of(&ModuleId::parse("nexora:core").unwrap()),
            Some(ModulePhase::Unloaded)
        );
    }

    #[test]
    fn circular_dependencies_are_reported_with_their_members() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(
                TestModule::new("nexora:a", journal.clone()).needing(&["nexora:b"]),
            ))
            .unwrap();
        manager
            .register(Box::new(
                TestModule::new("nexora:b", journal.clone()).needing(&["nexora:a"]),
            ))
            .unwrap();

        let err = manager
            .resolve(RuntimeMode::Test)
            .expect_err("a cycle must be refused");
        let rendered = err.to_string();
        assert!(rendered.contains("cycle"), "{rendered}");
        assert!(rendered.contains("nexora:a"), "{rendered}");
        assert!(rendered.contains("nexora:b"), "{rendered}");
    }

    #[test]
    fn a_missing_required_dependency_names_the_chain() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(
                TestModule::new("nexora:world", journal).needing(&["nexora:absent"]),
            ))
            .unwrap();

        let err = manager
            .resolve(RuntimeMode::Test)
            .expect_err("missing dependency must fail");
        let rendered = err.to_string();
        assert!(rendered.contains("module=nexora:world"), "{rendered}");
        assert!(rendered.contains("requires=nexora:absent"), "{rendered}");
        assert!(rendered.contains("not registered"), "{rendered}");
    }

    #[test]
    fn optional_dependencies_may_be_absent() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(
                TestModule::new("nexora:world", journal.clone())
                    .optionally_needing("nexora:profiler"),
            ))
            .unwrap();

        manager.resolve(RuntimeMode::Test).unwrap();
        manager.initialize_all(&context()).unwrap();
        assert_eq!(journal.entries(), ["init nexora:world"]);
    }

    #[test]
    fn a_client_only_module_does_not_block_a_dedicated_server() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(TestModule::new("nexora:core", journal.clone())))
            .unwrap();
        manager
            .register(Box::new(
                TestModule::new("nexora:renderer", journal.clone())
                    .needing(&["nexora:core"])
                    .only_on(&[RuntimeMode::Client]),
            ))
            .unwrap();

        // This is the invariant from ENGINE MODULE SYSTEM.md: the server starts.
        manager.resolve(RuntimeMode::DedicatedServer).unwrap();
        assert_eq!(
            manager
                .resolved_order()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["nexora:core"]
        );
        manager.initialize_all(&context()).unwrap();
        assert_eq!(journal.entries(), ["init nexora:core"]);
    }

    #[test]
    fn depending_on_a_mode_excluded_module_is_a_named_error() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(
                TestModule::new("nexora:renderer", journal.clone()).only_on(&[RuntimeMode::Client]),
            ))
            .unwrap();
        manager
            .register(Box::new(
                TestModule::new("nexora:hud", journal).needing(&["nexora:renderer"]),
            ))
            .unwrap();

        let err = manager
            .resolve(RuntimeMode::DedicatedServer)
            .expect_err("hud cannot run without the renderer");
        assert!(
            err.to_string().contains("excluded by runtime mode"),
            "{err}"
        );
    }

    #[test]
    fn a_failed_initialization_rolls_back_what_already_started() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(TestModule::new("nexora:core", journal.clone())))
            .unwrap();
        manager
            .register(Box::new(
                TestModule::new("nexora:broken", journal.clone())
                    .needing(&["nexora:core"])
                    .failing(),
            ))
            .unwrap();

        manager.resolve(RuntimeMode::Test).unwrap();
        let err = manager
            .initialize_all(&context())
            .expect_err("startup must fail");
        assert!(err.to_string().contains("nexora:broken"), "{err}");

        // core came up, then was rolled back; nothing is left running.
        assert_eq!(journal.entries(), ["init nexora:core", "stop nexora:core"]);
        assert_eq!(
            manager.phase_of(&ModuleId::parse("nexora:core").unwrap()),
            Some(ModulePhase::Unloaded)
        );
    }

    #[test]
    fn duplicate_module_ids_are_rejected() {
        let journal = Arc::new(Journal::default());
        let mut manager = ModuleManager::new();
        manager
            .register(Box::new(TestModule::new("nexora:core", journal.clone())))
            .unwrap();
        assert!(manager
            .register(Box::new(TestModule::new("nexora:core", journal)))
            .is_err());
    }

    #[test]
    fn resolution_order_is_deterministic_across_registration_orders() {
        // Independent modules must still come out in a stable order, otherwise
        // startup differs run to run and bugs become unreproducible.
        let build = |reversed: bool| {
            let journal = Arc::new(Journal::default());
            let mut manager = ModuleManager::new();
            let names = ["nexora:zulu", "nexora:alpha", "nexora:mike"];
            let ordered: Vec<&str> = if reversed {
                names.iter().rev().copied().collect()
            } else {
                names.to_vec()
            };
            for name in ordered {
                manager
                    .register(Box::new(TestModule::new(name, journal.clone())))
                    .unwrap();
            }
            manager.resolve(RuntimeMode::Test).unwrap();
            manager
                .resolved_order()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        };

        assert_eq!(build(false), build(true));
        assert_eq!(build(false), ["nexora:alpha", "nexora:mike", "nexora:zulu"]);
    }

    #[test]
    fn shutdown_continues_past_a_failing_module() {
        struct FailingShutdown {
            id: ModuleId,
            counter: Arc<AtomicUsize>,
        }
        impl EngineModule for FailingShutdown {
            fn id(&self) -> ModuleId {
                self.id.clone()
            }
            fn initialize(&mut self, _context: &ModuleContext) -> Result<()> {
                Ok(())
            }
            fn shutdown(&mut self, _context: &ModuleContext) -> Result<()> {
                self.counter.fetch_add(1, Ordering::SeqCst);
                Err(Error::new(Domain::Module, "test", "cleanup failed"))
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let mut manager = ModuleManager::new();
        for name in ["nexora:one", "nexora:two"] {
            manager
                .register(Box::new(FailingShutdown {
                    id: ModuleId::parse(name).unwrap(),
                    counter: counter.clone(),
                }))
                .unwrap();
        }
        manager.resolve(RuntimeMode::Test).unwrap();
        manager.initialize_all(&context()).unwrap();
        manager.shutdown_all(&context());

        // Both were asked to stop even though the first attempt errored.
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
