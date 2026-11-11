//! Passive device models.

use crate::model::{
    DeviceModel, IntegrationMethod, branch_index, parameter, pins_exact, positive_parameter,
    stamp_conductance, stamp_inductor_branch, stamp_voltage_branch,
};
use sim_core::{
    Device, DeviceId, DeviceInstance, MnaContext, NOMINAL_TEMPERATURE, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// An ideal resistor.
pub struct ResistorModel {
    resistance: f64,
    pins: [NodeId; 2],
}

impl ResistorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let resistance = resistance_value(instance.parameters())?;
        Ok(Self {
            resistance,
            pins: [pins[0], pins[1]],
        })
    }
}

fn resistance_value(params: &BTreeMap<String, f64>) -> Result<f64, SimError> {
    let nominal = positive_parameter(params, "resistance", None)?;
    let temperature = parameter(params, "temp", NOMINAL_TEMPERATURE);
    let delta = temperature - NOMINAL_TEMPERATURE;
    let tc1 = parameter(params, "tc1", 0.0);
    let tc2 = parameter(params, "tc2", 0.0);
    let scale = 1.0 + tc1 * delta + tc2 * delta * delta;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "resistor temperature scale is invalid".to_string(),
        ));
    }
    Ok(nominal * scale)
}

impl DeviceModel for ResistorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_conductance(ctx, self.pins[0], self.pins[1], 1.0 / self.resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// An ideal capacitor using backward Euler.
pub struct CapacitorModel {
    capacitance: f64,
    voltage_prev: f64,
    voltage_prev2: f64,
    current_prev: f64,
    dt_last: f64,
    method_last: IntegrationMethod,
    pins: [NodeId; 2],
}

impl CapacitorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let capacitance = positive_parameter(instance.parameters(), "capacitance", None)?;
        Ok(Self {
            capacitance,
            voltage_prev: 0.0,
            voltage_prev2: 0.0,
            current_prev: 0.0,
            dt_last: 0.0,
            method_last: IntegrationMethod::BackwardEuler,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for CapacitorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.stamp_method(ctx, solution, dt, IntegrationMethod::BackwardEuler)
    }

    fn stamp_method(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
        method: IntegrationMethod,
    ) -> Result<(), SimError> {
        let dt = match dt {
            Some(dt) => dt,
            None => return Ok(()),
        };
        self.dt_last = dt;
        self.method_last = method;
        match method {
            IntegrationMethod::BackwardEuler => self.stamp_backward(ctx, dt),
            IntegrationMethod::Trapezoidal => self.stamp_trapezoidal(ctx, dt),
            IntegrationMethod::Gear2 => self.stamp_gear(ctx, dt),
        }
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let voltage = first - second;
        let current = self.compute_current(voltage);
        self.voltage_prev2 = self.voltage_prev;
        self.voltage_prev = voltage;
        self.current_prev = current;
        Ok(())
    }
}

impl CapacitorModel {
    fn stamp_backward(&mut self, ctx: &mut MnaContext, dt: f64) -> Result<(), SimError> {
        let conductance = self.capacitance / dt;
        let rhs = conductance * self.voltage_prev;
        self.stamp_companion(ctx, conductance, rhs)
    }

    fn stamp_trapezoidal(&mut self, ctx: &mut MnaContext, dt: f64) -> Result<(), SimError> {
        let conductance = 2.0 * self.capacitance / dt;
        let rhs = conductance * self.voltage_prev + self.current_prev;
        self.stamp_companion(ctx, conductance, rhs)
    }

    fn stamp_gear(&mut self, ctx: &mut MnaContext, dt: f64) -> Result<(), SimError> {
        let conductance = 1.5 * self.capacitance / dt;
        let rhs = 2.0 * self.capacitance / dt * self.voltage_prev
            - 0.5 * self.capacitance / dt * self.voltage_prev2;
        self.stamp_companion(ctx, conductance, rhs)
    }

    fn stamp_companion(
        &mut self,
        ctx: &mut MnaContext,
        conductance: f64,
        rhs: f64,
    ) -> Result<(), SimError> {
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        let a = ctx.node_index(self.pins[0]);
        let b = ctx.node_index(self.pins[1]);
        ctx.add_rhs_opt(a, rhs)?;
        ctx.add_rhs_opt(b, -rhs)
    }

    fn compute_current(&self, voltage: f64) -> f64 {
        if self.dt_last <= 0.0 {
            return self.current_prev;
        }
        match self.method_last {
            IntegrationMethod::BackwardEuler => {
                self.capacitance / self.dt_last * (voltage - self.voltage_prev)
            }
            IntegrationMethod::Trapezoidal => {
                let conductance = 2.0 * self.capacitance / self.dt_last;
                conductance * (voltage - self.voltage_prev) - self.current_prev
            }
            IntegrationMethod::Gear2 => {
                1.5 * self.capacitance / self.dt_last * voltage
                    - 2.0 * self.capacitance / self.dt_last * self.voltage_prev
                    + 0.5 * self.capacitance / self.dt_last * self.voltage_prev2
            }
        }
    }
}

/// An ideal inductor using backward Euler.
pub struct InductorModel {
    id: DeviceId,
    inductance: f64,
    current_prev: f64,
    current_prev2: f64,
    voltage_prev: f64,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl InductorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let inductance = positive_parameter(instance.parameters(), "inductance", None)?;
        Ok(Self {
            id: instance.id(),
            inductance,
            current_prev: 0.0,
            current_prev2: 0.0,
            voltage_prev: 0.0,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }
}

impl InductorModel {
    fn companion(&self, dt: f64, method: IntegrationMethod) -> (f64, f64) {
        match method {
            IntegrationMethod::BackwardEuler => self.backward_companion(dt),
            IntegrationMethod::Trapezoidal => self.trapezoidal_companion(dt),
            IntegrationMethod::Gear2 => self.gear_companion(dt),
        }
    }

    fn backward_companion(&self, dt: f64) -> (f64, f64) {
        let resistance = self.inductance / dt;
        (resistance, -resistance * self.current_prev)
    }

    fn trapezoidal_companion(&self, dt: f64) -> (f64, f64) {
        let resistance = 2.0 * self.inductance / dt;
        (
            resistance,
            -resistance * self.current_prev - self.voltage_prev,
        )
    }

    fn gear_companion(&self, dt: f64) -> (f64, f64) {
        let resistance = 1.5 * self.inductance / dt;
        (
            resistance,
            -2.0 * self.inductance / dt * self.current_prev
                + 0.5 * self.inductance / dt * self.current_prev2,
        )
    }
}

impl DeviceModel for InductorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.stamp_method(ctx, solution, dt, IntegrationMethod::BackwardEuler)
    }

    fn stamp_method(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
        method: IntegrationMethod,
    ) -> Result<(), SimError> {
        self.branch = Some(branch_index(ctx, self.id)?);
        let dt = match dt {
            Some(dt) => dt,
            None => return stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], 0.0),
        };
        let (resistance, rhs) = self.companion(dt, method);
        stamp_inductor_branch(ctx, self.id, self.pins[0], self.pins[1], resistance, rhs)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        if let Some(branch) = self.branch {
            let current = solution.branch_current(branch).unwrap_or(0.0);
            let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
            let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
            self.current_prev2 = self.current_prev;
            self.current_prev = current;
            self.voltage_prev = first - second;
        }
        Ok(())
    }
}
