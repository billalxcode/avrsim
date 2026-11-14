//! Generic component catalog with model classes and validation.

#[path = "catalog_data.rs"]
mod catalog_data;

use sim_core::DeviceKind;

/// Top-level component family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    Passive,
    Source,
    Diode,
    Transistor,
    Protection,
    Switch,
    Analog,
    Logic,
    Memory,
    Mcu,
    Power,
    Sensor,
    Actuator,
    Rf,
    Behavioral,
    Board,
}

/// Generic model class used by the catalog and the Python builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelClass {
    Resistor,
    Capacitor,
    Inductor,
    Transformer,
    Crystal,
    TransmissionLine,
    VoltageSource,
    CurrentSource,
    Vcvs,
    Vccs,
    Cccs,
    Ccvs,
    BehavioralSource,
    DigitalSource,
    Battery,
    Diode,
    Led,
    Photodiode,
    Optocoupler,
    Bjt,
    Jfet,
    Mosfet,
    OtherTransistor,
    Thyristor,
    Protection,
    Switch,
    Relay,
    AnalogSwitch,
    Connector,
    OpAmp,
    Comparator,
    Timer,
    VoltageReference,
    LogicGate,
    FlipFlop,
    Counter,
    Multiplexer,
    Adder,
    LevelShifter,
    ProgrammableLogic,
    Memory,
    Mcu,
    Adc,
    Dac,
    SerialInterface,
    TimerPeripheral,
    Regulator,
    DcDc,
    ChargePump,
    Supervisor,
    PowerSwitch,
    MotorDriver,
    Sensor,
    Actuator,
    Display,
    RfBlock,
    BehavioralBlock,
    BoardModule,
}

/// One catalog entry.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub class: ModelClass,
    pub device: Option<DeviceKind>,
    pub pins: u8,
    pub parameter: &'static str,
    pub default: f64,
    pub unit: &'static str,
}

/// Catalog validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogError {
    pub id: &'static str,
    pub reason: &'static str,
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.id, self.reason)
    }
}

impl std::error::Error for CatalogError {}

/// Returns all 500 generic component entries.
#[must_use]
pub fn catalog() -> &'static [ComponentSpec] {
    catalog_data::CATALOG
}

impl Family {
    /// Returns a stable family name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Passive => "passive",
            Self::Source => "source",
            Self::Diode => "diode",
            Self::Transistor => "transistor",
            Self::Protection => "protection",
            Self::Switch => "switch",
            Self::Analog => "analog",
            Self::Logic => "logic",
            Self::Memory => "memory",
            Self::Mcu => "mcu",
            Self::Power => "power",
            Self::Sensor => "sensor",
            Self::Actuator => "actuator",
            Self::Rf => "rf",
            Self::Behavioral => "behavioral",
            Self::Board => "board",
        }
    }

    /// Returns the primary reference for a family.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Self::Passive => "ngspice manual, passive devices",
            Self::Source => "ngspice manual, independent and controlled sources",
            Self::Diode => "ngspice manual, diode and optoelectronic devices",
            Self::Transistor => "ngspice manual, BJT, JFET, and MOSFET models",
            Self::Protection => "datasheet, overcurrent and overvoltage devices",
            Self::Switch => "ngspice manual, switch and relay behavior",
            Self::Analog => "datasheet and ngspice op-amp models",
            Self::Logic => "ngspice XSPICE digital models and 74xx library",
            Self::Memory => "datasheet, memory timing and retention",
            Self::Mcu => "datasheet, instruction and peripheral model",
            Self::Power => "datasheet, regulator and converter behavior",
            Self::Sensor => "datasheet, transducer transfer function",
            Self::Actuator => "datasheet, electrical to physical response",
            Self::Rf => "datasheet and S-parameter reference",
            Self::Behavioral => "user-defined transfer function",
            Self::Board => "board and module datasheet",
        }
    }
}

