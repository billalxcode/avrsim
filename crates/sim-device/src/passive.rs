//! Passive device models.

use crate::model::{
    DeviceModel, branch_index, pins_exact, positive_parameter, stamp_conductance,
    stamp_inductor_transient, stamp_voltage_branch,
};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// An ideal resistor.
pub struct ResistorModel {
    resistance: f64,
    pins: [NodeId; 2],
}

impl ResistorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let resistance = positive_parameter(instance.parameters(), "resistance", None)?;
        Ok(Self {
            resistance,
            pins: [pins[0], pins[1]],
        })
    }
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
    pins: [NodeId; 2],
}

impl CapacitorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let capacitance = positive_parameter(instance.parameters(), "capacitance", None)?;
        Ok(Self {
            capacitance,
            voltage_prev: 0.0,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for CapacitorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let dt = match dt {
            Some(dt) => dt,
            None => return Ok(()),
        };
        let conductance = self.capacitance / dt;
        let current = conductance * self.voltage_prev;
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        let a = ctx.node_index(self.pins[0]);
        let b = ctx.node_index(self.pins[1]);
        ctx.add_rhs_opt(a, current)?;
        ctx.add_rhs_opt(b, -current)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        self.voltage_prev = first - second;
        Ok(())
    }
}

/// An ideal inductor using backward Euler.
pub struct InductorModel {
    id: DeviceId,
    inductance: f64,
    current_prev: f64,
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
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for InductorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.branch = Some(branch_index(ctx, self.id)?);
        match dt {
            Some(dt) => stamp_inductor_transient(
                ctx,
                self.id,
                self.pins[0],
                self.pins[1],
                self.inductance,
                dt,
                self.current_prev,
            ),
            None => stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], 0.0),
        }
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        if let Some(branch) = self.branch {
            self.current_prev = solution.branch_current(branch).unwrap_or(0.0);
        }
        Ok(())
    }
}
