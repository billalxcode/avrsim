//! Electromechanical relay with a coil and changeover contacts.

use crate::model::{DeviceModel, parameter, pins_exact, positive_parameter, stamp_conductance};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A five-pin relay with pins `coil+, coil-, common, NO, NC`.
pub struct RelayModel {
    coil: [NodeId; 2],
    common: NodeId,
    normally_open: NodeId,
    normally_closed: NodeId,
    coil_resistance: f64,
    on_resistance: f64,
    off_resistance: f64,
    threshold: f64,
}

impl RelayModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 5)?;
        let (coil_resistance, on_resistance, off_resistance, threshold) =
            relay_parameters(instance)?;
        Ok(Self {
            coil: [pins[0], pins[1]],
            common: pins[2],
            normally_open: pins[3],
            normally_closed: pins[4],
            coil_resistance,
            on_resistance,
            off_resistance,
            threshold,
        })
    }

    fn energized(&self, solution: &Solution) -> bool {
        let positive = solution.voltage(self.coil[0]).unwrap_or(0.0);
        let negative = solution.voltage(self.coil[1]).unwrap_or(0.0);
        positive - negative > self.threshold
    }

    fn contact_conductances(&self, energized: bool) -> (f64, f64) {
        let open = 1.0 / self.off_resistance;
        let closed = 1.0 / self.on_resistance;
        if energized {
            (closed, open)
        } else {
            (open, closed)
        }
    }
}

fn relay_parameters(instance: &DeviceInstance) -> Result<(f64, f64, f64, f64), SimError> {
    let coil = positive_parameter(instance.parameters(), "rcoil", Some(100.0))?;
    let on = positive_parameter(instance.parameters(), "ron", Some(0.1))?;
    let off = positive_parameter(instance.parameters(), "roff", Some(1e9))?;
    let threshold = parameter(instance.parameters(), "vth", 2.5);
    if !threshold.is_finite() {
        return Err(SimError::InvalidNetlist(
            "relay threshold must be finite".to_string(),
        ));
    }
    Ok((coil, on, off, threshold))
}

impl DeviceModel for RelayModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let coil = 1.0 / self.coil_resistance;
        stamp_conductance(ctx, self.coil[0], self.coil[1], coil)?;
        let (open_contact, closed_contact) = self.contact_conductances(self.energized(solution));
        stamp_conductance(ctx, self.common, self.normally_open, open_contact)?;
        stamp_conductance(ctx, self.common, self.normally_closed, closed_contact)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RelayModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};

    fn model(pins: usize) -> RelayModel {
        let instance = DeviceInstance::new(
            DeviceId::new(0),
            "relay",
            DeviceKind::Relay,
            (1..=pins).map(|raw| NodeId::new(raw as u32)).collect(),
            Default::default(),
        );
        RelayModel::from_instance(&instance).expect("model")
    }

    #[test]
    fn relay_requires_five_pins() {
        let instance = DeviceInstance::new(
            DeviceId::new(0),
            "relay",
            DeviceKind::Relay,
            vec![NodeId::GROUND; 4],
            Default::default(),
        );
        assert!(RelayModel::from_instance(&instance).is_err());
    }

    #[test]
    fn relay_switches_contact_state() {
        let relay = model(5);
        let (open_off, closed_off) = relay.contact_conductances(false);
        let (open_on, closed_on) = relay.contact_conductances(true);
        assert!(open_off <= 1e-9);
        assert!((closed_off - 10.0).abs() < 1e-9);
        assert!((open_on - 10.0).abs() < 1e-9);
        assert!(closed_on <= 1e-9);
    }

    #[test]
    fn relay_coil_energizes_above_threshold() {
        let relay = model(5);
        let mut solution = Solution::zero(6, 0);
        solution.node_voltages[1] = 3.0;
        solution.node_voltages[2] = 0.0;
        assert!(relay.energized(&solution));
        solution.node_voltages[1] = 1.0;
        assert!(!relay.energized(&solution));
    }
}
