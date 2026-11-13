//! Built-in component library.

use crate::model::{
    Category, ComponentDefinition, GridPos, GridSize, PinDefinition, SymbolKind, find_definition,
};

const RESISTOR_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "1",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "2",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const LED_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "A",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "K",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const BUTTON_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "1",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "2",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const VCC_PINS: &[PinDefinition] = &[PinDefinition {
    name: "VCC",
    number: "1",
    offset: GridPos::new(1, 2),
}];

const GND_PINS: &[PinDefinition] = &[PinDefinition {
    name: "GND",
    number: "1",
    offset: GridPos::new(1, 0),
}];

const MCU_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "RESET",
        number: "1",
        offset: GridPos::new(0, 2),
    },
    PinDefinition {
        name: "D2",
        number: "2",
        offset: GridPos::new(0, 4),
    },
    PinDefinition {
        name: "D3",
        number: "3",
        offset: GridPos::new(0, 6),
    },
    PinDefinition {
        name: "D4",
        number: "4",
        offset: GridPos::new(0, 8),
    },
    PinDefinition {
        name: "D5",
        number: "5",
        offset: GridPos::new(0, 10),
    },
    PinDefinition {
        name: "D6",
        number: "6",
        offset: GridPos::new(0, 12),
    },
    PinDefinition {
        name: "D7",
        number: "7",
        offset: GridPos::new(0, 14),
    },
    PinDefinition {
        name: "VCC",
        number: "8",
        offset: GridPos::new(10, 2),
    },
    PinDefinition {
        name: "GND",
        number: "9",
        offset: GridPos::new(10, 4),
    },
    PinDefinition {
        name: "A0",
        number: "10",
        offset: GridPos::new(10, 6),
    },
    PinDefinition {
        name: "A1",
        number: "11",
        offset: GridPos::new(10, 8),
    },
    PinDefinition {
        name: "A2",
        number: "12",
        offset: GridPos::new(10, 10),
    },
    PinDefinition {
        name: "A3",
        number: "13",
        offset: GridPos::new(10, 12),
    },
    PinDefinition {
        name: "A4",
        number: "14",
        offset: GridPos::new(10, 14),
    },
];

const DEFINITIONS: &[ComponentDefinition] = &[
    ComponentDefinition {
        id: "resistor",
        name: "Resistor",
        category: Category::Passive,
        reference_prefix: "R",
        default_value: "220",
        size: GridSize::new(4, 2),
        pins: RESISTOR_PINS,
        symbol: SymbolKind::Resistor,
    },
    ComponentDefinition {
        id: "led",
        name: "LED",
        category: Category::Passive,
        reference_prefix: "D",
        default_value: "LED",
        size: GridSize::new(4, 2),
        pins: LED_PINS,
        symbol: SymbolKind::Led,
    },
    ComponentDefinition {
        id: "button",
        name: "Button",
        category: Category::Passive,
        reference_prefix: "SW",
        default_value: "TACT",
        size: GridSize::new(4, 2),
        pins: BUTTON_PINS,
        symbol: SymbolKind::Button,
    },
    ComponentDefinition {
        id: "vcc",
        name: "VCC",
        category: Category::Power,
        reference_prefix: "PWR",
        default_value: "5V",
        size: GridSize::new(2, 2),
        pins: VCC_PINS,
        symbol: SymbolKind::Vcc,
    },
    ComponentDefinition {
        id: "gnd",
        name: "GND",
        category: Category::Power,
        reference_prefix: "GND",
        default_value: "0V",
        size: GridSize::new(2, 2),
        pins: GND_PINS,
        symbol: SymbolKind::Gnd,
    },
    ComponentDefinition {
        id: "atmega328p",
        name: "ATmega328P",
        category: Category::Board,
        reference_prefix: "U",
        default_value: "ATmega328P",
        size: GridSize::new(10, 16),
        pins: MCU_PINS,
        symbol: SymbolKind::Mcu,
    },
];

/// Returns the built-in component definitions.
#[must_use]
pub fn builtin() -> &'static [ComponentDefinition] {
    DEFINITIONS
}

/// Finds a definition by ID.
#[must_use]
pub fn find(id: &str) -> Option<&'static ComponentDefinition> {
    find_definition(DEFINITIONS, id)
}

#[cfg(test)]
mod tests {
    use super::{DEFINITIONS, builtin, find};

    #[test]
    fn definition_ids_are_unique() {
        let mut ids: Vec<&str> = builtin().iter().map(|definition| definition.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), DEFINITIONS.len());
    }

    #[test]
    fn pins_are_inside_definition_bounds() {
        for definition in builtin() {
            for pin in definition.pins {
                assert!(pin.offset.x >= 0);
                assert!(pin.offset.y >= 0);
                assert!(pin.offset.x <= definition.size.width);
                assert!(pin.offset.y <= definition.size.height);
            }
        }
    }

    #[test]
    fn find_returns_known_definition() {
        assert_eq!(find("resistor").expect("resistor").name, "Resistor");
        assert!(find("missing").is_none());
    }
}
