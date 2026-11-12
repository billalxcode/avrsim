//! Shockley diode model.

use crate::model::{
    DeviceModel, MAX_EXPONENT, MAX_FORWARD_VOLTAGE, junction_critical_voltage,
    non_negative_parameter, pins_exact, pnjlim, positive_parameter, stamp_capacitor,
    stamp_conductance, terminal_conductance, thermal_voltage,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};
use std::collections::BTreeMap;

/// A two-terminal Shockley diode.
pub struct DiodeModel {
    saturation_current: f64,
    emission: f64,
    thermal_voltage: f64,
    critical_voltage: f64,
    series_resistance: f64,
    junction_cap: JunctionCapacitance,
    voltage_prev: f64,
    junction_voltage: f64,
    cap_voltage_prev: f64,
    pins: [NodeId; 2],
}

#[derive(Clone, Copy)]
struct JunctionCapacitance {
    zero: f64,
    potential: f64,
    grading: f64,
}

fn junction_capacitance(params: &BTreeMap<String, f64>) -> Result<JunctionCapacitance, SimError> {
    Ok(JunctionCapacitance {
        zero: non_negative_parameter(params, "cj0", 0.0)?,
        potential: positive_parameter(params, "vj", Some(0.7))?,
        grading: non_negative_parameter(params, "m", 0.5)?,
    })
}

/// Returns the small-signal junction capacitance at a bias voltage.
pub fn diode_capacitance(params: &BTreeMap<String, f64>, voltage: f64) -> Result<f64, SimError> {
    Ok(capacitance_at(&junction_capacitance(params)?, voltage))
}

fn capacitance_at(cap: &JunctionCapacitance, voltage: f64) -> f64 {
    if cap.zero <= 0.0 {
        return 0.0;
    }
    let ratio = (1.0 - voltage / cap.potential).max(0.1);
    cap.zero / ratio.powf(cap.grading)
}

impl DiodeModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let saturation_current = positive_parameter(instance.parameters(), "is", Some(1e-12))?;
        let emission = positive_parameter(instance.parameters(), "n", Some(1.0))?;
        let thermal_voltage = thermal_voltage(instance.parameters())?;
        let series_resistance = non_negative_parameter(instance.parameters(), "rs", 0.0)?;
        let junction_cap = junction_capacitance(instance.parameters())?;
        Ok(Self {
            saturation_current,
            emission,
            thermal_voltage,
            critical_voltage: junction_critical_voltage(saturation_current, thermal_voltage),
            series_resistance,
            junction_cap,
            voltage_prev: 0.0,
            junction_voltage: 0.0,
            cap_voltage_prev: 0.0,
            pins: [pins[0], pins[1]],
        })
    }

    fn capacitance(&self, voltage: f64) -> f64 {
        capacitance_at(&self.junction_cap, voltage)
    }

    fn stamp_capacitance(
        &self,
        ctx: &mut MnaContext,
        dt: Option<f64>,
        voltage: f64,
    ) -> Result<(), SimError> {
        let dt = match dt {
            Some(value) => value,
            None => return Ok(()),
        };
        let capacitance = self.capacitance(voltage);
        stamp_capacitor(
            ctx,
            self.pins[0],
            self.pins[1],
            capacitance,
            self.cap_voltage_prev,
            dt,
        )
    }

    fn solve_junction(&self, terminal: f64) -> f64 {
        if self.series_resistance <= 0.0 {
            return terminal;
        }
        let mut low = terminal.min(0.0) - 1.0;
        let mut high = MAX_FORWARD_VOLTAGE;
        for _ in 0..50 {
            let middle = 0.5 * (low + high);
            let value = middle + self.series_resistance * self.current(middle) - terminal;
            if value > 0.0 {
                high = middle;
            } else {
                low = middle;
            }
        }
        0.5 * (low + high)
    }

    fn limited_voltage(&mut self, voltage: f64) -> f64 {
        let limited = pnjlim(
            voltage,
            self.voltage_prev,
            self.thermal_voltage,
            self.critical_voltage,
        );
        self.voltage_prev = limited;
        limited
    }

    fn linearize(&mut self, voltage: f64) -> (f64, f64) {
        let junction = self.solve_junction(voltage);
        let limited = self.limited_voltage(junction);
        self.junction_voltage = limited;
        let current = self.current(limited);
        let junction_conductance = self.conductance(limited);
        let conductance = terminal_conductance(junction_conductance, self.series_resistance);
        (conductance, current)
    }

    fn linearization_voltage(&self, terminal: f64) -> f64 {
        if self.series_resistance > 0.0 {
            terminal
        } else {
            self.junction_voltage.min(MAX_FORWARD_VOLTAGE)
        }
    }

    fn current(&self, voltage: f64) -> f64 {
        let limited = voltage.min(MAX_FORWARD_VOLTAGE);
        let exponent = (limited / (self.emission * self.thermal_voltage)).min(MAX_EXPONENT);
        self.saturation_current * (exponent.exp() - 1.0)
    }

    fn conductance(&self, voltage: f64) -> f64 {
        let limited = voltage.min(MAX_FORWARD_VOLTAGE);
        let exponent = (limited / (self.emission * self.thermal_voltage)).min(MAX_EXPONENT);
        self.saturation_current / (self.emission * self.thermal_voltage) * exponent.exp()
    }
}

