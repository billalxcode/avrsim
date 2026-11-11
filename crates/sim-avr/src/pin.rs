//! Electrical model for one GPIO pin.

use sim_core::SimError;

/// Electrical configuration for a GPIO pin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinElectricalConfig {
    /// Supply voltage in volts.
    pub supply: f64,
    /// Output driver resistance in ohms.
    pub output_resistance: f64,
    /// Pull-up resistance in ohms.
    pub pull_up_resistance: f64,
    /// Digital input threshold in volts.
    pub input_threshold: f64,
}

impl Default for PinElectricalConfig {
    fn default() -> Self {
        Self {
            supply: 5.0,
            output_resistance: 45.0,
            pull_up_resistance: 35_000.0,
            input_threshold: 2.5,
        }
    }
}

/// Digital mode of one pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinMode {
    /// Input without pull-up.
    Input,
    /// Input with pull-up.
    InputPullUp,
    /// Output driven low.
    OutputLow,
    /// Output driven high.
    OutputHigh,
}

impl PinMode {
    /// Returns true for input modes.
    #[must_use]
    pub const fn is_input(self) -> bool {
        matches!(self, Self::Input | Self::InputPullUp)
    }
}

/// External electrical connection to a pin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExternalPin {
    /// Nothing connected.
    HighZ,
    /// A Thevenin source.
    Thevenin {
        /// Open-circuit voltage.
        voltage: f64,
        /// Series resistance.
        resistance: f64,
    },
}

/// Electrical state of one pin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinElectrical {
    /// Pin voltage in volts.
    pub voltage: f64,
    /// Current sourced by the pin into the external connection.
    pub current: f64,
    /// True when the pin is high impedance.
    pub high_z: bool,
    /// True when the input pull-up is active.
    pub pull_up: bool,
    /// True when an external driver overpowers the output.
    pub contention: bool,
}

impl PinElectrical {
    /// Returns the digital level for a configuration.
    #[must_use]
    pub fn logic_level(&self, config: &PinElectricalConfig) -> bool {
        self.voltage >= config.input_threshold
    }
}

/// Computes the pin state from its mode and external connection.
pub fn measure_pin(
    mode: PinMode,
    external: ExternalPin,
    config: &PinElectricalConfig,
) -> Result<PinElectrical, SimError> {
    validate_config(config)?;
    let source = internal_source(mode, config);
    let external = external_source(external)?;
    match (source, external) {
        (None, None) => Ok(floating()),
        (None, Some((voltage, _))) => Ok(high_z_external(voltage)),
        (Some((voltage, _)), None) => Ok(driven(mode, voltage)),
        (Some(source), Some(load)) => Ok(combined(mode, source, load, config)),
    }
}

fn validate_config(config: &PinElectricalConfig) -> Result<(), SimError> {
    positive(config.supply, "supply")?;
    positive(config.output_resistance, "output resistance")?;
    positive(config.pull_up_resistance, "pull-up resistance")?;
    positive(config.input_threshold, "input threshold")?;
    if config.input_threshold >= config.supply {
        return Err(SimError::InvalidNetlist(
            "input threshold must be below supply".to_string(),
        ));
    }
    Ok(())
}

fn positive(value: f64, name: &str) -> Result<(), SimError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(SimError::InvalidNetlist(format!("{name} must be positive")));
    }
    Ok(())
}

fn external_source(external: ExternalPin) -> Result<Option<(f64, f64)>, SimError> {
    match external {
        ExternalPin::HighZ => Ok(None),
        ExternalPin::Thevenin {
            voltage,
            resistance,
        } => {
            if !voltage.is_finite() {
                return Err(SimError::InvalidNetlist(
                    "external voltage must be finite".to_string(),
                ));
            }
            positive(resistance, "external resistance")?;
            Ok(Some((voltage, resistance)))
        }
    }
}

fn internal_source(mode: PinMode, config: &PinElectricalConfig) -> Option<(f64, f64)> {
    match mode {
        PinMode::Input => None,
        PinMode::InputPullUp => Some((config.supply, config.pull_up_resistance)),
        PinMode::OutputLow => Some((0.0, config.output_resistance)),
        PinMode::OutputHigh => Some((config.supply, config.output_resistance)),
    }
}

