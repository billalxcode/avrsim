//! Independent source models.

use crate::model::{
    DeviceModel, parameter, pins_exact, stamp_current_source, stamp_voltage_branch,
};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// An independent voltage source.
pub struct VoltageSourceModel {
    id: DeviceId,
    voltage: f64,
    pins: [NodeId; 2],
}

impl VoltageSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let voltage = parameter(instance.parameters(), "voltage", 0.0);
        Ok(Self {
            id: instance.id(),
            voltage,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for VoltageSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], self.voltage)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// An independent current source.
pub struct CurrentSourceModel {
    current: f64,
    pins: [NodeId; 2],
}

impl CurrentSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let current = parameter(instance.parameters(), "current", 0.0);
        Ok(Self {
            current,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for CurrentSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_current_source(ctx, self.pins[0], self.pins[1], self.current)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}
