//! Independent source models.

use crate::model::{
    DeviceModel, non_negative_parameter, parameter, pins_exact, positive_parameter,
    stamp_current_source, stamp_voltage_branch,
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

/// An independent pulse voltage source.
pub struct PulseSourceModel {
    id: DeviceId,
    initial: f64,
    pulsed: f64,
    timing: PulseTiming,
    time: f64,
    step: Option<f64>,
    pins: [NodeId; 2],
}

#[derive(Clone, Copy)]
struct PulseTiming {
    delay: f64,
    rise: f64,
    fall: f64,
    width: f64,
    period: f64,
}

impl PulseSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        let initial = parameter(params, "v1", 0.0);
        let pulsed = parameter(params, "v2", 5.0);
        let timing = pulse_timing(params)?;
        Ok(Self {
            id: instance.id(),
            initial,
            pulsed,
            timing,
            time: 0.0,
            step: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn value_at(&self, time: f64) -> f64 {
        if time < self.timing.delay {
            return self.initial;
        }
        let phase = (time - self.timing.delay) % self.timing.period;
        if self.timing.rise > 0.0 && phase < self.timing.rise {
            return self.initial + (self.pulsed - self.initial) * phase / self.timing.rise;
        }
        let fall_start = self.timing.rise + self.timing.width;
        if phase < fall_start {
            return self.pulsed;
        }
        if self.timing.fall > 0.0 && phase < fall_start + self.timing.fall {
            let ratio = (phase - fall_start) / self.timing.fall;
            return self.pulsed + (self.initial - self.pulsed) * ratio;
        }
        self.initial
    }
}

fn pulse_timing(params: &std::collections::BTreeMap<String, f64>) -> Result<PulseTiming, SimError> {
    Ok(PulseTiming {
        delay: non_negative_parameter(params, "delay", 0.0)?,
        rise: non_negative_parameter(params, "rise", 1e-9)?,
        fall: non_negative_parameter(params, "fall", 1e-9)?,
        width: non_negative_parameter(params, "width", 1e-6)?,
        period: positive_parameter(params, "period", Some(2e-6))?,
    })
}

impl DeviceModel for PulseSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        let voltage = self.value_at(self.time);
        stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], voltage)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
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

#[cfg(test)]
mod tests {
    use super::{PulseSourceModel, PulseTiming};
    use crate::model::DeviceModel;
    use sim_core::{DeviceId, NodeId, Solution};

    fn pulse() -> PulseSourceModel {
        PulseSourceModel {
            id: DeviceId::new(0),
            initial: 0.0,
            pulsed: 5.0,
            timing: PulseTiming {
                delay: 1e-6,
                rise: 1e-6,
                fall: 1e-6,
                width: 2e-6,
                period: 10e-6,
            },
            time: 0.0,
            step: None,
            pins: [NodeId::new(1), NodeId::GROUND],
        }
    }

    #[test]
    fn pulse_follows_rise_high_fall_and_repeat() {
        let model = pulse();
        assert!(model.value_at(0.5e-6).abs() < 1e-12);
        assert!((model.value_at(1.5e-6) - 2.5).abs() < 1e-12);
        assert!(
            (model.value_at(3.0e-6) - 5.0).abs() < 1e-12,
            "v={}",
            model.value_at(3.0e-6)
        );
        assert!((model.value_at(4.5e-6) - 2.5).abs() < 1e-12);
        assert!(model.value_at(5.0e-6).abs() < 1e-12);
        assert!((model.value_at(11.5e-6) - 2.5).abs() < 1e-12);
    }

    #[test]
    fn update_advances_pulse_time() {
        let mut model = pulse();
        model.step = Some(1e-6);
        model.update(&Solution::zero(2, 0)).expect("update");
        assert!((model.time - 1e-6).abs() < 1e-12);
    }
}
