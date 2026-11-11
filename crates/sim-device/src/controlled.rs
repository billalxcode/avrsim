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

/// A current-controlled current source with pins `out+, out-, ctrl+, ctrl-`.
pub struct CccsModel {
    id: DeviceId,
    gain: f64,
    pins: [NodeId; 4],
}

impl CccsModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let gain = parameter(instance.parameters(), "gain", 1.0);
        if !gain.is_finite() {
            return Err(SimError::InvalidNetlist(
                "cccs gain must be finite".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            gain,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }
}

impl DeviceModel for CccsModel {
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
        ctx.add_matrix_opt(control, Some(branch), 1.0)?;
        ctx.add_matrix_opt(control_reference, Some(branch), -1.0)?;
        ctx.add_matrix_opt(Some(branch), control, 1.0)?;
        ctx.add_matrix_opt(Some(branch), control_reference, -1.0)?;
        ctx.add_matrix_opt(output, Some(branch), -self.gain)?;
        ctx.add_matrix_opt(reference, Some(branch), self.gain)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A current-controlled voltage source with pins `out+, out-, ctrl+, ctrl-`.
pub struct CcvsModel {
    id: DeviceId,
    transresistance: f64,
    pins: [NodeId; 4],
}

impl CcvsModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let transresistance = parameter(instance.parameters(), "rm", 1000.0);
        if !transresistance.is_finite() {
            return Err(SimError::InvalidNetlist(
                "ccvs rm must be finite".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            transresistance,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }
}

impl DeviceModel for CcvsModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let sense = branch_index(ctx, self.id)?;
        self.stamp_sense(ctx, sense)?;
        self.stamp_output(ctx, sense + 1, sense)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

impl CcvsModel {
    fn stamp_sense(&self, ctx: &mut MnaContext, sense: usize) -> Result<(), SimError> {
        let control = ctx.node_index(self.pins[2]);
        let reference = ctx.node_index(self.pins[3]);
        ctx.add_matrix_opt(control, Some(sense), 1.0)?;
        ctx.add_matrix_opt(reference, Some(sense), -1.0)?;
        ctx.add_matrix_opt(Some(sense), control, 1.0)?;
        ctx.add_matrix_opt(Some(sense), reference, -1.0)
    }

    fn stamp_output(
        &self,
        ctx: &mut MnaContext,
        output: usize,
        sense: usize,
    ) -> Result<(), SimError> {
        let positive = ctx.node_index(self.pins[0]);
        let negative = ctx.node_index(self.pins[1]);
        ctx.add_matrix_opt(positive, Some(output), 1.0)?;
        ctx.add_matrix_opt(negative, Some(output), -1.0)?;
        ctx.add_matrix_opt(Some(output), positive, 1.0)?;
        ctx.add_matrix_opt(Some(output), negative, -1.0)?;
        ctx.add_matrix_opt(Some(output), Some(sense), -self.transresistance)
    }
}