impl ModelClass {
    /// Returns the top-level family.
    #[must_use]
    pub const fn family(self) -> Family {
        if let Some(family) = self.passive_source_family() {
            return family;
        }
        if let Some(family) = self.semiconductor_family() {
            return family;
        }
        if let Some(family) = self.digital_family() {
            return family;
        }
        self.power_family()
    }

    const fn passive_source_family(self) -> Option<Family> {
        match self {
            Self::Resistor
            | Self::Capacitor
            | Self::Inductor
            | Self::Transformer
            | Self::Crystal
            | Self::TransmissionLine => Some(Family::Passive),
            Self::VoltageSource
            | Self::CurrentSource
            | Self::Vcvs
            | Self::Vccs
            | Self::Cccs
            | Self::Ccvs
            | Self::BehavioralSource
            | Self::DigitalSource
            | Self::Battery => Some(Family::Source),
            _ => None,
        }
    }

    const fn semiconductor_family(self) -> Option<Family> {
        match self {
            Self::Diode | Self::Led | Self::Photodiode | Self::Optocoupler => Some(Family::Diode),
            Self::Bjt | Self::Jfet | Self::Mosfet | Self::OtherTransistor => {
                Some(Family::Transistor)
            }
            Self::Thyristor | Self::Protection => Some(Family::Protection),
            Self::Switch | Self::Relay | Self::AnalogSwitch | Self::Connector => {
                Some(Family::Switch)
            }
            Self::OpAmp | Self::Comparator | Self::Timer | Self::VoltageReference => {
                Some(Family::Analog)
            }
            _ => None,
        }
    }

    const fn digital_family(self) -> Option<Family> {
        match self {
            Self::LogicGate
            | Self::FlipFlop
            | Self::Counter
            | Self::Multiplexer
            | Self::Adder
            | Self::LevelShifter
            | Self::ProgrammableLogic => Some(Family::Logic),
            Self::Memory => Some(Family::Memory),
            Self::Mcu | Self::Adc | Self::Dac | Self::SerialInterface | Self::TimerPeripheral => {
                Some(Family::Mcu)
            }
            _ => None,
        }
    }

    const fn power_family(self) -> Family {
        match self {
            Self::Regulator
            | Self::DcDc
            | Self::ChargePump
            | Self::Supervisor
            | Self::PowerSwitch
            | Self::MotorDriver => Family::Power,
            Self::Sensor => Family::Sensor,
            Self::Actuator | Self::Display => Family::Actuator,
            Self::RfBlock => Family::Rf,
            Self::BehavioralBlock => Family::Behavioral,
            _ => Family::Board,
        }
    }

    /// Returns the model fidelity tier.
    #[must_use]
    pub const fn tier(self) -> u8 {
        if self.tier_zero() {
            return 0;
        }
        if self.tier_one() {
            return 1;
        }
        if self.tier_two() {
            return 2;
        }
        4
    }

    const fn tier_zero(self) -> bool {
        matches!(
            self,
            Self::Resistor
                | Self::Capacitor
                | Self::Inductor
                | Self::VoltageSource
                | Self::CurrentSource
                | Self::Battery
        )
    }

    const fn tier_one(self) -> bool {
        matches!(
            self,
            Self::Transformer
                | Self::Crystal
                | Self::TransmissionLine
                | Self::Vcvs
                | Self::Vccs
                | Self::Cccs
                | Self::Ccvs
                | Self::BehavioralSource
                | Self::DigitalSource
                | Self::Switch
                | Self::Relay
                | Self::AnalogSwitch
                | Self::Connector
                | Self::Protection
                | Self::PowerSwitch
        )
    }

    const fn tier_two(self) -> bool {
        matches!(
            self,
            Self::Diode
                | Self::Led
                | Self::Photodiode
                | Self::Optocoupler
                | Self::Bjt
                | Self::Jfet
                | Self::Mosfet
                | Self::OtherTransistor
                | Self::Thyristor
        )
    }