impl DeviceModel for DiodeModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let voltage = first - second;
        let (conductance, current) = self.linearize(voltage);
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        let linearization = self.linearization_voltage(voltage);
        let constant = conductance * linearization - current;
        let a = ctx.node_index(self.pins[0]);
        let b = ctx.node_index(self.pins[1]);
        ctx.add_rhs_opt(a, constant)?;
        ctx.add_rhs_opt(b, -constant)?;
        self.stamp_capacitance(ctx, dt, self.junction_voltage)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        self.voltage_prev = self.junction_voltage;
        self.junction_voltage = self.solve_junction(first - second);
        self.cap_voltage_prev = first - second;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DiodeModel, JunctionCapacitance, MAX_FORWARD_VOLTAGE, diode_capacitance};
    use sim_core::NodeId;
    use std::collections::BTreeMap;

    fn model() -> DiodeModel {
        DiodeModel {
            saturation_current: 1e-12,
            emission: 1.0,
            thermal_voltage: 0.02585,
            critical_voltage: 0.611,
            series_resistance: 0.0,
            junction_cap: JunctionCapacitance {
                zero: 0.0,
                potential: 0.7,
                grading: 0.5,
            },
            voltage_prev: 0.0,
            junction_voltage: 0.0,
            cap_voltage_prev: 0.0,
            pins: [NodeId::new(1), NodeId::GROUND],
        }
    }

    fn cap_model() -> DiodeModel {
        let mut model = model();
        model.junction_cap = JunctionCapacitance {
            zero: 1e-12,
            potential: 0.7,
            grading: 0.5,
        };
        model
    }

    #[test]
    fn zero_voltage_has_zero_current() {
        assert_eq!(model().current(0.0), 0.0);
    }

    #[test]
    fn forward_bias_increases_current() {
        let model = model();
        assert!(model.current(0.6) > model.current(0.5));
    }

    #[test]
    fn reverse_bias_is_small() {
        let model = model();
        assert!(model.current(-1.0).abs() < 1e-9);
    }

    #[test]
    fn conductance_is_positive() {
        let model = model();
        assert!(model.conductance(0.0) > 0.0);
        assert!(model.conductance(0.5) > model.conductance(0.4));
    }

    #[test]
    fn forward_voltage_is_limited() {
        let model = model();
        let at_limit = model.current(MAX_FORWARD_VOLTAGE);
        let above_limit = model.current(MAX_FORWARD_VOLTAGE + 1.0);
        assert!((at_limit - above_limit).abs() < 1e-9);
    }

    #[test]
    fn series_resistance_lowers_junction_voltage() {
        let mut model = model();
        model.series_resistance = 10.0;
        let junction = model.solve_junction(5.0);
        assert!(junction > 0.6 && junction < 0.9, "junction={junction}");
    }

    #[test]
    fn public_capacitance_matches_bias() {
        let mut params = BTreeMap::new();
        params.insert("cj0".to_string(), 1e-12);
        params.insert("vj".to_string(), 0.7);
        params.insert("m".to_string(), 0.5);
        let zero = diode_capacitance(&params, 0.0).expect("zero bias");
        let reverse = diode_capacitance(&params, -0.7).expect("reverse bias");
        assert!((zero - 1e-12).abs() < 1e-20);
        assert!((reverse - 0.70710678e-12).abs() < 1e-18);
    }

    #[test]
    fn junction_capacitance_matches_bias() {
        let model = cap_model();
        assert!((model.capacitance(0.0) - 1e-12).abs() < 1e-20);
        assert!((model.capacitance(0.35) - 1.41421356e-12).abs() < 1e-18);
        assert!((model.capacitance(-0.7) - 0.70710678e-12).abs() < 1e-18);
    }
}
