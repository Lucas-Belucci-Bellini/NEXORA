//! Layered configuration.
//!
//! Implements `CONFIGURATION AND SETTINGS SYSTEM.md`. Three of its invariants
//! drive the design:
//!
//! * **No hidden global mutable configuration** - there is no static here; a
//!   [`Settings`] value is owned by whoever created it and passed explicitly.
//! * **Protected settings require explicit authority** - a user profile cannot
//!   override a server policy just by sitting higher in the layer order.
//! * **Mods receive only declared namespaces** - a mod writing outside its own
//!   namespace is rejected, not silently ignored.

use std::collections::BTreeMap;

use crate::error::{Domain, Error, Recovery, Result};
use crate::ident::{Identifier, Namespace};

/// Configuration layers, in ascending order of precedence.
///
/// The order matches the layer list in the settings document. A higher layer
/// wins unless the setting is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// Values declared by the schema itself.
    Defaults,
    /// Values shipped with the project.
    Project,
    /// Values stored with a specific world.
    World,
    /// Values set by a server operator.
    Server,
    /// Values chosen by the local user.
    User,
    /// Values injected for one run, e.g. by a test or a command-line flag.
    RuntimeOverride,
}

impl Scope {
    /// Every scope, lowest precedence first.
    pub const ALL: [Self; 6] = [
        Self::Defaults,
        Self::Project,
        Self::World,
        Self::Server,
        Self::User,
        Self::RuntimeOverride,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Defaults => "defaults",
            Self::Project => "project",
            Self::World => "world",
            Self::Server => "server",
            Self::User => "user",
            Self::RuntimeOverride => "runtime-override",
        }
    }
}

/// A configuration value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A flag.
    Bool(bool),
    /// A whole number.
    Integer(i64),
    /// A real number.
    Float(f64),
    /// Free text.
    Text(String),
}

impl Value {
    /// The type name, for error messages and schema checks.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Text(_) => "text",
        }
    }

    /// Whether two values share a type.
    #[must_use]
    pub const fn same_type_as(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Bool(_), Self::Bool(_))
                | (Self::Integer(_), Self::Integer(_))
                | (Self::Float(_), Self::Float(_))
                | (Self::Text(_), Self::Text(_))
        )
    }
}

/// Who is attempting a configuration change.
///
/// Authority is a parameter rather than ambient state so that a call site
/// cannot accidentally inherit more privilege than it should have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Authority {
    /// The engine itself during bootstrap.
    Engine,
    /// A server operator.
    Server,
    /// The local user.
    User,
    /// A mod, restricted to its own namespace.
    Mod(Namespace),
}

/// Declaration of one setting.
#[derive(Debug, Clone)]
pub struct SettingSchema {
    key: Identifier,
    default: Value,
    description: &'static str,
    protected: bool,
    validator: Option<fn(&Value) -> Result<()>>,
}

impl SettingSchema {
    /// Declare a setting with a default value.
    #[must_use]
    pub fn new(key: Identifier, default: Value, description: &'static str) -> Self {
        Self {
            key,
            default,
            description,
            protected: false,
            validator: None,
        }
    }

    /// Mark the setting as changeable only by [`Authority::Engine`] or
    /// [`Authority::Server`].
    #[must_use]
    pub const fn protected(mut self) -> Self {
        self.protected = true;
        self
    }

    /// Attach a domain validator, run on every write.
    #[must_use]
    pub const fn with_validator(mut self, validator: fn(&Value) -> Result<()>) -> Self {
        self.validator = Some(validator);
        self
    }

    /// The setting key.
    #[must_use]
    pub const fn key(&self) -> &Identifier {
        &self.key
    }

    /// The default value.
    #[must_use]
    pub const fn default_value(&self) -> &Value {
        &self.default
    }

    /// The human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    /// Whether the setting is protected.
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        self.protected
    }
}