    /// Returns true when the class maps to a solver device kind.
    #[must_use]
    pub const fn requires_device(self) -> bool {
        self.requires_passive_device() || self.requires_semiconductor_device()
    }

    const fn requires_passive_device(self) -> bool {
        matches!(
            self,
            Self::Resistor
                | Self::Capacitor
                | Self::Inductor
                | Self::TransmissionLine
                | Self::VoltageSource
                | Self::CurrentSource
                | Self::Vcvs
                | Self::Vccs
                | Self::Cccs
                | Self::Ccvs
                | Self::Battery
        )
    }

    const fn requires_semiconductor_device(self) -> bool {
        matches!(
            self,
            Self::Diode
                | Self::Led
                | Self::Photodiode
                | Self::Bjt
                | Self::Jfet
                | Self::Mosfet
                | Self::Switch
        )
    }

    /// Returns a short working principle.
    #[must_use]
    pub const fn principle(self) -> &'static str {
        match self.family() {
            Family::Passive => self.passive_principle(),
            Family::Source => self.source_principle(),
            Family::Diode => self.diode_principle(),
            Family::Transistor => self.transistor_principle(),
            Family::Protection => self.protection_principle(),
            Family::Switch => self.switch_principle(),
            Family::Analog => self.analog_principle(),
            Family::Logic => self.logic_principle(),
            Family::Memory => "Stores state or charge and exposes read and write timing.",
            Family::Mcu => self.mcu_principle(),
            Family::Power => self.power_principle(),
            Family::Sensor => "Converts a physical quantity into an electrical signal.",
            Family::Actuator => "Converts an electrical signal into physical output.",
            Family::Rf => "Frequency-domain gain, filtering, or radiation behavior.",
            Family::Behavioral => "User-defined algebraic or table-driven transfer function.",
            Family::Board => "Board-level interconnect and module behavior.",
        }
    }

    const fn passive_principle(self) -> &'static str {
        match self {
            Self::Resistor => "Linear voltage-current relation V = I x R.",
            Self::Capacitor => "Charge integration I = C x dV/dt.",
            Self::Inductor => "Flux integration V = L x dI/dt.",
            Self::Transformer => "Coupled magnetic windings with a turns ratio.",
            Self::Crystal => "Piezoelectric resonance with series and parallel modes.",
            _ => "Distributed transmission-line behavior.",
        }
    }

    const fn source_principle(self) -> &'static str {
        match self {
            Self::VoltageSource => "Holds a defined voltage across its terminals.",
            Self::CurrentSource => "Forces a defined current through its branch.",
            Self::Vcvs => "Output voltage is proportional to an input voltage.",
            Self::Vccs => "Output current is proportional to an input voltage.",
            Self::Cccs => "Output current is proportional to a control current.",
            Self::Ccvs => "Output voltage is proportional to a control current.",
            Self::BehavioralSource => "Output follows an expression or lookup table.",
            Self::DigitalSource => "Generates a clock, pattern, or pseudo-random sequence.",
            _ => "Stores or delivers electrical energy.",
        }
    }

    const fn diode_principle(self) -> &'static str {
        match self {
            Self::Diode => "Shockley junction with forward and reverse regions.",
            Self::Led => "Diode with radiative recombination and forward voltage.",
            Self::Photodiode => "Photoelectric current proportional to incident light.",
            _ => "Optical isolation between input and output.",
        }
    }

    const fn transistor_principle(self) -> &'static str {
        match self {
            Self::Bjt => "Ebers-Moll or Gummel-Poon current gain.",
            Self::Jfet => "Depletion channel controlled by a gate junction.",
            Self::Mosfet => "Field-effect channel controlled by gate voltage.",
            _ => "Datasheet-level power or high-frequency transistor behavior.",
        }
    }

    const fn protection_principle(self) -> &'static str {
        match self {
            Self::Thyristor => "Four-layer latch triggered by gate current.",
            _ => "Clamps or interrupts overcurrent and overvoltage.",
        }
    }

    const fn switch_principle(self) -> &'static str {
        match self {
            Self::Switch => "Controlled two-state conductance.",
            Self::Relay => "Coil or optical isolation drives a mechanical or solid-state contact.",
            Self::Connector => "Board-level electrical and mechanical interconnect.",
            _ => "Controlled analog signal path.",
        }
    }

    const fn analog_principle(self) -> &'static str {
        match self {
            Self::OpAmp => "High-gain differential input with feedback.",
            Self::Comparator => "Open-loop differential input with saturated output.",
            Self::Timer => "RC threshold and comparator timing.",
            _ => "Temperature-compensated voltage or current reference.",
        }
    }

    const fn logic_principle(self) -> &'static str {
        match self {
            Self::LogicGate => "Boolean function with propagation delay.",
            Self::FlipFlop => "Bistable state clocked by an input event.",
            Self::Counter => "Clocked state machine with a count output.",
            Self::Multiplexer => "Selects one of several inputs or outputs.",
            Self::Adder => "Combinational arithmetic or comparison.",
            Self::LevelShifter => "Translates between voltage domains.",
            _ => "Programmable array or lookup logic.",
        }
    }

    const fn mcu_principle(self) -> &'static str {
        match self {
            Self::Mcu => "Instruction-driven core with memory and peripherals.",
            Self::Adc => "Samples and quantizes an analog input.",
            Self::Dac => "Converts a digital code into an analog output.",
            Self::SerialInterface => "Framed serial protocol with timing.",
            _ => "Counter, compare, or watchdog peripheral.",
        }
    }

    const fn power_principle(self) -> &'static str {
        match self {
            Self::Regulator => "Feedback controls a pass element.",
            Self::DcDc => "Switched inductor or capacitor energy transfer.",
            Self::ChargePump => "Capacitive charge transfer between stages.",
            Self::Supervisor => "Threshold monitor with reset or reference output.",
            Self::PowerSwitch => "Low-resistance pass element with enable control.",
            _ => "Power bridge or driver stage.",
        }
    }

    /// Returns a scalar smoke response for catalog checks.
    #[must_use]
    pub fn scalar_response(self, input: f64) -> f64 {
        match self.family() {
            Family::Passive | Family::Source | Family::Board => input,
            Family::Diode => diode_response(input),
            Family::Transistor => transistor_response(self, input),
            Family::Protection => protection_response(self, input),
            Family::Switch => input,
            Family::Analog => analog_response(self, input),
            Family::Logic => logic_response(self, input),
            Family::Memory => input,
            Family::Mcu => mcu_response(self, input),
            Family::Power => power_response(self, input),
            Family::Sensor => input,
            Family::Actuator => input,
            Family::Rf => input,
            Family::Behavioral => input,
        }
    }
}