fn combined(
    mode: PinMode,
    source: (f64, f64),
    load: (f64, f64),
    config: &PinElectricalConfig,
) -> PinElectrical {
    let (source_voltage, source_resistance) = source;
    let (load_voltage, load_resistance) = load;
    let conductance = 1.0 / source_resistance + 1.0 / load_resistance;
    let voltage =
        (source_voltage / source_resistance + load_voltage / load_resistance) / conductance;
    PinElectrical {
        voltage,
        current: (source_voltage - voltage) / source_resistance,
        high_z: false,
        pull_up: matches!(mode, PinMode::InputPullUp),
        contention: contention(mode, voltage, config),
    }
}

fn contention(mode: PinMode, voltage: f64, config: &PinElectricalConfig) -> bool {
    match mode {
        PinMode::OutputHigh => voltage < config.input_threshold,
        PinMode::OutputLow => voltage > config.input_threshold,
        _ => false,
    }
}

fn floating() -> PinElectrical {
    PinElectrical {
        voltage: 0.0,
        current: 0.0,
        high_z: true,
        pull_up: false,
        contention: false,
    }
}

fn high_z_external(voltage: f64) -> PinElectrical {
    PinElectrical {
        voltage,
        ..floating()
    }
}

fn driven(mode: PinMode, voltage: f64) -> PinElectrical {
    PinElectrical {
        voltage,
        current: 0.0,
        high_z: false,
        pull_up: matches!(mode, PinMode::InputPullUp),
        contention: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{ExternalPin, PinElectricalConfig, PinMode, measure_pin};

    fn config() -> PinElectricalConfig {
        PinElectricalConfig::default()
    }

    #[test]
    fn output_high_drives_supply() {
        let state = measure_pin(PinMode::OutputHigh, ExternalPin::HighZ, &config()).expect("pin");
        assert!((state.voltage - 5.0).abs() < 1e-12);
        assert!(!state.high_z);
        assert!(!state.contention);
    }

    #[test]
    fn input_without_pull_up_is_high_z() {
        let state = measure_pin(PinMode::Input, ExternalPin::HighZ, &config()).expect("pin");
        assert!(state.high_z);
        assert_eq!(state.voltage, 0.0);
    }

    #[test]
    fn input_follows_external_source() {
        let external = ExternalPin::Thevenin {
            voltage: 3.3,
            resistance: 1000.0,
        };
        let state = measure_pin(PinMode::Input, external, &config()).expect("pin");
        assert!(state.high_z);
        assert!((state.voltage - 3.3).abs() < 1e-12);
        assert!(state.logic_level(&config()));
    }

    #[test]
    fn pull_up_sets_idle_high() {
        let state = measure_pin(PinMode::InputPullUp, ExternalPin::HighZ, &config()).expect("pin");
        assert!(state.pull_up);
        assert!((state.voltage - 5.0).abs() < 1e-12);
    }

    #[test]
    fn load_forms_voltage_divider() {
        let external = ExternalPin::Thevenin {
            voltage: 0.0,
            resistance: 1000.0,
        };
        let state = measure_pin(PinMode::OutputHigh, external, &config()).expect("pin");
        assert!(
            (state.voltage - 4.7847).abs() < 0.001,
            "v={}",
            state.voltage
        );
        assert!(
            (state.current - 0.0047847).abs() < 1e-6,
            "i={}",
            state.current
        );
        assert!(!state.contention);
    }

    #[test]
    fn defaults_match_datasheet_derivation() {
        let config = PinElectricalConfig::default();
        let high_voltage = config.supply - 0.02 * config.output_resistance;
        assert!((high_voltage - 4.1).abs() < 1e-9, "voh={high_voltage}");
        assert!((20_000.0..=50_000.0).contains(&config.pull_up_resistance));
    }

    #[test]
    fn strong_external_low_causes_contention() {
        let external = ExternalPin::Thevenin {
            voltage: 0.0,
            resistance: 10.0,
        };
        let state = measure_pin(PinMode::OutputHigh, external, &config()).expect("pin");
        assert!(state.contention);
        assert!(state.voltage < config().input_threshold);
    }

    #[test]
    fn rejects_invalid_configuration() {
        let mut invalid = config();
        invalid.supply = 0.0;
        assert!(measure_pin(PinMode::Input, ExternalPin::HighZ, &invalid).is_err());
        let bad_external = ExternalPin::Thevenin {
            voltage: 0.0,
            resistance: 0.0,
        };
        assert!(measure_pin(PinMode::Input, bad_external, &config()).is_err());
    }
}