/// A registered set of settings and their layered values.
#[derive(Debug, Default)]
pub struct Settings {
    schemas: BTreeMap<Identifier, SettingSchema>,
    layers: BTreeMap<Scope, BTreeMap<Identifier, Value>>,
}

impl Settings {
    /// Create an empty settings store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a setting declaration.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is already registered. Silently replacing
    /// a schema would let one subsystem change another's contract.
    pub fn register(&mut self, schema: SettingSchema) -> Result<()> {
        if self.schemas.contains_key(schema.key()) {
            return Err(error("setting is already registered")
                .with_context("key", schema.key().to_string()));
        }
        self.schemas.insert(schema.key().clone(), schema);
        Ok(())
    }

    /// The schema for a key, if registered.
    #[must_use]
    pub fn schema(&self, key: &Identifier) -> Option<&SettingSchema> {
        self.schemas.get(key)
    }

    /// Write a value into a layer.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is unregistered, the value type does not
    /// match the schema, the validator rejects it, or the authority is
    /// insufficient.
    pub fn set(
        &mut self,
        key: &Identifier,
        value: Value,
        scope: Scope,
        authority: &Authority,
    ) -> Result<()> {
        let schema = self.schemas.get(key).ok_or_else(|| {
            error("setting is not registered")
                .with_recovery(Recovery::Reject)
                .with_context("key", key.to_string())
        })?;

        if !value.same_type_as(schema.default_value()) {
            return Err(error("setting value has the wrong type")
                .with_recovery(Recovery::Reject)
                .with_context("key", key.to_string())
                .with_context("expected", schema.default_value().type_name())
                .with_context("found", value.type_name()));
        }

        authorize(schema, key, authority)?;

        if let Some(validate) = schema.validator {
            validate(&value).map_err(|cause| {
                error("setting value failed validation")
                    .with_recovery(Recovery::Reject)
                    .with_context("key", key.to_string())
                    .with_source(cause)
            })?;
        }

        self.layers
            .entry(scope)
            .or_default()
            .insert(key.clone(), value);
        Ok(())
    }

    /// Resolve a setting through the layer order.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is not registered.
    pub fn get(&self, key: &Identifier) -> Result<&Value> {
        let schema = self.schemas.get(key).ok_or_else(|| {
            error("setting is not registered")
                .with_recovery(Recovery::Reject)
                .with_context("key", key.to_string())
        })?;

        // Walk from the highest-precedence layer down to the schema default.
        for scope in Scope::ALL.iter().rev() {
            if let Some(value) = self.layers.get(scope).and_then(|layer| layer.get(key)) {
                return Ok(value);
            }
        }
        Ok(schema.default_value())
    }

    /// Resolve a boolean setting.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is unregistered or holds another type.
    pub fn get_bool(&self, key: &Identifier) -> Result<bool> {
        match self.get(key)? {
            Value::Bool(value) => Ok(*value),
            other => Err(type_mismatch(key, "bool", other)),
        }
    }

    /// Resolve an integer setting.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is unregistered or holds another type.
    pub fn get_integer(&self, key: &Identifier) -> Result<i64> {
        match self.get(key)? {
            Value::Integer(value) => Ok(*value),
            other => Err(type_mismatch(key, "integer", other)),
        }
    }

    /// Resolve a float setting.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is unregistered or holds another type.
    pub fn get_float(&self, key: &Identifier) -> Result<f64> {
        match self.get(key)? {
            Value::Float(value) => Ok(*value),
            other => Err(type_mismatch(key, "float", other)),
        }
    }

    /// Resolve a text setting.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is unregistered or holds another type.
    pub fn get_text(&self, key: &Identifier) -> Result<&str> {
        match self.get(key)? {
            Value::Text(value) => Ok(value),
            other => Err(type_mismatch(key, "text", other)),
        }
    }