fn diode_response(input: f64) -> f64 {
    let voltage = input.clamp(-40.0, 40.0);
    (voltage.exp() - 1.0).clamp(-1.0e6, 1.0e6)
}

fn transistor_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Bjt => input * 100.0,
        ModelClass::Jfet | ModelClass::Mosfet => input * input,
        _ => input,
    }
}

fn protection_response(class: ModelClass, input: f64) -> f64 {
    if class == ModelClass::Thyristor {
        return if input > 0.0 { 1.0 } else { 0.0 };
    }
    if input.abs() > 1.0 {
        return 0.0;
    }
    1.0
}

fn analog_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::OpAmp | ModelClass::Comparator => (input * 1.0e5).clamp(-1.0e6, 1.0e6),
        ModelClass::Timer => {
            if input > 0.0 {
                1.0
            } else {
                0.0
            }
        }
        _ => input.clamp(-10.0, 10.0),
    }
}

fn logic_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Counter => input.floor().clamp(0.0, 1.0e6),
        ModelClass::Adder | ModelClass::LevelShifter => input + 1.0,
        ModelClass::LogicGate | ModelClass::FlipFlop => {
            if input > 0.5 {
                1.0
            } else {
                0.0
            }
        }
        _ => input,
    }
}

fn mcu_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Adc => input.round().clamp(0.0, 4095.0),
        ModelClass::Dac => input.clamp(0.0, 4095.0),
        _ => input,
    }
}

