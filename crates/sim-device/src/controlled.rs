//! Linear controlled sources.

use crate::model::{DeviceModel, branch_index, parameter, pins_exact};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A voltage-controlled voltage source with pins `out+, out-, ctrl+, ctrl-`.
pub struct VcvsModel {
    id: DeviceId,
    gain: f64,
    pins: [NodeId; 4],
}

impl VcvsModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let gain = parameter(instance.parameters(), "gain", 1.0);
        if !gain.is_finite() {
            return Err(SimError::InvalidNetlist(
                "vcvs gain must be finite".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            gain,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }
}

impl DeviceModel for VcvsModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let branch = branch_index(ctx, self.id)?;
        let output = ctx.node_index(self.pins[0]);
        let reference = ctx.node_index(self.pins[1]);
        let control = ctx.node_index(self.pins[2]);
        let control_reference = ctx.node_index(self.pins[3]);
        ctx.add_matrix_opt(output, Some(branch), 1.0)?;
        ctx.add_matrix_opt(reference, Some(branch), -1.0)?;
        ctx.add_matrix_opt(Some(branch), output, 1.0)?;
        ctx.add_matrix_opt(Some(branch), reference, -1.0)?;
        ctx.add_matrix_opt(Some(branch), control, -self.gain)?;
        ctx.add_matrix_opt(Some(branch), control_reference, self.gain)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A voltage-controlled current source with pins `out+, out-, ctrl+, ctrl-`.
pub struct VccsModel {
    transconductance: f64,
    pins: [NodeId; 4],
}

impl VccsModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let transconductance = parameter(instance.parameters(), "gm", 1e-3);
        if !transconductance.is_finite() {
            return Err(SimError::InvalidNetlist(
                "vccs gm must be finite".to_string(),
            ));
        }
        Ok(Self {
            transconductance,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }
}

impl DeviceModel for VccsModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let output = ctx.node_index(self.pins[0]);
        let reference = ctx.node_index(self.pins[1]);
        let control = ctx.node_index(self.pins[2]);
        let control_reference = ctx.node_index(self.pins[3]);
        let gm = self.transconductance;
        ctx.add_matrix_opt(output, control, -gm)?;
        ctx.add_matrix_opt(output, control_reference, gm)?;
        ctx.add_matrix_opt(reference, control, gm)?;
        ctx.add_matrix_opt(reference, control_reference, -gm)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}