    /// Resolve every registered setting.
    ///
    /// # Errors
    ///
    /// Returns an error only if a registered key cannot be resolved, which
    /// would indicate an internal inconsistency.
    pub fn snapshot(&self) -> Result<BTreeMap<Identifier, Value>> {
        let mut out = BTreeMap::new();
        for key in self.schemas.keys() {
            out.insert(key.clone(), self.get(key)?.clone());
        }
        Ok(out)
    }

    /// Re-run every validator against the currently resolved values.
    ///
    /// Returns the keys that fail. Used at startup, after migration, and after
    /// hot reload, where individually valid layers can still combine badly.
    #[must_use]
    pub fn validate_all(&self) -> Vec<(Identifier, Error)> {
        let mut failures = Vec::new();
        for (key, schema) in &self.schemas {
            let Some(validate) = schema.validator else {
                continue;
            };
            match self.get(key) {
                Ok(value) => {
                    if let Err(cause) = validate(value) {
                        failures.push((key.clone(), cause));
                    }
                }
                Err(cause) => failures.push((key.clone(), cause)),
            }
        }
        failures
    }
}

fn authorize(schema: &SettingSchema, key: &Identifier, authority: &Authority) -> Result<()> {
    if let Authority::Mod(namespace) = authority {
        // A mod owns its namespace and nothing else, protected or not.
        if key.namespace() != namespace {
            return Err(error("a mod may only configure keys in its own namespace")
                .with_recovery(Recovery::Reject)
                .with_context("key", key.to_string())
                .with_context("mod_namespace", namespace.to_string()));
        }
        if schema.is_protected() {
            return Err(error("a mod may not change a protected setting")
                .with_recovery(Recovery::Reject)
                .with_context("key", key.to_string()));
        }
        return Ok(());
    }

    if schema.is_protected() && !matches!(authority, Authority::Engine | Authority::Server) {
        return Err(
            error("this setting is protected and requires server authority")
                .with_recovery(Recovery::Reject)
                .with_context("key", key.to_string()),
        );
    }
    Ok(())
}

fn type_mismatch(key: &Identifier, expected: &'static str, found: &Value) -> Error {
    error("setting was read as the wrong type")
        .with_recovery(Recovery::Reject)
        .with_context("key", key.to_string())
        .with_context("expected", expected)
        .with_context("found", found.type_name())
}