fn power_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Regulator => input.clamp(-20.0, 20.0),
        ModelClass::DcDc => input * 0.9,
        ModelClass::ChargePump => input * 2.0,
        ModelClass::Supervisor => {
            if input > 2.5 {
                1.0
            } else {
                0.0
            }
        }
        _ => input,
    }
}

impl ComponentSpec {
    /// Validates catalog invariants for this entry.
    pub fn validate(&self) -> Result<(), CatalogError> {
        self.validate_text()?;
        self.validate_values()?;
        self.validate_class()
    }

    fn validate_text(&self) -> Result<(), CatalogError> {
        if self.id.is_empty() || !self.id.is_ascii() {
            return Err(self.error("id must be non-empty ASCII"));
        }
        if self.name.is_empty() || !self.name.is_ascii() {
            return Err(self.error("name must be non-empty ASCII"));
        }
        if self.parameter.is_empty() || !self.parameter.is_ascii() {
            return Err(self.error("parameter must be non-empty ASCII"));
        }
        if self.unit.is_empty() || !self.unit.is_ascii() {
            return Err(self.error("unit must be non-empty ASCII"));
        }
        Ok(())
    }

    fn validate_values(&self) -> Result<(), CatalogError> {
        if self.pins == 0 {
            return Err(self.error("pins must be positive"));
        }
        if !self.default.is_finite() {
            return Err(self.error("default must be finite"));
        }
        Ok(())
    }

    fn validate_class(&self) -> Result<(), CatalogError> {
        if self.class.principle().is_empty() {
            return Err(self.error("principle must be non-empty"));
        }
        if self.class.family().source().is_empty() {
            return Err(self.error("source must be non-empty"));
        }
        Ok(())
    }

