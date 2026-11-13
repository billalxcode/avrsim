//! Internal component provider trait.

use crate::library;
use crate::model::ComponentDefinition;

/// Supplies component definitions to the GUI.
pub trait ComponentProvider {
    /// Returns the built-in definitions.
    fn definitions(&self) -> &'static [ComponentDefinition];
}

/// Built-in Rust component library.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuiltinComponents;

impl ComponentProvider for BuiltinComponents {
    fn definitions(&self) -> &'static [ComponentDefinition] {
        library::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltinComponents, ComponentProvider};

    #[test]
    fn builtin_provider_returns_definitions() {
        let definitions = BuiltinComponents.definitions();
        assert!(
            definitions
                .iter()
                .any(|definition| definition.id == "resistor")
        );
    }
}