fn error(message: &'static str) -> Error {
    Error::new(Domain::Config, "settings", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test key must be valid")
    }

    fn positive_integer(value: &Value) -> Result<()> {
        match value {
            Value::Integer(n) if *n > 0 => Ok(()),
            _ => Err(Error::new(
                Domain::Config,
                "validator",
                "value must be a positive integer",
            )),
        }
    }

    fn settings_with_defaults() -> Settings {
        let mut settings = Settings::new();
        settings
            .register(SettingSchema::new(
                key("nexora:performance/simulation_distance"),
                Value::Integer(16),
                "Radius in chunks simulated around a player.",
            ))
            .unwrap();
        settings
            .register(
                SettingSchema::new(
                    key("nexora:server/max_players"),
                    Value::Integer(20),
                    "Maximum simultaneous players.",
                )
                .protected()
                .with_validator(positive_integer),
            )
            .unwrap();
        settings
    }

    #[test]
    fn unset_keys_fall_back_to_their_default() {
        let settings = settings_with_defaults();
        assert_eq!(
            settings
                .get_integer(&key("nexora:performance/simulation_distance"))
                .unwrap(),
            16
        );
    }

    #[test]
    fn higher_layers_win_over_lower_ones() {
        let mut settings = settings_with_defaults();
        let k = key("nexora:performance/simulation_distance");

        settings
            .set(&k, Value::Integer(8), Scope::Project, &Authority::Engine)
            .unwrap();
        assert_eq!(settings.get_integer(&k).unwrap(), 8);

        settings
            .set(&k, Value::Integer(12), Scope::World, &Authority::Engine)
            .unwrap();
        assert_eq!(settings.get_integer(&k).unwrap(), 12);

        settings
            .set(&k, Value::Integer(24), Scope::User, &Authority::User)
            .unwrap();
        assert_eq!(settings.get_integer(&k).unwrap(), 24);

        settings
            .set(
                &k,
                Value::Integer(2),
                Scope::RuntimeOverride,
                &Authority::Engine,
            )
            .unwrap();
        assert_eq!(settings.get_integer(&k).unwrap(), 2);
    }

    #[test]
    fn protected_settings_reject_user_authority() {
        let mut settings = settings_with_defaults();
        let k = key("nexora:server/max_players");

        let err = settings
            .set(&k, Value::Integer(500), Scope::User, &Authority::User)
            .expect_err("a user must not raise a server limit");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert_eq!(settings.get_integer(&k).unwrap(), 20);

        settings
            .set(&k, Value::Integer(64), Scope::Server, &Authority::Server)
            .unwrap();
        assert_eq!(settings.get_integer(&k).unwrap(), 64);
    }

    #[test]
    fn mods_are_confined_to_their_own_namespace() {
        let mut settings = settings_with_defaults();
        let mod_namespace = Namespace::parse("example").unwrap();
        settings
            .register(SettingSchema::new(
                key("example:feature/enabled"),
                Value::Bool(false),
                "Mod-owned flag.",
            ))
            .unwrap();

        // Its own key: allowed.
        settings
            .set(
                &key("example:feature/enabled"),
                Value::Bool(true),
                Scope::User,
                &Authority::Mod(mod_namespace.clone()),
            )
            .unwrap();
        assert!(settings.get_bool(&key("example:feature/enabled")).unwrap());

        // Someone else's key: rejected even though it is not protected.
        let err = settings
            .set(
                &key("nexora:performance/simulation_distance"),
                Value::Integer(1),
                Scope::User,
                &Authority::Mod(mod_namespace),
            )
            .expect_err("a mod must not configure engine keys");
        assert_eq!(err.recovery(), Recovery::Reject);
    }

    #[test]
    fn wrong_types_and_unknown_keys_are_rejected() {
        let mut settings = settings_with_defaults();
        let k = key("nexora:performance/simulation_distance");

        assert!(settings
            .set(&k, Value::Bool(true), Scope::User, &Authority::User)
            .is_err());
        assert!(settings.get_bool(&k).is_err());
        assert!(settings.get(&key("nexora:missing/key")).is_err());
        assert!(settings
            .set(
                &key("nexora:missing/key"),
                Value::Bool(true),
                Scope::User,
                &Authority::User
            )
            .is_err());
    }

    #[test]
    fn validators_run_on_write_and_on_demand() {
        let mut settings = settings_with_defaults();
        let k = key("nexora:server/max_players");

        let err = settings
            .set(&k, Value::Integer(-1), Scope::Server, &Authority::Server)
            .expect_err("negative player limit must be rejected");
        assert!(err.to_string().contains("failed validation"), "{err}");

        assert!(settings.validate_all().is_empty());
    }

    #[test]
    fn duplicate_registration_is_an_error() {
        let mut settings = settings_with_defaults();
        let duplicate = SettingSchema::new(
            key("nexora:performance/simulation_distance"),
            Value::Integer(1),
            "Conflicting redeclaration.",
        );
        assert!(settings.register(duplicate).is_err());
    }

    #[test]
    fn snapshot_resolves_every_registered_key() {
        let mut settings = settings_with_defaults();
        settings
            .set(
                &key("nexora:performance/simulation_distance"),
                Value::Integer(4),
                Scope::World,
                &Authority::Engine,
            )
            .unwrap();

        let snapshot = settings.snapshot().unwrap();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(
            snapshot.get(&key("nexora:performance/simulation_distance")),
            Some(&Value::Integer(4))
        );
        assert_eq!(
            snapshot.get(&key("nexora:server/max_players")),
            Some(&Value::Integer(20))
        );
    }
}