    fn error(&self, reason: &'static str) -> CatalogError {
        CatalogError {
            id: self.id,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Family, catalog};
    use std::collections::BTreeSet;

    #[test]
    fn catalog_has_500_unique_entries() {
        let entries = catalog();
        assert_eq!(entries.len(), 500);
        let ids: BTreeSet<_> = entries.iter().map(|entry| entry.id).collect();
        assert_eq!(ids.len(), 500);
    }

    #[test]
    fn every_entry_validates() {
        for entry in catalog() {
            entry.validate().unwrap_or_else(|error| panic!("{error}"));
        }
    }

    #[test]
    fn ids_are_lowercase_ascii_slugs() {
        for entry in catalog() {
            assert!(
                entry.id.bytes().all(|byte| byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || byte == b'.'
                    || byte == b'_'),
                "bad id: {}",
                entry.id
            );
        }
    }

    #[test]
    fn all_classes_have_entries() {
        let classes: BTreeSet<_> = catalog().iter().map(|entry| entry.class).collect();
        assert_eq!(classes.len(), 57);
    }

    #[test]
    fn family_counts_match_design() {
        assert_family(Family::Passive, 55);
        assert_family(Family::Source, 25);
        assert_family(Family::Diode, 40);
        assert_family(Family::Transistor, 55);
        assert_family(Family::Protection, 18);
        assert_family(Family::Switch, 25);
        assert_family(Family::Analog, 40);
        assert_family(Family::Logic, 60);
        assert_family(Family::Memory, 15);
        assert_family(Family::Mcu, 25);
        assert_family(Family::Power, 30);
        assert_family(Family::Sensor, 35);
        assert_family(Family::Actuator, 20);
        assert_family(Family::Rf, 18);
        assert_family(Family::Behavioral, 25);
        assert_family(Family::Board, 14);
    }

    fn assert_family(family: Family, expected: usize) {
        let count = catalog()
            .iter()
            .filter(|entry| entry.class.family() == family)
            .count();
        assert_eq!(count, expected, "family {}", family.name());
    }

    #[test]
    fn scalar_responses_are_finite() {
        for entry in catalog() {
            for input in [-10.0, -1.0, 0.0, 1.0, 10.0] {
                assert!(
                    entry.class.scalar_response(input).is_finite(),
                    "{} at {input}",
                    entry.id
                );
            }
        }
    }

    #[test]
    fn every_entry_has_positive_pins_and_default() {
        for entry in catalog() {
            assert!(entry.pins > 0, "{}", entry.id);
            assert!(entry.default.is_finite(), "{}", entry.id);
        }
    }

    #[test]
    fn analog_classes_expose_a_solver_kind() {
        for entry in catalog() {
            if entry.class.requires_device() {
                assert!(entry.device.is_some(), "{}", entry.id);
            }
        }
    }

    #[test]
    fn device_presence_matches_class_contract() {
        for entry in catalog() {
            assert_eq!(
                entry.device.is_some(),
                entry.class.requires_device(),
                "{}",
                entry.id
            );
        }
    }

    #[test]
    fn device_kinds_are_solver_supported() {
        for entry in catalog() {
            let Some(device) = &entry.device else {
                continue;
            };
            assert!(supported_device(device), "{}", entry.id);
        }
    }

    fn supported_device(device: &sim_core::DeviceKind) -> bool {
        supported_passive(device) || supported_active(device)
    }

    fn supported_passive(device: &sim_core::DeviceKind) -> bool {
        matches!(
            device,
            sim_core::DeviceKind::Resistor
                | sim_core::DeviceKind::Capacitor
                | sim_core::DeviceKind::Inductor
                | sim_core::DeviceKind::VoltageSource
                | sim_core::DeviceKind::CurrentSource
                | sim_core::DeviceKind::Vcvs
                | sim_core::DeviceKind::Vccs
                | sim_core::DeviceKind::Cccs
                | sim_core::DeviceKind::Ccvs
                | sim_core::DeviceKind::Switch
                | sim_core::DeviceKind::TransmissionLine
        )
    }

    fn supported_active(device: &sim_core::DeviceKind) -> bool {
        matches!(
            device,
            sim_core::DeviceKind::Nmos
                | sim_core::DeviceKind::Pmos
                | sim_core::DeviceKind::Npn
                | sim_core::DeviceKind::Pnp
                | sim_core::DeviceKind::JfetN
                | sim_core::DeviceKind::JfetP
                | sim_core::DeviceKind::Diode
        )
    }

    #[test]
    fn passive_defaults_are_positive() {
        for entry in catalog() {
            if entry.class.family() == Family::Passive {
                assert!(entry.default > 0.0, "{}", entry.id);
            }
        }
    }

    #[test]
    fn catalog_is_deterministic() {
        let first = catalog();
        let second = catalog();
        assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter()) {
            assert_eq!(left.id, right.id);
            assert_eq!(left.default.to_bits(), right.default.to_bits());
        }
    }

    #[test]
    fn tiers_are_within_design() {
        for entry in catalog() {
            assert!(entry.class.tier() <= 4, "{}", entry.id);
        }
    }

    #[test]
    fn entries_have_distinct_names_per_family() {
        for entry in catalog() {
            let duplicates = catalog()
                .iter()
                .filter(|other| {
                    other.class.family() == entry.class.family() && other.name == entry.name
                })
                .count();
            assert_eq!(duplicates, 1, "{}", entry.id);
        }
    }

    #[test]
    fn principles_and_sources_are_present() {
        for entry in catalog() {
            assert!(!entry.class.principle().is_empty(), "{}", entry.id);
            assert!(!entry.class.family().source().is_empty(), "{}", entry.id);
        }
    }
}
