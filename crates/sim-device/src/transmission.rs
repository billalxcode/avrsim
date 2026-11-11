//! Lossless transmission line with a discrete delay history.

use crate::model::{
    DeviceModel, pins_exact, positive_parameter, stamp_conductance, stamp_current_source,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};
use std::collections::VecDeque;

const MAX_STEPS: usize = 1_000_000;

/// A lossless transmission line with pins `p1+, p1-, p2+, p2-`.
pub struct TransmissionLineModel {
    impedance: f64,
    delay: f64,
    step: Option<f64>,
    steps: usize,
    history: VecDeque<PortState>,
    equivalent: (f64, f64),
    pins: [NodeId; 4],
}

#[derive(Clone, Copy, Default)]
struct PortState {
    v1: f64,
    i1: f64,
    v2: f64,
    i2: f64,
}

impl TransmissionLineModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let impedance = positive_parameter(instance.parameters(), "z0", Some(50.0))?;
        let delay = positive_parameter(instance.parameters(), "delay", Some(1e-9))?;
        Ok(Self {
            impedance,
            delay,
            step: None,
            steps: 0,
            history: VecDeque::new(),
            equivalent: (0.0, 0.0),
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn ensure_history(&mut self, dt: f64) -> Result<(), SimError> {
        if let Some(step) = self.step {
            if (step - dt).abs() > 1e-18 {
                return Err(SimError::UnsupportedDevice(
                    "transmission line requires a constant time step".to_string(),
                ));
            }
            return Ok(());
        }
        let steps = (self.delay / dt).ceil() as usize;
        if steps == 0 || steps > MAX_STEPS {
            return Err(SimError::UnsupportedDevice(
                "transmission line delay is out of range".to_string(),
            ));
        }
        self.history = vec![PortState::default(); steps].into();
        self.steps = steps;
        self.step = Some(dt);
        Ok(())
    }

    fn port_voltages(&self, solution: &Solution) -> (f64, f64) {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[2]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        (first, second)
    }

    fn stamp_port(
        &self,
        ctx: &mut MnaContext,
        positive: NodeId,
        negative: NodeId,
        equivalent: f64,
    ) -> Result<(), SimError> {
        stamp_conductance(ctx, positive, negative, 1.0 / self.impedance)?;
        stamp_current_source(ctx, negative, positive, equivalent / self.impedance)
    }
}

impl DeviceModel for TransmissionLineModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let dt = dt.ok_or_else(|| {
            SimError::UnsupportedDevice(
                "transmission line requires a transient time step".to_string(),
            )
        })?;
        self.ensure_history(dt)?;
        let delayed = self.history.front().copied().unwrap_or_default();
        self.equivalent = (
            delayed.v2 + self.impedance * delayed.i2,
            delayed.v1 + self.impedance * delayed.i1,
        );
        self.stamp_port(ctx, self.pins[0], self.pins[1], self.equivalent.0)?;
        self.stamp_port(ctx, self.pins[2], self.pins[3], self.equivalent.1)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let (v1, v2) = self.port_voltages(solution);
        let i1 = (v1 - self.equivalent.0) / self.impedance;
        let i2 = (v2 - self.equivalent.1) / self.impedance;
        self.history.push_back(PortState { v1, i1, v2, i2 });
        while self.history.len() > self.steps {
            self.history.pop_front();
        }
        Ok(())
    }
}
