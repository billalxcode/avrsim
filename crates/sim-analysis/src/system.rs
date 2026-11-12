//! Simulation system mapping and device model orchestration.

use sim_core::{Circuit, Device, DeviceKind, MnaContext, SimError, Solution};
use sim_device::{DeviceModel, IntegrationMethod, model_from_instance};
use std::sync::Arc;

/// A circuit prepared for analysis.
pub struct SimulationSystem {
    node_map: Arc<Vec<Option<usize>>>,
    branch_map: Arc<Vec<Option<usize>>>,
    models: Vec<Box<dyn DeviceModel>>,
    node_count: usize,
    branch_count: usize,
    node_slots: usize,
    linear: bool,
}

impl SimulationSystem {
    /// Builds mappings and device models from a circuit.
    pub fn new(circuit: &Circuit) -> Result<Self, SimError> {
        let node_map = build_node_map(circuit);
        let node_count = count_nodes(&node_map);
        let (branch_map, branch_count) = build_branch_map(circuit);
        let models = build_models(circuit)?;
        let linear = is_linear(circuit);
        Ok(Self {
            node_map: Arc::new(node_map),
            branch_map: Arc::new(branch_map),
            models,
            node_count,
            branch_count,
            node_slots: circuit.nodes().len(),
            linear,
        })
    }

    /// Returns true when the circuit has no nonlinear device.
    #[must_use]
    pub(crate) fn is_linear(&self) -> bool {
        self.linear
    }

    /// Returns a zero solution with matching dimensions.
    pub(crate) fn zero_solution(&self) -> Solution {
        Solution::zero(self.node_slots, self.node_count + self.branch_count)
    }

    /// Stamps all models and solves one linearized system.
    pub(crate) fn solve(
        &mut self,
        solution: &Solution,
        dt: Option<f64>,
        gmin: f64,
    ) -> Result<Solution, SimError> {
        self.solve_with_method(solution, dt, gmin, IntegrationMethod::BackwardEuler)
    }

    pub(crate) fn solve_with_method(
        &mut self,
        solution: &Solution,
        dt: Option<f64>,
        gmin: f64,
        method: IntegrationMethod,
    ) -> Result<Solution, SimError> {
        let mut context = MnaContext::from_shared(
            self.node_count,
            self.branch_count,
            Arc::clone(&self.node_map),
            Arc::clone(&self.branch_map),
        )?;
        self.add_gmin(&mut context, gmin)?;
        for model in &mut self.models {
            model.stamp_method(&mut context, solution, dt, method)?;
        }
        context.solve()
    }

    pub(crate) fn stamp_dc(&mut self, solution: &Solution) -> Result<MnaContext, SimError> {
        let mut context = MnaContext::from_shared(
            self.node_count,
            self.branch_count,
            Arc::clone(&self.node_map),
            Arc::clone(&self.branch_map),
        )?;
        for model in &mut self.models {
            model.stamp(&mut context, solution, None)?;
        }
        Ok(context)
    }

    /// Updates all model states after convergence.
    pub(crate) fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        for model in &mut self.models {
            model.update(solution)?;
        }
        Ok(())
    }

    fn add_gmin(&self, context: &mut MnaContext, gmin: f64) -> Result<(), SimError> {
        for index in 0..self.node_count {
            context.add_matrix(index, index, gmin)?;
        }
        Ok(())
    }
}

fn build_models(circuit: &Circuit) -> Result<Vec<Box<dyn DeviceModel>>, SimError> {
    circuit.devices().iter().map(model_from_instance).collect()
}

fn is_linear(circuit: &Circuit) -> bool {
    !circuit
        .devices()
        .iter()
        .any(|device| is_nonlinear(device.kind()))
}

fn is_nonlinear(kind: &DeviceKind) -> bool {
    matches!(
        kind,
        DeviceKind::Diode
            | DeviceKind::Nmos
            | DeviceKind::Pmos
            | DeviceKind::Npn
            | DeviceKind::Pnp
            | DeviceKind::JfetN
            | DeviceKind::JfetP
    )
}

pub(crate) fn build_node_map(circuit: &Circuit) -> Vec<Option<usize>> {
    let mut map = Vec::with_capacity(circuit.nodes().len());
    let mut index = 0;
    for node in circuit.nodes() {
        if node.kind().is_ground() {
            map.push(None);
        } else {
            map.push(Some(index));
            index += 1;
        }
    }
    map
}

pub(crate) fn count_nodes(map: &[Option<usize>]) -> usize {
    map.iter().filter(|slot| slot.is_some()).count()
}

pub(crate) fn build_branch_map(circuit: &Circuit) -> (Vec<Option<usize>>, usize) {
    let mut map = Vec::with_capacity(circuit.devices().len());
    let mut index = non_ground_node_count(circuit);
    let mut branches = 0;
    for device in circuit.devices() {
        let slots = branch_slots(device.kind());
        if slots == 0 {
            map.push(None);
        } else {
            map.push(Some(index));
            index += slots;
            branches += slots;
        }
    }
    (map, branches)
}

fn branch_slots(kind: &DeviceKind) -> usize {
    match kind {
        DeviceKind::Ccvs => 2,
        DeviceKind::VoltageSource
        | DeviceKind::PulseVoltageSource
        | DeviceKind::Vcvs
        | DeviceKind::Cccs
        | DeviceKind::Inductor => 1,
        _ => 0,
    }
}

fn non_ground_node_count(circuit: &Circuit) -> usize {
    circuit
        .nodes()
        .iter()
        .filter(|node| !node.kind().is_ground())
        .count()
}
